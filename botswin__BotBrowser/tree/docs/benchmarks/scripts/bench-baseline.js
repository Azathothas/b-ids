#!/usr/bin/env node

/**
 * BotBrowser Benchmark -Dimension 1: Single Instance Baseline
 *
 * Compares Stock Chrome (no profile) vs BotBrowser with Android/Windows/macOS profiles.
 * Tests: Speedometer 3.0, JetStream 3, MotionMark, fingerprint API overhead, page loads.
 *
 * Usage:
 *   node scripts/bench-baseline.js --mode headless
 *   node scripts/bench-baseline.js --mode headed --runs 5
 */

import { chromium } from 'playwright-core';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { getConfig } from '../config.js';
import {
  stats, saveResults, serveDir, parseArgs, printTable,
  waitForPageCondition, timedMsAsync, getSystemInfo,
} from '../utils.js';

const config = getConfig();
const args = parseArgs();
const HEADLESS = args.mode === 'headless';
const RUNS = args.runs;
const BENCH_DIR = config.benchDir;

// ── Test Configurations ────────────────────────────────────────────────

function buildConfigs() {
  const configs = [
    { name: 'Stock Chrome (no profile)', profile: null },
  ];

  for (const [platform, profilePath] of Object.entries(config.profiles)) {
    if (profilePath) {
      configs.push({ name: `BotBrowser + ${platform}`, profile: profilePath });
    }
  }

  return configs;
}

// ── Browser Launch ─────────────────────────────────────────────────────

async function launchBrowser(profilePath) {
  const launchArgs = [
    '--no-first-run',
    '--disable-sync',
    '--disable-extensions',
    '--disable-background-networking',
  ];

  // Linux needs extra flags (especially when running as root)
  if (process.platform === 'linux') {
    launchArgs.push(
      '--no-sandbox',
      '--disable-dev-shm-usage',
      '--disable-gpu',
    );
  }

  if (profilePath) {
    launchArgs.push(`--bot-profile=${profilePath}`);
  }

  return chromium.launch({
    executablePath: config.browserPath,
    headless: HEADLESS,
    args: launchArgs,
  });
}

// ── Benchmark Suite Runners ────────────────────────────────────────────

async function runSpeedometer(browser, serverUrl) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();
  try {
    await page.goto(`${serverUrl}/`, { waitUntil: 'load', timeout: 60000 });

    // Click "Start Test" button
    await page.click('button.start-tests-button', { timeout: 10000 });

    // Wait for benchmark to complete -score appears in #result-number
    // The page transitions from #running to #summary section when done
    await waitForPageCondition(page, () => {
      const el = document.querySelector('#result-number');
      return el && el.textContent && el.textContent.trim().length > 0 && parseFloat(el.textContent) > 0;
    }, 300000); // 5 min timeout

    // Extract score
    const score = await page.evaluate(() => {
      const el = document.querySelector('#result-number');
      return el ? parseFloat(el.textContent) : null;
    });

    return score;
  } finally {
    await context.close();
  }
}

async function runJetStream(browser, serverUrl) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();
  try {
    // Use startAutomatically=true to auto-start (JetStream doesn't start by default)
    await page.goto(`${serverUrl}/?startAutomatically=true`, { waitUntil: 'load', timeout: 120000 });

    // Wait for completion: JetStream.isDone becomes true or result-summary gets 'done' class
    // JetStream 3 can take 10-30 minutes depending on hardware
    await waitForPageCondition(page, () => {
      return window.JetStream?.isDone === true
        || document.getElementById('result-summary')?.classList.contains('done');
    }, 1800000); // 30 min timeout

    const score = await page.evaluate(() => {
      const el = document.querySelector('#result-summary .score');
      return el ? parseFloat(el.textContent) : null;
    });

    return score;
  } finally {
    await context.close();
  }
}

