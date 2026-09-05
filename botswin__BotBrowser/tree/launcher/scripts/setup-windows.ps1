# BotBrowser Launcher Setup Script for Windows
# Run: powershell -ExecutionPolicy Bypass -File setup-windows.ps1

$ErrorActionPreference = "Stop"

$NODE_VERSION = "24.15.0"
$NODE_URL = "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-win-x64.zip"
$REPO_ZIP_URL = "https://github.com/botswin/BotBrowser/archive/refs/heads/main.zip"
$INSTALL_DIR = "$env:LOCALAPPDATA\BotBrowser"
$NODE_DIR = "$INSTALL_DIR\node"
$NPM_CACHE_DIR = "$INSTALL_DIR\npm-cache"
$REPO_DIR = "$INSTALL_DIR\BotBrowser"
$DIST_DIR = "$REPO_DIR\launcher\dist\BotBrowserLauncher"
$EXE_PATH = "$DIST_DIR\BotBrowserLauncher-win_x64.exe"
$DESKTOP_SHORTCUT = [System.IO.Path]::Combine([Environment]::GetFolderPath("Desktop"), "BotBrowser Launcher.lnk")
$STARTMENU_DIR = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs")
$STARTMENU_SHORTCUT = [System.IO.Path]::Combine($STARTMENU_DIR, "BotBrowser Launcher.lnk")

function Install-NodeJS {
    if (Test-Path "$NODE_DIR\node.exe") {
        $installed = (& "$NODE_DIR\node.exe" --version).Trim()
        if ($installed -eq "v${NODE_VERSION}") {
            Write-Host "Node.js v${NODE_VERSION} already installed." -ForegroundColor Green
            return
        }
        Write-Host "Replacing Node.js $installed with v${NODE_VERSION} (Angular 22 requires >= 24.15.0)..." -ForegroundColor Yellow
        Remove-Item -Recurse -Force $NODE_DIR
    }

    Write-Host "Downloading Node.js v${NODE_VERSION}..." -ForegroundColor Yellow
    $zipPath = "$env:TEMP\node.zip"
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $NODE_URL -OutFile $zipPath -UseBasicParsing

    Write-Host "Extracting Node.js..." -ForegroundColor Yellow
    Expand-Archive -Path $zipPath -DestinationPath $INSTALL_DIR -Force
    Rename-Item "$INSTALL_DIR\node-v${NODE_VERSION}-win-x64" $NODE_DIR -Force
    Remove-Item $zipPath
    Write-Host "Node.js installed." -ForegroundColor Green
}

function Install-Repository {
    if (Test-Path $REPO_DIR) {
        Remove-Item -Recurse -Force $REPO_DIR
    }
    Write-Host "Downloading repository..." -ForegroundColor Yellow
    $repoZipPath = "$env:TEMP\botbrowser.zip"
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $REPO_ZIP_URL -OutFile $repoZipPath -UseBasicParsing

    Write-Host "Extracting repository..." -ForegroundColor Yellow
    Expand-Archive -Path $repoZipPath -DestinationPath $INSTALL_DIR -Force
    Rename-Item "$INSTALL_DIR\BotBrowser-main" $REPO_DIR -Force
    Remove-Item $repoZipPath
}

function Build-Application {
    $env:PATH = "$NODE_DIR;$env:PATH"

    $nodeVersion = & "$NODE_DIR\node.exe" --version
    Write-Host "Node.js version: $nodeVersion" -ForegroundColor Green

    Push-Location "$REPO_DIR\launcher"

    Write-Host "Installing dependencies..." -ForegroundColor Yellow
    if (!(Test-Path $NPM_CACHE_DIR)) {
        New-Item -ItemType Directory -Path $NPM_CACHE_DIR | Out-Null
    }
    $env:NPM_CONFIG_CACHE = $NPM_CACHE_DIR
    $env:NPM_CONFIG_UPDATE_NOTIFIER = "false"
    & "$NODE_DIR\npm.cmd" ci

    Write-Host "Building application..." -ForegroundColor Yellow
    & "$NODE_DIR\npm.cmd" run build

    Pop-Location
}

function New-Shortcuts {
    Write-Host "Creating shortcuts..." -ForegroundColor Yellow
    $WshShell = New-Object -ComObject WScript.Shell

    $ICON_PATH = "$REPO_DIR\launcher\public\favicon.ico"

    # Desktop shortcut
    $Shortcut = $WshShell.CreateShortcut($DESKTOP_SHORTCUT)
    $Shortcut.TargetPath = $EXE_PATH
    $Shortcut.WorkingDirectory = $DIST_DIR
    $Shortcut.Description = "BotBrowser Launcher"
    if (Test-Path $ICON_PATH) {
        $Shortcut.IconLocation = "$ICON_PATH,0"
    }
    $Shortcut.Save()
    Write-Host "  Desktop shortcut: $DESKTOP_SHORTCUT" -ForegroundColor Green

    # Start Menu shortcut
    if (!(Test-Path $STARTMENU_DIR)) {
        New-Item -ItemType Directory -Path $STARTMENU_DIR | Out-Null
    }
    $Shortcut = $WshShell.CreateShortcut($STARTMENU_SHORTCUT)
    $Shortcut.TargetPath = $EXE_PATH
    $Shortcut.WorkingDirectory = $DIST_DIR
    $Shortcut.Description = "BotBrowser Launcher"
    if (Test-Path $ICON_PATH) {
        $Shortcut.IconLocation = "$ICON_PATH,0"
    }
    $Shortcut.Save()
    Write-Host "  Start Menu shortcut: $STARTMENU_SHORTCUT" -ForegroundColor Green
}

function Save-CommitHash {
    Write-Host "Saving launcher version info..." -ForegroundColor Yellow
    $commitDir = "$env:APPDATA\BotBrowser"
    if (!(Test-Path $commitDir)) {
        New-Item -ItemType Directory -Path $commitDir | Out-Null
    }
    try {
        $ProgressPreference = 'SilentlyContinue'
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/botswin/BotBrowser/commits?path=launcher&sha=main&per_page=1" -Headers @{ Accept = "application/vnd.github.v3+json" } -UseBasicParsing
        $response[0].sha | Out-File -FilePath "$commitDir\launcher-commit" -Encoding utf8 -NoNewline
        Write-Host "  Version: $($response[0].sha.Substring(0, 7))" -ForegroundColor Green
    } catch {
        Write-Host "  Warning: Could not save version info." -ForegroundColor Yellow
    }
}

function Start-Application {
    Write-Host "Starting BotBrowser Launcher..." -ForegroundColor Green
    Start-Process -FilePath $EXE_PATH -WorkingDirectory $DIST_DIR
}

# === Main ===

Write-Host "=== BotBrowser Launcher Setup ===" -ForegroundColor Cyan

# Create install directory
if (!(Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null
}

# Check if already installed
if (Test-Path $EXE_PATH) {
    Write-Host "BotBrowser Launcher is already installed." -ForegroundColor Green
    $choice = Read-Host "Update to latest version? (Y = update, N = launch existing) [N]"

    if ($choice -eq "Y" -or $choice -eq "y") {
        Write-Host "Updating..." -ForegroundColor Yellow
        Install-NodeJS
        Install-Repository
        Build-Application
        Save-CommitHash
        New-Shortcuts
        Start-Application
    } else {
        Start-Application
    }
} else {
    Write-Host "First time setup..." -ForegroundColor Yellow
    Install-NodeJS
    Install-Repository
    Build-Application
    Save-CommitHash
    New-Shortcuts
    Start-Application
}
