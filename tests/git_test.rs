//! Git 관련 기능 테스트
//!
//! Git 연동 기능을 테스트합니다.

use ai_cli::git_utils::*;
use tempfile::TempDir;
use std::fs;
use std::process::Command;

/// 테스트용 Git 리포지토리 설정
fn setup_test_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Git 리포지토리 초기화
    Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Git 설정
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    temp_dir
}

/// Git 리포지토리 열기 테스트
#[test]
fn test_open_repository() {
    let temp_dir = setup_test_repo();

    let repo = open_repository();
    assert!(repo.is_ok());
}

/// Git 리포지토리 없을 때 테스트
#[test]
fn test_open_repository_no_git() {
    let temp_dir = TempDir::new().unwrap();

    let result = open_repository();
    assert!(result.is_err());
}

/// 스테이징된 diff 읽기 테스트
#[test]
fn test_get_staged_diff() {
    let temp_dir = setup_test_repo();

    // 테스트 파일 생성
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();

    // 파일 스테이징
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to stage file");

    // 스테이징된 diff 읽기
    let diff = get_staged_diff();
    assert!(diff.is_ok());

    let diff_content = diff.unwrap();
    assert!(diff_content.contains("Hello, World!"));
    assert!(diff_content.contains("+++"));
}

/// 스테이징된 변경 없을 때 테스트
#[test]
fn test_get_staged_diff_empty() {
    let temp_dir = setup_test_repo();

    let result = get_staged_diff();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No changes found"));
}

/// 워킹 디렉토리 diff 읽기 테스트
#[test]
fn test_get_unstaged_diff() {
    let temp_dir = setup_test_repo();

    // 테스트 파일 생성 및 커밋
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();

    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to stage file");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    // 파일 수정
    fs::write(&test_file, "Hello, Rust!").unwrap();

    // 워킹 디렉토리 diff 읽기
    let diff = get_unstaged_diff();
    assert!(diff.is_ok());

    let diff_content = diff.unwrap();
    assert!(diff_content.contains("-Hello, World!"));
    assert!(diff_content.contains("+Hello, Rust!"));
}

/// 특정 커밋 diff 읽기 테스트
#[test]
fn test_get_commit_diff() {
    let temp_dir = setup_test_repo();

    // 테스트 파일 생성 및 커밋
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();

    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to stage file");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    // 커밋 해시 가져오기
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to get commit hash");

    let commit_hash = String::from_utf8_lossy(&output.stdout).trim();

    // 커밋 diff 읽기
    let diff = get_commit_diff(commit_hash);
    assert!(diff.is_ok());

    let diff_content = diff.unwrap();
    assert!(diff_content.contains("Hello, World!"));
    assert!(diff_content.contains("+++"));
}

/// 잘못된 커밋 해시 테스트
#[test]
fn test_get_commit_diff_invalid_hash() {
    let temp_dir = setup_test_repo();

    let result = get_commit_diff("invalid_hash");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid commit hash"));
}

/// 스테이징된 파일 목록 테스트
#[test]
fn test_get_staged_files() {
    let temp_dir = setup_test_repo();

    // 여러 테스트 파일 생성
    fs::write(temp_dir.path().join("file1.txt"), "Content 1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "Content 2").unwrap();

    // 파일들 스테이징
    Command::new("git")
        .args(["add", "file1.txt", "file2.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to stage files");

    // 스테이징된 파일 목록 읽기
    let files = get_staged_files().unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.contains("file1.txt")));
    assert!(files.iter().any(|f| f.contains("file2.txt")));
}

/// 현재 브랜치 이름 테스트
#[test]
fn test_get_current_branch() {
    let temp_dir = setup_test_repo();

    let branch = get_current_branch().unwrap();
    assert_eq!(branch, "main"); // Git의 기본 브랜치는 'main'
}

/// 리포지토리 상태 테스트
#[test]
fn test_get_repository_status() {
    let temp_dir = setup_test_repo();

    // 초기 상태 (변경 없음)
    let status = get_repository_status().unwrap();
    assert_eq!(status.staged, 0);
    assert_eq!(status.modified, 0);
    assert_eq!(status.untracked, 0);
    assert_eq!(status.branch, "main");

    // 파일 생성 (추적되지 않음)
    fs::write(temp_dir.path().join("new.txt"), "New content").unwrap();

    let status = get_repository_status().unwrap();
    assert_eq!(status.staged, 0);
    assert_eq!(status.modified, 0);
    assert_eq!(status.untracked, 1);

    // 파일 스테이징
    Command::new("git")
        .args(["add", "new.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to stage file");

    let status = get_repository_status().unwrap();
    assert_eq!(status.staged, 1);
    assert_eq!(status.modified, 0);
    assert_eq!(status.untracked, 0);

    // 커밋
    Command::new("git")
        .args(["commit", "-m", "Add new file"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    let status = get_repository_status().unwrap();
    assert_eq!(status.staged, 0);
    assert_eq!(status.modified, 0);
    assert_eq!(status.untracked, 0);
}

/// diff를 문자열로 변환 테스트
#[test]
fn test_diff_to_string() {
    let temp_dir = setup_test_repo();

    // 테스트 파일 생성
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();

    // 파일 스테이징
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to stage file");

    let repo = open_repository().unwrap();
    let head = repo.head().unwrap().peel_to_tree().unwrap();
    let mut diff = repo.diff_tree_to_index(Some(&head), None, None).unwrap();

    // diff를 문자열로 변환
    let diff_text = ai_cli::git_utils::diff_to_string(&diff).unwrap();
    assert!(diff_text.contains("Hello, World!"));
    assert!(diff_text.contains("+++"));
}