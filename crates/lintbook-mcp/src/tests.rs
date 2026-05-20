#[cfg(test)]
mod tests {
    use crate::server::LintbookMcpServer;
    use serde_json::json;

    #[tokio::test]
    async fn test_initialize_request() {
        let mut server = LintbookMcpServer::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(response_json["jsonrpc"], "2.0");
        assert_eq!(response_json["id"], 1);
        assert!(response_json["result"].is_object());
        assert_eq!(response_json["result"]["protocolVersion"], "2024-11-05");
        assert!(response_json["result"]["serverInfo"].is_object());
        assert_eq!(
            response_json["result"]["serverInfo"]["name"],
            "lintbook-mcp"
        );
    }

    #[tokio::test]
    async fn test_initialized_notification() {
        let mut server = LintbookMcpServer::new();

        let notification = json!({
            "jsonrpc": "2.0",
            "method": "initialized"
        });

        let response = server
            .handle_message(&notification.to_string())
            .await
            .unwrap();
        assert!(response.is_none()); // Notifications don't return responses
    }

    #[tokio::test]
    async fn test_tools_list() {
        let mut server = LintbookMcpServer::new();

        // Initialize first
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });
        let _ = server
            .handle_message(&init_request.to_string())
            .await
            .unwrap();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["tools"].is_array());

        let tools = response_json["result"]["tools"].as_array().unwrap();
        assert!(tools.len() >= 2); // At least lint and init tools

        // Check lint tool
        let lint_tool = tools.iter().find(|t| t["name"] == "lint").unwrap();
        assert_eq!(lint_tool["name"], "lint");
        assert!(lint_tool["description"].is_string());
        assert!(lint_tool["inputSchema"].is_object());
    }

    #[tokio::test]
    async fn test_tool_call_lint() {
        let mut server = LintbookMcpServer::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "lint",
                "arguments": {
                    "paths": ["src/main.rs"],
                    "fix": false
                }
            }
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["content"].is_array());
        assert_eq!(response_json["result"]["isError"], false);
    }

    #[tokio::test]
    async fn test_resources_list() {
        let mut server = LintbookMcpServer::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/list"
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["resources"].is_array());
        assert!(response_json["result"]["resourceTemplates"].is_array());

        let resources = response_json["result"]["resources"].as_array().unwrap();
        assert!(resources.iter().any(|r| r["uri"] == "lintbook://config"));
    }

    #[tokio::test]
    async fn test_resource_read() {
        let mut server = LintbookMcpServer::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": {
                "uri": "lintbook://config"
            }
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["contents"].is_array());
    }

    #[tokio::test]
    async fn test_prompts_list() {
        let mut server = LintbookMcpServer::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "prompts/list"
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["prompts"].is_array());

        let prompts = response_json["result"]["prompts"].as_array().unwrap();
        assert!(prompts.iter().any(|p| p["name"] == "code-review"));
    }

    #[tokio::test]
    async fn test_prompt_get() {
        let mut server = LintbookMcpServer::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "prompts/get",
            "params": {
                "name": "code-review",
                "arguments": {
                    "file": "src/main.rs"
                }
            }
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["messages"].is_array());

        let messages = response_json["result"]["messages"].as_array().unwrap();
        assert!(!messages.is_empty());
        assert_eq!(messages[0]["role"], "user");
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let mut server = LintbookMcpServer::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "unknown/method"
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["error"].is_object());
        assert_eq!(response_json["error"]["code"], -32601);
        assert_eq!(response_json["error"]["message"], "Method not found");
    }

    #[tokio::test]
    async fn test_invalid_json() {
        let mut server = LintbookMcpServer::new();

        let result = server.handle_message("invalid json").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_params() {
        let mut server = LintbookMcpServer::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "tools/call"
            // Missing params
        });

        let response = server.handle_message(&request.to_string()).await.unwrap();
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["error"].is_object());
    }
}
