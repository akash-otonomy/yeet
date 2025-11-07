# 🚀 yeet.sh

> **Yeet files across the internet at warp speed**

A blazingly fast™ retro-futuristic TUI file transfer tool powered by Cloudflare tunnels.

```
╦ ╦ ╔═╗ ╔═╗ ╔╦╗   ╔═╗ ╦ ╦
╚═╝ ║╣  ║╣   ║    ╚═╗ ╠═╣
╩   ╚═╝ ╚═╝  ╩  ╔ ╚═╝ ╩ ╩

    Y  E  E  T  .  S  H
   ━━━━━━━━━━━━━━━━━━━━
    WARP SPEED FILE XFER
```

## ✨ Features

- 🎨 **Retro-futuristic TUI** with neon cyberpunk aesthetics
- ⚡ **Lightning fast** - Faster than SCP/SFTP (no SSH overhead)
- 🌐 **Cloudflare tunnels** - Works behind NAT/firewalls
- 🔒 **Secure** - Only shares the file you specify
- 📦 **Any file size** - Transfer gigabytes without limits
- 🦀 **Rust powered** - Single binary, zero dependencies

## 🚀 Quick Start

```bash
# Yeet a file
yeet myfile.zip

# That's it! You'll get a public URL instantly
```

## 📦 Installation

### macOS / Linux

**Curl installer** (easiest):
```bash
curl -sSL https://yeet.sh/install | bash
```

**Homebrew**:
```bash
brew install yeet-sh
```

**From source**:
```bash
cargo install yeet-sh
```

### Requirements

- `cloudflared` (auto-installed by our installer, or: `brew install cloudflared`)
- `python3` (usually pre-installed)

## 🎯 Usage

```bash
# Basic usage
yeet path/to/file.zip

# Custom port
yeet --port 9000 bigfile.tar.gz

# Help
yeet --help
```

## 🎨 Why yeet.sh?

**Traditional file transfer sucks:**
- ❌ SCP is slow (SSH encryption overhead)
- ❌ SFTP requires SSH setup
- ❌ Email has size limits
- ❌ Dropbox/Drive requires uploading first
- ❌ WeTransfer has ads and limits

**yeet.sh is different:**
- ✅ One command to share
- ✅ Instant public URL
- ✅ Works anywhere (NAT/firewall friendly)
- ✅ No size limits
- ✅ Beautiful retro UI
- ✅ Free forever

## 🏗️ How It Works

1. Creates isolated temp directory with your file
2. Starts local HTTP server
3. Opens Cloudflare tunnel
4. Gives you a public HTTPS URL
5. Auto-cleanup on exit

## 🛠️ Development

```bash
# Clone
git clone https://github.com/yourusername/yeet.sh
cd yeet.sh

# Build
cargo build --release

# Run
cargo run -- testfile.txt
```

## 🎮 Controls

- `q` or `Esc` - Quit
- `Ctrl+C` - Quit

## 🤝 Contributing

PRs welcome! Let's make file transfer cool again.

## 📝 License

MIT License - YEET freely!

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=yourusername/yeet.sh&type=Date)](https://star-history.com/#yourusername/yeet.sh&Date)

---

**Made with 🦀 Rust + ❤️ by developers who are tired of slow file transfers**

*YEET YOUR FILES. YEET YOUR PROBLEMS. YEET INTO THE FUTURE.*
