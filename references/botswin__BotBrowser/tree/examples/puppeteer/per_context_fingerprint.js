/**
 * PRIVACY RESEARCH USE ONLY
 * Run exclusively in authorized privacy research labs that comply with all applicable laws.
 * See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

/**
 * BotBrowser Per-Context Fingerprint Example (Puppeteer)
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
 *   node per_context_fingerprint.js
 */

const puppeteer = require('puppeteer-core');

(async () => {
  const execPath = process.env.BOTBROWSER_EXEC_PATH;
  const profileA = process.env.PROFILE_A;
  const profileB = process.env.PROFILE_B;

  if (!execPath || !profileA || !profileB) {
    console.log('Usage:');
    console.log('  BOTBROWSER_EXEC_PATH=/path/to/chrome \\');
    console.log('  PROFILE_A=/path/to/profile-a.enc \\');
    console.log('  PROFILE_B=/path/to/profile-b.enc \\');
    console.log('  node per_context_fingerprint.js');
    process.exit(1);
  }

  // Launch browser with profile A as the base profile
  const browser = await puppeteer.launch({
    browser: 'chrome',
    executablePath: execPath,
    headless: true,
    ignoreDefaultArgs: [
      '--no-startup-window',
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
  const client = await browser.target().createCDPSession();

  try {
    // --- Context A: profile A ---
    const ctxA = await browser.createBrowserContext();
    await client.send('BotBrowser.setBrowserContextFlags', {
      browserContextId: ctxA._contextId,
      botbrowserFlags: [`--bot-profile=${profileA}`],
    });
    // Create page AFTER setting flags. The renderer reads flags at startup.
    const pageA = await ctxA.newPage();

    // --- Context B: profile B ---
    const ctxB = await browser.createBrowserContext();
    await client.send('BotBrowser.setBrowserContextFlags', {
      browserContextId: ctxB._contextId,
      botbrowserFlags: [`--bot-profile=${profileB}`],
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
