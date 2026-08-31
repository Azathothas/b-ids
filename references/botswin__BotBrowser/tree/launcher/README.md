# BotBrowser Launcher

Desktop GUI for **[BotBrowser](https://github.com/botswin/BotBrowser)**. Manage browser profiles, kernels, and proxies for fingerprint privacy protection.

<img width="800" alt="image" src="https://github.com/user-attachments/assets/0f003494-ec92-4c3a-b879-b08d3521a0fb">

## Platform Support

| Platform               | Architecture  |
| ---------------------- | ------------- |
| Windows                | x86_64        |
| macOS                  | ARM64, x86_64 |
| Linux (Ubuntu, Debian) | x86_64, ARM64 |

## Features

### Kernel Management

Download and manage BotBrowser kernels directly from the GUI. The launcher fetches releases from GitHub and handles platform-specific extraction (`.7z` for Windows, `.dmg` for macOS, `.deb` for Linux).

### Proxy Management

- Add, edit, and delete proxy configurations
- Bulk import proxies from text (one per line)
- Supports HTTP, HTTPS, SOCKS4, SOCKS5 protocols
- Quick-parse proxy strings (e.g., `socks5://user:pass@host:port`)
- **Check IP** — verify proxy exit IP, location, ISP, and datacenter detection via ip-api.com
- Quick proxy change from profile list without opening full editor
- Per-profile proxy assignment with auto-fill of detected exit IP

### Profile Management

- Create and configure browser profiles with fingerprint settings
- Clone existing profiles
- Import/export profiles for backup or sharing
- Auto-detect Android profiles and apply mobile settings

### Browser Launch

- Launch isolated browser instances with profile-specific settings
- Pre-warm sessions for controlled experiments
- Real-time download progress in sidebar

## Quick Setup

One-line install scripts that download Node.js, fetch the source, build, and create shortcuts (Desktop + Applications/Start Menu):

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/botswin/BotBrowser/main/launcher/scripts/setup-windows.ps1 -OutFile $env:TEMP\setup-windows.ps1; powershell -ExecutionPolicy Bypass -File $env:TEMP\setup-windows.ps1
```

**macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/botswin/BotBrowser/main/launcher/scripts/setup-macos.sh -o /tmp/setup.sh && bash /tmp/setup.sh
```

**Ubuntu/Debian:**

```bash
curl -fsSL https://raw.githubusercontent.com/botswin/BotBrowser/main/launcher/scripts/setup-ubuntu.sh -o /tmp/setup.sh && bash /tmp/setup.sh
```

## Manual Setup

### Prerequisites

- **Node.js 22 LTS (>=22.22.3)** or **24 LTS (>=24.15.0)** (Angular 22 requirement)

### Build & Run

```bash
npm ci          # installs dependencies + downloads Neutralino binary automatically
npm run build
npm run app
```

## Usage

1. Go to **Kernels** and download a BotBrowser kernel for your platform
2. Create a browser profile and select a bot profile (`.enc` file)
3. (Optional) Add proxies in **Proxies** tab
4. Launch the browser from the profile list

## Technical Stack

| Component | Technology                                         |
| --------- | -------------------------------------------------- |
| Frontend  | [Angular 22](https://angular.dev)                  |
| UI        | [Angular Material 21](https://material.angular.io) |
| Runtime   | [Neutralino.js 6.8](https://neutralino.js.org)     |

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**
