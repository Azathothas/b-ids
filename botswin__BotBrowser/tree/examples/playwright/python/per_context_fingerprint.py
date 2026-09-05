"""
PRIVACY RESEARCH USE ONLY
Run exclusively in authorized privacy research labs that comply with all applicable laws.
See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md

BotBrowser Per-Context Fingerprint + Proxy Example (Python Playwright)

Creates multiple browser contexts in a single browser instance, each with
a different fingerprint profile and optional proxy via BotBrowser.setBrowserContextFlags.

Requirements:
    pip install playwright
    BotBrowser binary (not stock Chromium)
    At least two .enc profile files

Usage:
    BOTBROWSER_EXEC_PATH=/path/to/chrome \
    PROFILE_A=/path/to/profile-a.enc \
    PROFILE_B=/path/to/profile-b.enc \
    PROXY_B=socks5://user:pass@host:1080 \
    python per_context_fingerprint.py
"""

import asyncio
import json
import os
import sys

from playwright.async_api import async_playwright


async def create_bb_context(browser, cdp_session, profile_path, proxy=None):
    """
    Create a BrowserContext with a BotBrowser fingerprint profile and optional proxy.

    Steps:
        1. Snapshot existing context IDs
        2. Create context via Playwright
        3. Find the new context ID by diffing before/after
        4. Set BotBrowser flags BEFORE creating any page
    """
    # Step 1: snapshot
    result = await cdp_session.send("Target.getBrowserContexts")
    before_ids = result["browserContextIds"]

    # Step 2: create context
    context = await browser.new_context()

    # Step 3: find new context ID
    result = await cdp_session.send("Target.getBrowserContexts")
    after_ids = result["browserContextIds"]
    context_id = None
    for cid in after_ids:
        if cid not in before_ids:
            context_id = cid
            break

    if not context_id:
        raise RuntimeError("Could not determine browserContextId")

    # Step 4: set flags (profile + optional proxy)
    flags = [f"--bot-profile={profile_path}"]
    if proxy:
        flags.append(f"--proxy-server={proxy}")

    await cdp_session.send("BotBrowser.setBrowserContextFlags", {
        "browserContextId": context_id,
        "botbrowserFlags": flags,
    })

    return context, context_id


async def collect_fingerprint(page):
    """Collect key fingerprint signals from a page."""
    return await page.evaluate("""async () => {
        const r = {};
        r.userAgent = navigator.userAgent;
        r.platform = navigator.platform;
        r.hardwareConcurrency = navigator.hardwareConcurrency;
        r.deviceMemory = navigator.deviceMemory;
        r.screenWidth = screen.width;
        r.screenHeight = screen.height;
        r.timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
        r.languages = JSON.stringify(navigator.languages);
        try {
            const c = document.createElement('canvas');
            const gl = c.getContext('webgl');
            if (gl) {
                const d = gl.getExtension('WEBGL_debug_renderer_info');
                if (d) r.webglRenderer = gl.getParameter(d.UNMASKED_RENDERER_WEBGL);
            }
        } catch (_) {}
        return r;
    }""")


def print_fingerprint(label, fp):
    print(f"\n  [{label}]")
    print(f"    UA:       {fp['userAgent'][:80]}")
    print(f"    Platform: {fp['platform']}")
    print(f"    HW:       {fp['hardwareConcurrency']} cores, {fp.get('deviceMemory', 'N/A')}GB")
    print(f"    Screen:   {fp['screenWidth']}x{fp['screenHeight']}")
    print(f"    WebGL:    {fp.get('webglRenderer', 'N/A')}")
    print(f"    Timezone: {fp['timezone']}")
    print(f"    Langs:    {fp.get('languages', 'N/A')}")


async def main():
    exec_path = os.environ.get("BOTBROWSER_EXEC_PATH")
    profile_a = os.environ.get("PROFILE_A")
    profile_b = os.environ.get("PROFILE_B")
    proxy_b = os.environ.get("PROXY_B")  # optional

    if not exec_path or not profile_a or not profile_b:
        print("Usage:")
        print("  BOTBROWSER_EXEC_PATH=/path/to/chrome \\")
        print("  PROFILE_A=/path/to/profile-a.enc \\")
        print("  PROFILE_B=/path/to/profile-b.enc \\")
        print("  PROXY_B=socks5://user:pass@host:1080 \\")
        print("  python per_context_fingerprint.py")
        sys.exit(1)

    async with async_playwright() as pw:
        # Launch with profile A as the base (default) profile
        browser = await pw.chromium.launch(
            executable_path=exec_path,
            headless=True,
            args=[
                "--disable-blink-features=AutomationControlled",
                "--disable-audio-output",
                f"--bot-profile={profile_a}",
            ],
        )
        print("Browser launched")

        cdp = await browser.new_browser_cdp_session()

        try:
            # --- Context A: profile A (no proxy) ---
            ctx_a, ctx_id_a = await create_bb_context(browser, cdp, profile_a)
            page_a = await ctx_a.new_page()
            print(f"\n  Context A created (id={ctx_id_a})")

            # --- Context B: profile B + optional proxy ---
            ctx_b, ctx_id_b = await create_bb_context(browser, cdp, profile_b, proxy=proxy_b)
            page_b = await ctx_b.new_page()
            print(f"  Context B created (id={ctx_id_b})")
            if proxy_b:
                print(f"  Context B proxy: {proxy_b}")

            # Navigate both
            await asyncio.gather(
                page_a.goto("https://example.com", wait_until="domcontentloaded"),
                page_b.goto("https://example.com", wait_until="domcontentloaded"),
            )

            # Collect fingerprints
            fp_a = await collect_fingerprint(page_a)
            fp_b = await collect_fingerprint(page_b)

            print_fingerprint("Context A", fp_a)
            print_fingerprint("Context B", fp_b)

            # Check proxy IP for context B
            if proxy_b:
                print("\n  --- Proxy check (Context B) ---")
                try:
                    await page_b.goto("https://httpbin.org/ip", timeout=20000)
                    body = await page_b.evaluate("() => document.body.innerText")
                    ip = json.loads(body).get("origin", "N/A")
                    print(f"    Exit IP: {ip}")
                except Exception as e:
                    print(f"    Proxy check failed: {e}")

            # Verify isolation
            print("\n  --- Isolation check ---")
            fields = ["userAgent", "platform", "hardwareConcurrency", "screenWidth", "webglRenderer"]
            diffs = 0
            for f in fields:
                same = fp_a.get(f) == fp_b.get(f)
                if not same:
                    diffs += 1
                print(f"    {f}: {'SAME' if same else 'DIFFERENT'}")
            print(f"\n  {diffs} of {len(fields)} fields differ between contexts.")

        finally:
            await browser.close()


if __name__ == "__main__":
    asyncio.run(main())
