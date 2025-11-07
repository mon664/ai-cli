//! MCP 클라이언트 구현
//!
//! Model Context Protocol 클라이언트의 핵심 기능을 구현합니다.

use anyhow::{Result, anyhow};
use serde_json;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::process::{Command as TokioCommand};
use tokio::io::{AsyncBufReadExt, BufReader};
use futures::StreamExt;

use super::protocol::*;

/// MCP 클라이언트
pub struct MCPClient {
    name: String,
    version: String,
    server_url: String,
    tools: Arc<Mutex<HashMap<String, Tool>>>,
    initialized: Arc<Mutex<bool>>,
}

impl MCPClient {
    /// 새 MCP 클라이언트 생성
    pub fn new(name: String, version: String, server_url: String) -> Self {
        Self {
            name,
            version,
            server_url,
            tools: Arc::new(Mutex::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// MCP 서버에 연결 및 초기화
    pub async fn initialize(&self) -> Result<()> {
        // stdio 방식의 서버 연결
        if self.server_url.starts_with("stdio://") {
            self.initialize_stdio().await
        } else if self.server_url.starts_with("http://") || self.server_url.starts_with("https://") {
            self.initialize_http().await
        } else {
            Err(anyhow!("Unsupported server URL format: {}", self.server_url))
        }
    }

    /// stdio를 통한 서버 초기화
    async fn initialize_stdio(&self) -> Result<()> {
        // GitHub MCP 서버 예시 (실제로는 설치된 서버 실행)
        let mut child = TokioCommand::new("npx")
            .args(["@modelcontextprotocol/server-github"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start MCP server: {}", e))?;

        let mut stdin = child.stdin.take()
            .ok_or_else(|| anyhow!("Failed to get stdin handle"))?;
        let mut stdout = BufReader::new(child.stdout.take()
            .ok_or_else(|| anyhow!("Failed to get stdout handle"))?);

        // 초기화 메시지 전송
        let init_message = MCPMessage::Initialize {
            jsonrpc: MCPMessage::JSONRPC_VERSION.to_string(),
            id: MCPMessage::new_request_id(),
            params: InitializeParams {
                protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                capabilities: ClientCapabilities {
                    tools: Some(ToolsCapability {
                        list_changed: Some(true),
                    }),
                },
                client_info: ClientInfo {
                    name: self.name.clone(),
                    version: self.version.clone(),
                },
            },
        };

        let init_json = serde_json::to_string(&init_message)?;
        use tokio::io::AsyncWriteExt;
        stdin.write_all(init_json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        // 응답 수신
        let mut response_line = String::new();
        stdout.read_line(&mut response_line).await?;

        let response: MCPMessage = serde_json::from_str(&response_line.trim())
            .map_err(|e| anyhow!("Failed to parse MCP response: {}", e))?;

        match response {
            MCPMessage::InitializeResult { result, .. } => {
                tracing::info!("MCP server initialized: {} {}", result.server_info.name, result.server_info.version);

                // 도구 목록 로드
                self.load_tools_stdio(&mut stdin, &mut stdout).await?;

                *self.initialized.lock().unwrap() = true;
                Ok(())
            }
            _ => Err(anyhow!("Unexpected MCP response format"))
        }
    }

    /// HTTP를 통한 서버 초기화 (추후 구현)
    async fn initialize_http(&self) -> Result<()> {
        // TODO: HTTP 기반 MCP 서버 연동 구현
        Err(anyhow!("HTTP MCP client not yet implemented"))
    }

    /// stdio를 통해 도구 목록 로드
    async fn load_tools_stdio(
        &self,
        stdin: &mut tokio::process::ChildStdin,
        stdout: &mut BufReader<tokio::process::ChildStdout>
    ) -> Result<()> {
        let tools_request = MCPMessage::ToolsList {
            jsonrpc: MCPMessage::JSONRPC_VERSION.to_string(),
            id: MCPMessage::new_request_id(),
            params: ToolsListParams { cursor: None },
        };

        let request_json = serde_json::to_string(&tools_request)?;
        use tokio::io::AsyncWriteExt;
        stdin.write_all(request_json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        let mut response_line = String::new();
        stdout.read_line(&mut response_line).await?;

        let response: MCPMessage = serde_json::from_str(&response_line.trim())
            .map_err(|e| anyhow!("Failed to parse tools list response: {}", e))?;

        match response {
            MCPMessage::ToolsListResult { result, .. } => {
                let mut tools = self.tools.lock().unwrap();
                for tool in result.tools {
                    tracing::debug!("Loaded MCP tool: {}", tool.name);
                    tools.insert(tool.name.clone(), tool);
                }
                tracing::info!("Loaded {} MCP tools", tools.len());
                Ok(())
            }
            _ => Err(anyhow!("Unexpected tools list response format"))
        }
    }

    /// 도구 호출
    pub async fn call_tool(&self, tool_name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult> {
        if !*self.initialized.lock().unwrap() {
            return Err(anyhow!("MCP client not initialized"));
        }

        let tools = self.tools.lock().unwrap();
        if !tools.contains_key(tool_name) {
            return Err(anyhow!("Tool '{}' not found", tool_name));
        }
        drop(tools);

        let call_request = MCPMessage::ToolsCall {
            jsonrpc: MCPMessage::JSONRPC_VERSION.to_string(),
            id: MCPMessage::new_request_id(),
            params: CallToolParams {
                name: tool_name.to_string(),
                arguments,
            },
        };

        // TODO: 실제 도구 호출 구현
        // 현재는 모의 응답 반환
        Ok(CallToolResult {
            content: vec![Content::Text {
                text: format!("Tool '{}' called successfully", tool_name),
            }],
            is_error: Some(false),
        })
    }

    /// 사용 가능한 도구 목록 반환
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.lock().unwrap()
            .keys()
            .cloned()
            .collect()
    }

    /// 도구 정보 조회
    pub fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.lock().unwrap()
            .get(name)
            .cloned()
    }

    /// 클라이언트가 초기화되었는지 확인
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }
}

/// GitHub MCP 서버용 도구 정의 (미리 정의된 도구들)
pub fn create_github_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "create_pull_request".to_string(),
            description: "Create a GitHub pull request".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("title".to_string(), ToolProperty {
                        property_type: "string".to_string(),
                        description: "Pull request title".to_string(),
                        enum_values: None,
                    });
                    props.insert("body".to_string(), ToolProperty {
                        property_type: "string".to_string(),
                        description: "Pull request description".to_string(),
                        enum_values: None,
                    });
                    props.insert("head".to_string(), ToolProperty {
                        property_type: "string".to_string(),
                        description: "Head branch".to_string(),
                        enum_values: None,
                    });
                    props.insert("base".to_string(), ToolProperty {
                        property_type: "string".to_string(),
                        description: "Base branch".to_string(),
                        enum_values: None,
                    });
                    props
                },
                required: vec!["title".to_string(), "head".to_string(), "base".to_string()],
            },
        },
        Tool {
            name: "create_issue".to_string(),
            description: "Create a GitHub issue".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("title".to_string(), ToolProperty {
                        property_type: "string".to_string(),
                        description: "Issue title".to_string(),
                        enum_values: None,
                    });
                    props.insert("body".to_string(), ToolProperty {
                        property_type: "string".to_string(),
                        description: "Issue description".to_string(),
                        enum_values: None,
                    });
                    props
                },
                required: vec!["title".to_string()],
            },
        },
    ]
}