#!/bin/bash
set -e

VERSION="0.1.1"

echo "🚀 Installing YEET v${VERSION}..."

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
                CLOUDFLARED_ARCH="amd64"
                ;;
            arm64)
                TARGET="aarch64-apple-darwin"
                CLOUDFLARED_ARCH="arm64"
                ;;
            *)
                echo "❌ Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        CLOUDFLARED_OS="darwin"
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "📦 Detected: $OS $ARCH -> $TARGET"

# Remove existing installations
if command -v yeet &> /dev/null; then
    EXISTING_PATH=$(command -v yeet)
    echo "🔄 Found existing installation at: $EXISTING_PATH"
    echo "   Removing old version..."
    if [ -w "$(dirname "$EXISTING_PATH")" ]; then
        rm -f "$EXISTING_PATH"
    else
        sudo rm -f "$EXISTING_PATH"
    fi
fi

# Determine install directory (prefer global)
INSTALL_DIR="/usr/local/bin"
NEEDS_SUDO=false

if [ ! -d "$INSTALL_DIR" ]; then
    sudo mkdir -p "$INSTALL_DIR" 2>/dev/null || {
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    }
elif [ ! -w "$INSTALL_DIR" ]; then
    NEEDS_SUDO=true
fi

echo "📂 Installing to: $INSTALL_DIR"

# Download yeet binary
YEET_URL="https://github.com/akash-otonomy/yeet/releases/latest/download/yeet-$TARGET"
TEMP_FILE="/tmp/yeet-install-$$"
echo "⬇️  Downloading yeet..."

if command -v curl &> /dev/null; then
    curl -fsSL "$YEET_URL" -o "$TEMP_FILE"
elif command -v wget &> /dev/null; then
    wget -q "$YEET_URL" -O "$TEMP_FILE"
else
    echo "❌ Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Install binary
chmod +x "$TEMP_FILE"
if [ "$NEEDS_SUDO" = true ]; then
    sudo mv "$TEMP_FILE" "$INSTALL_DIR/yeet"
else
    mv "$TEMP_FILE" "$INSTALL_DIR/yeet"
fi

echo "✅ yeet installed"

# Check if cloudflared is already installed
if command -v cloudflared &> /dev/null; then
    echo "✅ cloudflared already installed"
else
    echo "⬇️  Installing cloudflared..."
    TEMP_CF="/tmp/cloudflared-install-$$"

    if [ "$CLOUDFLARED_OS" = "darwin" ]; then
        # macOS binaries are distributed as .tgz archives
        CLOUDFLARED_URL="https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-${CLOUDFLARED_OS}-${CLOUDFLARED_ARCH}.tgz"

        if command -v curl &> /dev/null; then
            curl -fsSL "$CLOUDFLARED_URL" -o "${TEMP_CF}.tgz"
        else
            wget -q "$CLOUDFLARED_URL" -O "${TEMP_CF}.tgz"
        fi

        tar -xzf "${TEMP_CF}.tgz" -C "/tmp"
        mv "/tmp/cloudflared" "$TEMP_CF"
        rm "${TEMP_CF}.tgz"
    else
        # Linux binaries are distributed as plain executables
        CLOUDFLARED_URL="https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-${CLOUDFLARED_OS}-${CLOUDFLARED_ARCH}"

        if command -v curl &> /dev/null; then
            curl -fsSL "$CLOUDFLARED_URL" -o "$TEMP_CF"
        else
            wget -q "$CLOUDFLARED_URL" -O "$TEMP_CF"
        fi
    fi

    chmod +x "$TEMP_CF"
    if [ "$NEEDS_SUDO" = true ]; then
        sudo mv "$TEMP_CF" "$INSTALL_DIR/cloudflared"
    else
        mv "$TEMP_CF" "$INSTALL_DIR/cloudflared"
    fi

    echo "✅ cloudflared installed"
fi

echo ""
echo "✅ Installation complete! YEET v${VERSION}"
echo ""

# Check if PATH needs updating (only if installed to non-standard location)
if [[ "$INSTALL_DIR" != "/usr/local/bin" ]] && [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Add $INSTALL_DIR to your PATH:"
    echo ""
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
    echo ""
    if [ -f "$HOME/.zshrc" ]; then
        echo "   Run: echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        echo "   Run: echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
    else
        echo "   Add this to your shell config file (~/.bashrc or ~/.zshrc)"
    fi
    echo ""
fi

echo "Usage:"
echo "  yeet <file-or-directory>     # Share a file or directory"
echo "  yeet --status                # Check tunnel status"
echo "  yeet --kill                  # Stop background tunnel"
echo ""
echo "Example:"
echo "  yeet /path/to/file.zip"
echo ""
