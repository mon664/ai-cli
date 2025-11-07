use anyhow::{Result, anyhow};
use std::io::{self, Write};
use std::process::Command;
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::fs;

/// 보안 모듈
/// 다층적 보안 시스템: 신뢰 폴더 + 세션 기반 명령어 승인

/// 보안 레벨
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityLevel {
    Untrusted,  // 읽기 전용
    Trusted,    // 승인된 폴더
    Restricted, // 제한된 모드
}

/// 승인 옵션
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalOption {
    Yes,                  // 일회성 승인
    YesForSession,        // 세션 전체 승인
    No,                   // 거부
    EditAndRetry,         // 수정 후 재시도
}

/// 세션 승인 정보
#[derive(Debug, Clone)]
pub struct SessionApproval {
    pub command_type: String,
    pub approved_at: std::time::SystemTime,
    pub expires_at: std::time::SystemTime,
}

/// 보안 매니저
pub struct SecurityManager {
    trusted_folders: Vec<String>,
    session_approvals: Vec<SessionApproval>,
    current_level: SecurityLevel,
    session_duration: std::time::Duration,
}

impl SecurityManager {
    /// 새 보안 매니저 생성
    pub fn new() -> Self {
        Self {
            trusted_folders: Vec::new(),
            session_approvals: Vec::new(),
            current_level: SecurityLevel::Untrusted,
            session_duration: std::time::Duration::from_secs(3600), // 1시간
        }
    }

    /// 현재 디렉토리의 보안 레벨 결정
    pub fn determine_security_level(&mut self, current_dir: &Path) -> SecurityLevel {
        let current_dir_str = current_dir.to_string_lossy().to_lowercase();

        // 위험한 디렉토리 목록
        let dangerous_dirs = [
            "/",
            "/home",
            "/users",
            "/windows",
            "/system32",
            "c:\\",
            "c:\\windows",
            "c:\\program files",
        ];

        // 위험한 디렉토리 확인
        for dangerous_dir in &dangerous_dirs {
            if current_dir_str.starts_with(&dangerous_dir.to_lowercase()) {
                self.current_level = SecurityLevel::Restricted;
                return SecurityLevel::Restricted;
            }
        }

        // 신뢰 폴더 확인
        for trusted_folder in &self.trusted_folders {
            if current_dir_str.starts_with(&trusted_folder.to_lowercase()) {
                self.current_level = SecurityLevel::Trusted;
                return SecurityLevel::Trusted;
            }
        }

        self.current_level = SecurityLevel::Untrusted;
        SecurityLevel::Untrusted
    }

    /// 폴더 신뢰 상태 확인
    pub fn is_folder_trusted(&self, folder: &Path) -> bool {
        let folder_str = folder.to_string_lossy().to_lowercase();
        self.trusted_folders.iter()
            .any(|trusted| folder_str.starts_with(&trusted.to_lowercase()))
    }

    /// 폴더를 신뢰 목록에 추가
    pub fn trust_folder(&mut self, folder: &Path) -> Result<()> {
        let folder_str = folder.canonicalize()
            .map_err(|e| anyhow!("Cannot canonicalize path: {}", e))?
            .to_string_lossy()
            .to_string();

        if !self.trusted_folders.contains(&folder_str) {
            self.trusted_folders.push(folder_str.clone());
            self.save_trusted_folders()?;
        }

        println!("✅ Folder '{}' is now trusted", folder_str);
        Ok(())
    }

