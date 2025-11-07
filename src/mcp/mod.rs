//! MCP (Model Context Protocol) 클라이언트 모듈
//!
//! MCP는 AI 에이전트가 외부 도구와 상호작용하기 위한 개방형 표준입니다.
//! 이 모듈은 MCP 클라이언트의 기본 구조를 제공합니다.

pub mod client;
pub mod protocol;
pub mod tools;

pub use client::MCPClient;
pub use protocol::*;
pub use tools::*;

/// MCP 클라이언트 생성을 위한 빌더
pub struct MCPClientBuilder {
    name: String,
    version: String,
    server_url: Option<String>,
}

impl MCPClientBuilder {
    /// 새 MCP 클라이언트 빌더 생성
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            server_url: None,
        }
    }

    /// 클라이언트 버전 설정
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// MCP 서버 URL 설정
    pub fn server_url(mut self, url: impl Into<String>) -> Self {
        self.server_url = Some(url.into());
        self
    }

    /// MCP 클라이언트 생성
    pub fn build(self) -> MCPClient {
        MCPClient::new(
            self.name,
            self.version,
            self.server_url.unwrap_or_else(|| "stdio://".to_string()),
        )
    }
}