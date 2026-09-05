# BotBrowser Installation Guide

This guide provides step-by-step installation, troubleshooting, and deployment options across operating systems. Usage remains subject to the project [Legal Disclaimer](DISCLAIMER.md) and [Responsible Use Guidelines](RESPONSIBLE_USE.md).

<a id="choosing-a-build"></a>

## Choosing a Build

BotBrowser ships in two builds. Both share the same fingerprint protection model, profile format, CLI flag surface, and CDP commands.

- **Standard Build** is the default public release. Use this build for long-running sessions, interactive workflows, and scenarios that exercise the full browser feature surface. Available on the [Releases](https://github.com/botswin/BotBrowser/releases) page; the instructions on this page describe Standard Build installation.
- **Trimmed Build** (ENT Tier3) is a separately built distribution tuned for short-session, high-concurrency automation. Linux x64 benchmark: **62% lower wall time**, **85% faster per-context creation**, **68% lower CPU peak** versus Standard. Trimmed Build is not on the public Releases page; it is distributed through the enterprise channel. Product overview: [TRIMMED_BUILD.md](TRIMMED_BUILD.md). Full performance table: [BENCHMARK.md#trimmed-build](BENCHMARK.md#trimmed-build). Access: [Enterprise](https://botbrowser.io/enterprise/) or [Pricing](https://botbrowser.io/pricing/).

Profiles, automation code, and CLI flags from Standard Build work unchanged on Trimmed Build, so you can build against Standard locally and switch to Trimmed in production.

<a id="download--installation"></a>

## Download & Installation

### 1. Download Installer

Get the Standard Build installer for your OS from the [Releases](https://github.com/botswin/BotBrowser/releases) page. (Trimmed Build is distributed through the enterprise channel; see [Choosing a Build](#choosing-a-build).)

### 2. Windows Installation

#### Quick Install (PowerShell)

```powershell
# Install latest version
iwr -Uri "https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.ps1" -OutFile "$env:TEMP\install_botbrowser.ps1"; & "$env:TEMP\install_botbrowser.ps1"

# Install specific Chrome version
iwr -Uri "https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.ps1" -OutFile "$env:TEMP\install_botbrowser.ps1"; & "$env:TEMP\install_botbrowser.ps1" -Version 146

# Install to custom directory
iwr -Uri "https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.ps1" -OutFile "$env:TEMP\install_botbrowser.ps1"; & "$env:TEMP\install_botbrowser.ps1" -InstallDir "D:\BotBrowser"
```

> **Note:** Requires [7-Zip](https://www.7-zip.org/) for extraction.

#### Manual Installation
1. Extract the downloaded `.7z` archive
2. Run `chrome.exe` from the extracted folder

#### Common Windows Issues & Solutions

| Issue | Solution |
|-------|----------|
| **Profile file permission errors** | Ensure `.enc` file has read permissions |
| **BotBrowser won't start or crashes** | Check that your OS and Chromium version match the build; update BotBrowser to the latest release |
| **Antivirus blocking execution** | Add BotBrowser directory to antivirus exclusions |

#### Windows Command-Line Example

**CMD:**
```cmd
chrome.exe --bot-profile="C:\absolute\path\to\profile.enc" --user-data-dir="%TEMP%\botprofile_%RANDOM%"
```

**PowerShell:**
```powershell
.\chrome.exe --bot-profile="C:\absolute\path\to\profile.enc" --user-data-dir="$env:TEMP\botprofile_$(Get-Random)"
```

> **Note:** CMD uses `%VAR%` syntax while PowerShell uses `$env:VAR`. The `%RANDOM%` variable only works in CMD; use `$(Get-Random)` in PowerShell.

### 3. macOS Installation

#### Quick Install (Script)

```bash
# Install latest version
curl -sL https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.sh | bash

# Install specific Chrome version
curl -sL https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.sh | bash -s -- 146

# Download only, don't install
curl -sL https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.sh | bash -s -- --download
```

The script auto-detects your Mac's architecture (Apple Silicon or Intel), fetches the latest release via the GitHub API, and installs to `/Applications/`.

#### Manual Installation
1. Open the downloaded `.dmg` file
2. Drag `Chromium.app` into your Applications folder or any desired location
3. If you see the error:
   ```
   "Chromium" is damaged and can't be opened
   ```
   Run:
   ```bash
   xattr -rd com.apple.quarantine /Applications/Chromium.app
   ```

#### Common macOS Issues & Solutions

| Issue | Solution |
|-------|----------|
| **"Chromium" is damaged** | Run `xattr -rd com.apple.quarantine /Applications/Chromium.app` |
| **Permission denied** | Ensure you have admin rights and the app is in Applications folder |
| **Gatekeeper blocking** | Go to System Preferences > Security & Privacy and allow the app |

#### macOS Command-Line Example
```bash
/Applications/Chromium.app/Contents/MacOS/Chromium \
  --user-data-dir="$(mktemp -d)" \
  --bot-profile="/absolute/path/to/profile.enc"
```

### 4. Ubuntu Installation (ENT Tier1)

> **Note:** Ubuntu/Linux binaries require an ENT Plan Tier 1 or higher subscription.

#### Quick Install (Script)

```bash
# One-line install (auto-detects architecture: x86_64 or arm64)
curl -sL https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.sh | bash

# Install specific Chrome version
curl -sL https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.sh | bash -s -- 146

# Download only, don't install
curl -sL https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.sh | bash -s -- --download
```

#### Docker / CI

```dockerfile
RUN curl -sL https://raw.githubusercontent.com/botswin/BotBrowser/main/scripts/install_botbrowser.sh | bash
```

No hardcoded URLs needed. The script uses the GitHub Releases API to always fetch the latest build for your platform.

#### Manual Installation
1. Install via `dpkg`:
   ```bash
   sudo dpkg -i botbrowser_<version>_amd64.deb
   ```
2. If dependencies are missing, run:
   ```bash
   sudo apt-get install -f
   ```

#### Required Dependencies
```bash
# Essential libraries
sudo apt-get update
sudo apt-get install -y \
    libnss3 \
    libatk-bridge2.0-0 \
    libdrm2 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    libgbm1 \
    libxkbcommon0 \
    libgtk-3-0
```

#### Ubuntu Command-Line Example
```bash
chromium-browser \
  --user-data-dir="$(mktemp -d)" \
  --bot-profile="/absolute/path/to/profile.enc"
```

---

## Docker Deployment

For containerized deployment with full isolation and scalability:

### Quick Docker Setup

```bash
# Pull the latest BotBrowser Docker image
docker pull botbrowser/botbrowser:latest

# Run with profile mounting
docker run -d \
  --name botbrowser-instance \
  -v /path/to/profiles:/app/profiles \
  -p 9222:9222 \
  botbrowser/botbrowser:latest \
  --bot-profile="/absolute/path/to/profile.enc" \
  --remote-debugging-port=9222
```

### Docker Compose Deployment

For production environments, see the complete [Docker deployment guide](docker/README.md) with:
- Multi-instance orchestration
- Proxy configuration
- Volume management
- Health checks
- Scaling configurations

---

## First Launch Verification

### Basic Functionality Test

After installation, verify BotBrowser is working correctly:

```bash
# Test basic launch (replace with your executable path)
chrome.exe --bot-profile="C:\\absolute\\path\\to\\profile.enc" --version

# Test with remote debugging
chrome.exe --bot-profile="C:\\absolute\\path\\to\\profile.enc" --remote-debugging-port=9222

# Verify remote debugging is active
curl http://localhost:9222/json/version
```

### Profile Validation

Ensure your profiles are working correctly:

```bash
# Check profile file permissions (Linux/macOS)
ls -la ./profiles/*.enc

# Test profile loading
chrome.exe --bot-profile="C:\\absolute\\path\\to\\profile.enc" --headless --dump-dom https://httpbin.org/user-agent
```


---

## Security Considerations

### Firewall Configuration

BotBrowser may need network access for:
- Profile validation and updates
- Remote debugging (if enabled)
- Proxy authentication
- Timezone/locale auto-detection

#### Recommended Firewall Rules
```bash
# Allow outbound HTTPS (profile updates)
ufw allow out 443

# Allow remote debugging port (if used)
ufw allow 9222

# Allow proxy connections (if used)
ufw allow out on <proxy-port>
```

### Profile Security

- Store profile files (.enc) in secure directories with appropriate permissions
- Never commit profiles to version control systems
- Use temporary user data directories (`--user-data-dir`) to avoid persistent data
- Regularly update profiles from trusted sources

---

## Installation Support

### Getting Help

If you encounter installation issues:

1. **Check the troubleshooting table** above for common solutions
2. **Verify system requirements** match your OS version
3. **Test with a matching profile package**. BotBrowser 150 and newer profiles are available through subscription or support at [support@botbrowser.io](mailto:support@botbrowser.io) or [@botbrowser_support](https://t.me/botbrowser_support); legacy demo profiles remain in the profiles directory for older evaluation lines.
4. **Contact support** with detailed error messages

### Contact Information

<table>
  <tr><td>Email</td><td><a href="mailto:support@botbrowser.io">support@botbrowser.io</a></td></tr>
  <tr><td>Telegram</td><td><a href="https://t.me/botbrowser_support">@botbrowser_support</a></td></tr>
</table>

### Reporting Installation Issues

When reporting installation problems, please include:
- Operating system and version
- BotBrowser version
- Complete error messages
- Steps to reproduce the issue
- System specifications (RAM, disk space, etc.)

---

## Related Documentation

- [Main README](README.md) - Project overview and quick start
- [Guides](https://botbrowser.io/docs/) - Step-by-step guides for proxy, fingerprint, identity, platform emulation, and deployment
- [CLI Flags Reference](CLI_FLAGS.md) - Complete command-line options
- [Profile Configuration](profiles/PROFILE_CONFIGS.md) - Advanced profile customization
- [Docker Guide](docker/README.md) - Containerized deployment
- [Advanced Features](ADVANCED_FEATURES.md) - Detailed feature documentation

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
