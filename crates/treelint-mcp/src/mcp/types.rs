use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// JSON-RPC 2.0 types
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

// MCP-specific types
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<HashMap<String, Value>>,
    pub resources: Option<HashMap<String, Value>>,
    pub prompts: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub sampling: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

// Tool types
#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { resource: ResourceReference },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceReference {
    pub uri: String,
}

// Resource types
#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceTemplate {
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadResourceParams {
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContent>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResourceContent {
    #[serde(rename = "text")]
    Text {
        uri: String,
        text: String,
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
    },
    #[serde(rename = "blob")]
    Blob {
        uri: String,
        blob: String,
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
    },
}

// Prompt types
#[derive(Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPromptParams {
    pub name: String,
    pub arguments: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPromptResult {
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { resource: ResourceReference },
}
