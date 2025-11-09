#!/bin/bash
set -e

# 🚀 YEET.SH - One-liner installer for RunPod/Linux VMs
# curl -fsSL https://YOUR-TUNNEL.trycloudflare.com/install.sh | bash

echo "🚀 Installing YEET.SH..."

INSTALL_DIR="${HOME}/.local/bin"
TUNNEL_URL="${TUNNEL_URL:-https://guided-carry-pour-lakes.trycloudflare.com}"

mkdir -p "$INSTALL_DIR"

# Install cloudflared
if ! command -v cloudflared &> /dev/null; then
    echo "📥 Installing cloudflared..."
    curl -fsSL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o "$INSTALL_DIR/cloudflared"
    chmod +x "$INSTALL_DIR/cloudflared"
    echo "✅ cloudflared installed"
fi

# Download pre-built binary
echo "📥 Downloading yeet binary..."
curl -fsSL "$TUNNEL_URL/yeet" -o "$INSTALL_DIR/yeet"
chmod +x "$INSTALL_DIR/yeet"
echo "✅ yeet installed"

# Add to PATH
if ! grep -q ".local/bin" ~/.bashrc 2>/dev/null; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
    echo "Added to PATH"
fi

export PATH="$HOME/.local/bin:$PATH"

echo ""
echo "✅ Installation complete!"
echo ""
echo "Run:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\"  # or source ~/.bashrc"
echo "  yeet /workspace/your-data                 # Yeet your files!"
echo ""
