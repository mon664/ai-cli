# Contributing to AI CLI

감사합니다! AI CLI 프로젝트에 기여해 주셔서 대환영합니다. 이 가이드는 프로젝트에 기여하는 방법을 안내합니다.

## 🤝 기여 방식

다음과 같은 다양한 방식으로 기여하실 수 있습니다:

- 🐛 버그 리포트
- 💡 새 기능 제안
- 📝 문서 개선
- 🔧 코드 기여
- 🧪 테스트 작성
- 🌐 번역

## 🚀 시작하기

### 개발 환경 설정

1. **리포지토리 포크 및 클론**

```bash
# 리포지토리 포크 (GitHub 웹 인터페이스)
# 클론
git clone https://github.com/YOUR_USERNAME/ai-cli.git
cd ai-cli

# 업스트림 리모트 추가
git remote add upstream https://github.com/ORIGINAL_OWNER/ai-cli.git
```

2. **Rust 개발 환경 설치**

```bash
# Rust 설치 (없는 경우)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 설치 확인
rustc --version
cargo --version
```

3. **프로젝트 빌드**

```bash
# 의존성 설치 및 빌드
cargo build

# 개발 모드로 실행 테스트
cargo run -- --help
```

4. **Git 훅 설정 (선택사항)**

```bash
# pre-commit 훅 설정
cp scripts/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
```

### 프로젝트 구조 이해하기

```
ai-cli/
├── src/
│   ├── main.rs          # 프로그램 엔트리 포인트
│   ├── cli.rs           # CLI 인터페이스 정의 (clap)
│   ├── git_utils.rs     # Git 연동 모듈 (git2-rs)
│   ├── ai_utils.rs      # AI 백엔드 연동
│   ├── context.rs       # 컨텍스트 엔진
│   └── security.rs      # 보안 시스템
├── tests/               # 통합 테스트
├── benches/             # 성능 벤치마크
├── docs/               # 추가 문서
└── scripts/            # 빌드/개발 스크립트
```

## 📋 개발 워크플로우

### 1. 이슈 확인

