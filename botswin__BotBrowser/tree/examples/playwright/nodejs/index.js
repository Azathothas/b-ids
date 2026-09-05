/**
 * ⚠️ PRIVACY RESEARCH USE ONLY
 * Run exclusively in authorized privacy research labs that comply with all applicable laws.
 * See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

import { chromium } from "playwright";

const BOTBROWSER_EXEC_PATH = process.env.BOTBROWSER_EXEC_PATH; // Absolute path to the BotBrowser executable
const BOT_PROFILE_PATH = process.env.BOT_PROFILE_PATH; // Absolute or relative path to the profile

const browser = await chromium.launch({
  executablePath: BOTBROWSER_EXEC_PATH,
  headless: false, // Set to true for production
  ignoreDefaultArgs: [
    "--disable-crash-reporter",
    "--disable-crashpad-for-testing",
    "--disable-gpu-watchdog",
  ],
  args: [
    "--disable-blink-features=AutomationControlled",
    "--disable-audio-output",
    `--bot-profile=${BOT_PROFILE_PATH}`,
    // ⚠️ PROXY CONFIGURATION:
    // Use --proxy-server flag instead of playwright's proxy option in launch()
    // This ensures BotBrowser can retrieve geo information from proxy IP for accurate timezone/locale
    // '--proxy-server=http://user:pass@proxy.com:8080',
    // '--proxy-server=socks5://user:pass@proxy.com:1080',
  ],
});

const page = await browser.newPage();

// Remove Playwright's bindings to maintain consistent fingerprint.
await page.addInitScript(() => {
  delete window.__playwright__binding__;
  delete window.__pwInitScripts;
});

await page.goto("https://abrahamjuliot.github.io/creepjs/");
