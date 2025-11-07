# Makefile for AI CLI

.PHONY: help build test clean install lint format check publish

# Default target
help:
	@echo "Available targets:"
	@echo "  build     - Build the project"
	@echo "  test      - Run tests"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install locally"
	@echo "  lint      - Run clippy lints"
	@echo "  format    - Format code"
	@echo "  check     - Run all checks (format, lint, test)"
	@echo "  publish   - Publish to crates.io"

# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test

# Run tests with coverage
test-coverage:
	cargo llvm-cov --lcov --output-path lcov.info

# Clean build artifacts
clean:
	cargo clean

# Install locally
install: build
	cargo install --path .

# Run clippy lints
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Format code
format:
	cargo fmt

# Run all checks
check: format lint test

# Run security audit
audit:
	cargo audit

# Run benchmarks
bench:
	cargo bench

# Generate documentation
docs:
	cargo doc --no-deps --open

# Publish to crates.io
publish: check
	cargo publish

# Create release
release: clean test build
	@echo "Release ready in target/release/"

# Setup development environment
setup:
	rustup component add rustfmt clippy
	cargo install cargo-audit cargo-llvm-cov

# Run integration tests
integration-test:
	cargo test --test integration_test

# Run Git tests
git-test:
	cargo test --test git_test

# Docker build
docker-build:
	docker build -t ai-cli .

# Run in Docker
docker-run: docker-build
	docker run --rm -v $(PWD):/workspace ai-cli