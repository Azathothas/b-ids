/**
 * BotBrowser Benchmark -Platform configuration auto-detection
 *
 * Resolves browser binary paths and profile paths per platform.
 * All paths are configurable via environment variables.
 *
 * Environment variables:
 *   BROWSER_PATH     -Path to BotBrowser/Chromium executable
 *   PROFILES_BASE    -Root directory containing profile files
 *   CHROME_VERSION   -Chrome major version (default: "145")
 */

import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

const HOST_OS = process.platform; // darwin | linux | win32
const CHROME_VERSION = process.env.CHROME_VERSION || '145';

// ── Browser binary paths ──────────────────────────────────────────────
// Override with BROWSER_PATH env var for your setup.

const BROWSER_PATHS = {
  darwin: '/Applications/BotBrowser.app/Contents/MacOS/BotBrowser',
  linux: '/opt/chromium.org/chromium/chrome',
  win32: 'C:\\Program Files\\BotBrowser\\chrome.exe',
};

// ── Profiles base directory ───────────────────────────────────────────
// Override with PROFILES_BASE env var. Should point to the root of your
// BotBrowser profiles directory (the one containing public/{version}/{date}/).

const PROFILES_BASE = process.env.PROFILES_BASE || '';

// ── Profile discovery ────────────────────────────────────────────────

const PLATFORM_MAP = {
  android: ['android13'],
  windows: ['win11_x64', 'win10_x64'],
  macos: ['mac_arm64'],
};

/**
 * Scan PROFILES_BASE/public/{semver}/{date}/ to find the latest .enc
 * profile for a given Chrome major version + platform.
 */
function findLatestProfile(platform) {
  const slugs = PLATFORM_MAP[platform];
  if (!slugs) throw new Error(`Unknown platform: ${platform}`);

  if (!PROFILES_BASE) {
    throw new Error('PROFILES_BASE not set. Point it to your BotBrowser profiles directory.');
  }

  for (const slug of slugs) {
    const filename = `chrome${CHROME_VERSION}_${slug}.enc`;

    // Check premium_test directory first
    const premiumDir = findLatestVersionDateDir();
    if (premiumDir) {
      const premiumPath = path.join(premiumDir, 'tests', 'premium_test', filename);
      if (fs.existsSync(premiumPath)) return premiumPath;
    }

    // Scan version/date directories
    const publicDir = path.join(PROFILES_BASE, 'public');
    if (!fs.existsSync(publicDir)) continue;

    const candidates = [];
    for (const versionDir of fs.readdirSync(publicDir).sort()) {
      const versionPath = path.join(publicDir, versionDir);
      if (!fs.statSync(versionPath).isDirectory()) continue;
      for (const dateDir of fs.readdirSync(versionPath).sort()) {
        const datePath = path.join(versionPath, dateDir);
        if (!fs.statSync(datePath).isDirectory()) continue;
        const candidate = path.join(datePath, filename);
        if (fs.existsSync(candidate)) {
          candidates.push(candidate);
        }
      }
    }

    if (candidates.length > 0) {
      return candidates[candidates.length - 1]; // latest by lexicographic sort
    }
  }

  throw new Error(`No profile found for chrome${CHROME_VERSION} ${platform} in ${path.join(PROFILES_BASE, 'public')}`);
}

function findLatestVersionDateDir() {
  const publicDir = path.join(PROFILES_BASE, 'public');
  if (!fs.existsSync(publicDir)) return null;

  let latest = null;
  for (const versionDir of fs.readdirSync(publicDir).sort()) {
    const versionPath = path.join(publicDir, versionDir);
    if (!fs.statSync(versionPath).isDirectory()) continue;
    for (const dateDir of fs.readdirSync(versionPath).sort()) {
      const datePath = path.join(versionPath, dateDir);
      if (!fs.statSync(datePath).isDirectory()) continue;
      latest = datePath;
    }
  }
  return latest;
}

// ── Build config object ───────────────────────────────────────────────

export function getConfig() {
  const browserPath = process.env.BROWSER_PATH || BROWSER_PATHS[HOST_OS];
  if (!browserPath) {
    throw new Error(`Unsupported host OS: ${HOST_OS}. Set BROWSER_PATH env var.`);
  }

  const profiles = {};
  for (const platform of ['android', 'windows', 'macos']) {
    try {
      profiles[platform] = findLatestProfile(platform);
    } catch (e) {
      console.warn(`Warning: Could not find ${platform} profile: ${e.message}`);
      profiles[platform] = null;
    }
  }

  return {
    browserPath,
    profiles,
    platform: HOST_OS,
    arch: os.arch(),
    chromeVersion: CHROME_VERSION,
    benchDir: path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Z]:)/, '$1')),
  };
}

export { PLATFORM_MAP, CHROME_VERSION };
