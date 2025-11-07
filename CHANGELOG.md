# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial MVP implementation
- AI-powered conventional commit generation
- Code change explanation with AI
- Multi-backend AI support (local Ollama, OpenAI, Anthropic)
- Multi-layer security system with trusted folders
- Context engine with hierarchical configuration
- MCP (Model Context Protocol) client foundation
- Comprehensive test suite
- CI/CD pipeline with GitHub Actions
- Cross-platform builds (Windows, macOS, Linux)

### Security
- Multi-layer security model (untrusted/trusted/session approval)
- Dangerous command detection and warnings
- Safe Git operations with git2-rs
- No shell command injection vulnerabilities

### Documentation
- Complete README with usage examples
- Contributing guidelines
- Apache 2.0 license
- API documentation with rustdoc

## [0.1.0] - 2025-11-07

### Added
- Project initialization
- Core architecture design
- Basic CLI interface with clap
- Git integration with git2-rs
- AI backend integration structure
- Security system foundation
- Context engine implementation
- MCP client protocol support
- Comprehensive testing framework
- Build and release automation

### Features
- `ai-cli commit` - Generate conventional commit messages
- `ai-cli explain` - Explain code changes
- `ai-cli init` - Initialize configuration
- `ai-cli config` - Show configuration
- Support for multiple AI models
- Conventional Commit specification compliance
- Cross-platform compatibility

### Security
- Trusted folder system
- Session-based command approval
- Dangerous command detection
- Safe command execution

### Performance
- Fast native binary with Rust
- Efficient Git operations
- Minimal startup time
- Low memory footprint

[Unreleased]: https://github.com/your-username/ai-cli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-username/ai-cli/releases/tag/v0.1.0