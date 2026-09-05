/**
 * PRIVACY RESEARCH USE ONLY
 * Run exclusively in authorized privacy research labs that comply with all applicable laws.
 * See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

/**
 * BotBrowser Per-Context Fingerprint Example (Playwright)
 *
 * Creates multiple browser contexts in a single browser instance, each with
 * a different fingerprint profile loaded via BotBrowser.setBrowserContextFlags.
 *
 * Requirements:
 * - BotBrowser binary (not stock Chromium)
 * - At least two .enc profile files
 *
 * Usage:
 *   BOTBROWSER_EXEC_PATH=/path/to/chrome \
 *   PROFILE_A=/path/to/profile-a.enc \
 *   PROFILE_B=/path/to/profile-b.enc \
 *   PROXY_B=socks5://user:pass@host:1080 \
 *   node per_context_fingerprint.js
 */

const { chromium } = require('playwright-core');

(async () => {
  const execPath = process.env.BOTBROWSER_EXEC_PATH;
  const profileA = process.env.PROFILE_A;
  const profileB = process.env.PROFILE_B;
  const proxyB   = process.env.PROXY_B; // optional, e.g. socks5://user:pass@host:1080

  if (!execPath || !profileA || !profileB) {
    console.log('Usage:');
    console.log('  BOTBROWSER_EXEC_PATH=/path/to/chrome \\');
    console.log('  PROFILE_A=/path/to/profile-a.enc \\');
    console.log('  PROFILE_B=/path/to/profile-b.enc \\');
    console.log('  PROXY_B=socks5://user:pass@host:1080 \\');
    console.log('  node per_context_fingerprint.js');
    process.exit(1);
  }

  // Launch browser with profile A as the base profile
  const browser = await chromium.launch({
    executablePath: execPath,
    headless: true,
    ignoreDefaultArgs: [
      '--disable-crash-reporter',
      '--disable-crashpad-for-testing',
      '--disable-gpu-watchdog',
    ],
    args: [
      '--disable-blink-features=AutomationControlled',
      '--disable-audio-output',
      `--bot-profile=${profileA}`,
    ],
  });

  console.log('Browser launched\n');

  // Browser-level CDP session. Required for BotBrowser.* commands.
  // Page-level sessions (page.createCDPSession()) do NOT have access
  // to the BotBrowser domain.
  const browserCDP = await browser.newBrowserCDPSession();

  try {
    // --- Context A: profile A ---
    // 1. Snapshot existing context IDs
    const { browserContextIds: before1 } = await browserCDP.send('Target.getBrowserContexts');
    // 2. Create context via Playwright
    const ctxA = await browser.newContext();
    // 3. Find the new browserContextId
    const { browserContextIds: after1 } = await browserCDP.send('Target.getBrowserContexts');
    const ctxIdA = after1.filter(id => !before1.includes(id))[0];
    // 4. Set flags BEFORE creating any page
    await browserCDP.send('BotBrowser.setBrowserContextFlags', {
      browserContextId: ctxIdA,
      botbrowserFlags: [`--bot-profile=${profileA}`],
    });
    // 5. Now create the page. The renderer starts with the correct flags.
    const pageA = await ctxA.newPage();

    // --- Context B: profile B + optional proxy ---
    const { browserContextIds: before2 } = await browserCDP.send('Target.getBrowserContexts');
    const ctxB = await browser.newContext();
    const { browserContextIds: after2 } = await browserCDP.send('Target.getBrowserContexts');
    const ctxIdB = after2.filter(id => !before2.includes(id))[0];

    const flagsB = [`--bot-profile=${profileB}`];
    if (proxyB) flagsB.push(`--proxy-server=${proxyB}`);

    await browserCDP.send('BotBrowser.setBrowserContextFlags', {
      browserContextId: ctxIdB,
      botbrowserFlags: flagsB,
    });
    const pageB = await ctxB.newPage();

    // Navigate both contexts
    await Promise.all([
      pageA.goto('https://example.com', { waitUntil: 'domcontentloaded' }),
      pageB.goto('https://example.com', { waitUntil: 'domcontentloaded' }),
    ]);

    // Collect fingerprints from each context
    const fpA = await collectFingerprint(pageA);
    const fpB = await collectFingerprint(pageB);

    console.log('Context A fingerprint:');
    printFingerprint(fpA);

    console.log('\nContext B fingerprint:');
    printFingerprint(fpB);

    // Check proxy IP for context B (if proxy was set)
    if (proxyB) {
      console.log('\n--- Proxy check (Context B) ---');
      try {
        await pageB.goto('https://httpbin.org/ip', { timeout: 20000 });
        const ipBody = await pageB.evaluate(() => document.body.innerText);
        const ip = JSON.parse(ipBody).origin;
        console.log(`  Context B exit IP: ${ip}`);
      } catch (e) {
        console.log(`  Proxy check failed: ${e.message}`);
      }
    }

    // Verify isolation
    console.log('\n--- Isolation check ---');
    const fields = ['userAgent', 'platform', 'hardwareConcurrency', 'screenWidth', 'webglRenderer'];
    let differences = 0;
    for (const f of fields) {
      const same = fpA[f] === fpB[f];
      if (!same) differences++;
      console.log(`  ${f}: ${same ? 'SAME' : 'DIFFERENT'}`);
    }
    console.log(`\n${differences} of ${fields.length} fields differ between contexts.`);

  } finally {
    await browser.close();
  }
})();

async function collectFingerprint(page) {
  return page.evaluate(() => {
    const r = {};
    r.userAgent = navigator.userAgent;
    r.platform = navigator.platform;
    r.hardwareConcurrency = navigator.hardwareConcurrency;
    r.deviceMemory = navigator.deviceMemory;
    r.screenWidth = screen.width;
    r.screenHeight = screen.height;
    try {
      const c = document.createElement('canvas');
      const gl = c.getContext('webgl');
      if (gl) {
        const d = gl.getExtension('WEBGL_debug_renderer_info');
        if (d) r.webglRenderer = gl.getParameter(d.UNMASKED_RENDERER_WEBGL);
      }
    } catch (_) {}
    r.timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
    return r;
  });
}

function printFingerprint(fp) {
  console.log(`  UA:       ${fp.userAgent}`);
  console.log(`  Platform: ${fp.platform}`);
  console.log(`  HW:       ${fp.hardwareConcurrency} cores, ${fp.deviceMemory}GB`);
  console.log(`  Screen:   ${fp.screenWidth}x${fp.screenHeight}`);
  console.log(`  WebGL:    ${fp.webglRenderer || 'N/A'}`);
  console.log(`  Timezone: ${fp.timezone}`);
}
