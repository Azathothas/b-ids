#!/usr/bin/env node
// scripts/common/vendor-sync.mjs - fetch a pristine copy of a vendored upstream, and materialise
// a vendored tree from it.
//
// -- WHAT IT IS FOR ---------------------------------------------------------------------------
// docs/methodology/vendoring.md says the vendored tree is the truth and a derived patch series is
// regenerated from it. Both halves of that need one thing this repository does not otherwise
// have: a PRISTINE copy of the upstream at the recorded commit, to diff the tree against and to
// build the next reconciliation on. This fetches it, and it materialises the tree the first time.
//
// It refuses to overwrite a tree that already has content, because the tree carries local patches
// and a refresh that took upstream's copy would delete them with no diff to notice it by.
//
// -- WHY NODE, WITH NO POWERSHELL TWIN --------------------------------------------------------
// The same reason write-file.mjs and set-record.mjs have none, and scripts/README.md carries it:
// node is the same program on every host, and this is a helper rather than a check.
//
//   node scripts/common/vendor-sync.mjs pristine
//   node scripts/common/vendor-sync.mjs pristine --name rustls
//   node scripts/common/vendor-sync.mjs materialise --name rustls
//   node scripts/common/vendor-sync.mjs materialise --name rustls --force
//   node scripts/common/vendor-sync.mjs pristine --json
//
// The "pristine" mode clones the recorded ref into .tmp/vendor-pristine/NAME and asserts that it
// resolves to the recorded base. ⚠ A ref that has MOVED is reported and not followed: the recorded
// base is what the patch series was generated against, so following a moved tag would silently
// change what every patch is a diff from.
//
// The "materialise" mode copies that pristine tree into the manifest's directory, dropping every
// excluded path. ⛔ It is the FIRST vendoring only. Taking a new release is a reconciliation,
// which is a reading rather than a copy, and docs/methodology/vendoring.md says what it owes.
//
// Exit codes: 0 done, 1 refused, 2 could not run.
//
// ⛔ Read the exit code from this process, unpiped.

import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, cpSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, '..', '..');
const MANIFEST = join(ROOT, 'vendor', 'upstream.json');
const PRISTINE = join(ROOT, '.tmp', 'vendor-pristine');

function die(code, message) {
  process.stderr.write('vendor-sync: ' + message + '\n');
  process.exit(code);
}

function git(args, cwd) {
  const r = spawnSync('git', args, { cwd: cwd ?? ROOT, encoding: 'utf8' });
  if (r.error) die(2, 'git could not be run: ' + r.error.message);
  return { code: r.status, out: (r.stdout ?? '').trim(), err: (r.stderr ?? '').trim() };
}

function usage() {
  process.stdout.write(
    'usage: node scripts/common/vendor-sync.mjs <pristine|materialise> [--name NAME] [--force] [--json]\n'
    + '       read the header of this file for the full contract\n'
  );
}

const argv = process.argv.slice(2);
const mode = argv[0] ?? '';
let only = null;
let force = false;
let json = false;
for (let i = 1; i < argv.length; i += 1) {
  if (argv[i] === '--name') { only = argv[i + 1]; i += 1; }
  else if (argv[i] === '--force') force = true;
  else if (argv[i] === '--json') json = true;
  else if (argv[i] === '-h' || argv[i] === '--help') { usage(); process.exit(0); }
  else die(2, 'unknown argument: ' + argv[i]);
}

if (mode === '-h' || mode === '--help') { usage(); process.exit(0); }
if (mode !== 'pristine' && mode !== 'materialise') { usage(); process.exit(2); }
if (!existsSync(MANIFEST)) die(2, 'no manifest at vendor/upstream.json');

let manifest;
try {
  manifest = JSON.parse(readFileSync(MANIFEST, 'utf8'));
} catch (e) {
  die(2, 'vendor/upstream.json does not parse: ' + e.message);
}
const upstreams = (manifest.upstreams ?? []).filter((u) => only === null || u.name === only);
if (upstreams.length === 0) {
  die(2, only === null ? 'the manifest names no upstream' : 'no upstream named ' + only);
}

// ⚠ A shallow clone AT A TAG is the cheapest fetch that can still be checked, and checking is the
// point: the recorded base is what the patch series is a diff from. A clone that resolved to a
// different commit is reported rather than used.
function fetchPristine(u) {
  const dest = join(PRISTINE, u.name);
  if (existsSync(join(dest, '.git'))) {
    const head = git(['rev-parse', 'HEAD'], dest);
    if (head.code === 0 && head.out === u.base) return { dest, reused: true, head: head.out };
    rmSync(dest, { recursive: true, force: true });
  }
  mkdirSync(PRISTINE, { recursive: true });
  const cloned = git(['clone', '--quiet', '--depth', '1', '--branch', u.ref, u.repository, dest]);
  if (cloned.code !== 0) {
    die(1, 'clone of ' + u.repository + ' at ' + u.ref + ' failed: ' + cloned.err);
  }
  const head = git(['rev-parse', 'HEAD'], dest);
  if (head.code !== 0) die(2, 'cannot read HEAD of the pristine copy: ' + head.err);
  return { dest, reused: false, head: head.out };
}

const results = [];
for (const u of upstreams) {
  const got = fetchPristine(u);
  if (got.head !== u.base) {
    // ⛔ Reported, never followed. Following a moved ref changes what every patch is a diff from.
    die(1, u.name + ': ref ' + u.ref + ' now resolves to ' + got.head
      + ', and the manifest records ' + u.base + '. Reconcile deliberately.');
  }
  if (mode === 'pristine') {
    results.push({ name: u.name, action: 'pristine', path: got.dest, head: got.head, reused: got.reused });
    if (!json) process.stdout.write('pristine ' + u.name + ' at ' + got.head + ' in ' + got.dest + '\n');
    continue;
  }

  const target = join(ROOT, u.directory);
  if (existsSync(target) && readdirSync(target).length > 0 && !force) {
    die(1, u.directory + ' already has content. The tree is the truth and this would overwrite '
      + 'local patches. Pass --force only when you mean to discard them.');
  }
  rmSync(target, { recursive: true, force: true });
  mkdirSync(target, { recursive: true });
  const excluded = new Set(u.exclude ?? []);
  let copied = 0;
  let skipped = 0;
  for (const name of readdirSync(got.dest)) {
    if (excluded.has(name)) { skipped += 1; continue; }
    cpSync(join(got.dest, name), join(target, name), { recursive: true });
    copied += 1;
  }
  // ⚠ An exclusion that matched nothing upstream is reported rather than silently accepted. It is
  // either a path a release removed, or a name written defensively so a release that ADDS it does
  // not land silently, and only the record can say which.
  const unmatched = [...excluded].filter((name) => !existsSync(join(got.dest, name)));
  results.push({ name: u.name, action: 'materialise', path: target, copied, skipped, unmatched });
  if (!json) {
    process.stdout.write('materialise ' + u.name + ': ' + copied + ' path(s) copied, '
      + skipped + ' excluded\n');
    if (unmatched.length > 0) {
      process.stdout.write('  exclusions matching nothing upstream: ' + unmatched.join(' ') + '\n');
    }
  }
}

if (json) process.stdout.write(JSON.stringify({ schema: 'vendor-sync/1', results }) + '\n');
process.exit(0);
