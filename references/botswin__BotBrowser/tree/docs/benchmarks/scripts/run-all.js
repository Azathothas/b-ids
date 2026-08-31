#!/usr/bin/env node

/**
 * BotBrowser Benchmark -Full Orchestrator
 *
 * Runs all benchmark dimensions in sequence:
 *   1. Baseline (headless)
 *   2. Baseline (headed)
 *   3. Scale (headless)
 *   4. Scale (headed)
 *
 * Usage:
 *   node scripts/run-all.js                    # Run everything
 *   node scripts/run-all.js --skip-headed      # Headless only
 *   node scripts/run-all.js --skip-baseline    # Scale tests only
 *   node scripts/run-all.js --skip-scale       # Baseline tests only
 *   node scripts/run-all.js --max-scale 25     # Limit scale levels
 */

import { execSync, spawn } from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs';
import { getConfig } from '../config.js';
import { getSystemInfo } from '../utils.js';

const BENCH_DIR = path.dirname(new URL(import.meta.url).pathname.replace('/scripts', '').replace(/^\/([A-Z]:)/, '$1'));
const SCRIPTS_DIR = path.join(BENCH_DIR, 'scripts');

// Parse orchestrator-level args
const argv = process.argv.slice(2);
const skipHeaded = argv.includes('--skip-headed');
const skipBaseline = argv.includes('--skip-baseline');
const skipScale = argv.includes('--skip-scale');
const maxScaleIdx = argv.indexOf('--max-scale');
const maxScale = maxScaleIdx >= 0 ? argv[maxScaleIdx + 1] : '50';

// ── Pre-flight Checks ──────────────────────────────────────────────────

function preflight() {
  console.log('='.repeat(70));
  console.log('BotBrowser Performance Benchmark -Full Suite');
  console.log('='.repeat(70));

  // Check config
  let config;
  try {
    config = getConfig();
  } catch (e) {
    console.error(`Configuration error: ${e.message}`);
    process.exit(1);
  }

  console.log(`Platform:       ${config.platform} (${config.arch})`);
  console.log(`Browser:        ${config.browserPath}`);
  console.log(`Chrome version: ${config.chromeVersion}`);
  console.log('');

  // Check browser exists
  if (!fs.existsSync(config.browserPath)) {
    console.error(`Browser binary not found: ${config.browserPath}`);
    console.error('Set BROWSER_PATH environment variable to override.');
    process.exit(1);
  }

  // Check profiles
  const missingProfiles = [];
  for (const [platform, profilePath] of Object.entries(config.profiles)) {
    if (!profilePath) {
      missingProfiles.push(platform);
    } else if (!fs.existsSync(profilePath)) {
      missingProfiles.push(`${platform} (${profilePath})`);
    }
  }
  if (missingProfiles.length > 0) {
    console.warn(`Warning: Missing profiles for: ${missingProfiles.join(', ')}`);
    console.warn('These configurations will be skipped in baseline tests.');
    console.warn('');
  }

  // Check benchmark suites
  const benchmarksDir = path.join(BENCH_DIR, 'benchmarks');
  const suites = ['speedometer', 'jetstream', 'motionmark'];
  const missingSuites = suites.filter(s => !fs.existsSync(path.join(benchmarksDir, s, 'index.html')));
  if (missingSuites.length > 0) {
    console.warn(`Warning: Benchmark suites not found: ${missingSuites.join(', ')}`);
    console.warn('Run: bash setup-benchmarks.sh');
    console.warn('Standard benchmark tests (Speedometer/JetStream/MotionMark) will be skipped.');
    console.warn('Fingerprint API overhead and page load tests will still run.');
    console.warn('');
  }

  // Check playwright-core
  try {
    const pkg = path.join(BENCH_DIR, 'node_modules', 'playwright-core');
    if (!fs.existsSync(pkg)) {
      console.error('playwright-core not installed. Run: npm install');
      process.exit(1);
    }
  } catch {
    console.error('Cannot check dependencies. Run: npm install');
    process.exit(1);
  }

  // System info
  const sys = getSystemInfo();
  console.log(`System: ${sys.cpuModel} (${sys.cpuCores} cores), ${sys.totalMemoryGB}GB RAM`);
  console.log(`Node.js: ${sys.nodeVersion}`);
  console.log(`Date: ${sys.timestamp}`);
  console.log('');

  return config;
}

// ── Run a child script ─────────────────────────────────────────────────

function runScript(scriptName, extraArgs = []) {
  return new Promise((resolve, reject) => {
    const scriptPath = path.join(SCRIPTS_DIR, scriptName);
    const fullArgs = [scriptPath, ...extraArgs];

    console.log(`\n${'#'.repeat(70)}`);
    console.log(`# Running: node ${scriptName} ${extraArgs.join(' ')}`);
    console.log('#'.repeat(70) + '\n');

    const startTime = Date.now();

    const child = spawn('node', fullArgs, {
      cwd: BENCH_DIR,
      stdio: 'inherit',
      env: { ...process.env },
    });

    child.on('close', (code) => {
      const elapsed = Math.round((Date.now() - startTime) / 1000);
      if (code === 0) {
        console.log(`\n>>> ${scriptName} completed in ${elapsed}s`);
        resolve();
      } else {
        console.error(`\n>>> ${scriptName} failed with code ${code} after ${elapsed}s`);
        // Don't reject -continue with other tests
        resolve();
      }
    });

    child.on('error', (err) => {
      console.error(`\n>>> ${scriptName} spawn error: ${err.message}`);
      resolve();
    });
  });
}

// ── Main ───────────────────────────────────────────────────────────────

async function main() {
  const config = preflight();

  const startTotal = Date.now();
  const tasks = [];

  // Build task list
  if (!skipBaseline) {
    tasks.push({ script: 'bench-baseline.js', args: ['--mode', 'headless'] });
    if (!skipHeaded) {
      tasks.push({ script: 'bench-baseline.js', args: ['--mode', 'headed'] });
    }
  }

  if (!skipScale) {
    tasks.push({ script: 'bench-scale.js', args: ['--mode', 'headless', '--max-scale', maxScale] });
    if (!skipHeaded) {
      tasks.push({ script: 'bench-scale.js', args: ['--mode', 'headed', '--max-scale', maxScale] });
    }
  }

  console.log(`Planned tasks (${tasks.length}):`);
  for (const t of tasks) {
    console.log(`  - ${t.script} ${t.args.join(' ')}`);
  }
  console.log('');

  // Execute sequentially
  for (const task of tasks) {
    await runScript(task.script, task.args);
  }

  // Final summary
  const totalElapsed = Math.round((Date.now() - startTotal) / 1000);
  console.log('\n' + '='.repeat(70));
  console.log('ALL BENCHMARKS COMPLETE');
  console.log('='.repeat(70));
  console.log(`Total time: ${Math.floor(totalElapsed / 60)}m ${totalElapsed % 60}s`);
  console.log(`Results directory: ${path.join(BENCH_DIR, 'results')}`);

  // List result files
  const resultsDir = path.join(BENCH_DIR, 'results');
  if (fs.existsSync(resultsDir)) {
    const files = fs.readdirSync(resultsDir).filter(f => f.endsWith('.json'));
    console.log(`\nResult files (${files.length}):`);
    for (const f of files) {
      const stat = fs.statSync(path.join(resultsDir, f));
      console.log(`  ${f} (${Math.round(stat.size / 1024)}KB)`);
    }
  }
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