    /// 사용자에게 폴더 신뢰 여부 확인
    pub fn prompt_trust_folder(&mut self, folder: &Path) -> Result<bool> {
        println!("\n🔒 Security Notice");
        println!("This folder is not trusted: {}", folder.display());
        println!("AI CLI can only read files in untrusted folders.");
        println!("To enable AI features, you must trust this folder.");
        println!();

        print!("Do you want to trust this folder? [Y/n] ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        let response = response.trim().to_lowercase();
        if response.is_empty() || response == "y" || response == "yes" {
            self.trust_folder(folder)?;
            Ok(true)
        } else {
            println!("❌ Folder not trusted. AI CLI will run in read-only mode.");
            Ok(false)
        }
    }

    /// 명령어 실행 승인 요청
    pub fn prompt_command_approval(&mut self, command: &str, command_type: &str) -> Result<ApprovalOption> {
        // 세션 승인 확인
        if self.has_session_approval(command_type) {
            return Ok(ApprovalOption::Yes);
        }

        println!("\n⚠️  Security Approval Required");
        println!("Command to execute: {}", command);
        println!("Type: {}", command_type);
        println!();

        println!("Options:");
        println!("  [Y]es     - Execute this command once");
        println!("  [Y]es for session - Execute all {} commands this session", command_type);
        println!("  [N]o      - Cancel execution");
        println!("  [E]dit    - Modify the command and retry");
        println!();

        print!("Your choice [Y/N/E]: ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        match response.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(ApprovalOption::Yes),
            "yes for session" | "yf" => {
                self.add_session_approval(command_type)?;
                Ok(ApprovalOption::YesForSession)
            }
            "n" | "no" => Ok(ApprovalOption::No),
            "e" | "edit" => Ok(ApprovalOption::EditAndRetry),
            _ => {
                println!("Invalid choice. Assuming 'No'.");
                Ok(ApprovalOption::No)
            }
        }
    }

    /// 세션 승인 확인
    fn has_session_approval(&self, command_type: &str) -> bool {
        let now = std::time::SystemTime::now();

        self.session_approvals.iter()
            .any(|approval| {
                approval.command_type == command_type &&
                now >= approval.approved_at &&
                now <= approval.expires_at
            })
    }

    /// 세션 승인 추가
    fn add_session_approval(&mut self, command_type: &str) -> Result<()> {
        let now = std::time::SystemTime::now();
        let expires_at = now + self.session_duration;

        // 기존 승인 제거
        self.session_approvals.retain(|approval| {
            approval.command_type != command_type && now <= approval.expires_at
        });

        // 새 승인 추가
        self.session_approvals.push(SessionApproval {
            command_type: command_type.to_string(),
            approved_at: now,
            expires_at,
        });

        println!("✅ Approved all '{}' commands for this session (expires in 1 hour)", command_type);
        Ok(())
    }

    /// 신뢰 폴더 목록 저장
    fn save_trusted_folders(&self) -> Result<()> {
        if let Some(home_dir) = dirs::home_dir() {
            let config_dir = home_dir.join(".ai-cli");
            fs::create_dir_all(&config_dir)?;

            let trusted_file = config_dir.join("trusted_folders.json");
            let trusted_data = TrustedFoldersData {
                folders: self.trusted_folders.clone(),
            };

            let json = serde_json::to_string_pretty(&trusted_data)?;
            fs::write(trusted_file, json)?;
        }

        Ok(())
    }

    /// 신뢰 폴더 목록 로드
    fn load_trusted_folders(&mut self) -> Result<()> {
        if let Some(home_dir) = dirs::home_dir() {
            let trusted_file = home_dir.join(".ai-cli").join("trusted_folders.json");

            if trusted_file.exists() {
                let content = fs::read_to_string(trusted_file)?;
                let trusted_data: TrustedFoldersData = serde_json::from_str(&content)?;
                self.trusted_folders = trusted_data.folders;
            }
        }

        Ok(())
    }

    /// 위험한 명령어 확인
    pub fn is_dangerous_command(command: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf",
            "del /",
            "format",
            "sudo rm",
            "chmod 777",
            "> /dev/sda",
            "dd if=",
            ":(){ :|:& };:", // fork bomb
        ];

        let command_lower = command.to_lowercase();
        dangerous_patterns.iter()
            .any(|pattern| command_lower.contains(pattern))
    }

    /// 추가 경고 필요한 명령어 확인
    pub fn needs_warning(command: &str) -> bool {
        let warning_patterns = [
            "rm ",
            "del ",
            "mv ",
            "chmod ",
            "chown ",
            "git clean",
            "git reset --hard",
        ];

        let command_lower = command.to_lowercase();
        warning_patterns.iter()
            .any(|pattern| command_lower.contains(pattern))
    }

