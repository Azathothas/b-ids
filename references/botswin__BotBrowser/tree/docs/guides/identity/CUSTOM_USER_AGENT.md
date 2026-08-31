# Custom User-Agent

> Configure custom User-Agent and userAgentData/Client Hints while keeping all identity surfaces internally consistent to prevent fingerprint mismatches.

---

<a id="prerequisites"></a>

## Prerequisites

- **BotBrowser binary** installed. See [INSTALLATION.md](../../../INSTALLATION.md).
- **A profile file** (`.enc` or `.json`).
- **ENT Tier3 license** for custom User-Agent and full userAgentData control.

---

<a id="overview"></a>

## Overview

Browser identity extends far beyond the User-Agent string. It includes Client Hints, platform information, device properties, and other API surfaces, all of which must remain internally consistent.

BotBrowser provides a set of flags that work together to construct a unified browser identity. When you configure these flags, BotBrowser auto-generates matching Client Hints values (including brands, fullVersionList, and proper GREASE tokens) and all corresponding HTTP headers. Values stay consistent across the main thread, Workers, and HTTP requests.

---

<a id="quick-start"></a>

## Quick Start

```bash
# Android WebView identity with placeholders
chromium-browser \
  --bot-profile="/path/to/android-profile.enc" \
  --user-agent="Mozilla/5.0 (Linux; Android {platform-version}; {model} Build/TP1A.220624.021; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/{ua-full-version} Mobile Safari/537.36" \
  --bot-config-browser-brand=webview \
  --bot-config-platform=Android \
  --bot-config-platform-version=13 \
  --bot-config-model=SM-G991B \
  --bot-config-mobile=true \
  --bot-config-architecture=arm \
  --bot-config-bitness=64
```

Placeholders like `{platform-version}` and `{model}` get replaced at runtime from the corresponding flag values.

---

<a id="how-it-works"></a>

## How It Works

### Identity Flags

These flags define the building blocks of browser identity:

| Flag | Controls | Example |
|------|----------|---------|
| `--user-agent` | The raw User-Agent string. Supports placeholders. | `Mozilla/5.0 (Windows NT 10.0; Win64; x64) ...` |
| `--bot-config-platform` | `navigator.userAgentData.platform` | `Windows`, `Android`, `macOS`, `Linux` |
| `--bot-config-platform-version` | OS version in userAgentData | `13`, `10.0`, `14.0` |
| `--bot-config-model` | Device model (primarily for mobile) | `RMX3471`, `SM-G991B` |
| `--bot-config-architecture` | CPU architecture in userAgentData | `x86`, `arm`, `arm64` |
| `--bot-config-bitness` | System bitness in userAgentData | `32`, `64` |
| `--bot-config-mobile` | Mobile device flag in userAgentData | `true`, `false` |

### Supporting Flags

| Flag | Controls | Example |
|------|----------|---------|
| `--bot-config-browser-brand` | Brand identity (Chrome, Edge, etc.) | `chrome`, `edge`, `webview` |
| `--bot-config-ua-full-version` | Chromium full version string | `142.0.7444.60` |
| `--bot-config-brand-full-version` | Brand-specific version (Edge, Opera) | `142.0.3595.65` |

### Placeholder Tokens

The `--user-agent` flag supports these placeholders that get replaced at runtime:

| Placeholder | Source |
|-------------|--------|
| `{platform}` | `--bot-config-platform` |
| `{platform-version}` | `--bot-config-platform-version` |
| `{model}` | `--bot-config-model` |
| `{ua-full-version}` | `--bot-config-ua-full-version` |
| `{ua-major-version}` | Major version derived from `--bot-config-ua-full-version` |
| `{brand-full-version}` | `--bot-config-brand-full-version` |
| `{architecture}` | `--bot-config-architecture` |
| `{bitness}` | `--bot-config-bitness` |

### What BotBrowser Auto-Generates

When you set these flags, BotBrowser automatically produces:

- **Client Hints brands** with correct brand entries and GREASE tokens
- **High-entropy values** for all configured identity properties (platform, model, architecture, bitness, etc.)
- **All corresponding Client Hints HTTP headers** aligned with the JavaScript-visible values

### Client Hints Alignment

BotBrowser ensures that all Client Hints HTTP headers match their JavaScript-visible counterparts, including display, memory, and identity properties.

---

<a id="common-scenarios"></a>

## Common Scenarios

### Custom Windows desktop identity

```javascript
import { chromium } from "playwright-core";

const browser = await chromium.launch({
  executablePath: process.env.BOTBROWSER_EXEC_PATH,
  headless: true,
  args: [
    "--bot-profile=/path/to/windows-profile.enc",
    '--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{ua-full-version} Safari/537.36',
    "--bot-config-platform=Windows",
    "--bot-config-platform-version=10.0",
    "--bot-config-architecture=x86",
    "--bot-config-bitness=64",
    "--bot-config-mobile=false",
  ],
});
```

### Custom Android mobile identity

```javascript
const browser = await chromium.launch({
  executablePath: process.env.BOTBROWSER_EXEC_PATH,
  headless: true,
  args: [
    "--bot-profile=/path/to/android-profile.enc",
    '--user-agent=Mozilla/5.0 (Linux; Android {platform-version}; {model}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{ua-full-version} Mobile Safari/537.36',
    "--bot-config-platform=Android",
    "--bot-config-platform-version=14",
    "--bot-config-model=SM-G991B",
    "--bot-config-architecture=arm",
    "--bot-config-bitness=64",
    "--bot-config-mobile=true",
  ],
});
```

### Edge identity with custom version

```javascript
const browser = await chromium.launch({
  executablePath: process.env.BOTBROWSER_EXEC_PATH,
  headless: true,
  args: [
    "--bot-profile=/path/to/profile.enc",
    '--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{ua-full-version} Safari/537.36 Edg/{brand-full-version}',
    "--bot-config-browser-brand=edge",
    "--bot-config-ua-full-version=142.0.7444.60",
    "--bot-config-brand-full-version=142.0.3595.65",
    "--bot-config-platform=Windows",
    "--bot-config-platform-version=10.0",
  ],
});
```

---

<a id="troubleshooting"></a>

## Troubleshooting / FAQ

| Problem | Solution |
|---------|----------|
| Placeholder not replaced in UA string | Ensure the corresponding `--bot-config-*` flag is set. `{model}` requires `--bot-config-model`. |
| Client Hints mismatch | Keep `--bot-config-ua-full-version` aligned with the Chromium major version of your BotBrowser binary. |
| userAgentData returns wrong platform | Verify `--bot-config-platform` is set (e.g., `Windows`, `Android`, `macOS`, `Linux`). |
| Brand version not appearing | Use `--bot-config-brand-full-version` for the brand-specific version, separate from `--bot-config-ua-full-version`. |

---

<a id="next-steps"></a>

## Next Steps

- [Browser Brand Alignment](BROWSER_BRAND_ALIGNMENT.md). Switch between Chrome, Edge, Brave, Opera, and more.
- [Android Emulation](../platform/ANDROID_EMULATION.md). Full Android identity emulation on desktop.
- [Android WebView](../platform/ANDROID_WEBVIEW.md). WebView-specific identity configuration.
- [Identity flags](../../../CLI_FLAGS.md#flag-group-identity). CLI values and availability.

---

**Related documentation:** [CLI Flags Reference](../../../CLI_FLAGS.md) | [Profile Configuration](../../../profiles/PROFILE_CONFIGS.md) | [Advanced Features](../../../ADVANCED_FEATURES.md)

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
