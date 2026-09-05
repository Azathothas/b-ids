/**
 * BotBrowser Benchmark -Shared utilities
 *
 * Memory measurement, process counting, statistics, HTTP serving, result storage.
 */

import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import os from 'node:os';

// ── Memory & Process Measurement ──────────────────────────────────────

/**
 * Get total RSS (in MB) of a process tree rooted at `pid`.
 * Works on macOS, Linux, and Windows.
 */
export function measureMemoryMB(pid) {
  try {
    if (process.platform === 'win32') {
      const out = execSync(
        `wmic process where "ParentProcessId=${pid} or ProcessId=${pid}" get WorkingSetSize /value`,
        { encoding: 'utf8', timeout: 5000 }
      );
      let totalBytes = 0;
      for (const line of out.split('\n')) {
        const match = line.match(/WorkingSetSize=(\d+)/);
        if (match) totalBytes += parseInt(match[1], 10);
      }
      return Math.round(totalBytes / 1024 / 1024);
    }

    const out = execSync(
      `ps -o pid=,rss= -p $(pgrep -P ${pid} 2>/dev/null | tr '\\n' ',')${pid} 2>/dev/null || ps -o pid=,rss= -p ${pid}`,
      { encoding: 'utf8', timeout: 5000 }
    );
    let totalKB = 0;
    for (const line of out.trim().split('\n')) {
      const parts = line.trim().split(/\s+/);
      if (parts.length >= 2) totalKB += parseInt(parts[1], 10) || 0;
    }
    return Math.round(totalKB / 1024);
  } catch {
    return -1;
  }
}

/**
 * Measure total memory of ALL chrome/chromium processes on the system.
 * More reliable than tree-walking for multi-instance scenarios.
 */
export function measureAllChromeMemoryMB() {
  try {
    if (process.platform === 'win32') {
      const out = execSync(
        'wmic process where "name like \'%chrome%\'" get WorkingSetSize /value',
        { encoding: 'utf8', timeout: 5000 }
      );
      let totalBytes = 0;
      for (const line of out.split('\n')) {
        const match = line.match(/WorkingSetSize=(\d+)/);
        if (match) totalBytes += parseInt(match[1], 10);
      }
      return Math.round(totalBytes / 1024 / 1024);
    }

    const processName = process.platform === 'darwin' ? 'Chromium' : 'chrome';
    const out = execSync(
      `ps aux | grep -i "${processName}" | grep -v grep | awk '{sum += $6} END {print sum}'`,
      { encoding: 'utf8', timeout: 5000 }
    );
    const totalKB = parseInt(out.trim(), 10) || 0;
    return Math.round(totalKB / 1024);
  } catch {
    return -1;
  }
}

/**
 * Count all chrome/chromium processes on the system.
 */
export function countChromeProcesses() {
  try {
    if (process.platform === 'win32') {
      const out = execSync(
        'tasklist /fi "imagename eq chrome.exe" /nh',
        { encoding: 'utf8', timeout: 5000 }
      );
      return out.trim().split('\n').filter(l => l.includes('chrome')).length;
    }
    const out = execSync(
      `ps aux | grep -i -E "(Chromium|chrome)" | grep -v grep | wc -l`,
      { encoding: 'utf8', timeout: 5000 }
    );
    return parseInt(out.trim(), 10) || 0;
  } catch {
    return -1;
  }
}

// ── Statistics ─────────────────────────────────────────────────────────

export function stats(arr) {
  if (!arr || arr.length === 0) return { median: 0, p95: 0, mean: 0, min: 0, max: 0, stdev: 0, n: 0 };
  const sorted = [...arr].sort((a, b) => a - b);
  const n = sorted.length;
  const mean = sorted.reduce((s, v) => s + v, 0) / n;
  const variance = sorted.reduce((s, v) => s + (v - mean) ** 2, 0) / n;
  return {
    median: sorted[Math.floor(n / 2)],
    p95: sorted[Math.floor(n * 0.95)],
    mean: Math.round(mean * 100) / 100,
    min: sorted[0],
    max: sorted[n - 1],
    stdev: Math.round(Math.sqrt(variance) * 100) / 100,
    n,
  };
}

// ── System Info ────────────────────────────────────────────────────────

export function getSystemInfo() {
  return {
    platform: process.platform,
    arch: os.arch(),
    cpuModel: os.cpus()[0]?.model || 'unknown',
    cpuCores: os.cpus().length,
    totalMemoryGB: Math.round(os.totalmem() / 1024 / 1024 / 1024),
    osRelease: os.release(),
    nodeVersion: process.version,
    timestamp: new Date().toISOString(),
  };
}

