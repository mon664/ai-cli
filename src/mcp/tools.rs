//! MCP 도구 관리
//!
//! MCP를 통해 사용 가능한 도구들을 관리하고 래핑합니다.

use anyhow::{Result, anyhow};
use serde_json::Value;
use super::protocol::*;
use super::client::MCPClient;

/// 도구 관리자
pub struct ToolManager {
    mcp_client: MCPClient,
}

impl ToolManager {
    /// 새 도구 관리자 생성
    pub fn new(mcp_client: MCPClient) -> Self {
        Self { mcp_client }
    }

    /// GitHub Pull Request 생성
    pub async fn create_github_pull_request(
        &self,
        title: &str,
        body: Option<&str>,
        head: &str,
        base: &str,
    ) -> Result<()> {
        let mut args = serde_json::Map::new();
        args.insert("title".to_string(), Value::String(title.to_string()));
        args.insert("head".to_string(), Value::String(head.to_string()));
        args.insert("base".to_string(), Value::String(base.to_string()));

        if let Some(desc) = body {
            args.insert("body".to_string(), Value::String(desc.to_string()));
        }

        let result = self.mcp_client.call_tool("create_pull_request", Some(Value::Object(args))).await?;

        match result.is_error.unwrap_or(false) {
            false => {
                for content in result.content {
                    if let Content::Text { text } = content {
                        tracing::info!("Pull request created: {}", text);
                        return Ok(());
                    }
                }
                Ok(())
            }
            true => Err(anyhow!("Failed to create pull request")),
        }
    }

    /// GitHub Issue 생성
    pub async fn create_github_issue(
        &self,
        title: &str,
        body: Option<&str>,
    ) -> Result<()> {
        let mut args = serde_json::Map::new();
        args.insert("title".to_string(), Value::String(title.to_string()));

        if let Some(desc) = body {
            args.insert("body".to_string(), Value::String(desc.to_string()));
        }

        let result = self.mcp_client.call_tool("create_issue", Some(Value::Object(args))).await?;

        match result.is_error.unwrap_or(false) {
            false => {
                for content in result.content {
                    if let Content::Text { text } = content {
                        tracing::info!("Issue created: {}", text);
                        return Ok(());
                    }
                }
                Ok(())
            }
            true => Err(anyhow!("Failed to create issue")),
        }
    }

    /// 사용 가능한 도구 목록 반환
    pub fn list_available_tools(&self) -> Vec<String> {
        self.mcp_client.list_tools()
    }

    /// 도구 정보 반환
    pub fn get_tool_info(&self, name: &str) -> Option<Tool> {
        self.mcp_client.get_tool(name)
    }
}

/// AI CLI용 내장 도구들
#[derive(Debug, Clone)]
pub enum BuiltInTool {
    /// Git 명령어 실행
    GitCommand {
        command: String,
        args: Vec<String>,
    },
    /// 파일 읽기
    ReadFile {
        path: String,
    },
    /// 파일 쓰기
    WriteFile {
        path: String,
        content: String,
    },
    /// 디렉토리 목록
    ListDirectory {
        path: String,
    },
}

impl BuiltInTool {
    /// 내장 도구 실행
    pub async fn execute(&self) -> Result<Value> {
        match self {
            BuiltInTool::GitCommand { command, args } => {
                let output = tokio::process::Command::new("git")
                    .arg(command)
                    .args(args)
                    .output()
                    .await?;

                let result = serde_json::json!({
                    "success": output.status.success(),
                    "stdout": String::from_utf8_lossy(&output.stdout),
                    "stderr": String::from_utf8_lossy(&output.stderr),
                });

                Ok(result)
            }
            BuiltInTool::ReadFile { path } => {
                let content = tokio::fs::read_to_string(path).await?;
                Ok(serde_json::json!({
                    "content": content
                }))
            }
            BuiltInTool::WriteFile { path, content } => {
                tokio::fs::write(path, content).await?;
                Ok(serde_json::json!({
                    "success": true
                }))
            }
            BuiltInTool::ListDirectory { path } => {
                let mut entries = Vec::new();
                let mut dir = tokio::fs::read_dir(path).await?;

                while let Some(entry) = dir.next_entry().await? {
                    let metadata = entry.metadata().await?;
                    entries.push(serde_json::json!({
                        "name": entry.file_name().to_string_lossy(),
                        "is_file": metadata.is_file(),
                        "is_dir": metadata.is_dir(),
                    }));
                }

                Ok(serde_json::json!({
                    "entries": entries
                }))
            }
        }
    }
}