use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;

/// 컨텍스트 엔진 모듈
/// 다층적 컨텍스트 시스템 (전역/프로젝트/디렉토리) 구현

/// 컨텍스트 타입
#[derive(Debug, Clone)]
pub enum ContextType {
    Global,    // ~/.ai-cli/CONFIG.md
    Project,   // /path/to/project/PROJECT.md
    Directory, // /path/to/project/subdir/PROJECT.md
}

/// 컨텍스트 정보 구조체
#[derive(Debug)]
pub struct Context {
    pub path: PathBuf,
    pub content: String,
    pub context_type: ContextType,
}

/// 컨텍스트 엔진
pub struct ContextEngine {
    project_root: Option<PathBuf>,
    contexts: Vec<Context>,
}

impl ContextEngine {
    /// 새 컨텍스트 엔진 생성
    pub fn new() -> Self {
        Self {
            project_root: None,
            contexts: Vec::new(),
        }
    }

    /// 현재 디렉토리에서 프로젝트 루트 찾기
    pub fn find_project_root(&mut self, start_dir: &Path) -> Result<PathBuf> {
        let mut current_dir = start_dir.to_path_buf();

        loop {
            // .git 디렉토리가 있는지 확인
            if current_dir.join(".git").exists() {
                self.project_root = Some(current_dir.clone());
                return Ok(current_dir);
            }

            // 상위 디렉토리로 이동
            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break,
            }
        }

        Err(anyhow!("Could not find Git repository root"))
    }

    /// 모든 관련 컨텍스트 로드
    pub fn load_contexts(&mut self, current_dir: &Path) -> Result<()> {
        self.contexts.clear();

        // 프로젝트 루트 찾기
        let project_root = self.find_project_root(current_dir)?;

        // 전역 컨텍스트 로드
        if let Some(global_context) = self.load_global_context()? {
            self.contexts.push(global_context);
        }

        // 프로젝트 컨텍스트 로드
        if let Some(project_context) = self.load_project_context(&project_root)? {
            self.contexts.push(project_context);
        }

        // 현재 디렉토리 컨텍스트 로드
        if current_dir != project_root {
            if let Some(dir_context) = self.load_directory_context(current_dir)? {
                self.contexts.push(dir_context);
            }
        }

        Ok(())
    }

    /// 전역 컨텍스트 로드
    fn load_global_context(&self) -> Result<Option<Context>> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        let config_path = home_dir.join(".ai-cli").join("CONFIG.md");

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            Ok(Some(Context {
                path: config_path,
                content,
                context_type: ContextType::Global,
            }))
        } else {
            Ok(None)
        }
    }

    /// 프로젝트 컨텍스트 로드
    fn load_project_context(&self, project_root: &Path) -> Result<Option<Context>> {
        let project_config_path = project_root.join("PROJECT.md");

        if project_config_path.exists() {
            let content = fs::read_to_string(&project_config_path)?;
            Ok(Some(Context {
                path: project_config_path,
                content,
                context_type: ContextType::Project,
            }))
        } else {
            Ok(None)
        }
    }

    /// 디렉토리 컨텍스트 로드
    fn load_directory_context(&self, dir: &Path) -> Result<Option<Context>> {
        let dir_config_path = dir.join("PROJECT.md");

        if dir_config_path.exists() {
            let content = fs::read_to_string(&dir_config_path)?;
            Ok(Some(Context {
                path: dir_config_path,
                content,
                context_type: ContextType::Directory,
            }))
        } else {
            Ok(None)
        }
    }

    /// 결합된 컨텍스트 내용 가져오기
    pub fn get_combined_context(&self) -> String {
        self.contexts
            .iter()
            .map(|ctx| {
                format!(
                    "--- Context from {} ({}) ---\n{}\n\n",
                    ctx.path.display(),
                    match ctx.context_type {
                        ContextType::Global => "Global",
                        ContextType::Project => "Project",
                        ContextType::Directory => "Directory",
                    },
                    ctx.content
                )
            })
            .collect()
    }

    /// 파일 참조 (@filename) 해석
    pub fn resolve_file_reference(&self, reference: &str, current_dir: &Path) -> Result<PathBuf> {
        // @ 기호 제거
        let file_path = reference.trim_start_matches('@');

        let resolved_path = if file_path.starts_with('/') {
            // 절대 경로
            PathBuf::from(file_path)
        } else if let Some(project_root) = &self.project_root {
            // 프로젝트 루트 기준 상대 경로
            project_root.join(file_path)
        } else {
            // 현재 디렉토리 기준 상대 경로
            current_dir.join(file_path)
        };

        Ok(resolved_path)
    }

    /// 셸 히스토리 읽기
    pub fn read_shell_history(&self) -> Result<Vec<String>> {
        let mut history = Vec::new();

        // 가능한 셸 히스토리 파일들
        let history_files = [
            dirs::home_dir().map(|h| h.join(".zsh_history")),
            dirs::home_dir().map(|h| h.join(".bash_history")),
            dirs::home_dir().map(|h| h.join(".config/fish/fish_history")),
        ];

        for history_file in history_files.iter().flatten() {
            if history_file.exists() {
                if let Ok(content) = fs::read_to_string(history_file) {
                    // 최근 50개 명령어만 추출
                    let recent_commands: Vec<String> = content
                        .lines()
                        .rev()
                        .take(50)
                        .filter_map(|line| {
                            // zsh 형식: ': 1234567890:0;command'
                            if line.starts_with(':') {
                                line.split(';').nth(1)
                            } else {
                                Some(line)
                            }
                        })
                        .filter(|cmd| !cmd.trim().is_empty())
                        .collect();

                    history.extend(recent_commands);
                }
            }
        }

        Ok(history)
    }

    /// 관련성 있는 컨텍스트 조각 찾기 (간단한 키워드 매칭)
    pub fn find_relevant_context(&self, query: &str) -> Vec<String> {
        let query_keywords: Vec<String> = query
            .split_whitespace()
            .map(|word| word.to_lowercase())
            .collect();

        let mut relevant_chunks = Vec::new();

        for context in &self.contexts {
            // 단락으로 나누기
            let paragraphs: Vec<&str> = context.content.split("\n\n").collect();

            for paragraph in paragraphs {
                let paragraph_lower = paragraph.to_lowercase();

                // 키워드 매칭
                let match_count = query_keywords
                    .iter()
                    .filter(|keyword| paragraph_lower.contains(keyword))
                    .count();

                if match_count > 0 {
                    relevant_chunks.push(format!(
                        "[Relevance: {}/{}] {}",
                        match_count,
                        query_keywords.len(),
                        paragraph
                    ));
                }
            }
        }

        // 관련성 순으로 정렬
        relevant_chunks.sort_by(|a, b| {
            let a_relevance = Self::extract_relevance(a);
            let b_relevance = Self::extract_relevance(b);
            b_relevance.cmp(&a_relevance)
        });

        relevant_chunks
    }

    /// 관련성 점수 추출
    fn extract_relevance(chunk: &str) -> usize {
        if let Some(start) = chunk.find("[Relevance: ") {
            if let Some(end) = chunk[start..].find(']') {
                let relevance_str = &chunk[start + 13..start + end];
                return relevance_str.split('/').next().unwrap_or("0").parse().unwrap_or(0);
            }
        }
        0
    }
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 기본 전역 컨텍스트 파일 생성
pub fn create_default_global_config() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let ai_cli_dir = home_dir.join(".ai-cli");

    // 디렉토리 생성
    fs::create_dir_all(&ai_cli_dir)?;

    let config_path = ai_cli_dir.join("CONFIG.md");

    if !config_path.exists() {
        let default_config = r#"# AI CLI Global Configuration

## Developer Preferences
- I prefer conventional commits with clear descriptions
- Focus on user-facing changes in commit messages
- Include breaking changes warnings when applicable

## AI Model Preferences
- Default to local models for privacy
- Fall back to OpenAI GPT-4o-mini for complex reasoning
- Keep responses concise and actionable

## Coding Standards
- Follow Rust best practices
- Include proper error handling
- Write tests for new functionality
"#;

        fs::write(&config_path, default_config)?;
    }

    Ok(config_path)
}

