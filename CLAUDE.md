# CLAUDE.md - AI Assistant Guide for YEET

> **Last Updated:** 2025-11-20
> **Version:** 0.1.2
> **Purpose:** Comprehensive guide for AI assistants working with the YEET codebase

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Codebase Architecture](#codebase-architecture)
3. [Key Files & Responsibilities](#key-files--responsibilities)
4. [Development Workflow](#development-workflow)
5. [Coding Conventions](#coding-conventions)
6. [Common Tasks](#common-tasks)
7. [Dependencies Guide](#dependencies-guide)
8. [Testing & Debugging](#testing--debugging)
9. [Release Process](#release-process)
10. [Important Gotchas](#important-gotchas)

---

## Project Overview

**YEET** is a zero-config file sharing tool written in Rust that leverages Cloudflare Quick Tunnels to create instant public URLs for sharing files and directories.

### Core Features
- **Zero Configuration**: No accounts, no config files, no setup
- **Instant Public URLs**: Uses Cloudflare Quick Tunnels (trycloudflare.com)
- **Retro TUI**: Terminal UI with 8-bit neon aesthetic using Ratatui
- **Daemon Mode**: Background process keeps tunnels alive independently
- **Directory Support**: Serves entire folders with interactive file browser
- **Admin Dashboard**: Real-time stats and request logs at `/admin`
- **Cross-Platform**: Supports Linux (x86_64, ARM64) and macOS (Intel, Apple Silicon)

### Tech Stack
- **Language**: Rust 2021 Edition
- **TUI Framework**: Ratatui + Crossterm
- **Web Server**: Axum (Tokio-based)
- **Web UI**: Alpine.js (embedded), Dioxus components (planned migration)
- **CLI**: Clap (derive-based)
- **Process Management**: Nix crate (Unix fork/signals)
- **External Dependency**: cloudflared binary

### Repository
- **GitHub**: https://github.com/akash-otonomy/yeet
- **License**: MIT
- **Current Version**: 0.1.2

---

## Codebase Architecture

### Directory Structure

```
/home/user/yeet/
├── src/
│   ├── main.rs              # Main entry point (1,272 lines)
│   ├── shared/mod.rs        # Shared data structures (34 lines)
│   ├── tui/
│   │   ├── mod.rs           # TUI rendering & logo (80 lines)
│   │   └── theme.rs         # Color palette (14 lines)
│   └── web/mod.rs           # Dioxus components (270 lines, not yet integrated)
├── .github/workflows/
│   └── release.yml          # CI/CD pipeline for releases
├── assets/                  # Static assets (if any)
├── Cargo.toml               # Dependencies and build config
├── Cargo.lock               # Dependency lock file
├── install.sh               # One-liner installer script
├── LICENSE                  # MIT license
└── README.md                # User-facing documentation
```

### Module Organization

```
yeet (binary crate)
├── main::            Core application logic, CLI, daemon management
├── shared::          Data structures (ServerStats, RequestLog, FileStats)
├── tui::             Terminal UI rendering
│   └── theme::       Color constants
└── web::             Dioxus web components (planned future use)
```

### Application Flow

```
User executes: yeet <file> [--daemon] [--port 8000]
    ↓
main() parses CLI with clap
    ↓
Validate file/directory exists
    ↓
Load existing tunnel state from ~/.yeet/tunnel.state (if exists)
    ↓
Check if existing daemon is alive and serving same file
    ├─ YES → Reuse existing tunnel, display TUI, exit
    └─ NO  → Kill old daemon (if any), continue to spawn new
    ↓
spawn_daemon(file_path, port)
    ├─ Fork process via nix::unistd::fork()
    │   ├─ Parent: Waits for tunnel state file (~30s timeout)
    │   └─ Child: Becomes daemon
    │       ├─ setsid() → Detach from terminal
    │       ├─ Redirect stdin/stdout/stderr → /dev/null
    │       ├─ run_daemon_server() → Start Axum HTTP server
    │       └─ start_cloudflared_daemon() → Spawn tunnel, extract URL
    ↓
TUI displays tunnel info
    ├─ User presses 'q' or Esc → Exit TUI (daemon stays alive)
    └─ Daemon continues running in background
```

### Process Architecture

```
┌─────────────────────────────────────────┐
│  User's Shell (Parent Process)          │
│  - Spawns daemon                         │
│  - Displays TUI                          │
│  - Exits (daemon persists)               │
└─────────────────────────────────────────┘
                    │
                    │ fork()
                    ↓
┌─────────────────────────────────────────┐
│  Daemon Process (Child)                  │
│  - New session (setsid)                  │
│  - Detached from terminal                │
│  - Saves state to ~/.yeet/tunnel.state   │
│  ├─ Axum HTTP Server (tokio task)       │
│  └─ cloudflared subprocess               │
└─────────────────────────────────────────┘
```

---

## Key Files & Responsibilities

### src/main.rs (Primary Application Logic)

**Location**: `/home/user/yeet/src/main.rs`
**Lines**: 1,272
**Purpose**: Core orchestration of all application functionality

#### Key Components

| Component | Lines | Purpose |
|-----------|-------|---------|
| `Cli` struct | 46-74 | CLI argument parsing via clap derive macros |
| `App` struct | 76-86 | Application state for TUI rendering |
| `TunnelState` struct | 88-95 | Persistent state saved to JSON file |
| `main()` | 1102-1272 | Entry point, daemon lifecycle management |
| `spawn_daemon()` | 922-1045 | Fork process, daemonize, start services |
| `run_daemon_server()` | 694-855 | Axum HTTP server setup and routing |
| `start_cloudflared_daemon()` | 857-920 | Spawn cloudflared, extract tunnel URL |
| `ui()` | 1047-1100 | Ratatui render function for TUI |
| `run_app()` | 1216-1270 | TUI event loop with keyboard input |

#### Embedded HTML

| Component | Lines | Description |
|-----------|-------|-------------|
| Admin Dashboard | 136-385 | Alpine.js-powered dashboard with stats and logs |
| Directory Browser | 485-691 | File browser with search and sorting |

#### API Endpoints

```rust
// Single file mode
GET /              → serve file
GET /{filename}    → serve file
GET /admin         → admin dashboard HTML
GET /api/stats     → JSON ServerStats
GET /api/logs      → JSON Vec<RequestLog>

// Directory mode
GET /              → directory listing HTML
GET /{path}/*      → serve files recursively (tower-http)
GET /admin         → admin dashboard
GET /api/stats     → JSON stats
GET /api/logs      → JSON logs
```

#### Important Constants

```rust
const DEFAULT_PORT: u16 = 8000;
const STATE_FILE: &str = "~/.yeet/tunnel.state";  // Expands via dirs crate
const CLOUDFLARED_TIMEOUT_SECS: u64 = 30;
```

### src/shared/mod.rs (Data Structures)

**Location**: `/home/user/yeet/src/shared/mod.rs`
**Lines**: 34
**Purpose**: Shared type definitions for API responses

```rust
pub struct ServerStats {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub total_bytes_sent: u64,
    pub current_speed_bps: u64,
    pub active_connections: u32,
    pub unique_ips: u32,
    pub requests_per_minute: f64,
}

pub struct RequestLog {
    pub timestamp: i64,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub size_bytes: u64,
    pub user_agent: String,
    pub ip: String,
}

pub struct FileStats {
    pub name: String,
    pub requests: u64,
    pub bytes_sent: u64,
}
```

**Note**: Currently used for API contract. Stats are hardcoded in main.rs (TODO: implement real tracking).

### src/tui/mod.rs (Terminal UI)

**Location**: `/home/user/yeet/src/tui/mod.rs`
**Lines**: 80
**Purpose**: TUI rendering logic

```rust
pub struct YeetTui {
    throbber_state: throbber_widgets_tui::ThrobberState,
}

impl YeetTui {
    pub fn new() -> Self { /* ... */ }

    pub fn render_logo(&self, frame: &mut Frame, area: Rect) {
        // Renders colorful YEET ASCII art
        // Each letter has unique color from theme
    }

    pub fn tick(&mut self) {
        // Advances throbber animation
    }
}
```

### src/tui/theme.rs (Color Palette)

**Location**: `/home/user/yeet/src/tui/theme.rs`
**Lines**: 14
**Purpose**: Centralized color definitions

```rust
pub const CYAN: Color = Color::Rgb(0, 255, 255);
pub const MAGENTA: Color = Color::Rgb(255, 0, 255);
pub const YELLOW: Color = Color::Rgb(255, 255, 0);
pub const GREEN: Color = Color::Rgb(0, 255, 159);
pub const DARK_GRAY: Color = Color::Rgb(64, 64, 64);
```

**Usage**: Retro 8-bit aesthetic throughout TUI and web dashboards.

### src/web/mod.rs (Future Web UI)

**Location**: `/home/user/yeet/src/web/mod.rs`
**Lines**: 270
**Purpose**: Dioxus components for future web UI migration

**Status**: Components exist but not yet integrated. Current implementation uses Alpine.js embedded in main.rs.

**Components**:
- `App()` - Main container
- `Logo()` - Animated logo
- `Stats()` - Statistics grid
- `LiveLog()` - Request log table
- `LogEntry()` - Log entry renderer
- `ProgressBar()` - Activity indicator

**TODO**: Migrate from Alpine.js to Dioxus for better maintainability and type safety.

---

## Development Workflow

### Prerequisites

1. **Rust Toolchain**: Install via [rustup.rs](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **cloudflared**: Required for tunnel functionality
   ```bash
   # macOS
   brew install cloudflare/cloudflare/cloudflared

   # Linux
   curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /usr/local/bin/cloudflared
   chmod +x /usr/local/bin/cloudflared
   ```

3. **Cross-compilation (optional)**: For building ARM64 Linux binaries
   ```bash
   cargo install cross --git https://github.com/cross-rs/cross
   ```

### Building

```bash
# Development build
cargo build

# Release build (optimized for size)
cargo build --release

# Run directly
cargo run -- /tmp/test-file.txt

# Run with daemon mode
cargo run -- /tmp/test-file.txt --daemon

# Check tunnel status
cargo run -- --status

# Kill running daemon
cargo run -- --kill
```

### Development Tips

1. **Quick Testing**:
   ```bash
   # Create test file
   echo "Hello YEET" > /tmp/test.txt

   # Run in foreground
   cargo run -- /tmp/test.txt

   # In another terminal, test the tunnel URL
   curl <tunnel-url>/test.txt
   ```

2. **Debug Logging**:
   ```bash
   # Enable tracing logs
   RUST_LOG=debug cargo run -- /tmp/test.txt
   ```

3. **Formatting & Linting**:
   ```bash
   cargo fmt
   cargo clippy
   ```

4. **Checking Dependencies**:
   ```bash
   cargo tree
   cargo outdated  # Requires cargo-outdated
   ```

### Cross-Platform Testing

```bash
# macOS Apple Silicon (native)
cargo build --target aarch64-apple-darwin

# macOS Intel
cargo build --target x86_64-apple-darwin

# Linux x86_64
cargo build --target x86_64-unknown-linux-gnu

# Linux ARM64 (requires cross)
cross build --target aarch64-unknown-linux-gnu
```

---

## Coding Conventions

### Rust Style

1. **Follow Rust standard conventions**:
   - Use `rustfmt` defaults (no custom config)
   - Follow clippy recommendations
   - Use descriptive variable names

2. **Error Handling**:
   - Use `anyhow::Result<T>` for flexible error propagation
   - Use `context()` to add context to errors
   - Use `color-eyre` for panic hooks (already configured)

   ```rust
   use anyhow::{Context, Result};

   fn example() -> Result<()> {
       let state = read_tunnel_state()
           .context("Failed to read tunnel state")?;
       Ok(())
   }
   ```

3. **Async/Await**:
   - Use `tokio::spawn` for background tasks
   - Use `tokio::select!` for concurrent operations
   - Prefer structured concurrency

4. **Module Organization**:
   - Keep related functionality together
   - Use `pub(crate)` for internal APIs
   - Export only what's necessary from modules

### File Organization

1. **main.rs**:
   - Keep main() concise, delegate to functions
   - Group related functions together
   - Use clear section comments

2. **Embedded HTML**:
   - Keep inline HTML in main.rs for simplicity
   - Use constants for reusable HTML fragments
   - TODO: Migrate to Dioxus components when ready

3. **State Management**:
   - All persistent state goes in `TunnelState`
   - Use JSON for serialization (human-readable)
   - Store in `~/.yeet/tunnel.state`

### Naming Conventions

```rust
// Structs: PascalCase
struct TunnelState { }

// Functions: snake_case
fn spawn_daemon() { }

// Constants: SCREAMING_SNAKE_CASE
const DEFAULT_PORT: u16 = 8000;

// Module files: snake_case.rs
mod tunnel_manager;

// CLI flags: kebab-case
--daemon
--file-path
```

### Comments & Documentation

1. **Doc Comments** for public APIs:
   ```rust
   /// Spawns a background daemon process that runs the HTTP server and tunnel.
   ///
   /// # Arguments
   /// * `file_path` - Path to file or directory to share
   /// * `port` - Local port for HTTP server
   ///
   /// # Returns
   /// The PID of the spawned daemon process
   fn spawn_daemon(file_path: PathBuf, port: u16) -> Result<u32> { }
   ```

2. **Inline Comments** for complex logic:
   ```rust
   // Extract tunnel URL from cloudflared stderr using regex
   // Format: https://xxxxx-xxxxx.trycloudflare.com
   if let Some(captures) = tunnel_regex.captures(&line) { }
   ```

3. **TODO Comments** for planned work:
   ```rust
   // TODO: Replace hardcoded stats with real tracking
   // TODO: Migrate to Dioxus components from Alpine.js
   // TODO: Enable QR code feature when dependency issue is fixed
   ```

---

## Common Tasks

### Adding a New CLI Flag

1. **Edit `Cli` struct in main.rs**:
   ```rust
   #[derive(Parser)]
   struct Cli {
       #[arg(long)]
       new_flag: bool,
   }
   ```

2. **Handle flag in main()**:
   ```rust
   if cli.new_flag {
       // Handle new flag
   }
   ```

### Adding a New API Endpoint

1. **Define handler in main.rs** (around line 700-800):
   ```rust
   async fn new_endpoint() -> impl IntoResponse {
       Json(/* response data */)
   }
   ```

2. **Add route to Axum router**:
   ```rust
   let app = Router::new()
       .route("/api/new-endpoint", get(new_endpoint))
       // ... existing routes
   ```

### Modifying TUI Display

1. **Edit `App` struct** if new state is needed:
   ```rust
   struct App {
       // ... existing fields
       new_field: String,
   }
   ```

2. **Update `ui()` function** (line 1047-1100):
   ```rust
   fn ui(f: &mut Frame, app: &mut App) {
       // Add new widgets/text
       let new_paragraph = Paragraph::new(app.new_field.clone());
       f.render_widget(new_paragraph, area);
   }
   ```

### Adding Real Statistics Tracking

**Current Status**: Stats are hardcoded in main.rs (lines 95-130).

**Implementation Plan**:

1. **Create shared state with Arc<Mutex<>>**:
   ```rust
   use std::sync::{Arc, Mutex};

   #[derive(Clone)]
   struct AppState {
       stats: Arc<Mutex<ServerStats>>,
       logs: Arc<Mutex<Vec<RequestLog>>>,
   }
   ```

2. **Add middleware to track requests**:
   ```rust
   use tower::ServiceBuilder;
   use tower_http::trace::TraceLayer;

   let app = Router::new()
       .route("/", get(handler))
       .layer(ServiceBuilder::new()
           .layer(TraceLayer::new_for_http())
           .layer(middleware::from_fn(track_request)));
   ```

3. **Implement tracking middleware**:
   ```rust
   async fn track_request(
       req: Request<Body>,
       next: Next<Body>,
   ) -> impl IntoResponse {
       let start = Instant::now();
       let method = req.method().clone();
       let path = req.uri().path().to_string();

       let response = next.run(req).await;

       let duration = start.elapsed();
       // Update stats

       response
   }
   ```

### Migrating to Dioxus Components

**Current**: Alpine.js embedded in main.rs
**Goal**: Use Dioxus components from src/web/mod.rs

**Steps**:

1. **Remove Alpine.js HTML** from main.rs

2. **Integrate Dioxus SSR** in Axum handler:
   ```rust
   use dioxus::prelude::*;
   use dioxus_web::dioxus_core::VirtualDom;

   async fn admin_dashboard() -> impl IntoResponse {
       let mut vdom = VirtualDom::new(web::App);
       let _ = vdom.rebuild();
       let html = dioxus_ssr::render(&vdom);
       Html(html)
   }
   ```

3. **Update API calls** in Dioxus components to fetch from `/api/stats` and `/api/logs`

4. **Test thoroughly** across browsers

---

## Dependencies Guide

### Core Dependencies

#### Runtime & Web Server

| Crate | Version | Purpose | Key APIs |
|-------|---------|---------|----------|
| `tokio` | 1.42 | Async runtime | `tokio::spawn`, `tokio::select!`, `tokio::fs` |
| `axum` | 0.7 | HTTP framework | `Router`, `Json`, `Html`, `get()`, `post()` |
| `tower` | 0.5 | Service abstraction | Middleware, `ServiceBuilder` |
| `tower-http` | 0.6 | HTTP middleware | `ServeDir`, `TraceLayer` |

**Usage Notes**:
- All async code runs on Tokio runtime
- Axum handlers must return `impl IntoResponse`
- Use `tower-http::ServeDir` for static file serving

#### Terminal UI

| Crate | Version | Purpose | Key APIs |
|-------|---------|---------|----------|
| `ratatui` | 0.29 | TUI framework | `Terminal`, `Frame`, `Paragraph`, `Block` |
| `crossterm` | 0.28 | Terminal backend | `enable_raw_mode`, `event::read()` |
| `throbber-widgets-tui` | 0.8 | Spinner widget | `Throbber`, `ThrobberState` |

**Usage Notes**:
- Call `enable_raw_mode()` before TUI, `disable_raw_mode()` after
- Use `crossterm::event::poll()` for non-blocking input
- Render at ~30 FPS for smooth animations

#### CLI & Serialization

| Crate | Version | Purpose | Key APIs |
|-------|---------|---------|----------|
| `clap` | 4.5 | CLI parsing | `#[derive(Parser)]`, `#[arg(...)]` |
| `serde` | 1.0 | Serialization | `#[derive(Serialize, Deserialize)]` |
| `serde_json` | 1.0 | JSON codec | `to_string`, `from_str`, `json!()` |
| `regex` | 1.11 | Pattern matching | `Regex::new()`, `captures()` |

**Usage Notes**:
- Clap uses derive macros for zero-boilerplate CLI
- All API types must derive `Serialize + Deserialize`
- Compile regex patterns at runtime (or use `lazy_static`)

#### Error Handling

| Crate | Version | Purpose | Key APIs |
|-------|---------|---------|----------|
| `anyhow` | 1.0 | Flexible errors | `Result<T>`, `Context`, `bail!()` |
| `color-eyre` | 0.6 | Pretty panics | `install()` |

**Usage Notes**:
- Use `anyhow::Result` for all fallible functions
- Install `color-eyre` panic hook in main()
- Add context with `.context("message")?`

#### System Integration

| Crate | Version | Purpose | Key APIs |
|-------|---------|---------|----------|
| `nix` | 0.29 | Unix syscalls | `fork()`, `setsid()`, `kill()`, `dup2()` |
| `dirs` | 5.0 | User directories | `home_dir()`, `config_dir()` |

**Usage Notes**:
- `fork()` is unsafe, handle carefully
- Use `setsid()` to detach from terminal
- Signal handling with `kill()` for daemon management

### Optional Dependencies

#### Web UI (Not Yet Integrated)

| Crate | Version | Purpose | Status |
|-------|---------|---------|--------|
| `dioxus` | 0.6 | Web components | Components exist in web/mod.rs |
| `dioxus-web` | 0.6 | Web renderer | Not yet used |

#### Future Enhancements

```toml
# Commented out in Cargo.toml
# qrcode = "0.14"  # TODO: Enable when moxcms/image issue is fixed
# chrono = "0.4"   # Included but not actively used
# humansize = "2.1" # Included but not actively used
```

---

## Testing & Debugging

### Manual Testing

1. **Single File Sharing**:
   ```bash
   # Terminal 1: Start YEET
   cargo run -- /tmp/test.txt

   # Terminal 2: Test download
   curl <tunnel-url>/test.txt
   ```

2. **Directory Sharing**:
   ```bash
   # Create test directory
   mkdir -p /tmp/test-dir
   echo "File 1" > /tmp/test-dir/file1.txt
   echo "File 2" > /tmp/test-dir/file2.txt

   # Share directory
   cargo run -- /tmp/test-dir

   # Test browser UI
   open <tunnel-url>
   ```

3. **Daemon Mode**:
   ```bash
   # Start daemon
   cargo run -- /tmp/test.txt --daemon

   # Check status
   cargo run -- --status

   # Kill daemon
   cargo run -- --kill
   ```

4. **Admin Dashboard**:
   ```bash
   cargo run -- /tmp/test.txt
   # Visit <tunnel-url>/admin in browser
   ```

### Debugging Techniques

1. **Enable Tracing**:
   ```bash
   RUST_LOG=debug cargo run -- /tmp/test.txt
   RUST_LOG=yeet=trace cargo run -- /tmp/test.txt
   ```

2. **Check Tunnel State**:
   ```bash
   cat ~/.yeet/tunnel.state
   ```

3. **Debug Cloudflared**:
   ```bash
   # Run cloudflared manually
   cloudflared tunnel --url http://localhost:8000
   ```

4. **Check Process Status**:
   ```bash
   # Find YEET processes
   ps aux | grep yeet

   # Find cloudflared processes
   ps aux | grep cloudflared

   # Check if daemon is alive
   cargo run -- --status
   ```

5. **Network Debugging**:
   ```bash
   # Test local server
   curl http://localhost:8000/

   # Check if port is in use
   lsof -i :8000

   # Monitor HTTP requests
   cargo run -- /tmp/test.txt
   # Then: curl -v <tunnel-url>
   ```

### Common Issues & Solutions

#### Issue: "cloudflared not found"
**Solution**: Install cloudflared or ensure it's in PATH
```bash
which cloudflared
# If not found, install via install.sh or manually
```

#### Issue: "Port already in use"
**Solution**: Kill existing daemon or use different port
```bash
cargo run -- --kill
# OR
cargo run -- /tmp/test.txt --port 8001
```

#### Issue: "Tunnel state file not found"
**Solution**: This is normal for first run. File is created after daemon starts.

#### Issue: "Tunnel URL not appearing"
**Solution**: Wait up to 30 seconds. Check cloudflared is installed and accessible.

---

## Release Process

### Versioning

1. **Update version** in `Cargo.toml`:
   ```toml
   [package]
   version = "0.1.3"  # Increment based on semver
   ```

2. **Update version** in `install.sh`:
   ```bash
   VERSION="0.1.3"
   ```

3. **Update README.md** if needed (version numbers, new features)

4. **Commit changes**:
   ```bash
   git add Cargo.toml install.sh README.md
   git commit -m "Release v0.1.3: Description of changes"
   ```

### Creating a Release

1. **Create and push tag**:
   ```bash
   git tag v0.1.3
   git push origin v0.1.3
   ```

2. **CI/CD automatically**:
   - Builds for 4 platforms (x86_64/ARM64 on Linux/macOS)
   - Creates GitHub release
   - Uploads binaries as assets
   - Generates release notes

3. **Monitor GitHub Actions**:
   - Visit: https://github.com/akash-otonomy/yeet/actions
   - Ensure all builds pass
   - Verify release is created

### Build Matrix

The `.github/workflows/release.yml` builds for:

| Platform | Target Triple | Builder |
|----------|---------------|---------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `cargo build` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `cross build` |
| macOS Intel | `x86_64-apple-darwin` | `cargo build` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `cargo build` |

### Release Artifacts

After successful release, binaries are available at:
```
https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-x86_64-unknown-linux-gnu
https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-aarch64-unknown-linux-gnu
https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-x86_64-apple-darwin
https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-aarch64-apple-darwin
```

### Post-Release Checklist

- [ ] Verify all 4 binaries are attached to release
- [ ] Test install.sh on at least one platform
- [ ] Update documentation if needed
- [ ] Announce on relevant channels (if applicable)

---

## Important Gotchas

### 1. Daemon Process Management

**Issue**: Killing parent process doesn't kill daemon.

**Why**: Daemon uses `setsid()` to detach from parent's process group.

**Solution**: Always use `yeet --kill` or `kill -9 -<pid>` (negative PID to kill process group).

```bash
# Wrong (only kills parent)
kill <pid>

# Right (kills process group)
kill -9 -<pid>

# Best (uses built-in command)
yeet --kill
```

### 2. Tunnel State Persistence

**Issue**: Tunnel URLs change on restart even for same file.

**Why**: Cloudflare Quick Tunnels generate new URLs each time `cloudflared` starts.

**Current Behavior**:
- If daemon is alive, reuse existing tunnel
- If daemon died, new tunnel is created with new URL

**Future Enhancement**: Consider using named tunnels for persistent URLs.

### 3. File Path Changes

**Issue**: Daemon doesn't detect if shared file/directory changes.

**Critical Fix (v0.1.2)**: Daemon now restarts if file path changes.

**Code Reference**: `main.rs` lines 1180-1195
```rust
// Restart daemon when serving different file/directory
if let Some(state) = &tunnel_state {
    if state.file_path != cli.file_path.to_string_lossy().to_string() {
        eprintln!("File path changed, restarting daemon...");
        kill_daemon(state.pid)?;
        tunnel_state = None;
    }
}
```

### 4. Binary Size Optimization

**Profile Settings**: Optimized for size, not speed.

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
strip = true         # Strip debug symbols
```

**Implication**: Slightly slower runtime, but much smaller binary (~5-10 MB).

**Alternative**: For development, use `cargo build` without `--release` for faster compilation.

### 5. Hardcoded Statistics

**Current State**: `/api/stats` and `/api/logs` return mock data.

**Code Reference**: `main.rs` lines 95-130

**TODO**: Implement real request tracking with shared state.

**See**: [Adding Real Statistics Tracking](#adding-real-statistics-tracking) section.

### 6. Alpine.js vs Dioxus

**Current**: Admin dashboard and directory browser use Alpine.js (embedded HTML strings).

**Future**: Migrate to Dioxus components in `src/web/mod.rs`.

**Why Not Now**: Dioxus components exist but aren't yet integrated. Alpine.js is simpler for MVP.

**Migration Path**: See [Migrating to Dioxus Components](#migrating-to-dioxus-components) section.

### 7. Cross-Compilation for ARM64 Linux

**Issue**: `cargo build --target aarch64-unknown-linux-gnu` fails on x86_64 without cross.

**Solution**: Use `cross` tool (Docker-based cross-compiler).

```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-gnu
```

**Note**: GitHub Actions workflow handles this automatically.

### 8. Cloudflared Dependency

**Critical**: YEET requires `cloudflared` binary in PATH.

**Handled By**: `install.sh` automatically installs cloudflared.

**Manual Install**:
```bash
# macOS
brew install cloudflare/cloudflare/cloudflared

# Linux
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /usr/local/bin/cloudflared
chmod +x /usr/local/bin/cloudflared
```

### 9. File Descriptor Redirection

**Code**: Daemon redirects stdin/stdout/stderr to `/dev/null`.

**Why**: Prevents interference with terminal after parent exits.

**Code Reference**: `main.rs` lines 970-985
```rust
let devnull = File::open("/dev/null")?;
dup2(devnull.as_raw_fd(), 0)?; // stdin
dup2(devnull.as_raw_fd(), 1)?; // stdout
dup2(devnull.as_raw_fd(), 2)?; // stderr
```

**Implication**: Daemon logs are not visible. Future: consider logging to file.

### 10. Retro Color Theme

**Palette**: Defined in `src/tui/theme.rs`.

**Consistency**: Use these colors throughout TUI and web dashboards for cohesive aesthetic.

```rust
CYAN: #00FFFF
MAGENTA: #FF00FF
YELLOW: #FFFF00
GREEN: #00FF9F
DARK_GRAY: #404040
```

**Web CSS**: Embedded dashboards use same hex values.

---

## Additional Resources

### Documentation Links

- **Ratatui Guide**: https://ratatui.rs/
- **Axum Docs**: https://docs.rs/axum/
- **Tokio Tutorial**: https://tokio.rs/tokio/tutorial
- **Cloudflare Tunnels**: https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/
- **Rust Book**: https://doc.rust-lang.org/book/

### Useful Commands

```bash
# Check current version
yeet --version  # (TODO: implement version flag)

# View help
yeet --help

# Build documentation
cargo doc --open

# Check for unused dependencies
cargo +nightly udeps

# Security audit
cargo audit

# Benchmark (if benchmarks added)
cargo bench
```

### Project Structure Best Practices

1. **Keep main.rs focused**: Delegate complex logic to modules
2. **Use type system**: Leverage Rust's type safety
3. **Prefer composition**: Use traits and generics over inheritance
4. **Document public APIs**: Always add doc comments
5. **Test edge cases**: Especially daemon lifecycle and file serving

---

## Contributing Guidelines

When contributing to YEET:

1. **Fork and branch**: Create feature branches from `main`
2. **Follow conventions**: Match existing code style
3. **Test locally**: Ensure builds and runs on your platform
4. **Update docs**: Update this file if architecture changes
5. **Small PRs**: Keep changes focused and reviewable
6. **Descriptive commits**: Use conventional commit messages

### Conventional Commits Format

```
<type>: <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples**:
```
feat: Add QR code generation for tunnel URLs
fix: Restart daemon when serving different file/directory
docs: Update CLAUDE.md with new API endpoints
refactor: Extract statistics tracking to separate module
```

---

## Quick Reference

### File Locations

```
Main binary:           src/main.rs
Data structures:       src/shared/mod.rs
TUI rendering:         src/tui/mod.rs
Color theme:           src/tui/theme.rs
Web components:        src/web/mod.rs (not yet used)
Dependencies:          Cargo.toml
CI/CD:                 .github/workflows/release.yml
Installer:             install.sh
Tunnel state:          ~/.yeet/tunnel.state (runtime)
```

### Key Functions

```rust
main()                      // Entry point, CLI parsing
spawn_daemon()              // Fork and daemonize
run_daemon_server()         // Start Axum HTTP server
start_cloudflared_daemon()  // Spawn tunnel, extract URL
ui()                        // TUI render function
run_app()                   // TUI event loop
```

### Port & Paths

```
Default port:    8000
Config dir:      ~/.yeet/
State file:      ~/.yeet/tunnel.state
Install path:    /usr/local/bin/yeet (or ~/.local/bin/yeet)
```

### Build Targets

```
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
```

---

**Last Updated**: 2025-11-20
**Maintainer**: Claude Code (AI Assistant)
**For Questions**: See README.md or open an issue on GitHub
