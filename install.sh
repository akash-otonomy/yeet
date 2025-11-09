#!/bin/bash
set -e

echo "🚀 Installing YEET..."

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-unknown-linux-gnu"
                CLOUDFLARED_ARCH="amd64"
                ;;
            aarch64|arm64)
                TARGET="aarch64-unknown-linux-gnu"
                CLOUDFLARED_ARCH="arm64"
                ;;
            *)
                echo "❌ Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        CLOUDFLARED_OS="linux"
        ;;
    Darwin*)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-apple-darwin"
                ;;
            arm64)
                TARGET="aarch64-apple-darwin"
                ;;
            *)
                echo "❌ Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        CLOUDFLARED_OS="darwin"
        CLOUDFLARED_ARCH="amd64"  # Cloudflare only provides amd64 for macOS (works with Rosetta)
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "📦 Detected: $OS $ARCH -> $TARGET"

# Set install directory
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo "📂 Installing to: $INSTALL_DIR"

# Download yeet binary
YEET_URL="https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-$TARGET"
echo "⬇️  Downloading yeet..."

if command -v curl &> /dev/null; then
    curl -fsSL "$YEET_URL" -o "$INSTALL_DIR/yeet"
elif command -v wget &> /dev/null; then
    wget -q "$YEET_URL" -O "$INSTALL_DIR/yeet"
else
    echo "❌ Neither curl nor wget found. Please install one of them."
    exit 1
fi

chmod +x "$INSTALL_DIR/yeet"
echo "✅ yeet installed"

# Check if cloudflared is already installed
if command -v cloudflared &> /dev/null; then
    echo "✅ cloudflared already installed"
else
    echo "⬇️  Installing cloudflared..."
    CLOUDFLARED_URL="https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-${CLOUDFLARED_OS}-${CLOUDFLARED_ARCH}"
    
    if command -v curl &> /dev/null; then
        curl -fsSL "$CLOUDFLARED_URL" -o "$INSTALL_DIR/cloudflared"
    else
        wget -q "$CLOUDFLARED_URL" -O "$INSTALL_DIR/cloudflared"
    fi
    
    chmod +x "$INSTALL_DIR/cloudflared"
    echo "✅ cloudflared installed"
fi

# Add to PATH if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "⚠️  Add $INSTALL_DIR to your PATH:"
    echo "   export PATH=\"$INSTALL_DIR:\$PATH\""
    echo ""
    echo "   Add this to your ~/.bashrc or ~/.zshrc to make it permanent"
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "Usage:"
echo "  yeet <file-or-directory>     # Share a file or directory"
echo "  yeet --status                # Check tunnel status"
echo "  yeet --kill                  # Stop background tunnel"
echo ""
echo "Example:"
echo "  yeet /path/to/file.zip"
echo ""