- 새 기여를 시작하기 전 [이슈](https://github.com/your-username/ai-cli/issues)를 확인하세요
- 비슷한 이슈나 Pull Request가 없는지 검색하세요
- 논의가 필요한 경우 새 이슈를 생성하여 논의하세요

### 2. 브랜치 생성

```bash
# 최신 변경 사항 동기화
git checkout main
git pull upstream main

# 새 브랜치 생성 (Conventional Commits 명명 규칙 사용)
git checkout -b feature/your-feature-name
# 또는
git checkout -b fix/bug-description
```

### 3. 개발

- 코드 스타일 따르기 (`cargo fmt`)
- 린터 통과 (`cargo clippy`)
- 테스트 작성 및 통과 (`cargo test`)
- 커밋 메시지는 [Conventional Commits](https://www.conventionalcommits.org/) 표준 따르기

### 4. 테스트

```bash
# 모든 테스트 실행
cargo test

# 특정 모듈 테스트
cargo test git_utils

# 통합 테스트
cargo test --test integration

# 벤치마크 실행
cargo bench
```

### 5. 커밋 및 푸시

```bash
# 변경 사항 스테이징
git add .

# 커밋 (Conventional Commits)
git commit -m "feat: add new feature description"

# 브랜치 푸시
git push origin feature/your-feature-name
```

### 6. Pull Request 생성

- GitHub에서 Pull Request 생성
- 제목과 본문을 명확하게 작성
- 관련 이슈 참조 (`Closes #123`)
- 변경 사항을 설명하고 테스트 방법을 명시

## 📝 코드 스타일 가이드

### Rust 코드 스타일

- `rustfmt` 사용: `cargo fmt`
- `clippy` 경고 해결: `cargo clippy -- -D warnings`
- 명확한 변수명과 함수명 사용
- 적절한 문서 주석 포함 (`///`)

### 예시

```rust
/// 좋은 예시:
/// Gets the staged diff content from the current Git repository.
///
/// # Errors
///
/// Returns an error if the repository cannot be opened or no staged changes exist.
///
/// # Examples
///
/// ```
/// use ai_cli::git_utils::get_staged_diff;
/// let diff = get_staged_diff()?;
/// println!("Staged changes: {}", diff);
/// ```
pub fn get_staged_diff() -> Result<String> {
    // 구현
}
```

### 커밋 메시지 규칙

```
feat: add conventional commit generation
fix: resolve git repository detection issue
docs: update installation instructions
style: format code with rustfmt
refactor: simplify AI backend selection logic
test: add integration tests for git operations
chore: update dependencies
```

## 🧪 테스트 가이드

### 단위 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_commit_prompt() {
        let diff = "+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"Hello\");\n }\n";
        let prompt = create_commit_prompt(diff, None);

        assert!(prompt.contains("Conventional Commits"));
        assert!(prompt.contains(diff));
    }
}
```

### 통합 테스트

```rust
// tests/integration_test.rs
use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_commit_command() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.arg("commit").arg("--help");

    cmd.assert().success();
}
```

## 🐛 버그 리포트

버그를 발견하면 다음 정보를 포함하여 이슈를 생성하세요:

- **버그 설명**: 무엇이 잘못되었나요?
- **재현 단계**: 버그를 재현하는 방법
- **기대 동작**: 무엇이 일어나야 했나요?
- **실제 동작**: 실제로 무엇이 일어났나요?
- **환경 정보**: OS, Rust 버전, AI CLI 버전
- **관련 로그**: 에러 메시지, 백트레이스 등

## 💡 기능 요청

새 기능을 제안할 때 다음을 포함하세요:

- **기능 설명**: 제안하는 기능은 무엇인가요?
- **사용 사례**: 이 기능이 왜 필요한가요?
- **제안 구현**: 어떻게 구현하는 것을 제안하시나요?
- **대안**: 고려한 다른 접근 방식이 있나요?

## 📚 문서 기여

- README.md 업데이트
- API 문서 개선
- 코드 예시 추가
- 튜토리얼 작성

## 🔍 코드 리뷰

### 리뷰어를 위한 가이드

- 코드가 명확하고 이해하기 쉬운가요?
- 테스트가 충분한가요?
- 문서가 적절한가요?
- 성능에 영향을 미치는가요?
- 보안 고려사항이 있나요?

### PR 작성자를 위한 가이드

- 작고 집중된 PR로 유지하세요
- 명확한 제목과 설명을 작성하세요
- 테스트를 포함하세요
- 문서를 업데이트하세요
- 피드백에 적극적으로 반응하세요

## 🏷️ 라벨 가이드

- `good first issue`: 초보자에게 좋은 이슈
- `help wanted`: 도움이 필요한 이슈
- `bug`: 버그 리포트
- `enhancement`: 기능 개선
- `documentation`: 문서 관련
- `security`: 보안 관련

## 🚀 릴리스 프로세스

릴리스는 다음 단계를 따릅니다:

1. `main` 브랜치로 머지
2. 버전 번호 업데이트 (`Cargo.toml`)
3. 체인지로그 업데이트
4. 태그 생성 (`git tag v0.1.0`)
5. GitHub Release 생성

## 💬 커뮤니티

- [GitHub Discussions](https://github.com/your-username/ai-cli/discussions)에서 질문하고 아이디어를 공유하세요
- [Discord 서버](https://discord.gg/your-server)에 참여하여 실시간으로 소통하세요

## 📜 행동 강령

모든 기여자는 [행동 강령](CODE_OF_CONDUCT.md)을 따라야 합니다. 존중하고 포용적인 환경을 만들어 함께 기여해 주세요.

## 🙏 감사

AI CLI 프로젝트에 기여해 주셔서 감사합니다! 여러분의 기여가 더 나은 개발자 도구를 만드는 데 도움이 됩니다.

---

질문이 있으시면 [이슈](https://github.com/your-username/ai-cli/issues)를 생성하거나 [Discussions](https://github.com/your-username/ai-cli/discussions)에 참여해 주세요.