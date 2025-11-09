# YEET 🚀

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

### macOS/Linux (from source)
```bash
cargo install --git https://github.com/akash-otonomy/yeet
```

### RunPod/Linux VMs (one-liner)
```bash
# Coming soon - GitHub release with pre-built binaries
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
