#!/usr/bin/env node

/**
 * BotBrowser Benchmark -Dimension 2: Per-Context FP vs Multi-Instance
 *
 * Compares launching N separate browser instances (each with a different profile)
 * against running N BrowserContexts within a single browser instance.
 *
 * Usage:
 *   node scripts/bench-scale.js --mode headless --max-scale 50
 *   node scripts/bench-scale.js --mode headed --max-scale 100
 */

import { chromium } from 'playwright-core';
import path from 'node:path';
import { getConfig } from '../config.js';
import {
  stats, saveResults, serveDir, parseArgs, printTable,
  measureAllChromeMemoryMB, countChromeProcesses, getSystemInfo,
} from '../utils.js';

const config = getConfig();
const args = parseArgs();
const HEADLESS = args.mode === 'headless';
const MAX_SCALE = args.maxScale;
const BENCH_DIR = config.benchDir;

// Scale levels to test (filtered by max-scale)
const SCALE_LEVELS = [1, 10, 25, 50, 100].filter(n => n <= MAX_SCALE);

// Linux-specific launch flags (needed when running as root)
const LINUX_FLAGS = process.platform === 'linux'
  ? ['--no-sandbox', '--disable-dev-shm-usage', '--disable-gpu']
  : [];

// Pick a base profile for testing (prefer android, fallback to any available)
const BASE_PROFILE = config.profiles.android || config.profiles.windows || config.profiles.macos;
if (!BASE_PROFILE) {
  console.error('Error: No profile found. Check PROFILES_BASE and PROFILE_FORMAT.');
  process.exit(1);
}

// ── Local page for load testing ────────────────────────────────────────

const LOCAL_HTML = '<!DOCTYPE html><html><head></head><body><h1>Benchmark Page</h1><canvas id="c" width="256" height="256"></canvas><script>const c=document.getElementById("c").getContext("2d");c.fillStyle="red";c.fillRect(0,0,128,128);c.font="14px Arial";c.fillText("test",10,200);</script></body></html>';

function setupRouteIntercept(page) {
  return page.route('https://example.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'text/html',
      body: LOCAL_HTML,
    });
  });
}

// ── Canvas hash for isolation check ────────────────────────────────────

async function getCanvasHash(page) {
  return page.evaluate(() => {
    const c = document.createElement('canvas');
    c.width = 256; c.height = 256;
    const ctx = c.getContext('2d');
    ctx.fillStyle = '#ff6600';
    ctx.fillRect(0, 0, 128, 128);
    ctx.font = '18px Arial';
    ctx.fillText('FP isolation test', 10, 200);
    ctx.arc(128, 128, 50, 0, Math.PI * 2);
    ctx.fill();
    return c.toDataURL();
  });
}

// ── Canvas toDataURL latency under load ────────────────────────────────

async function measureCanvasLatency(page) {
  return page.evaluate(() => {
    const c = document.createElement('canvas');
    c.width = 256; c.height = 256;
    const ctx = c.getContext('2d');
    ctx.fillStyle = '#0066ff';
    ctx.fillRect(0, 0, 256, 256);
    ctx.font = '16px Arial';
    ctx.fillText('latency test', 10, 128);

    const times = [];
    for (let i = 0; i < 100; i++) {
      const start = performance.now();
      c.toDataURL();
      times.push(performance.now() - start);
    }
    const sorted = times.sort((a, b) => a - b);
    return {
      median: sorted[Math.floor(sorted.length / 2)],
      p95: sorted[Math.floor(sorted.length * 0.95)],
      mean: sorted.reduce((s, v) => s + v, 0) / sorted.length,
    };
  });
}

// ── Multi-Instance Test ────────────────────────────────────────────────

