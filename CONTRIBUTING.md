# Contributing to YEET

Thank you for your interest in contributing to YEET! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Submitting Changes](#submitting-changes)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before contributing.

## Getting Started

### Prerequisites

1. **Rust** (latest stable version)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **cloudflared** (required for tunnel functionality)
   ```bash
   # macOS
   brew install cloudflare/cloudflare/cloudflared

   # Linux
   curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /usr/local/bin/cloudflared
   chmod +x /usr/local/bin/cloudflared
   ```

3. **Development tools** (optional but recommended)
   ```bash
   cargo install cargo-watch cargo-audit
   ```

### Setting Up Your Development Environment

1. Fork the repository on GitHub

2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/yeet.git
   cd yeet
   ```

3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/akash-otonomy/yeet.git
   ```

4. Build the project:
   ```bash
   cargo build
   ```

5. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Making Changes

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bugfix-name
   ```

2. Make your changes, following our [coding standards](#coding-standards)

3. Test your changes:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. Commit your changes using conventional commits:
   ```bash
   git commit -m "feat: add new feature"
   git commit -m "fix: resolve issue with path handling"
   git commit -m "docs: update README"
   ```

### Conventional Commits

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

### Running the Project Locally

```bash
# Build and run
cargo run -- /path/to/file.txt

# Run with daemon mode
cargo run -- /path/to/directory --daemon

# Check daemon status
cargo run -- --status

# Kill daemon
cargo run -- --kill
```

## Coding Standards

### Rust Style Guide

1. **Follow Rust standard formatting**:
   ```bash
   cargo fmt
   ```

2. **Address all Clippy warnings**:
   ```bash
   cargo clippy -- -D warnings
   ```

3. **Error Handling**:
   - Use `anyhow::Result<T>` for functions that can fail
   - Add context to errors with `.context("description")`
   - Never use `unwrap()` or `expect()` in production code
   - Use proper error propagation with `?` operator

   ```rust
   use anyhow::{Context, Result};

   fn example() -> Result<()> {
       let data = read_file()
           .context("Failed to read configuration file")?;
       Ok(())
   }
   ```

4. **Documentation**:
   - Add doc comments to all public functions
   - Include examples in doc comments where helpful
   - Keep comments up-to-date with code changes

   ```rust
   /// Validates that a path is within the allowed base directory.
   ///
   /// # Arguments
   /// * `base` - The base directory path
   /// * `requested` - The requested path to validate
   ///
   /// # Returns
   /// The canonicalized safe path if valid
   ///
   /// # Errors
   /// Returns an error if path traversal is detected
   fn validate_path_within_base(base: &PathBuf, requested: &PathBuf) -> Result<PathBuf> {
       // implementation
   }
   ```

5. **Security**:
   - Always validate and sanitize user input
   - Use HTML escaping for user-controlled content in HTML responses
   - Implement path traversal prevention for file serving
   - Set restrictive file permissions for sensitive files

### Code Organization

- Keep functions focused and small (ideally < 50 lines)
- Use meaningful variable and function names
- Group related functionality together
- Add section comments for major code blocks

## Submitting Changes

### Pull Request Process

1. **Update documentation**:
   - Update README.md if adding new features
   - Update CLAUDE.md if changing architecture
   - Update CHANGELOG.md following Keep a Changelog format

2. **Ensure all checks pass**:
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt -- --check
   cargo build --release
   ```

3. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create a Pull Request**:
   - Go to the original repository on GitHub
   - Click "New Pull Request"
   - Select your fork and branch
   - Fill out the PR template with details
   - Link any related issues

5. **Respond to feedback**:
   - Address all review comments
   - Push additional commits as needed
   - Keep the PR focused on a single feature/fix

### Pull Request Guidelines

- **Title**: Use conventional commit format
- **Description**: Explain what and why (not how)
- **Size**: Keep PRs focused and reasonably sized
- **Tests**: Include tests for new functionality
- **Documentation**: Update docs for user-facing changes
- **No breaking changes**: Unless discussed and approved
- **Clean history**: Squash fixup commits before merging

## Reporting Bugs

### Before Submitting a Bug Report

1. Check the [existing issues](https://github.com/akash-otonomy/yeet/issues)
2. Try the latest version from `main` branch
3. Gather relevant information:
   - OS and version
   - Rust version (`rustc --version`)
   - YEET version (`yeet --version`)
   - Steps to reproduce
   - Expected vs actual behavior

### Submitting a Bug Report

Use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md) and include:

- Clear and descriptive title
- Detailed steps to reproduce
- Expected behavior
- Actual behavior
- Screenshots if applicable
- Environment information
- Relevant logs or error messages

## Feature Requests

We welcome feature requests! Please use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md) and include:

- Clear description of the feature
- Use case and motivation
- Proposed implementation (if you have ideas)
- Alternatives you've considered
- Additional context

## Development Tips

### Useful Commands

```bash
# Watch for changes and rebuild
cargo watch -x check -x test

# Run security audit
cargo audit

# Check for unused dependencies
cargo +nightly udeps

# Generate documentation
cargo doc --open

# Profile build
cargo build --release --timings
```

### Project Structure

```
yeet/
├── src/
│   ├── main.rs          # Main application logic
│   ├── shared/mod.rs    # Shared types
│   ├── tui/             # Terminal UI
│   └── web/             # Web UI components (planned)
├── tests/               # Integration tests
├── .github/             # CI/CD and templates
└── docs/                # Additional documentation
```

### Testing

- Write unit tests for new functions
- Add integration tests for user-facing features
- Test edge cases and error conditions
- Ensure tests are deterministic and fast

## Getting Help

- **Questions**: Open a [Discussion](https://github.com/akash-otonomy/yeet/discussions)
- **Bugs**: Open an [Issue](https://github.com/akash-otonomy/yeet/issues)
- **Chat**: (Add link to Discord/Slack if available)

## License

By contributing to YEET, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to YEET! 🚀
