#!/bin/bash
# BotBrowser Launcher Setup Script for Ubuntu/Debian
# Run: curl -fsSL https://raw.githubusercontent.com/botswin/BotBrowser/main/launcher/scripts/setup-ubuntu.sh -o /tmp/setup.sh && bash /tmp/setup.sh

set -e

NODE_VERSION="24.15.0"
REPO_ZIP_URL="https://github.com/botswin/BotBrowser/archive/refs/heads/main.zip"
INSTALL_DIR="$HOME/.botbrowser"
NODE_DIR="$INSTALL_DIR/node"
NPM_CACHE_DIR="$INSTALL_DIR/npm-cache"
REPO_DIR="$INSTALL_DIR/BotBrowser"
DIST_DIR="$REPO_DIR/launcher/dist/BotBrowserLauncher"
ICON_PATH="$REPO_DIR/launcher/public/logo.svg"
DESKTOP_FILE="$HOME/Desktop/BotBrowser-Launcher.desktop"
APPS_DESKTOP_FILE="$HOME/.local/share/applications/BotBrowser-Launcher.desktop"

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    NODE_URL="https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-arm64.tar.xz"
    NODE_EXTRACTED="node-v${NODE_VERSION}-linux-arm64"
    EXE_NAME="BotBrowserLauncher-linux_arm64"
else
    NODE_URL="https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz"
    NODE_EXTRACTED="node-v${NODE_VERSION}-linux-x64"
    EXE_NAME="BotBrowserLauncher-linux_x64"
fi

EXE_PATH="$DIST_DIR/$EXE_NAME"

install_dependencies() {
    if command -v unzip &> /dev/null && command -v curl &> /dev/null && command -v xz &> /dev/null; then
        return
    fi
    echo "Installing required tools..."
    sudo apt-get update
    sudo apt-get install -y curl unzip xz-utils
}

install_nodejs() {
    if [ -f "$NODE_DIR/bin/node" ]; then
        installed=$("$NODE_DIR/bin/node" --version)
        if [ "$installed" = "v${NODE_VERSION}" ]; then
            echo "Node.js v${NODE_VERSION} already installed."
            return
        fi
        echo "Replacing Node.js $installed with v${NODE_VERSION} (Angular 22 requires >= 24.15.0)..."
        rm -rf "$NODE_DIR"
    fi

    echo "Downloading Node.js v${NODE_VERSION} for ${ARCH}..."
    curl -fsSL "$NODE_URL" -o /tmp/node.tar.xz

    echo "Extracting Node.js..."
    tar -xJf /tmp/node.tar.xz -C "$INSTALL_DIR"
    mv "$INSTALL_DIR/$NODE_EXTRACTED" "$NODE_DIR"
    rm /tmp/node.tar.xz
    echo "Node.js installed."
}

install_repository() {
    if [ -d "$REPO_DIR" ]; then
        rm -rf "$REPO_DIR"
    fi
    echo "Downloading repository..."
    curl -fsSL "$REPO_ZIP_URL" -o /tmp/botbrowser.zip

    echo "Extracting repository..."
    unzip -q /tmp/botbrowser.zip -d "$INSTALL_DIR"
    mv "$INSTALL_DIR/BotBrowser-main" "$REPO_DIR"
    rm /tmp/botbrowser.zip
}

build_application() {
    export PATH="$NODE_DIR/bin:$PATH"
    echo "Node.js version: $("$NODE_DIR/bin/node" --version)"

    cd "$REPO_DIR/launcher"

    echo "Installing dependencies..."
    mkdir -p "$NPM_CACHE_DIR"
    export NPM_CONFIG_CACHE="$NPM_CACHE_DIR"
    export NPM_CONFIG_UPDATE_NOTIFIER=false
    "$NODE_DIR/bin/npm" ci

    echo "Building application..."
    "$NODE_DIR/bin/npm" run build
}

create_desktop_shortcut() {
    echo "Creating desktop shortcut..."

    # Create .desktop file content
    DESKTOP_CONTENT="[Desktop Entry]
Name=BotBrowser Launcher
Comment=BotBrowser Profile Manager
Exec=$EXE_PATH
Icon=$ICON_PATH
Terminal=false
Type=Application
Categories=Utility;Network;"

    # Create desktop shortcut
    mkdir -p "$(dirname "$DESKTOP_FILE")"
    echo "$DESKTOP_CONTENT" > "$DESKTOP_FILE"
    chmod +x "$DESKTOP_FILE"

    # Also add to application menu
    mkdir -p "$(dirname "$APPS_DESKTOP_FILE")"
    echo "$DESKTOP_CONTENT" > "$APPS_DESKTOP_FILE"

    # Trust the desktop file (GNOME)
    if command -v gio &> /dev/null; then
        gio set "$DESKTOP_FILE" metadata::trusted true 2>/dev/null || true
    fi

    echo "Desktop shortcut created: $DESKTOP_FILE"
}

save_commit_hash() {
    echo "Saving launcher version info..."
    local commit_dir="$HOME/.config/BotBrowser"
    mkdir -p "$commit_dir"
    local sha
    sha=$(curl -fsSL -H "Accept: application/vnd.github.v3+json" "https://api.github.com/repos/botswin/BotBrowser/commits?path=launcher&sha=main&per_page=1" 2>/dev/null | grep '"sha"' | head -1 | sed 's/.*"sha": *"\([^"]*\)".*/\1/')
    if [ -n "$sha" ]; then
        printf '%s' "$sha" > "$commit_dir/launcher-commit"
        echo "  Version: ${sha:0:7}"
    else
        echo "  Warning: Could not save version info."
    fi
}

launch_application() {
    echo "Starting BotBrowser Launcher..."
    cd "$DIST_DIR"
    "./$EXE_NAME" &
    disown
}

# === Main ===

echo "=== BotBrowser Launcher Setup ==="

# Create install directory
mkdir -p "$INSTALL_DIR"

# Install system dependencies
install_dependencies

# Check if already installed
if [ -f "$EXE_PATH" ]; then
    echo "BotBrowser Launcher is already installed."
    read -p "Update to latest version? (y = update, n = launch existing) [n]: " choice < /dev/tty
    choice=${choice:-n}

    if [ "$choice" = "y" ] || [ "$choice" = "Y" ]; then
        echo "Updating..."
        install_nodejs
        install_repository
        build_application
        save_commit_hash
        create_desktop_shortcut
        launch_application
    else
        launch_application
    fi
else
    echo "First time setup..."
    install_nodejs
    install_repository
    build_application
    save_commit_hash
    create_desktop_shortcut
    launch_application
fi
