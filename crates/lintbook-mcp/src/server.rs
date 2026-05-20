use crate::mcp::types::*;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

pub struct LintbookMcpServer {
    initialized: bool,
    #[allow(dead_code)]
    project_root: PathBuf,
}

impl LintbookMcpServer {
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
                name: "lintbook-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_tools_list(&self) -> Result<Value> {
        let tools = vec![
            Tool {
                name: "lint".to_string(),
                description: "Run lintbook on specified files or directories".to_string(),
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
                name: "setup".to_string(),
                description: "Explain how to initialize lintbook in the current project"
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
            Tool {
                name: "compile".to_string(),
                description: "Compile .lintbook/rules Markdown plus .lintbook/gen Datafox queries into generated artifacts"
                    .to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "rule_authoring_guide".to_string(),
                description: "Return the lintbook custom rule authoring contract".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
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
            "setup" => self.tool_setup(params.arguments).await?,
            "compile" => self.tool_compile(params.arguments).await?,
            "rule_authoring_guide" => self.tool_rule_authoring_guide(params.arguments).await?,
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
        // TODO: Implement actual linting using lintbook-scanner
        let content = vec![ToolContent::Text {
            text: "Linting functionality not yet implemented".to_string(),
        }];

        Ok(ToolResult {
            content,
            is_error: Some(false),
        })
    }

    async fn tool_setup(&self, _args: Option<Value>) -> Result<ToolResult> {
        let content = vec![ToolContent::Text {
            text: "Run `lintbook setup` in the project. lintbook never edits MCP configuration; add { \"command\": \"lintbook\", \"args\": [\"mcp\"] } to your MCP client manually.".to_string(),
        }];

        Ok(ToolResult {
            content,
            is_error: Some(false),
        })
    }

    async fn tool_compile(&self, _args: Option<Value>) -> Result<ToolResult> {
        let report = lintbook_rules::compile_project(&self.project_root)?;
        let content = vec![ToolContent::Text {
            text: format!(
                "Compiled {} rule(s), skipped {} incomplete rule(s).",
                report.compiled.len(),
                report.skipped_incomplete.len()
            ),
        }];

        Ok(ToolResult {
            content,
            is_error: Some(false),
        })
    }

    async fn tool_rule_authoring_guide(&self, _args: Option<Value>) -> Result<ToolResult> {
        let content = vec![ToolContent::Text {
            text: lintbook_rules::RULE_AUTHORING_GUIDE.to_string(),
        }];

        Ok(ToolResult {
            content,
            is_error: Some(false),
        })
    }

    async fn handle_resources_list(&self) -> Result<Value> {
        let resources = vec![Resource {
            uri: "lintbook://config".to_string(),
            name: "lintbook.toml".to_string(),
            description: Some("Current lintbook configuration".to_string()),
            mime_type: Some("application/toml".to_string()),
        }];

        let templates = vec![ResourceTemplate {
            uri_template: "lintbook://file/{path}".to_string(),
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

        let text = match params.uri.as_str() {
            "lintbook://config" => {
                let path = self.project_root.join("lintbook.toml");
                std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| "No lintbook.toml found. Run `lintbook setup`.".to_string())
            }
            _ => "Resource reading not implemented for this URI".to_string(),
        };

        let contents = vec![ResourceContent::Text {
            uri: params.uri.clone(),
            text,
            mime_type: Some("text/plain".to_string()),
        }];

        Ok(json!({ "contents": contents }))
    }

    async fn handle_prompts_list(&self) -> Result<Value> {
        let prompts = vec![
            Prompt {
                name: "write-rule".to_string(),
                description: Some("Create a lintbook Markdown + Datafox rule pair".to_string()),
                arguments: Some(vec![PromptArgument {
                    name: "description".to_string(),
                    description: Some(
                        "Detailed natural-language description of the lint".to_string(),
                    ),
                    required: Some(true),
                }]),
            },
            Prompt {
                name: "code-review".to_string(),
                description: Some("Review code for common issues and style violations".to_string()),
                arguments: Some(vec![PromptArgument {
                    name: "file".to_string(),
                    description: Some("Path to the file to review".to_string()),
                    required: Some(true),
                }]),
            },
        ];

        Ok(json!({ "prompts": prompts }))
    }

    async fn handle_prompt_get(&self, params: Option<Value>) -> Result<Value> {
        let params: GetPromptParams =
            serde_json::from_value(params.ok_or_else(|| anyhow!("Missing prompt get params"))?)?;

        match params.name.as_str() {
            "write-rule" => {
                let description = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("description"))
                    .ok_or_else(|| anyhow!("Missing description argument"))?;

                let messages = vec![PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text {
                        text: format!(
                            "Create a lintbook rule for this check: {description}\n\n{}",
                            lintbook_rules::RULE_AUTHORING_GUIDE
                        ),
                    },
                }];

                Ok(json!({
                    "description": "Lintbook rule authoring prompt",
                    "messages": messages
                }))
            }
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
