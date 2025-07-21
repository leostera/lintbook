use crate::mcp::types::*;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

pub struct TreelintMcpServer {
    initialized: bool,
    #[allow(dead_code)]
    project_root: PathBuf,
}

impl TreelintMcpServer {
    pub fn new() -> Self {
        Self {
            initialized: false,
            project_root: std::env::current_dir().unwrap_or_default(),
        }
    }

    pub async fn handle_message(&mut self, message: &str) -> Result<Option<String>> {
        let value: Value = serde_json::from_str(message)?;

        // Check if it's a notification (no id field) or request
        if value.get("id").is_none() {
            // It's a notification
            if let Ok(notification) = serde_json::from_value::<JsonRpcNotification>(value.clone()) {
                self.handle_notification(notification).await?;
                return Ok(None);
            }
        }

        // It's a request
        let request: JsonRpcRequest = serde_json::from_value(value)?;
        let response = self.handle_request(request).await?;
        let response_json = serde_json::to_string(&response)?;
        Ok(Some(response_json))
    }

    async fn handle_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling request: {}", request.method);

        let result = match request.method.as_str() {
            "initialize" => match self.handle_initialize(request.params).await {
                Ok(result) => result,
                Err(e) => {
                    return Ok(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: e.to_string(),
                            data: None,
                        }),
                    })
                }
            },
            "tools/list" => self.handle_tools_list().await?,
            "tools/call" => match self.handle_tool_call(request.params).await {
                Ok(result) => result,
                Err(e) => {
                    return Ok(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: e.to_string(),
                            data: None,
                        }),
                    })
                }
            },
            "resources/list" => self.handle_resources_list().await?,
            "resources/read" => match self.handle_resource_read(request.params).await {
                Ok(result) => result,
                Err(e) => {
                    return Ok(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: e.to_string(),
                            data: None,
                        }),
                    })
                }
            },
            "prompts/list" => self.handle_prompts_list().await?,
            "prompts/get" => match self.handle_prompt_get(request.params).await {
                Ok(result) => result,
                Err(e) => {
                    return Ok(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: e.to_string(),
                            data: None,
                        }),
                    })
                }
            },
            _ => {
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                });
            }
        };

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result),
            error: None,
        })
    }

    async fn handle_notification(&mut self, notification: JsonRpcNotification) -> Result<()> {
        debug!("Handling notification: {}", notification.method);

        match notification.method.as_str() {
            "initialized" => {
                info!("Client initialized");
                self.initialized = true;
            }
            "notifications/cancelled" => {
                warn!("Request cancelled");
            }
            _ => {
                debug!("Unknown notification: {}", notification.method);
            }
        }

        Ok(())
    }

    async fn handle_initialize(&mut self, params: Option<Value>) -> Result<Value> {
        let _params: InitializeParams =
            serde_json::from_value(params.ok_or_else(|| anyhow!("Missing initialize params"))?)?;

        let capabilities = ServerCapabilities {
            tools: Some(HashMap::new()),
            resources: Some(HashMap::new()),
            prompts: Some(HashMap::new()),
        };

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities,
            server_info: ServerInfo {
                name: "treelint-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_tools_list(&self) -> Result<Value> {
        let tools = vec![
            Tool {
                name: "lint".to_string(),
                description: "Run treelint on specified files or directories".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "paths": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Files or directories to lint"
                        },
                        "fix": {
                            "type": "boolean",
                            "description": "Automatically fix issues where possible",
                            "default": false
                        }
                    }
                }),
            },
            Tool {
                name: "init".to_string(),
                description: "Initialize treelint configuration in the current directory"
                    .to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "force": {
                            "type": "boolean",
                            "description": "Overwrite existing configuration",
                            "default": false
                        }
                    }
                }),
            },
        ];

        Ok(json!({ "tools": tools }))
    }

    async fn handle_tool_call(&self, params: Option<Value>) -> Result<Value> {
        let params: ToolCallParams =
            serde_json::from_value(params.ok_or_else(|| anyhow!("Missing tool call params"))?)?;

        let result = match params.name.as_str() {
            "lint" => self.tool_lint(params.arguments).await?,
            "init" => self.tool_init(params.arguments).await?,
            _ => {
                return Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Unknown tool: {}", params.name)
                    }],
                    "isError": true
                }));
            }
        };

        Ok(serde_json::to_value(result)?)
    }

    async fn tool_lint(&self, _args: Option<Value>) -> Result<ToolResult> {
        // TODO: Implement actual linting using treelint-scanner
        let content = vec![ToolContent::Text {
            text: "Linting functionality not yet implemented".to_string(),
        }];

        Ok(ToolResult {
            content,
            is_error: Some(false),
        })
    }

    async fn tool_init(&self, _args: Option<Value>) -> Result<ToolResult> {
        // TODO: Implement actual init using treelint-config
        let content = vec![ToolContent::Text {
            text: "Init functionality not yet implemented".to_string(),
        }];

        Ok(ToolResult {
            content,
            is_error: Some(false),
        })
    }

    async fn handle_resources_list(&self) -> Result<Value> {
        let resources = vec![Resource {
            uri: "treelint://config".to_string(),
            name: "treelint.toml".to_string(),
            description: Some("Current treelint configuration".to_string()),
            mime_type: Some("application/toml".to_string()),
        }];

        let templates = vec![ResourceTemplate {
            uri_template: "treelint://file/{path}".to_string(),
            name: "Source file".to_string(),
            description: Some("Read a source file for linting".to_string()),
            mime_type: Some("text/plain".to_string()),
        }];

        Ok(json!({
            "resources": resources,
            "resourceTemplates": templates
        }))
    }

    async fn handle_resource_read(&self, params: Option<Value>) -> Result<Value> {
        let params: ReadResourceParams =
            serde_json::from_value(params.ok_or_else(|| anyhow!("Missing resource read params"))?)?;

        // TODO: Implement actual resource reading
        let contents = vec![ResourceContent::Text {
            uri: params.uri.clone(),
            text: "Resource reading not yet implemented".to_string(),
            mime_type: Some("text/plain".to_string()),
        }];

        Ok(json!({ "contents": contents }))
    }

    async fn handle_prompts_list(&self) -> Result<Value> {
        let prompts = vec![Prompt {
            name: "code-review".to_string(),
            description: Some("Review code for common issues and style violations".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "file".to_string(),
                description: Some("Path to the file to review".to_string()),
                required: Some(true),
            }]),
        }];

        Ok(json!({ "prompts": prompts }))
    }

    async fn handle_prompt_get(&self, params: Option<Value>) -> Result<Value> {
        let params: GetPromptParams =
            serde_json::from_value(params.ok_or_else(|| anyhow!("Missing prompt get params"))?)?;

        match params.name.as_str() {
            "code-review" => {
                let file = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("file"))
                    .ok_or_else(|| anyhow!("Missing file argument"))?;

                let messages = vec![PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text {
                        text: format!(
                            "Please review the following file for code quality issues: {}",
                            file
                        ),
                    },
                }];

                Ok(json!({
                    "description": "Code review prompt",
                    "messages": messages
                }))
            }
            _ => Err(anyhow!("Unknown prompt: {}", params.name)),
        }
    }
}
