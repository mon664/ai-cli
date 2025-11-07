//! 통합 테스트
//!
//! AI CLI의 전체 기능을 테스트합니다.

use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;
use std::process::Stdio;

/// 기본 CLI 명령어 테스트
#[tokio::test]
async fn test_cli_help() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.arg("--help");

    cmd.assert().success()
        .stdout(predicates::str::contains("AI-powered CLI Git Assistant"))
        .stdout(predicates::str::contains("commit"))
        .stdout(predicates::str::contains("explain"))
        .stdout(predicates::str::contains("init"))
        .stdout(predicates::str::contains("config"));
}

/// commit 명령어 도움말 테스트
#[tokio::test]
async fn test_commit_help() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.args(["commit", "--help"]);

    cmd.assert().success()
        .stdout(predicates::str::contains("Generate a conventional commit message"))
        .stdout(predicates::str::contains("--message"))
        .stdout(predicates::str::contains("--all"))
        .stdout(predicates::str::contains("--model"));
}

/// explain 명령어 도움말 테스트
#[tokio::test]
async fn test_explain_help() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.args(["explain", "--help"]);

    cmd.assert().success()
        .stdout(predicates::str::contains("Explain the staged changes"))
        .stdout(predicates::str::contains("--hash"))
        .stdout(predicates::str::contains("--detailed"))
        .stdout(predicates::str::contains("--format"));
}

/// init 명령어 테스트
#[tokio::test]
async fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("PROJECT.md");

    // init 명령어 실행
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["init"]);

    cmd.assert().success()
        .stdout(predicates::str::contains("Initializing AI CLI configuration"))
        .stdout(predicates::str::contains("initialization complete"));

    // PROJECT.md 파일 생성 확인
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("Project Configuration"));
    assert!(content.contains("Architecture"));
}

/// config 명령어 테스트
#[tokio::test]
async fn test_config_command() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.args(["config"]);

    cmd.assert().success()
        .stdout(predicates::str::contains("AI CLI Configuration"));
}

/// config 명령어 상세 모드 테스트
#[tokio::test]
async fn test_config_verbose() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.args(["config", "--verbose"]);

    cmd.assert().success()
        .stdout(predicates::str::contains("Environment Variables"));
}

/// Git 리포지토리 없을 때 commit 시도 테스트
#[tokio::test]
async fn test_commit_without_git_repo() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["commit"]);

    cmd.assert().failure()
        .stderr(predicates::str::contains("Git repository"));
}

/// 프롬프트 엔지니어링 테스트
#[test]
fn test_commit_prompt_generation() {
    let diff = r#"+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
"#;

    let prompt = ai_cli::ai_utils::create_commit_prompt(diff, None);

    assert!(prompt.contains("Conventional Commits"));
    assert!(prompt.contains(diff));
    assert!(prompt.contains("feat"));
    assert!(prompt.contains("fix"));
}

/// Conventional Commit 정제 테스트
#[test]
fn test_conventional_commit_refinement() {
    let test_cases = vec![
        // 유효한 형식
        ("feat: add new feature", "feat: add new feature"),
        // 불필요한 접두사 제거
        ("Commit message: feat: add new feature", "feat: add new feature"),
        // 따옴표 제거
        ("\"feat: add new feature\"", "feat: add new feature"),
        // 코드 블록 제거
        ("```\nfeat: add new feature\n```", "feat: add new feature"),
        // 타입 없는 메시지에 타입 추가
        ("add new functionality", "feat: add new functionality"),
        ("fix the bug", "fix: fix the bug"),
        ("update documentation", "docs: update documentation"),
        ("run tests", "test: run tests"),
    ];

    for (input, expected) in test_cases {
        let result = ai_cli::ai_utils::refine_conventional_commit(input);
        assert_eq!(result, expected, "Input: {}", input);
    }
}

/// 컨텍스트 엔진 테스트
#[test]
fn test_context_engine() {
    let engine = ai_cli::context::ContextEngine::new();
    assert!(engine.project_root.is_none());
    assert!(engine.contexts.is_empty());
}

/// 파일 참조 해석 테스트
#[test]
fn test_file_reference_resolution() {
    let engine = ai_cli::context::ContextEngine::new();
    let temp_dir = tempfile::tempdir().unwrap();

    // 상대 경로 참조
    let resolved = engine.resolve_file_reference("@src/main.rs", temp_dir.path()).unwrap();
    assert_eq!(resolved, temp_dir.path().join("src/main.rs"));
}

/// 보안 매니저 테스트
#[test]
fn test_security_manager() {
    let manager = ai_cli::security::SecurityManager::new();
    assert_eq!(manager.current_level, ai_cli::security::SecurityLevel::Untrusted);
    assert!(manager.trusted_folders.is_empty());
}

/// 위험한 명령어 감지 테스트
#[test]
fn test_dangerous_command_detection() {
    assert!(ai_cli::security::SecurityManager::is_dangerous_command("rm -rf /"));
    assert!(ai_cli::security::SecurityManager::is_dangerous_command("sudo rm -rf /home"));
    assert!(!ai_cli::security::SecurityManager::is_dangerous_command("git status"));
    assert!(!ai_cli::security::SecurityManager::is_dangerous_command("ls -la"));
}

/// 경고 필요한 명령어 테스트
#[test]
fn test_warning_command_detection() {
    assert!(ai_cli::security::SecurityManager::needs_warning("rm file.txt"));
    assert!(ai_cli::security::SecurityManager::needs_warning("git reset --hard"));
    assert!(!ai_cli::security::SecurityManager::needs_warning("git status"));
    assert!(!ai_cli::security::SecurityManager::needs_warning("echo hello"));
}

/// MCP 클라이언트 빌더 테스트
#[test]
fn test_mcp_client_builder() {
    let client = ai_cli::mcp::MCPClientBuilder::new("test-client")
        .version("1.0.0")
        .server_url("stdio://")
        .build();

    assert!(!client.is_initialized());
}

/// AI 백엔드 선택 테스트
#[test]
fn test_ai_backend_selection() {
    // 로컬 백엔드
    let backend = ai_cli::ai_utils::get_ai_backend("local").unwrap();
    match backend {
        ai_cli::ai_utils::AIBackend::Local { model, .. } => {
            assert!(!model.is_empty());
        }
        _ => panic!("Expected Local backend"),
    }

    // 잘못된 백엔드
    let result = ai_cli::ai_utils::get_ai_backend("invalid");
    assert!(result.is_err());
}

/// 설명 프롬프트 생성 테스트
#[test]
fn test_explain_prompt_generation() {
    let diff = r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
-    println!("Hello");
+    println!("Hello, world!");
 }
"#;

    // 간단한 설명 프롬프트
    let simple_prompt = ai_cli::ai_utils::create_explain_prompt(diff, false);
    assert!(simple_prompt.contains("software engineer"));
    assert!(simple_prompt.contains("2-3 paragraphs"));
    assert!(simple_prompt.contains(diff));

    // 상세한 설명 프롬프트
    let detailed_prompt = ai_cli::ai_utils::create_explain_prompt(diff, true);
    assert!(detailed_prompt.contains("comprehensive explanation"));
    assert!(detailed_prompt.contains("High-level summary"));
    assert!(detailed_prompt.contains("Technical details"));
    assert!(detailed_prompt.contains(diff));
}