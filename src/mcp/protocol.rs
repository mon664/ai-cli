//! MCP 프로토콜 정의
//!
//! MCP (Model Context Protocol)의 핵심 데이터 구조와 메시지 형식을 정의합니다.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// MCP 요청 ID
pub type RequestId = String;

/// MCP JSON-RPC 메시지 기반
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum MCPMessage {
    /// 클라이언트 초기화
    #[serde(rename = "initialize")]
    Initialize {
        jsonrpc: String,
        id: RequestId,
        params: InitializeParams,
    },
    /// 초기화 응답
    #[serde(rename = "initialize/result")]
    InitializeResult {
        jsonrpc: String,
        id: RequestId,
        result: InitializeResult,
    },
    /// 도구 목록 요청
    #[serde(rename = "tools/list")]
    ToolsList {
        jsonrpc: String,
        id: RequestId,
        params: ToolsListParams,
    },
    /// 도구 목록 응답
    #[serde(rename = "tools/list/result")]
    ToolsListResult {
        jsonrpc: String,
        id: RequestId,
        result: ToolsListResult,
    },
    /// 도구 호출
    #[serde(rename = "tools/call")]
    ToolsCall {
        jsonrpc: String,
        id: RequestId,
        params: CallToolParams,
    },
    /// 도구 호출 결과
    #[serde(rename = "tools/call/result")]
    ToolsCallResult {
        jsonrpc: String,
        id: RequestId,
        result: CallToolResult,
    },
}

/// 클라이언트 초기화 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

/// 클라이언트 기능
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub tools: Option<ToolsCapability>,
}

/// 도구 기능
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

/// 클라이언트 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// 초기화 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

/// 서버 기능
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
}

/// 리소스 기능
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

/// 서버 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// 도구 목록 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListParams {
    pub cursor: Option<String>,
}

/// 도구 목록 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<Tool>,
}

/// 도구 정의
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
}

/// 도구 입력 스키마
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: HashMap<String, ToolProperty>,
    pub required: Vec<String>,
}

/// 도구 속성
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProperty {
    #[serde(rename = "type")]
    pub property_type: String,
    pub description: String,
    pub enum_values: Option<Vec<String>>,
}

/// 도구 호출 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}

/// 도구 호출 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    pub is_error: Option<bool>,
}

/// 콘텐츠
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text {
        text: String,
    },
    #[serde(rename = "image")]
    Image {
        data: String,
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource {
        uri: String,
        mime_type: Option<String>,
        text: Option<String>,
        blob: Option<String>,
    },
}

impl MCPMessage {
    /// 새 JSON-RPC 요청 ID 생성
    pub fn new_request_id() -> RequestId {
        Uuid::new_v4().to_string()
    }

    /// JSON-RPC 2.0 기본 설정
    pub const JSONRPC_VERSION: &'static str = "2.0";
}

impl Default for InitializeParams {
    fn default() -> Self {
        Self {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(true),
                }),
            },
            client_info: ClientInfo {
                name: "ai-cli".to_string(),
                version: "0.1.0".to_string(),
            },
        }
    }
}

/// MCP 프로토콜 버전
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";