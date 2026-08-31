"""
PRIVACY RESEARCH USE ONLY
Run exclusively in authorized privacy research labs that comply with all applicable laws.
See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md

BotBrowser Per-Context Proxy Example (Python Playwright)

Demonstrates multiple contexts with different proxies in a single browser,
each with automatic GeoIP-driven timezone, locale, and language detection.

Requirements:
    pip install playwright
    BotBrowser binary (not stock Chromium)

Usage:
    BOTBROWSER_EXEC_PATH=/path/to/chrome \
    BOT_PROFILE_PATH=/path/to/profile.enc \
    PROXY_US=socks5://user:pass@us-proxy:1080 \
    PROXY_JP=socks5://user:pass@jp-proxy:1080 \
    python per_context_proxy.py
"""

import asyncio
import json
import os
import sys

from playwright.async_api import async_playwright


async def create_bb_context_with_proxy(browser, cdp_session, proxy):
    """
    Create a BrowserContext with a per-context proxy.

    BotBrowser.setBrowserContextProxy sets the proxy and automatically
    triggers GeoIP detection for timezone, locale, and languages.
    """
    result = await cdp_session.send("Target.getBrowserContexts")
    before_ids = result["browserContextIds"]

    context = await browser.new_context()

    result = await cdp_session.send("Target.getBrowserContexts")
    after_ids = result["browserContextIds"]
    context_id = None
    for cid in after_ids:
        if cid not in before_ids:
            context_id = cid
            break

    if not context_id:
        raise RuntimeError("Could not determine browserContextId")

    await cdp_session.send("BotBrowser.setBrowserContextProxy", {
        "browserContextId": context_id,
        "proxyServer": proxy,
    })

    return context, context_id


async def test_context(page, label):
    """Test a context's proxy IP and GeoIP-driven settings."""
    print(f"\n  [{label}]")

    try:
        await page.goto("https://httpbin.org/ip", timeout=20000)
        body = await page.evaluate("() => document.body.innerText")
        ip = json.loads(body).get("origin", "N/A")
        print(f"    IP:       {ip}")
    except Exception as e:
        print(f"    IP check failed: {e}")

    # GeoIP-driven values (automatically set by BotBrowser based on proxy exit IP)
    info = await page.evaluate("""() => ({
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        locale: Intl.NumberFormat().resolvedOptions().locale,
        languages: JSON.stringify(navigator.languages),
        offset: new Date().getTimezoneOffset(),
    })""")

    print(f"    Timezone: {info['timezone']} (offset: {info['offset']}min)")
    print(f"    Locale:   {info['locale']}")
    print(f"    Langs:    {info['languages']}")


async def main():
    exec_path = os.environ.get("BOTBROWSER_EXEC_PATH")
    profile = os.environ.get("BOT_PROFILE_PATH")
    proxy_us = os.environ.get("PROXY_US")
    proxy_jp = os.environ.get("PROXY_JP")

    if not exec_path or not profile:
        print("Usage:")
        print("  BOTBROWSER_EXEC_PATH=/path/to/chrome \\")
        print("  BOT_PROFILE_PATH=/path/to/profile.enc \\")
        print("  PROXY_US=socks5://user:pass@us-proxy:1080 \\")
        print("  PROXY_JP=socks5://user:pass@jp-proxy:1080 \\")
        print("  python per_context_proxy.py")
        sys.exit(1)

    if not proxy_us or not proxy_jp:
        print("Warning: Set PROXY_US and PROXY_JP for full demo. Using placeholders.")
        proxy_us = proxy_us or "socks5://user:pass@us-proxy.example.com:1080"
        proxy_jp = proxy_jp or "socks5://user:pass@jp-proxy.example.com:1080"

    async with async_playwright() as pw:
        browser = await pw.chromium.launch(
            executable_path=exec_path,
            headless=True,
            args=[
                "--disable-blink-features=AutomationControlled",
                "--disable-audio-output",
                f"--bot-profile={profile}",
            ],
        )
        print("Browser launched")

        cdp = await browser.new_browser_cdp_session()

        try:
            # Context 1: US proxy - will auto-detect US timezone/locale
            ctx_us, _ = await create_bb_context_with_proxy(browser, cdp, proxy_us)
            page_us = await ctx_us.new_page()

            # Context 2: JP proxy - will auto-detect JP timezone/locale
            ctx_jp, _ = await create_bb_context_with_proxy(browser, cdp, proxy_jp)
            page_jp = await ctx_jp.new_page()

            # Test each context
            await test_context(page_us, "US Proxy")
            await test_context(page_jp, "JP Proxy")

            # Verify isolation
            tz_us = await page_us.evaluate("() => Intl.DateTimeFormat().resolvedOptions().timeZone")
            tz_jp = await page_jp.evaluate("() => Intl.DateTimeFormat().resolvedOptions().timeZone")
            print(f"\n  Timezone isolation: {'PASS' if tz_us != tz_jp else 'SAME'} (US={tz_us}, JP={tz_jp})")

            await page_us.close()
            await page_jp.close()
            await ctx_us.close()
            await ctx_jp.close()

        finally:
            await browser.close()
            print("\nDone!")


if __name__ == "__main__":
    asyncio.run(main())
