use git2::{Repository, Diff, DiffFormat, Tree, Oid};
use anyhow::{Result, anyhow};
use std::path::Path;

/// Git 리포지토리 유틸리티 모듈
/// git2-rs를 사용하여 Git 작업을 안전하게 처리

/// 현재 디렉토리에서 Git 리포지토리 열기
pub fn open_repository() -> Result<Repository> {
    Repository::open_from_env()
        .or_else(|_| Repository::open("."))
        .map_err(|_| anyhow!("Failed to open Git repository in current directory"))
}

/// 스테이징된 변경 사항 가져오기 (git diff --cached)
pub fn get_staged_diff() -> Result<String> {
    let repo = open_repository()?;

    let head = repo.head()?.peel_to_tree()
        .map_err(|_| anyhow!("Could not find HEAD tree. Is the repository empty or no commits exist?"))?;

    // 스테이징된 변경 사항(index)과 HEAD 트리 간의 diff 생성
    let mut diff = repo.diff_tree_to_index(
        Some(&head),
        None, // None은 현재 인덱스(스테이징 영역)를 의미
        None,
    )?;

    diff_to_string(&diff)
}

/// 워킹 디렉토리의 변경 사항 가져오기 (git diff)
pub fn get_unstaged_diff() -> Result<String> {
    let repo = open_repository()?;

    let head = repo.head()?.peel_to_tree()
        .map_err(|_| anyhow!("Could not find HEAD tree."))?;

    // HEAD와 워킹 디렉토리 간의 diff 생성
    let mut diff = repo.diff_tree_to_workdir(
        Some(&head),
        None,
    )?;

    diff_to_string(&diff)
}

/// 특정 커밋의 변경 사항 가져오기
pub fn get_commit_diff(commit_hash: &str) -> Result<String> {
    let repo = open_repository()?;

    let oid = Oid::from_str(commit_hash)
        .map_err(|_| anyhow!("Invalid commit hash: {}", commit_hash))?;

    let commit = repo.find_commit(oid)?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None // 첫 커밋인 경우
    };

    let commit_tree = commit.tree()?;

    let mut diff = repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&commit_tree),
        None,
    )?;

    diff_to_string(&diff)
}

/// Diff 객체를 문자열로 변환
fn diff_to_string(diff: &Diff) -> Result<String> {
    let mut diff_text = String::new();

    diff.print(DiffFormat::Patch, |_, _, line| {
        diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
        true // 계속 진행
    })?;

    if diff_text.is_empty() {
        anyhow::bail!("No changes found to analyze.");
    }

    Ok(diff_text)
}

/// 스테이징된 파일 목록 가져오기
pub fn get_staged_files() -> Result<Vec<String>> {
    let repo = open_repository()?;
    let mut files = Vec::new();

    let head = repo.head()?.peel_to_tree()
        .map_err(|_| anyhow!("Could not find HEAD tree."))?;

    let mut diff = repo.diff_tree_to_index(
        Some(&head),
        None,
        None,
    )?;

    diff.foreach(
        &mut |delta, _| {
            if let Some(file) = delta.new_file().path() {
                files.push(file.to_string_lossy().to_string());
            }
            true
        },
        None,
        None,
        None,
    )?;

    Ok(files)
}

/// 현재 브랜치 이름 가져오기
pub fn get_current_branch() -> Result<String> {
    let repo = open_repository()?;

    let head = repo.head()?;
    let branch_name = head.shorthand()
        .ok_or_else(|| anyhow!("Not on any branch (detached HEAD)"))?;

    Ok(branch_name.to_string())
}

/// 리포지토리 상태 확인
pub fn get_repository_status() -> Result<GitStatus> {
    let repo = open_repository()?;
    let mut statuses = repo.statuses(None)?;

    let mut staged = 0;
    let mut modified = 0;
    let mut untracked = 0;

    for status in statuses.iter() {
        let status_bits = status.status();

        if status_bits.contains(git2::Status::INDEX_NEW) ||
           status_bits.contains(git2::Status::INDEX_MODIFIED) ||
           status_bits.contains(git2::Status::INDEX_DELETED) ||
           status_bits.contains(git2::Status::INDEX_RENAMED) ||
           status_bits.contains(git2::Status::INDEX_TYPECHANGE) {
            staged += 1;
        }

        if status_bits.contains(git2::Status::WT_NEW) {
            untracked += 1;
        } else if status_bits.contains(git2::Status::WT_MODIFIED) ||
                  status_bits.contains(git2::Status::WT_DELETED) ||
                  status_bits.contains(git2::Status::WT_RENAMED) ||
                  status_bits.contains(git2::Status::WT_TYPECHANGE) {
            modified += 1;
        }
    }

    Ok(GitStatus {
        staged,
        modified,
        untracked,
        branch: get_current_branch()?,
    })
}

/// Git 상태 정보 구조체
#[derive(Debug)]
pub struct GitStatus {
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub branch: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::process::Command;

    #[test]
    fn test_diff_to_string() {
        // 이 테스트는 실제 Git 리포지토리가 필요
        // TODO: 임시 리포지토리 생성으로 테스트 개선
    }
}