/// 기본 프로젝트 컨텍스트 파일 생성
pub fn create_default_project_config(project_root: &Path) -> Result<PathBuf> {
    let config_path = project_root.join("PROJECT.md");

    if !config_path.exists() {
        let default_config = format!(
            r#"# Project Configuration: {}

## Project Overview
This is an AI-powered CLI tool for Git workflow automation.

## Architecture
- Language: Rust
- CLI Framework: clap
- AI Backends: Local (Ollama), OpenAI, Anthropic
- Git Operations: git2-rs

## Development Guidelines
- Follow conventional commits specification
- Maintain backward compatibility
- Prioritize user privacy and security
- Write comprehensive tests
- Document all public APIs

## Testing Strategy
- Unit tests for core functionality
- Integration tests for Git operations
- Mock AI responses for testing

## Deployment
- Single binary distribution via cargo install
- Cross-platform support (Windows, macOS, Linux)
"#,
            project_root.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("ai-cli")
        );

        fs::write(&config_path, default_config)?;
    }

    Ok(config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_context_engine_creation() {
        let engine = ContextEngine::new();
        assert!(engine.project_root.is_none());
        assert!(engine.contexts.is_empty());
    }

    #[test]
    fn test_file_reference_resolution() {
        let engine = ContextEngine::new();
        let temp_dir = TempDir::new().unwrap();
        let current_dir = temp_dir.path();

        // 상대 경로 참조
        let resolved = engine.resolve_file_reference("@src/main.rs", current_dir).unwrap();
        assert_eq!(resolved, current_dir.join("src/main.rs"));
    }

    #[test]
    fn test_relevance_extraction() {
        let chunk = "[Relevance: 2/3] This is a relevant paragraph";
        let relevance = ContextEngine::extract_relevance(chunk);
        assert_eq!(relevance, 2);
    }
}