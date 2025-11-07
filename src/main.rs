use clap::{Parser, Subcommand};
use anyhow::Result;

mod cli;
mod git_utils;
mod ai_utils;
mod context;
mod security;
mod mcp;

use cli::*;
use git_utils::*;
use ai_utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 로깅 초기화
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Commit { message, all } => {
            println!("🤖 AI is generating your commit message...");

            // 모든 변경 사항 스테이징 (옵션)
            if *all {
                println!("📋 Staging all changes...");
                // TODO: git add -A 구현
            }

            // 스테이징된 diff 읽기
            let diff = get_staged_diff()?;
            println!("📝 Analyzing {} lines of changes...", diff.lines().count());

            // 커밋 메시지 생성
            let commit_message = generate_commit_message(&diff).await?;

            // 사용자 승인 및 커밋 실행
            security::prompt_and_commit(&commit_message)?;
        }
        Commands::Explain { hash, model, detailed, format } => {
            println!("🔍 AI is analyzing the changes...");

            // diff 또는 특정 커밋 분석
            let diff = if let Some(commit_hash) = hash {
                get_commit_diff(commit_hash)?
            } else {
                get_staged_diff()?
            };

            // AI 백엔드 선택
            let backend = get_ai_backend(model)?;

            // 변경 사항 설명 생성
            let explanation = generate_explanation(&diff, *detailed, &backend).await?;

            match format.as_str() {
                "json" => {
                    let output = serde_json::json!({
                        "analysis": explanation,
                        "model": backend,
                        "detailed": detailed
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "markdown" => {
                    println!("## Code Change Analysis\n\n{}", explanation);
                }
                _ => {
                    println!("\n📄 AI Analysis:\n{}", explanation);
                }
            }
        }
        Commands::Init { model, openai_key, anthropic_key, ollama_url } => {
            println!("🔧 Initializing AI CLI configuration...");

            // 환경 변수 설정 안내
            if let Some(m) = model {
                println!("✓ Default model set to: {}", m);
                std::env::set_var("AI_CLI_DEFAULT_MODEL", m);
            }

            if let Some(key) = openai_key {
                println!("✓ OpenAI API key configured");
                std::env::set_var("OPENAI_API_KEY", key);
            }

            if let Some(key) = anthropic_key {
                println!("✓ Anthropic API key configured");
                std::env::set_var("ANTHROPIC_API_KEY", key);
            }

            if ollama_url != "http://localhost:11434" {
                println!("✓ Ollama URL set to: {}", ollama_url);
                std::env::set_var("AI_CLI_OLLAMA_URL", ollama_url);
            }

            // 기본 설정 파일 생성
            let current_dir = std::env::current_dir()?;
            if let Ok(config_path) = context::create_default_project_config(&current_dir) {
                println!("✓ Created PROJECT.md at: {}", config_path.display());
            }

            if let Ok(config_path) = context::create_default_global_config() {
                println!("✓ Created global config at: {}", config_path.display());
            }

            // MCP 클라이언트 초기화 테스트
            let mcp_client = mcp::MCPClientBuilder::new("ai-cli")
                .version("0.1.0")
                .server_url("stdio://")
                .build();

            match mcp_client.initialize().await {
                Ok(()) => {
                    println!("✓ MCP client initialized successfully");
                    let tools = mcp_client.list_tools();
                    if !tools.is_empty() {
                        println!("✓ Available MCP tools: {}", tools.join(", "));
                    }
                }
                Err(e) => {
                    println!("⚠ MCP client initialization failed: {}", e);
                    println!("  This is normal if no MCP server is installed.");
                }
            }

            println!("\n🎉 AI CLI initialization complete!");
            println!("Run 'ai-cli commit' to generate your first AI-powered commit message.");
        }
        Commands::Config { verbose } => {
            println!("⚙️  AI CLI Configuration");

            if *verbose {
                // 현재 설정 상세 출력
                println!("\nEnvironment Variables:");
                if let Ok(model) = std::env::var("AI_CLI_LOCAL_MODEL") {
                    println!("  Local Model: {}", model);
                }
                if let Ok(url) = std::env::var("AI_CLI_OLLAMA_URL") {
                    println!("  Ollama URL: {}", url);
                }
                if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                    println!("  OpenAI API: ✓ configured");
                }
                if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
                    println!("  Anthropic API: ✓ configured");
                }

                // 컨텍스트 파일 상태
                let current_dir = std::env::current_dir()?;
                let project_config = current_dir.join("PROJECT.md");
                if project_config.exists() {
                    println!("  Project Context: ✓ {}", project_config.display());
                }

                let home_config = dirs::home_dir()
                    .map(|h| h.join(".ai-cli").join("CONFIG.md"));
                if let Some(ref config_path) = home_config {
                    if config_path.exists() {
                        println!("  Global Context: ✓ {}", config_path.display());
                    }
                }
            } else {
                println!("Use --verbose for detailed configuration");
                println!("Run 'ai-cli init' to configure");
            }
        }
    }

    Ok(())
}