async function runMotionMark(browser, serverUrl) {
  // MotionMark requires landscape viewport -create page with explicit wide viewport
  // This is needed especially for mobile profiles (Android) which set smaller screen sizes
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();
  try {
    // MotionMark root index.html redirects to MotionMark/ subdirectory
    await page.goto(`${serverUrl}/MotionMark/`, { waitUntil: 'load', timeout: 60000 });

    // Wait for start button to exist in DOM (may be hidden due to CSS orientation checks)
    await page.waitForSelector('#start-button', { state: 'attached', timeout: 30000 });

    // Wait for frame rate detection, then force-start via JS (bypasses visibility/disabled checks)
    let started = false;
    for (let attempt = 0; attempt < 20 && !started; attempt++) {
      await new Promise(r => setTimeout(r, 2000));
      try {
        started = await page.evaluate(() => {
          const btn = document.getElementById('start-button');
          if (btn && !btn.disabled) {
            benchmarkController.startBenchmark();
            return true;
          }
          // If still detecting frame rate, check if we can force it
          if (btn && btn.disabled && btn.textContent.includes('Run')) {
            btn.disabled = false;
            benchmarkController.startBenchmark();
            return true;
          }
          return false;
        });
      } catch { /* retry */ }
    }

    if (!started) throw new Error('Could not start MotionMark benchmark');

    // Wait for completion -body gets class 'showing-results' and score is populated
    await waitForPageCondition(page, () => {
      return document.body.classList.contains('showing-results')
        && document.querySelector('#results .score')?.textContent?.trim().length > 0;
    }, 600000);

    // Score format: "{score} @ {fps}fps" -parseFloat extracts the numeric score
    const score = await page.evaluate(() => {
      const el = document.querySelector('#results .score');
      return el ? parseFloat(el.textContent) : null;
    });

    return score;
  } finally {
    await context.close();
  }
}

// ── Fingerprint API Overhead ───────────────────────────────────────────

async function runFingerprintBench(browser, pagesServerUrl) {
  const page = await browser.newPage();
  try {
    await page.goto(`${pagesServerUrl}/fingerprint-bench.html`, { waitUntil: 'load', timeout: 30000 });

    // Wait for benchmark to finish
    await waitForPageCondition(page, () => window.__benchDone === true, 120000);

    const results = await page.evaluate(() => window.__benchResults);
    return results;
  } finally {
    await page.close();
  }
}

// ── Page Load Timing ───────────────────────────────────────────────────

async function measurePageLoad(browser, url, label) {
  const times = [];

  for (let i = 0; i < 10; i++) {
    const page = await browser.newPage();
    try {
      const start = performance.now();
      await page.goto(url, { waitUntil: 'load', timeout: 30000 });
      const elapsed = performance.now() - start;
      times.push(elapsed);
    } catch (e) {
      times.push(-1);
    } finally {
      await page.close();
    }
  }

  return { label, url, ...stats(times.filter(t => t >= 0)) };
}

async function measurePageLoadLocal(browser) {
  // Use route interception to serve pages locally
  const page = await browser.newPage();
  const times = [];

  await page.route('https://example.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'text/html',
      body: '<!DOCTYPE html><html><head></head><body><h1>Benchmark</h1><p>Local page for load timing.</p></body></html>',
    });
  });

  for (let i = 0; i < 10; i++) {
    const start = performance.now();
    await page.goto('https://example.com/bench', { waitUntil: 'load' });
    times.push(performance.now() - start);
  }

  await page.close();
  return { label: 'Local intercepted page', ...stats(times) };
}

// ── Main ───────────────────────────────────────────────────────────────

