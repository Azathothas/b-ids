# Console Suppression

> Control CDP console message forwarding to maintain consistent browser behavior during automation.

---

<a id="prerequisites"></a>

## Prerequisites

- **BotBrowser** installed and running. See [Installation Guide](../../../INSTALLATION.md).
- **A profile file** (`.enc` for production).
- **ENT Tier1** subscription (for `--bot-disable-console-message`).

---

<a id="quick-start"></a>

## Quick Start

Console suppression is enabled by default on ENT Tier1. No additional configuration is needed:

```bash
chromium-browser \
    --headless \
    --bot-profile="/path/to/profile.enc" \
    --proxy-server=socks5://user:pass@proxy.example.com:1080
```

To explicitly enable or disable it:

```bash
# Active by default on ENT Tier1; pass =false to disable for debugging
chromium-browser \
    --bot-profile="/path/to/profile.enc" \
    --bot-disable-console-message

# Disable for debugging (console messages will be visible in CDP)
chromium-browser \
    --bot-profile="/path/to/profile.enc" \
    --bot-disable-console-message=false
```

---

<a id="how-it-works"></a>

## How It Works

The `--bot-disable-console-message` flag ensures that console-related behavior remains consistent with a non-instrumented browser session. The console API still works normally in page JavaScript, but CDP clients do not receive forwarded console messages.

On ENT Tier1 and above, this protection is enabled by default (`true`). You only need to set the flag explicitly if you want to disable it for debugging purposes.

---

<a id="common-scenarios"></a>

## Common Scenarios

### Production deployment

In production, keep console suppression enabled (the default). Your automation script will not receive `console.log` output via CDP, but this is the correct tradeoff for consistency:

```javascript
const browser = await chromium.launch({
    executablePath: BOTBROWSER_EXEC_PATH,
    headless: true,
    args: [
        `--bot-profile=${BOT_PROFILE_PATH}`,
        "--proxy-server=socks5://user:pass@proxy.example.com:1080",
    ],
});

const page = await browser.newPage();
// page.on("console") will not fire - this is expected behavior
await page.goto("https://example.com");
```

### Debugging with console output

When you need to see console messages during development, disable the suppression:

```javascript
const browser = await chromium.launch({
    executablePath: BOTBROWSER_EXEC_PATH,
    headless: true,
    args: [
        `--bot-profile=${BOT_PROFILE_PATH}`,
        "--bot-disable-console-message=false",
    ],
});

const page = await browser.newPage();
page.on("console", (msg) => console.log("PAGE:", msg.text()));
await page.goto("https://example.com");
```

> **Important:** Disabling console suppression during debugging changes the browser's CDP behavior. Results from fingerprint-sensitive pages may differ from production.

---

<a id="troubleshooting"></a>

## Troubleshooting / FAQ

| Problem | Solution |
|---------|----------|
| `page.on("console")` never fires | This is expected when console suppression is active. Disable it with `--bot-disable-console-message=false` for debugging. |
| Console suppression not working | Verify your subscription tier is ENT Tier1 or above. The flag is not available on lower tiers. |
| Need console output in production | Consider logging at the application level (write to files or external services) instead of relying on CDP console forwarding. |

---

<a id="next-steps"></a>

## Next Steps

- [Automation Consistency Practices](../getting-started/AUTOMATION_CONSISTENCY.md). Additional techniques for maintaining a consistent browser environment.
- [Performance Optimization](../deployment/PERFORMANCE_OPTIMIZATION.md). Production tuning guide.
- [CLI Flags Reference](../../../CLI_FLAGS.md). Complete list of behavior and protection toggles.

---

**Related documentation:** [Advanced Features](../../../ADVANCED_FEATURES.md) | [Playwright Guide](../getting-started/PLAYWRIGHT.md)

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