async function testMultiInstance(n) {
  console.log(`\n  [Multi-Instance] Scale: ${n} browsers`);
  const browsers = [];
  const pages = [];

  const startCreate = performance.now();

  // Launch N browsers sequentially (parallel launch can overwhelm the system)
  for (let i = 0; i < n; i++) {
    const browser = await chromium.launch({
      executablePath: config.browserPath,
      headless: HEADLESS,
      args: [
        `--bot-profile=${BASE_PROFILE}`,
        `--bot-noise-seed=${i + 1}`,
        '--no-first-run',
        '--disable-sync',
        '--disable-extensions',
        '--disable-background-networking',
        ...LINUX_FLAGS,
      ],
    });
    browsers.push(browser);

    const page = await browser.newPage();
    await setupRouteIntercept(page);
    pages.push(page);
  }
  const createTimeMs = performance.now() - startCreate;

  // Navigate all pages concurrently
  const startLoad = performance.now();
  await Promise.all(pages.map(p => p.goto('https://example.com/test', { waitUntil: 'load' })));
  const loadTimeMs = performance.now() - startLoad;

  // Let things settle
  await new Promise(r => setTimeout(r, 2000));

  // Measure memory and processes
  const memoryMB = measureAllChromeMemoryMB();
  const processCount = countChromeProcesses();

  // Measure canvas latency under load (sample from first 5 pages)
  const latencies = [];
  for (let i = 0; i < Math.min(5, pages.length); i++) {
    try {
      const lat = await measureCanvasLatency(pages[i]);
      latencies.push(lat.median);
    } catch { /* skip */ }
  }

  // Check canvas hash isolation
  const hashes = new Set();
  for (let i = 0; i < Math.min(10, pages.length); i++) {
    try {
      const hash = await getCanvasHash(pages[i]);
      hashes.add(hash);
    } catch { /* skip */ }
  }

  // Cleanup
  for (const browser of browsers) {
    await browser.close().catch(() => {});
  }

  // Wait for processes to die
  await new Promise(r => setTimeout(r, 1000));

  return {
    scale: n,
    architecture: 'multi-instance',
    createTimeMs: Math.round(createTimeMs),
    loadTimeMs: Math.round(loadTimeMs),
    memoryMB,
    processCount,
    canvasLatencyMedianMs: latencies.length > 0 ? stats(latencies).median : -1,
    uniqueCanvasHashes: hashes.size,
    expectedUniqueHashes: Math.min(10, n),
  };
}

// ── Per-Context Test ───────────────────────────────────────────────────
//
// Correct CDP flow (matching reference test_noise-seed.js):
//   1. browser-level CDP session
//   2. Target.createBrowserContext
//   3. BotBrowser.setBrowserContextFlags (BEFORE any page/renderer exists)
//   4. browser.newContext() + newPage() for Playwright interaction
//
// Key insight: setBrowserContextFlags must be called on the browser CDP
// session BEFORE any renderer (page) is created in that context.

async function testPerContext(n) {
  console.log(`\n  [Per-Context] Scale: ${n} contexts`);

  const browser = await chromium.launch({
    executablePath: config.browserPath,
    headless: HEADLESS,
    args: [
      `--bot-profile=${BASE_PROFILE}`,
      '--no-first-run',
      '--disable-sync',
      '--disable-extensions',
      '--disable-background-networking',
      ...LINUX_FLAGS,
    ],
  });

  // Browser-level CDP session for setBrowserContextFlags
  const browserCDP = await browser.newBrowserCDPSession();

  const contexts = [];
  const pages = [];

  const startCreate = performance.now();

  for (let i = 0; i < n; i++) {
    // 1. Snapshot existing context IDs
    const { browserContextIds: before } = await browserCDP.send('Target.getBrowserContexts');

    // 2. Create context via Playwright (which internally calls Target.createBrowserContext)
    const context = await browser.newContext();

    // 3. Find the new browserContextId by diffing
    const { browserContextIds: after } = await browserCDP.send('Target.getBrowserContexts');
    const newIds = after.filter(id => !before.includes(id));
    const browserContextId = newIds[0];

    // 4. Set flags on browser CDP BEFORE any page is created
    if (browserContextId) {
      try {
        await browserCDP.send('BotBrowser.setBrowserContextFlags', {
          browserContextId,
          botbrowserFlags: [
            `--bot-profile=${BASE_PROFILE}`,
            `--bot-noise-seed=${i + 1}`,
          ],
        });
      } catch (e) {
        if (args.verbose) console.log(`    setBrowserContextFlags ctx ${i}: ${e.message}`);
      }
    } else if (args.verbose) {
      console.log(`    WARNING: Could not find browserContextId for context ${i}`);
    }

    // 5. NOW create the page (renderer will pick up the pre-set flags)
    const page = await context.newPage();
    await setupRouteIntercept(page);
    contexts.push(context);
    pages.push(page);
  }
  const createTimeMs = performance.now() - startCreate;

  // Navigate all pages concurrently
  const startLoad = performance.now();
  await Promise.all(pages.map(p => p.goto('https://example.com/test', { waitUntil: 'load' }).catch(() => {})));
  const loadTimeMs = performance.now() - startLoad;

  // Let things settle
  await new Promise(r => setTimeout(r, 2000));

  // Measure memory and processes
  const memoryMB = measureAllChromeMemoryMB();
  const processCount = countChromeProcesses();

  // Canvas latency
  const latencies = [];
  for (let i = 0; i < Math.min(5, pages.length); i++) {
    try {
      const lat = await measureCanvasLatency(pages[i]);
      latencies.push(lat.median);
    } catch { /* skip */ }
  }

  // Canvas hash isolation
  const hashes = new Set();
  for (let i = 0; i < Math.min(10, pages.length); i++) {
    try {
      const hash = await getCanvasHash(pages[i]);
      hashes.add(hash);
    } catch { /* skip */ }
  }

  // Cleanup
  await browser.close().catch(() => {});
  await new Promise(r => setTimeout(r, 1000));

  return {
    scale: n,
    architecture: 'per-context',
    createTimeMs: Math.round(createTimeMs),
    loadTimeMs: Math.round(loadTimeMs),
    memoryMB,
    processCount,
    canvasLatencyMedianMs: latencies.length > 0 ? stats(latencies).median : -1,
    uniqueCanvasHashes: hashes.size,
    expectedUniqueHashes: Math.min(10, n),
  };
}

