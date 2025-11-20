.PHONY: help build test lint fmt fmt-check clean install dev watch audit doc release

help:  ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build:  ## Build the project in debug mode
	cargo build

release:  ## Build the project in release mode (optimized)
	cargo build --release

test:  ## Run all tests
	cargo test --all

lint:  ## Run clippy linter
	cargo clippy -- -D warnings

fmt:  ## Format code with rustfmt
	cargo fmt --all

fmt-check:  ## Check if code is formatted correctly
	cargo fmt --all -- --check

clean:  ## Remove build artifacts
	cargo clean
	rm -rf target/

install:  ## Install yeet to system (requires cargo install)
	cargo install --path .

dev:  ## Run in development mode with test file
	@echo "Creating test file..."
	@echo "Hello from YEET!" > /tmp/yeet-test.txt
	cargo run -- /tmp/yeet-test.txt

watch:  ## Watch for changes and rebuild (requires cargo-watch)
	cargo watch -x check -x test -x build

audit:  ## Run security audit (requires cargo-audit)
	cargo audit

doc:  ## Generate and open documentation
	cargo doc --open --no-deps

check-all:  ## Run all checks (fmt, clippy, test)
	@echo "Running format check..."
	@make fmt-check
	@echo "\nRunning clippy..."
	@make lint
	@echo "\nRunning tests..."
	@make test
	@echo "\n✅ All checks passed!"

pre-commit:  ## Run pre-commit checks
	@make fmt-check || (echo "❌ Code not formatted. Run 'make fmt'" && exit 1)
	@make lint || (echo "❌ Clippy warnings found" && exit 1)
	@make test || (echo "❌ Tests failed" && exit 1)
	@echo "✅ Pre-commit checks passed!"

install-dev-tools:  ## Install development tools
	cargo install cargo-watch cargo-audit cargo-outdated

outdated:  ## Check for outdated dependencies (requires cargo-outdated)
	cargo outdated

bench:  ## Run benchmarks (if any)
	cargo bench

# Cross-compilation targets
build-linux-x64:  ## Build for Linux x86_64
	cargo build --release --target x86_64-unknown-linux-gnu

build-linux-arm:  ## Build for Linux ARM64 (requires cross)
	cross build --release --target aarch64-unknown-linux-gnu

build-macos-intel:  ## Build for macOS Intel
	cargo build --release --target x86_64-apple-darwin

build-macos-arm:  ## Build for macOS Apple Silicon
	cargo build --release --target aarch64-apple-darwin

build-all:  ## Build for all platforms
	@echo "Building for all platforms..."
	@make build-linux-x64
	@make build-linux-arm
	@make build-macos-intel
	@make build-macos-arm
	@echo "✅ All builds complete!"

# Git helpers
git-check:  ## Check git status and branch
	@git status
	@echo "\nCurrent branch: $$(git branch --show-current)"

commit:  ## Quick commit (usage: make commit MSG="your message")
	@if [ -z "$(MSG)" ]; then \
		echo "Error: MSG is required. Usage: make commit MSG=\"your message\""; \
		exit 1; \
	fi
	@make pre-commit
	git add -A
	git commit -m "$(MSG)"

# Quick commands
run:  ## Run with a test file (alias for dev)
	@make dev

size:  ## Show binary size
	@ls -lh target/release/yeet 2>/dev/null || echo "No release binary found. Run 'make release' first."

version:  ## Show version from Cargo.toml
	@grep '^version' Cargo.toml | head -1 | cut -d'"' -f2