async function main() {
  console.log('='.repeat(70));
  console.log('BotBrowser Benchmark -Dimension 1: Single Instance Baseline');
  console.log('='.repeat(70));
  console.log(`Platform: ${config.platform} (${config.arch})`);
  console.log(`Browser:  ${config.browserPath}`);
  console.log(`Mode:     ${args.mode}`);
  console.log(`Runs:     ${RUNS}`);
  console.log('');

  const testConfigs = buildConfigs();
  console.log(`Test configurations: ${testConfigs.map(c => c.name).join(', ')}`);
  console.log('');

  // Start local HTTP servers
  const pagesDir = path.join(BENCH_DIR, 'pages');
  const pagesServer = await serveDir(pagesDir);
  console.log(`Pages server: ${pagesServer.url}`);

  // Check for benchmark suites
  const benchmarksDir = path.join(BENCH_DIR, 'benchmarks');
  const hasSpeedometer = fs.existsSync(path.join(benchmarksDir, 'speedometer', 'index.html'));
  const hasJetStream = fs.existsSync(path.join(benchmarksDir, 'jetstream', 'index.html'));
  const hasMotionMark = fs.existsSync(path.join(benchmarksDir, 'motionmark', 'MotionMark', 'index.html'));

  let speedometerServer, jetStreamServer, motionMarkServer;
  if (hasSpeedometer) {
    speedometerServer = await serveDir(path.join(benchmarksDir, 'speedometer'));
    console.log(`Speedometer server: ${speedometerServer.url}`);
  }
  if (hasJetStream) {
    jetStreamServer = await serveDir(path.join(benchmarksDir, 'jetstream'));
    console.log(`JetStream server: ${jetStreamServer.url}`);
  }
  if (hasMotionMark) {
    motionMarkServer = await serveDir(path.join(benchmarksDir, 'motionmark'));
    console.log(`MotionMark server: ${motionMarkServer.url}`);
  }

  console.log('');

  const allResults = {
    mode: args.mode,
    configurations: {},
  };

  // Initialize result structures for each config
  for (const cfg of testConfigs) {
    allResults.configurations[cfg.name] = {
      name: cfg.name,
      profile: cfg.profile,
      benchmarks: {},
      fingerprint: null,
      pageLoads: [],
    };
  }

  // ── Standard Benchmarks (INTERLEAVED to eliminate warm-up bias) ──────
  // Instead of running all Stock runs then all BB runs, we alternate:
  //   Run 1: Stock → BB+android → BB+windows → ...
  //   Run 2: Stock → BB+android → BB+windows → ...
  // This ensures each config gets the same thermal/cache conditions.

  const benchmarks = [
    { key: 'speedometer', server: speedometerServer, runner: runSpeedometer, label: 'Speedometer 3.0' },
    { key: 'jetstream', server: jetStreamServer, runner: runJetStream, label: 'JetStream 3' },
    { key: 'motionmark', server: motionMarkServer, runner: runMotionMark, label: 'MotionMark' },
  ];

  for (const bench of benchmarks) {
    if (!bench.server) continue;

    console.log('='.repeat(50));
    console.log(`${bench.label} -${RUNS} runs x ${testConfigs.length} configs (interleaved)`);
    console.log('='.repeat(50));

    // Collect scores per config
    const scoresByConfig = {};
    for (const cfg of testConfigs) scoresByConfig[cfg.name] = [];

    // Warm-up run (discarded) -run each config once to warm caches
    console.log('  Warm-up round (discarded)...');
    for (const cfg of testConfigs) {
      const browser = await launchBrowser(cfg.profile);
      try {
        await bench.runner(browser, bench.server.url);
        console.log(`    ${cfg.name}: done`);
      } catch (e) {
        console.log(`    ${cfg.name}: ${e.message}`);
      } finally {
        await browser.close();
      }
    }

    // Interleaved measurement runs
    for (let run = 0; run < RUNS; run++) {
      console.log(`  Run ${run + 1}/${RUNS}:`);
      for (const cfg of testConfigs) {
        const browser = await launchBrowser(cfg.profile);
        try {
          const score = await bench.runner(browser, bench.server.url);
          if (score !== null) scoresByConfig[cfg.name].push(score);
          console.log(`    ${cfg.name}: ${score}`);
        } catch (e) {
          console.log(`    ${cfg.name}: ERROR - ${e.message}`);
        } finally {
          await browser.close();
        }
      }
    }

    // Store stats
    for (const cfg of testConfigs) {
      const scores = scoresByConfig[cfg.name];
      allResults.configurations[cfg.name].benchmarks[bench.key] = stats(scores);
      const s = stats(scores);
      console.log(`  ${cfg.name} -median: ${s.median}, stddev: ${s.stdev?.toFixed(2) ?? 'N/A'}, n=${s.n}`);
    }
    console.log('');
  }

  // ── Fingerprint API Overhead (per config, single run) ──────────────
  for (const cfg of testConfigs) {
    console.log(`  Fingerprint API: ${cfg.name}...`);
    const browser = await launchBrowser(cfg.profile);
    try {
      allResults.configurations[cfg.name].fingerprint = await runFingerprintBench(browser, pagesServer.url);
      const apiCount = Object.keys(allResults.configurations[cfg.name].fingerprint || {}).length;
      console.log(`    ${apiCount} APIs tested`);
    } catch (e) {
      console.log(`    ERROR: ${e.message}`);
    } finally {
      await browser.close();
    }
  }

  // ── Page Load Timing (per config) ──────────────────────────────────
  for (const cfg of testConfigs) {
    console.log(`  Page load: ${cfg.name}...`);
    const browser = await launchBrowser(cfg.profile);
    try {
      const localResult = await measurePageLoadLocal(browser);
      allResults.configurations[cfg.name].pageLoads.push(localResult);
      console.log(`    Local page: ${localResult.median.toFixed(1)}ms`);

      const blankResult = await measurePageLoad(browser, 'about:blank', 'about:blank');
      allResults.configurations[cfg.name].pageLoads.push(blankResult);
      console.log(`    about:blank: ${blankResult.median.toFixed(1)}ms`);
    } catch (e) {
      console.log(`    ERROR: ${e.message}`);
    } finally {
      await browser.close();
    }
  }

  // ── Summary ──────────────────────────────────────────────────────────
  console.log('='.repeat(70));
  console.log('SUMMARY');
  console.log('='.repeat(70));

  // Benchmark scores comparison
  const benchNames = ['speedometer', 'jetstream', 'motionmark'];
  for (const benchName of benchNames) {
    const rows = [];
    for (const [cfgName, cfgResult] of Object.entries(allResults.configurations)) {
      const b = cfgResult.benchmarks[benchName];
      if (b && b.n > 0) {
        rows.push([cfgName, b.median, b.mean, b.stdev, b.n]);
      }
    }
    if (rows.length > 0) {
      console.log(`\n${benchName.toUpperCase()}:`);
      printTable(['Configuration', 'Median', 'Mean', 'StdDev', 'Runs'], rows);
    }
  }

  // Fingerprint API comparison
  console.log('\nFINGERPRINT API OVERHEAD (median ms):');
  const fpHeaders = ['API'];
  const fpAPIs = new Set();
  for (const cfgResult of Object.values(allResults.configurations)) {
    fpHeaders.push(cfgResult.name.substring(0, 25));
    if (cfgResult.fingerprint) {
      for (const key of Object.keys(cfgResult.fingerprint)) fpAPIs.add(key);
    }
  }
  const fpRows = [];
  for (const api of fpAPIs) {
    const row = [api];
    for (const cfgResult of Object.values(allResults.configurations)) {
      const val = cfgResult.fingerprint?.[api]?.median;
      row.push(val !== undefined ? val.toFixed(4) : 'N/A');
    }
    fpRows.push(row);
  }
  if (fpRows.length > 0) {
    printTable(fpHeaders, fpRows);
  }

  // Page load comparison
  console.log('\nPAGE LOAD TIMING (median ms):');
  const pageHeaders = ['Page'];
  for (const cfgResult of Object.values(allResults.configurations)) {
    pageHeaders.push(cfgResult.name.substring(0, 25));
  }
  const pageLabels = new Set();
  for (const cfgResult of Object.values(allResults.configurations)) {
    for (const pl of cfgResult.pageLoads) pageLabels.add(pl.label);
  }
  const pageRows = [];
  for (const label of pageLabels) {
    const row = [label];
    for (const cfgResult of Object.values(allResults.configurations)) {
      const pl = cfgResult.pageLoads.find(p => p.label === label);
      row.push(pl ? pl.median.toFixed(1) : 'N/A');
    }
    pageRows.push(row);
  }
  if (pageRows.length > 0) {
    printTable(pageHeaders, pageRows);
  }

  // Save results
  const resultPath = saveResults(`baseline-${args.mode}`, allResults, BENCH_DIR);
  console.log(`\nFull results saved to: ${resultPath}`);

  // Cleanup
  await pagesServer.close();
  if (speedometerServer) await speedometerServer.close();
  if (jetStreamServer) await jetStreamServer.close();
  if (motionMarkServer) await motionMarkServer.close();
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