// ── Context Create/Destroy Cycle Test ──────────────────────────────────

async function testContextLifecycle() {
  console.log('\n  [Per-Context] Lifecycle test: create/destroy 200 contexts...');

  const browser = await chromium.launch({
    executablePath: config.browserPath,
    headless: HEADLESS,
    args: [
      `--bot-profile=${BASE_PROFILE}`,
      '--no-first-run',
      '--disable-sync',
      '--disable-extensions',
      '--disable-background-networking',
      ...LINUX_FLAGS,
    ],
  });

  const CYCLES = 200;
  const createTimes = [];
  const destroyTimes = [];
  const memorySnapshots = [];

  for (let i = 0; i < CYCLES; i++) {
    const startCreate = performance.now();
    const context = await browser.newContext();
    const page = await context.newPage();
    createTimes.push(performance.now() - startCreate);

    await setupRouteIntercept(page);
    await page.goto('https://example.com/test', { waitUntil: 'load' });

    // Sample memory every 20 cycles
    if (i % 20 === 0) {
      memorySnapshots.push({
        cycle: i,
        memoryMB: measureAllChromeMemoryMB(),
      });
    }

    const startDestroy = performance.now();
    await context.close();
    destroyTimes.push(performance.now() - startDestroy);
  }

  // Final memory
  memorySnapshots.push({
    cycle: CYCLES,
    memoryMB: measureAllChromeMemoryMB(),
  });

  await browser.close().catch(() => {});

  return {
    cycles: CYCLES,
    createTime: stats(createTimes),
    destroyTime: stats(destroyTimes),
    memorySnapshots,
    memoryTrend: {
      start: memorySnapshots[0]?.memoryMB || -1,
      end: memorySnapshots[memorySnapshots.length - 1]?.memoryMB || -1,
    },
  };
}

// ── Main ───────────────────────────────────────────────────────────────

