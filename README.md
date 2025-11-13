# YEET 🚀


https://github.com/user-attachments/assets/e0678561-5559-4e8a-8dd4-44f363f26bf2


> Yeet files across the internet at warp speed

Zero-config file sharing via Cloudflare tunnels with a retro TUI and web dashboard.

## Features

- **Zero Configuration**: No account required, no config files
- **Instant Public URLs**: Uses Cloudflare Quick Tunnels
- **Retro TUI**: Colorful terminal interface with 8-bit aesthetic
- **Admin Dashboard**: Real-time stats at `/admin`
- **Daemon Mode**: Tunnel stays alive in background
- **Directory Support**: Share entire folders with file browser
- **One-liner Install**: Deploy to RunPod/Linux VMs instantly

## Installation

### One-liner (Recommended)

**macOS / Linux**
```bash
curl -fsSL https://raw.githubusercontent.com/akash-otonomy/yeet/master/install.sh | bash
```

This automatically:
- ✅ Detects your OS and architecture
- ✅ Downloads the correct binary
- ✅ Installs cloudflared dependency
- ✅ Sets up permissions

### Manual Install (Pre-built binaries)

**macOS (Apple Silicon)**
```bash
curl -L https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-aarch64-apple-darwin -o yeet
chmod +x yeet
sudo mv yeet /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -L https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-x86_64-apple-darwin -o yeet
chmod +x yeet
sudo mv yeet /usr/local/bin/
```

**Linux x86_64**
```bash
curl -L https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-x86_64-unknown-linux-gnu -o yeet
chmod +x yeet
sudo mv yeet /usr/local/bin/
```

**Linux ARM64 (RunPod GPU servers)**
```bash
curl -L https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-aarch64-unknown-linux-gnu -o yeet
chmod +x yeet
# For RunPod, use ~/.local/bin instead
mkdir -p ~/.local/bin
mv yeet ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

### From Source (requires Rust)

First [install Rust](https://rustup.rs/), then:
```bash
cargo install --git https://github.com/akash-otonomy/yeet
```

## Usage

### Share a single file
```bash
yeet /path/to/file.zip
```

### Share a directory
```bash
yeet /path/to/directory
```

### Daemon mode (background)
```bash
yeet /workspace/data --daemon
```

### Check tunnel status
```bash
yeet --status
```

### Stop daemon
```bash
yeet --kill
```

## Screenshots

```
  ██╗   ██╗███████╗███████╗████████╗   v0.1.0
  ╚██╗ ██╔╝██╔════╝██╔════╝╚══██╔══╝
   ╚████╔╝ █████╗  █████╗     ██║
    ╚██╔╝  ██╔══╝  ██╔══╝     ██║
     ██║   ███████╗███████╗   ██║
     ╚═╝   ╚══════╝╚══════╝   ╚═╝
>> yeet stuff across the internet
```

## Tech Stack

- **Rust** - Blazing fast and memory safe
- **Ratatui** - Terminal UI framework
- **Axum** - Web server
- **Cloudflare Tunnels** - Instant public URLs
- **Dioxus** - Web UI components (ready for integration)

## Development

```bash
# Clone repo
git clone https://github.com/akash-otonomy/yeet.git
cd yeet

# Build
cargo build --release

# Run
cargo run -- /tmp/test-file.txt
```

## License

MIT

## Contributing

Contributions welcome! This is a fun side project - feel free to open issues or PRs.

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