// ── Result Storage ─────────────────────────────────────────────────────

export function saveResults(name, data, benchDir) {
  const resultsDir = path.join(benchDir, 'results');
  fs.mkdirSync(resultsDir, { recursive: true });

  const platformTag = process.platform === 'darwin' ? 'macos' : process.platform === 'win32' ? 'windows' : 'linux';
  const filename = `${name}-${platformTag}.json`;
  const filepath = path.join(resultsDir, filename);

  const output = {
    system: getSystemInfo(),
    ...data,
  };

  fs.writeFileSync(filepath, JSON.stringify(output, null, 2));
  console.log(`Results saved to ${filepath}`);
  return filepath;
}

// ── HTTP Static File Server ────────────────────────────────────────────

const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.mjs': 'application/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.gif': 'image/gif',
  '.ico': 'image/x-icon',
  '.txt': 'text/plain',
  '.xml': 'application/xml',
};

/**
 * Start a simple static file server for a directory.
 * Returns { server, url, close() }.
 */
export function serveDir(dirPath, port = 0) {
  const resolvedDir = path.resolve(dirPath);
  return new Promise((resolve, reject) => {
    const server = http.createServer((req, res) => {
      let urlPath = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
      if (urlPath === '/') urlPath = '/index.html';

      const filePath = path.resolve(path.join(resolvedDir, urlPath));

      // Security: prevent directory traversal
      if (!filePath.startsWith(resolvedDir)) {
        res.writeHead(403);
        res.end('Forbidden');
        return;
      }

      if (!fs.existsSync(filePath)) {
        res.writeHead(404);
        res.end('Not Found');
        return;
      }

      const stat = fs.statSync(filePath);
      if (stat.isDirectory()) {
        const indexPath = path.join(filePath, 'index.html');
        if (fs.existsSync(indexPath)) {
          res.writeHead(200, { 'Content-Type': 'text/html' });
          fs.createReadStream(indexPath).pipe(res);
          return;
        }
        res.writeHead(404);
        res.end('Not Found');
        return;
      }

      const ext = path.extname(filePath).toLowerCase();
      const contentType = MIME_TYPES[ext] || 'application/octet-stream';
      res.writeHead(200, {
        'Content-Type': contentType,
        'Cache-Control': 'no-cache',
      });
      fs.createReadStream(filePath).pipe(res);
    });

    server.on('error', reject);
    server.listen(port, '127.0.0.1', () => {
      const addr = server.address();
      const url = `http://127.0.0.1:${addr.port}`;
      resolve({
        server,
        url,
        close: () => new Promise(r => server.close(r)),
      });
    });
  });
}

// ── Timing Helpers ─────────────────────────────────────────────────────

export function timedMs(fn) {
  const start = performance.now();
  const result = fn();
  const elapsed = performance.now() - start;
  return { result, elapsed };
}

export async function timedMsAsync(fn) {
  const start = performance.now();
  const result = await fn();
  const elapsed = performance.now() - start;
  return { result, elapsed };
}

// ── CLI Argument Parsing ───────────────────────────────────────────────

export function parseArgs() {
  const args = process.argv.slice(2);
  const parsed = {
    mode: 'headless', // headed | headless
    maxScale: 100,
    runs: 3,
    verbose: false,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--mode':
        parsed.mode = args[++i];
        break;
      case '--max-scale':
        parsed.maxScale = parseInt(args[++i], 10);
        break;
      case '--runs':
        parsed.runs = parseInt(args[++i], 10);
        break;
      case '--verbose':
        parsed.verbose = true;
        break;
    }
  }

  return parsed;
}

// ── Console Formatting ─────────────────────────────────────────────────

export function printTable(headers, rows) {
  const widths = headers.map((h, i) =>
    Math.max(h.length, ...rows.map(r => String(r[i] ?? '').length))
  );
  const sep = widths.map(w => '-'.repeat(w + 2)).join('+');
  const fmt = (row) => row.map((c, i) => ` ${String(c ?? '').padEnd(widths[i])} `).join('|');

  console.log(fmt(headers));
  console.log(sep);
  for (const row of rows) {
    console.log(fmt(row));
  }
}

/**
 * Wait for a condition to become true in the page, with timeout.
 */
export async function waitForPageCondition(page, conditionFn, timeoutMs = 300000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const result = await page.evaluate(conditionFn).catch(() => false);
    if (result) return;
    await new Promise(r => setTimeout(r, 1000));
  }
  throw new Error(`Timeout waiting for condition after ${timeoutMs}ms`);
}
