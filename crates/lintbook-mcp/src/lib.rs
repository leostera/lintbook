//! lintbook-mcp: MCP server library for lintbook
//!
//! This library provides MCP (Model Context Protocol) server functionality
//! for lintbook, exposing linting tools, resources, and prompts to LLM applications.

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{debug, error, info};

pub mod mcp;
mod server;
#[cfg(test)]
mod tests;

pub use mcp::types::*;
pub use server::LintbookMcpServer;

/// Run the MCP server using stdio transport
pub async fn run_stdio_server() -> Result<()> {
    info!("Starting lintbook MCP server");

    let mut server = LintbookMcpServer::new();

    // Read from stdin and write to stdout for MCP stdio transport
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let mut reader = AsyncBufReader::new(stdin);
    let mut writer = stdout;

    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("EOF reached, shutting down");
                break;
            }
            Ok(_) => {
                if line.trim().is_empty() {
                    continue;
                }

                debug!("Received message: {}", line.trim());

                match server.handle_message(&line).await {
                    Ok(Some(response)) => {
                        debug!("Sending response: {}", response);
                        writer.write_all(response.as_bytes()).await?;
                        writer.write_all(b"\n").await?;
                        writer.flush().await?;
                    }
                    Ok(None) => {
                        debug!("No response needed");
                    }
                    Err(e) => {
                        error!("Error handling message: {}", e);
                        // Send error response if possible
                        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&line) {
                            let error_response = JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: "Internal error".to_string(),
                                    data: None,
                                }),
                            };
                            if let Ok(error_json) = serde_json::to_string(&error_response) {
                                let _ = writer.write_all(error_json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                                let _ = writer.flush().await;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    info!("lintbook MCP server shutting down");
    Ok(())
}

/// Initialize MCP server with lintbook context
pub fn init_mcp_server() -> LintbookMcpServer {
    LintbookMcpServer::new()
}
