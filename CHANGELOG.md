# Changelog

All notable changes to YEET will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive security hardening
- Path traversal attack prevention with path validation
- HTML escaping to prevent XSS attacks in directory listings
- Streaming file responses for large files (>100MB)
- Health check endpoint at `/health`
- Structured logging with tracing framework
- Restrictive file permissions (0o600) for tunnel state file
- Complete documentation suite (CLAUDE.md, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md)
- GitHub issue and PR templates
- CI/CD workflow for linting and testing
- Rust formatting configuration (.rustfmt.toml, .editorconfig)
- Makefile for common development tasks

### Changed
- Replaced all `unwrap()` and `expect()` calls with proper error handling
- Optimized regex compilation using `once_cell` for performance
- Improved error messages with context using `anyhow::Context`
- Enhanced daemon startup error handling

### Fixed
- Path traversal vulnerability in directory serving
- XSS vulnerability in file name rendering
- Panic conditions from unsafe unwrap calls
- Memory issues with large file serving

### Security
- **CRITICAL**: Fixed path traversal vulnerability (CVE-pending)
- **HIGH**: Fixed XSS vulnerability in directory listings
- **MEDIUM**: Added secure file permissions for state files
- Improved input validation and sanitization

## [0.1.2] - 2025-11-20

### Fixed
- Daemon restart when serving different file/directory
- Tunnel persistence improvements

## [0.1.1] - 2025-11-19

### Added
- Admin dashboard at `/admin`
- Real-time stats display
- Request logging in dashboard

### Changed
- Improved TUI aesthetics

## [0.1.0] - 2025-11-15

### Added
- Initial release
- Zero-config file sharing via Cloudflare tunnels
- Retro TUI with colorful interface
- Daemon mode for background operation
- Directory browsing support
- Cross-platform support (Linux and macOS)
- Single file sharing
- Admin dashboard

[Unreleased]: https://github.com/akash-otonomy/yeet/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/akash-otonomy/yeet/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/akash-otonomy/yeet/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/akash-otonomy/yeet/releases/tag/v0.1.0
