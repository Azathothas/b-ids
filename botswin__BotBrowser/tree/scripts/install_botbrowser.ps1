# BotBrowser Installer for Windows
# Automatically downloads and extracts the latest release.
#
# Usage:
#   .\install_botbrowser.ps1                      # Install latest
#   .\install_botbrowser.ps1 -Version 145         # Install latest Chrome 145 build
#   .\install_botbrowser.ps1 -InstallDir "D:\BB"  # Custom directory

param(
    [string]$Version = "",
    [string]$InstallDir = "$env:LOCALAPPDATA\BotBrowser",
    [switch]$Help
)

if ($Help) {
    Write-Host "BotBrowser Installer for Windows"
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\install_botbrowser.ps1                        # Install latest"
    Write-Host "  .\install_botbrowser.ps1 -Version 145           # Chrome 145"
    Write-Host "  .\install_botbrowser.ps1 -InstallDir 'D:\BB'    # Custom directory"
    exit 0
}

$ErrorActionPreference = "Stop"
$Repo = "botswin/BotBrowser"
$ApiBase = "https://api.github.com/repos/$Repo/releases"
$AssetPattern = ".7z"

Write-Host "Platform: Windows x86_64"

# Fetch release
$downloadUrl = $null

if ($Version) {
    Write-Host "1. Finding latest Chrome $Version release..."
    $releases = Invoke-RestMethod -Uri "$ApiBase`?per_page=100" -UseBasicParsing

    $candidates = @()
    foreach ($release in $releases) {
        if ($release.tag_name -match "^$Version\.") {
            foreach ($asset in $release.assets) {
                if ($asset.name -like "*$AssetPattern") {
                    $candidates += $asset
                }
            }
        }
    }
    # Sort by name descending to get the latest date (YYYYMMDD in filename)
    if ($candidates.Count -gt 0) {
        $latest = $candidates | Sort-Object -Property name -Descending | Select-Object -First 1
        $downloadUrl = $latest.browser_download_url
    }
} else {
    Write-Host "1. Fetching latest release info..."
    $release = Invoke-RestMethod -Uri "$ApiBase/latest" -UseBasicParsing

    $candidates = @()
    foreach ($asset in $release.assets) {
        if ($asset.name -like "*$AssetPattern") {
            $candidates += $asset
        }
    }
    # Sort by name descending to get the latest date (YYYYMMDD in filename)
    if ($candidates.Count -gt 0) {
        $latest = $candidates | Sort-Object -Property name -Descending | Select-Object -First 1
        $downloadUrl = $latest.browser_download_url
    }
}

if (-not $downloadUrl) {
    Write-Host "Error: Could not find download URL for $AssetPattern"
    if ($Version) {
        Write-Host "       No release found for Chrome $Version."
    }
    exit 1
}

$fileName = [System.IO.Path]::GetFileName($downloadUrl)
$tempFile = Join-Path $env:TEMP $fileName

# Extract version from filename: botbrowser_20260210_145.0.7632.46_win_x86_64.7z
$detectedVersion = if ($fileName -match 'botbrowser_\d+_(\d+\.\d+\.\d+\.\d+)_') { $Matches[1] } else { "unknown" }

Write-Host "   Version: $detectedVersion"
Write-Host "2. Downloading..."

$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
$ProgressPreference = 'Continue'

Write-Host "   Saved to: $tempFile"

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-Host "3. Extracting to $InstallDir ..."

# Find 7-Zip
$sevenZip = Get-Command "7z" -ErrorAction SilentlyContinue
if (-not $sevenZip) {
    $sevenZipPath = "C:\Program Files\7-Zip\7z.exe"
    if (Test-Path $sevenZipPath) {
        $sevenZip = Get-Command $sevenZipPath
    }
}

if ($sevenZip) {
    & $sevenZip.Source x $tempFile -o"$InstallDir" -y | Out-Null
} else {
    Write-Host "Error: 7-Zip is required to extract .7z files."
    Write-Host "       Install from: https://www.7-zip.org/"
    Write-Host "       Or extract manually: $tempFile"
    exit 1
}

# Handle nested archives (the outer 7z may contain chrome.7z or other archives)
$nested7zFiles = Get-ChildItem -Path $InstallDir -Filter "*.7z" -Recurse -ErrorAction SilentlyContinue
foreach ($nested in $nested7zFiles) {
    Write-Host "   Extracting nested archive: $($nested.Name)"
    & $sevenZip.Source x $nested.FullName -o"$InstallDir" -y | Out-Null
    Remove-Item $nested.FullName -Force
}

$nestedZipFiles = Get-ChildItem -Path $InstallDir -Filter "*.zip" -Recurse -ErrorAction SilentlyContinue
foreach ($nested in $nestedZipFiles) {
    Write-Host "   Extracting nested archive: $($nested.Name)"
    Expand-Archive -Path $nested.FullName -DestinationPath $InstallDir -Force
    Remove-Item $nested.FullName -Force
}

Write-Host "4. Cleaning up..."
Remove-Item $tempFile -Force

$chromePath = Join-Path $InstallDir "chrome.exe"
Write-Host ""
Write-Host "Installation complete! BotBrowser $detectedVersion"
Write-Host "Location: $InstallDir"
if (Test-Path $chromePath) {
    Write-Host "Executable: $chromePath"
}
