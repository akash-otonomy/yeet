#!/bin/bash
set -e

echo "🚀 Installing YEET.SH on RunPod..."

INSTALL_DIR="${HOME}/.local/bin"
TUNNEL="https://guided-carry-pour-lakes.trycloudflare.com"

mkdir -p "$INSTALL_DIR"

# Install cloudflared
if ! command -v cloudflared &> /dev/null; then
    echo "📥 Installing cloudflared..."
    curl -fsSL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o "$INSTALL_DIR/cloudflared"
    chmod +x "$INSTALL_DIR/cloudflared"
fi

# Install Rust if needed
if ! command -v cargo &> /dev/null; then
    echo "📥 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    source "$HOME/.cargo/env"
else
    source "$HOME/.cargo/env" 2>/dev/null || true
fi

# Download source
echo "📥 Downloading source..."
cd /tmp && rm -rf yeet.sh
mkdir -p yeet.sh/src/tui yeet.sh/src/web yeet.sh/src/shared yeet.sh/assets
cd yeet.sh

curl -fsSL "$TUNNEL/Cargo.toml" -o Cargo.toml
curl -fsSL "$TUNNEL/Cargo.lock" -o Cargo.lock
curl -fsSL "$TUNNEL/src/main.rs" -o src/main.rs
curl -fsSL "$TUNNEL/src/tui/mod.rs" -o src/tui/mod.rs 2>/dev/null || true
curl -fsSL "$TUNNEL/src/tui/theme.rs" -o src/tui/theme.rs 2>/dev/null || true
curl -fsSL "$TUNNEL/src/web/mod.rs" -o src/web/mod.rs 2>/dev/null || true
curl -fsSL "$TUNNEL/src/shared/mod.rs" -o src/shared/mod.rs 2>/dev/null || true
curl -fsSL "$TUNNEL/assets/style.css" -o assets/style.css 2>/dev/null || true
echo "🔨 Building yeet..."
cargo build --release
cp target/release/yeet "$INSTALL_DIR/yeet"

# Add to PATH
grep -q ".local/bin" ~/.bashrc || echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
export PATH="$HOME/.local/bin:$PATH"

echo "✅ Done! Run: yeet /workspace/your-data"
