#!/usr/bin/env python3
"""
⚠️ PRIVACY RESEARCH USE ONLY
Run exclusively in authorized privacy research labs that comply with all applicable laws.
See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
"""

import asyncio
import os

from playwright.async_api import async_playwright

BOTBROWSER_EXEC_PATH: str = os.getenv("BOTBROWSER_EXEC_PATH", "") # Absolute path to the BotBrowser executable
BOT_PROFILE_PATH: str = os.getenv("BOT_PROFILE_PATH", "") # Absolute or relative path to the profile

if not BOTBROWSER_EXEC_PATH or not BOT_PROFILE_PATH:
    raise ValueError("Both BOTBROWSER_EXEC_PATH and BOT_PROFILE_PATH environment variables must be set.")

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch_persistent_context(
            user_data_dir="./user_data",
            executable_path=BOTBROWSER_EXEC_PATH,
            headless=False, # Set to True for production
            ignore_default_args=[
                "--disable-crash-reporter",
                "--disable-crashpad-for-testing",
                "--disable-gpu-watchdog",
            ],
            args=[
                "--disable-blink-features=AutomationControlled",
                "--disable-audio-output",
                f"--bot-profile={BOT_PROFILE_PATH}",
                # ⚠️ PROXY CONFIGURATION:
                # Use --proxy-server flag instead of playwright's proxy parameter in launch()
                # This ensures BotBrowser can retrieve geo information from proxy IP for accurate timezone/locale
                # "--proxy-server=http://user:pass@proxy.com:8080",
                # "--proxy-server=socks5://user:pass@proxy.com:1080",
            ],
        )
        page = await browser.new_page()

        # Remove Playwright's bindings to maintain consistent fingerprint
        await page.add_init_script("""
            delete window.__playwright__binding__;
            delete window.__pwInitScripts;
        """)

        await page.goto("https://abrahamjuliot.github.io/creepjs/")
        await asyncio.Event().wait()

asyncio.run(main())
