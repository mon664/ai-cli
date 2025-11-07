# AI CLI - AI-powered Git Assistant

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.91+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)]()
[![Website](https://img.shields.io/badge/website-Live-green.svg)](https://mon664.github.io/ai-cli/)
[![Demo](https://img.shields.io/badge/demo-Interactive-purple.svg)](https://mon664.github.io/ai-cli/demo.html)

AI CLI는 개발자의 Git 워크플로우를 혁신하는 지능형 커맨드 라인 도구입니다. 로컬 및 클라우드 AI 모델을 활용하여 전문적인 커밋 메시지를 자동 생성하고 코드 변경 사항을 설명해줍니다.

## ✨ 주요 기능

- 🤖 **AI 기반 커밋 메시지 생성**: 스테이징된 변경 사항을 분석하여 Conventional Commit 표준에 맞는 전문적인 메시지 생성
- 📝 **코드 변경 사항 설명**: 복잡한 코드 변경을 이해하기 쉬운 자연어로 설명
- 🔒 **다층적 보안 시스템**: 신뢰 폴더와 세션 기반 승인으로 안전한 명령어 실행 보장
- 🌐 **다중 AI 백엔드 지원**: 로컬(Ollama), OpenAI, Anthropic 모델 유연하게 사용
- 📁 **컨텍스트 인식**: 프로젝트 설정과 셸 히스토리를 활용한 맞춤형 응답
- 🚀 **고성능**: Rust로 구현된 빠른 네이티브 바이너리

## 🌐 웹사이트 및 데모

- **[공식 웹사이트](https://mon664.github.io/ai-cli/)**: 전체 기능 소개 및 문서
- **[인터랙티브 데모](https://mon664.github.io/ai-cli/demo.html)**: 브라우저에서 AI CLI 직접 체험하기

## 🚀 빠른 시작

### 설치

```bash
# Cargo를 통해 설치 (권장)
cargo install ai-cli

# 또는 직접 빌드
git clone https://github.com/your-username/ai-cli.git
cd ai-cli
cargo build --release
```

### 기본 사용법

```bash
# Git 리포지토리에서
cd your-project

# 커밋 메시지 생성 (변경 사항 먼저 스테이징)
git add .
ai-cli commit

# 특정 모델 사용
ai-cli commit --model openai

# 변경 사항 설명
ai-cli explain

# 특정 커밋 분석
ai-cli explain --hash abc1234

# 설정 초기화
ai-cli init --model local --openai-key YOUR_API_KEY
```

## 📋 사용 예시

### 커밋 메시지 생성

```bash
$ ai-cli commit
🤖 AI is generating your commit message...
📋 Staging all changes...
📝 Analyzing 42 lines of changes...

--- AI Generated Commit Message ---
feat(cli): add conventional commit generation with AI integration

- Implement clap-based CLI interface with commit and explain subcommands
- Add git2-rs integration for safe Git operations
- Support multiple AI backends (local Ollama, OpenAI, Anthropic)
- Include multi-layer security model with trusted folders
-----------------------------------

Do you want to execute this commit? [Y/n] y
🔄 Executing git commit...
✅ Commit successful!
```

### 코드 변경 설명

```bash
$ ai-cli explain --detailed
🔍 AI is analyzing the changes...

📄 AI Analysis:
## High-level Summary
This change introduces the core AI CLI functionality with comprehensive Git integration and multi-backend AI support.

## Technical Details
- **CLI Interface**: Implemented using clap with derive macros for type-safe command parsing
- **Git Operations**: Safe diff extraction using git2-rs library
- **AI Integration**: Modular backend system supporting local (Ollama) and remote (OpenAI, Anthropic) models
- **Security**: Multi-layer protection with trusted folders and session-based command approval

## Reasoning
The modular architecture allows for flexible AI model selection while maintaining security and performance. The use of Rust ensures memory safety and fast execution.

## Impact
This foundation enables all subsequent AI-powered Git workflow automation features.
```

## 🏗️ 아키텍처

AI CLI는 다음과 같은 핵심 구성 요소로 이루어져 있습니다:

```
ai-cli/
├── src/
│   ├── main.rs          # 엔트리 포인트
│   ├── cli.rs           # CLI 인터페이스 정의
│   ├── git_utils.rs     # Git 연동 모듈
│   ├── ai_utils.rs      # AI 백엔드 연동
│   ├── context.rs       # 컨텍스트 엔진
│   └── security.rs      # 보안 시스템
└── tests/               # 통합 테스트
```

### 컨텍스트 엔진
- **전역 컨텍스트** (`~/.ai-cli/CONFIG.md`): 사용자의 전체 선호도
- **프로젝트 컨텍스트** (`PROJECT.md`): 프로젝트별 설정과 아키텍처
- **디렉토리 컨텍스트**: 특정 모듈에 대한 상세 지침

### 보안 모델
1. **1층 (비신뢰)**: 읽기 전용 모드
2. **2층 (신뢰 폴더)**: 승인된 폴더에서 AI 기능 활성화
3. **3층 (세션 승인)**: 명령어 타입별 세션 승인

## ⚙️ 설정

### 환경 변수

```bash
# 로컬 모델 설정
export AI_CLI_LOCAL_MODEL="gemma2:9b"
export AI_CLI_OLLAMA_URL="http://localhost:11434"

# OpenAI 설정
export OPENAI_API_KEY="your-openai-api-key"
export AI_CLI_OPENAI_MODEL="gpt-4o-mini"

# Anthropic 설정
export ANTHROPIC_API_KEY="your-anthropic-api-key"
export AI_CLI_ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"
```

### 컨텍스트 파일

**전역 설정** (`~/.ai-cli/CONFIG.md`):
```markdown
# AI CLI Global Configuration

## Developer Preferences
- I prefer conventional commits with clear descriptions
- Focus on user-facing changes in commit messages

## AI Model Preferences
- Default to local models for privacy
- Fall back to OpenAI GPT-4o-mini for complex reasoning
```

**프로젝트 설정** (`PROJECT.md`):
```markdown
# Project Configuration

## Architecture
- Language: Rust
- CLI Framework: clap
- Follow conventional commits specification

## Development Guidelines
- Include proper error handling
- Write tests for new functionality
```

## 🔧 개발

### 빌드 요구사항
- Rust 1.91 이상
- Git 2.0 이상

### 로컬 개발 환경 설정

```bash
# 리포지토리 클론
git clone https://github.com/your-username/ai-cli.git
cd ai-cli

# 의존성 설치
cargo build

# 테스트 실행
cargo test

# 개발 모드로 실행
cargo run -- commit --help
```

### 테스트

```bash
# 모든 테스트 실행
cargo test

# 특정 모듈 테스트
cargo test git_utils

# 통합 테스트
cargo test --test integration
```

## 🤝 기여하기

기여를 환영합니다! 다음 단계를 따라주세요:

1. 이 리포지토리를 포크하세요
2. 기능 브랜치를 생성하세요 (`git checkout -b feature/amazing-feature`)
3. 변경 사항을 커밋하세요 (`git commit -m 'feat: add amazing feature'`)
4. 브랜치에 푸시하세요 (`git push origin feature/amazing-feature`)
5. Pull Request를 생성하세요

### 기여 가이드라인
- Conventional Commit 표준 따르기
- 테스트 포함하기
- 문서 업데이트하기
- `cargo fmt`와 `cargo clippy` 실행하기

## 📄 라이선스

이 프로젝트는 Apache License 2.0 하에 라이선스가 부여됩니다. [LICENSE](LICENSE) 파일을 참조하세요.

## 🔗 관련 프로젝트

- [aicommits](https://github.com/NVIDIA/ai-commits) - 커밋 메시지 생성
- [git-ai](https://github.com/gpt-engineer-org/git-ai) - Git 작업 자동화
- [diff-explainer](https://github.com/pwwang/diff-explainer) - Diff 설명

## 🙏 감사

AI CLI는 다음 프로젝트에서 영감을 받았습니다:
- [git2-rs](https://github.com/rust-lang/git2-rs) - Git 바인딩
- [clap](https://github.com/clap-rs/clap) - CLI 프레임워크
- [ollama-rs](https://github.com/pepperoni21/ollama-rs) - Ollama 클라이언트

## 📞 지원

- 🐛 [버그 리포트](https://github.com/your-username/ai-cli/issues)
- 💡 [기능 요청](https://github.com/your-username/ai-cli/issues)
- 💬 [토론](https://github.com/your-username/ai-cli/discussions)

---

**AI CLI** - 개발자 워크플로우를 위한 스마트한 AI 파트너 🚀