async function main() {
  console.log('='.repeat(70));
  console.log('BotBrowser Benchmark -Dimension 2: Per-Context vs Multi-Instance');
  console.log('='.repeat(70));
  console.log(`Platform: ${config.platform} (${config.arch})`);
  console.log(`Browser:  ${config.browserPath}`);
  console.log(`Profile:  ${BASE_PROFILE}`);
  console.log(`Mode:     ${args.mode}`);
  console.log(`Max scale: ${MAX_SCALE}`);
  console.log(`Scale levels: ${SCALE_LEVELS.join(', ')}`);
  console.log('');

  const allResults = {
    mode: args.mode,
    baseProfile: BASE_PROFILE,
    scaleLevels: SCALE_LEVELS,
    multiInstance: [],
    perContext: [],
    lifecycle: null,
  };

  // ── Scale Tests ──────────────────────────────────────────
  for (const n of SCALE_LEVELS) {
    console.log(`\n${'='.repeat(50)}`);
    console.log(`Scale level: ${n}`);
    console.log('='.repeat(50));

    // Multi-Instance first
    try {
      const miResult = await testMultiInstance(n);
      allResults.multiInstance.push(miResult);
      console.log(`    Memory: ${miResult.memoryMB} MB | Processes: ${miResult.processCount} | Create: ${miResult.createTimeMs}ms | Load: ${miResult.loadTimeMs}ms`);
    } catch (e) {
      console.log(`    Multi-Instance FAILED at scale ${n}: ${e.message}`);
      allResults.multiInstance.push({ scale: n, architecture: 'multi-instance', error: e.message });
    }

    // Per-Context second
    try {
      const pcResult = await testPerContext(n);
      allResults.perContext.push(pcResult);
      console.log(`    Memory: ${pcResult.memoryMB} MB | Processes: ${pcResult.processCount} | Create: ${pcResult.createTimeMs}ms | Load: ${pcResult.loadTimeMs}ms`);
    } catch (e) {
      console.log(`    Per-Context FAILED at scale ${n}: ${e.message}`);
      allResults.perContext.push({ scale: n, architecture: 'per-context', error: e.message });
    }
  }

  // ── Lifecycle Test ───────────────────────────────────────
  try {
    allResults.lifecycle = await testContextLifecycle();
    console.log(`\n  Lifecycle: ${allResults.lifecycle.cycles} cycles`);
    console.log(`    Create median: ${allResults.lifecycle.createTime.median.toFixed(1)}ms`);
    console.log(`    Destroy median: ${allResults.lifecycle.destroyTime.median.toFixed(1)}ms`);
    console.log(`    Memory trend: ${allResults.lifecycle.memoryTrend.start}MB -> ${allResults.lifecycle.memoryTrend.end}MB`);
  } catch (e) {
    console.log(`  Lifecycle test FAILED: ${e.message}`);
  }

  // ── Summary ──────────────────────────────────────────────
  console.log('\n' + '='.repeat(70));
  console.log('SUMMARY: Multi-Instance vs Per-Context');
  console.log('='.repeat(70));

  const rows = [];
  for (const n of SCALE_LEVELS) {
    const mi = allResults.multiInstance.find(r => r.scale === n);
    const pc = allResults.perContext.find(r => r.scale === n);

    if (mi && pc && !mi.error && !pc.error) {
      const memSavings = mi.memoryMB > 0 && pc.memoryMB > 0
        ? Math.round((1 - pc.memoryMB / mi.memoryMB) * 100) + '%'
        : 'N/A';
      const speedup = mi.createTimeMs > 0 && pc.createTimeMs > 0
        ? (mi.createTimeMs / pc.createTimeMs).toFixed(1) + 'x'
        : 'N/A';

      rows.push([
        n,
        `${mi.memoryMB}MB`, `${pc.memoryMB}MB`, memSavings,
        `${mi.processCount}`, `${pc.processCount}`,
        `${mi.createTimeMs}ms`, `${pc.createTimeMs}ms`, speedup,
      ]);
    }
  }

  if (rows.length > 0) {
    printTable(
      ['Scale', 'MI Mem', 'PC Mem', 'Savings', 'MI Procs', 'PC Procs', 'MI Create', 'PC Create', 'Speedup'],
      rows
    );
  }

  // Canvas isolation summary
  console.log('\nCANVAS FINGERPRINT ISOLATION:');
  const isoRows = [];
  for (const r of [...allResults.multiInstance, ...allResults.perContext]) {
    if (!r.error && r.uniqueCanvasHashes !== undefined) {
      isoRows.push([
        r.architecture, r.scale,
        `${r.uniqueCanvasHashes}/${r.expectedUniqueHashes}`,
        r.uniqueCanvasHashes >= r.expectedUniqueHashes ? 'PASS' : 'CHECK',
      ]);
    }
  }
  if (isoRows.length > 0) {
    printTable(['Architecture', 'Scale', 'Unique/Expected', 'Status'], isoRows);
  }

  // Save
  const resultPath = saveResults(`scale-${args.mode}`, allResults, BENCH_DIR);
  console.log(`\nFull results saved to: ${resultPath}`);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