    /// 명령어 실행 전 최종 확인
    pub fn confirm_dangerous_command(command: &str) -> Result<bool> {
        println!("\n🚨 DANGEROUS COMMAND WARNING");
        println!("This command may cause irreversible damage:");
        println!("  {}", command);
        println!();

        print!("Are you absolutely sure you want to execute this? Type 'YES' to confirm: ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        Ok(response.trim() == "YES")
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        let mut manager = Self::new();
        let _ = manager.load_trusted_folders(); // 실패해도 계속 진행
        manager
    }
}

/// 신뢰 폴더 데이터 구조체
#[derive(Debug, Serialize, Deserialize)]
struct TrustedFoldersData {
    folders: Vec<String>,
}

/// 커밋 승인 및 실행
pub fn prompt_and_commit(commit_message: &str) -> Result<()> {
    let mut security_manager = SecurityManager::default();

    println!("\n--- AI Generated Commit Message ---");
    println!("{}", commit_message);
    println!("-----------------------------------");

    // 승인 요청
    match security_manager.prompt_command_approval(
        &format!("git commit -m \"{}\"", commit_message),
        "git_commit"
    )? {
        ApprovalOption::Yes | ApprovalOption::YesForSession => {
            execute_git_commit(commit_message)?;
        }
        ApprovalOption::No => {
            println!("❌ Commit cancelled by user.");
        }
        ApprovalOption::EditAndRetry => {
            print!("Enter custom commit message: ");
            io::stdout().flush()?;

            let mut custom_message = String::new();
            io::stdin().read_line(&mut custom_message)?;
            let custom_message = custom_message.trim();

            if !custom_message.is_empty() {
                execute_git_commit(custom_message)?;
            } else {
                println!("❌ Empty commit message. Commit cancelled.");
            }
        }
    }

    Ok(())
}

/// Git 커밋 실행
fn execute_git_commit(commit_message: &str) -> Result<()> {
    println!("\n🔄 Executing git commit...");

    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(commit_message)
        .output()?;

    if output.status.success() {
        println!("✅ Commit successful!");
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        println!("❌ Commit failed!");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        return Err(anyhow!("Git commit failed"));
    }

    Ok(())
}

/// 안전한 명령어 실행
pub fn execute_command_safely(command: &str) -> Result<std::process::Output> {
    let mut security_manager = SecurityManager::default();

    // 위험한 명령어 확인
    if SecurityManager::is_dangerous_command(command) {
        if !SecurityManager::confirm_dangerous_command(command)? {
            return Err(anyhow!("Dangerous command cancelled by user"));
        }
    } else if SecurityManager::needs_warning(command) {
        match security_manager.prompt_command_approval(command, "file_operation")? {
            ApprovalOption::Yes | ApprovalOption::YesForSession => {
                // 계속 진행
            }
            ApprovalOption::No => {
                return Err(anyhow!("Command cancelled by user"));
            }
            ApprovalOption::EditAndRetry => {
                print!("Enter modified command: ");
                io::stdout().flush()?;

                let mut modified_command = String::new();
                io::stdin().read_line(&mut modified_command)?;
                let modified_command = modified_command.trim();

                if modified_command.is_empty() {
                    return Err(anyhow!("Empty command. Execution cancelled."));
                }

                return execute_command_safely(modified_command);
            }
        }
    }

    // 명령어 실행 (Windows: cmd, Unix/Mac: sh)
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", command])
        .output()?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .args(["-c", command])
        .output()?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert_eq!(manager.current_level, SecurityLevel::Untrusted);
        assert!(manager.trusted_folders.is_empty());
    }

    #[test]
    fn test_dangerous_command_detection() {
        assert!(SecurityManager::is_dangerous_command("rm -rf /"));
        assert!(SecurityManager::is_dangerous_command("sudo rm -rf /home"));
        assert!(!SecurityManager::is_dangerous_command("git status"));
        assert!(!SecurityManager::is_dangerous_command("ls -la"));
    }

    #[test]
    fn test_warning_command_detection() {
        assert!(SecurityManager::needs_warning("rm file.txt"));
        assert!(SecurityManager::needs_warning("git reset --hard"));
        assert!(!SecurityManager::needs_warning("git status"));
        assert!(!SecurityManager::needs_warning("echo hello"));
    }

    #[test]
    fn test_trusted_folder_operations() {
        let mut manager = SecurityManager::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!manager.is_folder_trusted(temp_dir.path()));

        let _ = manager.trust_folder(temp_dir.path());
        assert!(manager.is_folder_trusted(temp_dir.path()));
    }
}