#!/bin/bash
# BotBrowser Launcher Setup Script for macOS
# Run: curl -fsSL https://raw.githubusercontent.com/botswin/BotBrowser/main/launcher/scripts/setup-macos.sh -o /tmp/setup.sh && bash /tmp/setup.sh

set -e

NODE_VERSION="24.15.0"
REPO_ZIP_URL="https://github.com/botswin/BotBrowser/archive/refs/heads/main.zip"
INSTALL_DIR="$HOME/.botbrowser"
NODE_DIR="$INSTALL_DIR/node"
NPM_CACHE_DIR="$INSTALL_DIR/npm-cache"
REPO_DIR="$INSTALL_DIR/BotBrowser"
DIST_DIR="$REPO_DIR/launcher/dist/BotBrowserLauncher"
DESKTOP_APP="$HOME/Desktop/BotBrowser Launcher.app"
APPLICATIONS_APP="/Applications/BotBrowser Launcher.app"

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    NODE_URL="https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-darwin-arm64.tar.gz"
    NODE_EXTRACTED="node-v${NODE_VERSION}-darwin-arm64"
    EXE_NAME="BotBrowserLauncher-mac_arm64"
else
    NODE_URL="https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-darwin-x64.tar.gz"
    NODE_EXTRACTED="node-v${NODE_VERSION}-darwin-x64"
    EXE_NAME="BotBrowserLauncher-mac_x64"
fi

EXE_PATH="$DIST_DIR/$EXE_NAME"

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
    curl -fsSL "$NODE_URL" -o /tmp/node.tar.gz

    echo "Extracting Node.js..."
    tar -xzf /tmp/node.tar.gz -C "$INSTALL_DIR"
    mv "$INSTALL_DIR/$NODE_EXTRACTED" "$NODE_DIR"
    rm /tmp/node.tar.gz
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

generate_icns() {
    local icns_path="$1"
    local svg_src="$REPO_DIR/launcher/public/logo.svg"
    local tmp_png="/tmp/botbrowser_icon.png"
    local iconset_dir="/tmp/BotBrowser.iconset"

    echo "  Generating app icon..."

    # Render SVG to 1024px PNG using Quick Look (built into macOS)
    qlmanage -t -s 1024 -o /tmp "$svg_src" > /dev/null 2>&1
    mv "/tmp/logo.svg.png" "$tmp_png" 2>/dev/null || true

    if [ ! -f "$tmp_png" ]; then
        echo "  Warning: Could not generate icon (qlmanage failed), skipping."
        return
    fi

    # Create iconset with all required sizes
    rm -rf "$iconset_dir"
    mkdir -p "$iconset_dir"
    sips -z 16 16 "$tmp_png" --out "$iconset_dir/icon_16x16.png" > /dev/null 2>&1
    sips -z 32 32 "$tmp_png" --out "$iconset_dir/icon_16x16@2x.png" > /dev/null 2>&1
    sips -z 32 32 "$tmp_png" --out "$iconset_dir/icon_32x32.png" > /dev/null 2>&1
    sips -z 64 64 "$tmp_png" --out "$iconset_dir/icon_32x32@2x.png" > /dev/null 2>&1
    sips -z 128 128 "$tmp_png" --out "$iconset_dir/icon_128x128.png" > /dev/null 2>&1
    sips -z 256 256 "$tmp_png" --out "$iconset_dir/icon_128x128@2x.png" > /dev/null 2>&1
    sips -z 256 256 "$tmp_png" --out "$iconset_dir/icon_256x256.png" > /dev/null 2>&1
    sips -z 512 512 "$tmp_png" --out "$iconset_dir/icon_256x256@2x.png" > /dev/null 2>&1
    sips -z 512 512 "$tmp_png" --out "$iconset_dir/icon_512x512.png" > /dev/null 2>&1
    sips -z 1024 1024 "$tmp_png" --out "$iconset_dir/icon_512x512@2x.png" > /dev/null 2>&1

    # Create .icns from iconset
    iconutil -c icns "$iconset_dir" -o "$icns_path" 2>/dev/null

    # Cleanup temp files
    rm -rf "$iconset_dir" "$tmp_png"

    if [ -f "$icns_path" ]; then
        echo "  Icon generated successfully."
    else
        echo "  Warning: iconutil failed, app will have no icon."
    fi
}

create_app_bundle() {
    local target_path="$1"
    rm -rf "$target_path"

    mkdir -p "$target_path/Contents/MacOS"
    mkdir -p "$target_path/Contents/Resources"

    # Generate icon if not already cached
    local icns_cache="/tmp/BotBrowser.icns"
    if [ ! -f "$icns_cache" ]; then
        generate_icns "$icns_cache"
    fi

    # Copy icon to bundle
    if [ -f "$icns_cache" ]; then
        cp "$icns_cache" "$target_path/Contents/Resources/AppIcon.icns"
    fi

    cat > "$target_path/Contents/MacOS/launcher" << LAUNCHER
#!/bin/bash
cd "$DIST_DIR"
exec "./$EXE_NAME"
LAUNCHER
    chmod +x "$target_path/Contents/MacOS/launcher"

    cat > "$target_path/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>BotBrowser Launcher</string>
    <key>CFBundleDisplayName</key>
    <string>BotBrowser Launcher</string>
    <key>CFBundleIdentifier</key>
    <string>com.botbrowser.launcher</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleExecutable</key>
    <string>launcher</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
</dict>
</plist>
PLIST
}

create_shortcuts() {
    echo "Creating shortcuts..."

    # Desktop shortcut
    create_app_bundle "$DESKTOP_APP"
    echo "  Desktop shortcut: $DESKTOP_APP"

    # /Applications shortcut (try with sudo, fallback to ~/Applications)
    if [ -w "/Applications" ]; then
        create_app_bundle "$APPLICATIONS_APP"
        echo "  Applications shortcut: $APPLICATIONS_APP"
    else
        echo "  Installing to /Applications requires admin privileges."
        local tmp_app="/tmp/BotBrowser Launcher.app"
        create_app_bundle "$tmp_app"
        if sudo -n true 2>/dev/null || sudo true; then
            sudo rm -rf "$APPLICATIONS_APP"
            sudo mv "$tmp_app" "$APPLICATIONS_APP"
            echo "  Applications shortcut: $APPLICATIONS_APP"
        else
            echo "  Admin access denied, falling back to ~/Applications."
            mkdir -p "$HOME/Applications"
            rm -rf "$HOME/Applications/BotBrowser Launcher.app"
            mv "$tmp_app" "$HOME/Applications/BotBrowser Launcher.app"
            echo "  Applications shortcut: $HOME/Applications/BotBrowser Launcher.app"
        fi
    fi
}

save_commit_hash() {
    echo "Saving launcher version info..."
    local commit_dir="$HOME/Library/Application Support/BotBrowser"
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
        create_shortcuts
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
    create_shortcuts
    launch_application
fi
