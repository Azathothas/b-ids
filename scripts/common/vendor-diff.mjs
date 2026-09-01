#!/usr/bin/env node
// scripts/common/vendor-diff.mjs - regenerate the patch series from the vendored tree.
//
// -- WHAT IT IS FOR ---------------------------------------------------------------------------
// docs/methodology/vendoring.md settles that the vendored TREE is the truth and that a derived
// patch series is regenerated from it, never applied to anything. A series bought two things a
// working tree cannot say: a review of a change to somebody else's code on its own, and the
// attribution a licence asks for when a distributor changes a file.
//
// ⛔ THE PATCHES ARE OUTPUT, NOT INPUT. Nothing in this repository applies them. Editing one
// changes nothing about what is built, which is why every generated file carries a header saying
// so and why --check exists.
//
// -- WHAT IT NEEDS ----------------------------------------------------------------------------
// A pristine copy of each upstream at the commit vendor/upstream.json records, which is what
// scripts/common/vendor-sync.mjs fetches. ⚠ That needs the network, so this is NOT a gate check.
// The offline half of the same question, whether every patch still names a file the tree has,
// belongs to scripts/common/check-vendor.sh and runs in the gate.
//
//   node scripts/common/vendor-diff.mjs
//   node scripts/common/vendor-diff.mjs --name rustls
//   node scripts/common/vendor-diff.mjs --check
//   node scripts/common/vendor-diff.mjs --json
//
// --check regenerates into memory and compares with what is on disk. It writes nothing and exits
// 1 on any difference, which is what makes the series a derived artefact rather than a second
// copy of the truth that drifts from it.
//
// Exit codes: 0 in sync (or written), 1 out of sync, 2 could not run.
//
// ⛔ Read the exit code from this process, unpiped.

import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, join, posix, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, '..', '..');
const MANIFEST = join(ROOT, 'vendor', 'upstream.json');
const PRISTINE = join(ROOT, '.tmp', 'vendor-pristine');
const PATCHES = join(ROOT, 'patches');

function die(code, message) {
  process.stderr.write('vendor-diff: ' + message + '\n');
  process.exit(code);
}

function usage() {
  process.stdout.write(
    'usage: node scripts/common/vendor-diff.mjs [--name NAME] [--check] [--json]\n'
    + '       read the header of this file for the full contract\n'
  );
}

const argv = process.argv.slice(2);
let only = null;
let check = false;
let json = false;
for (let i = 0; i < argv.length; i += 1) {
  if (argv[i] === '--name') { only = argv[i + 1]; i += 1; }
  else if (argv[i] === '--check') check = true;
  else if (argv[i] === '--json') json = true;
  else if (argv[i] === '-h' || argv[i] === '--help') { usage(); process.exit(0); }
  else die(2, 'unknown argument: ' + argv[i]);
}

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

// ⚠ Relative paths, always POSIX-separated, so a patch generated on Windows and one generated on
// Linux are the same bytes. A series that differs by host is a series --check can never pass.
function walk(base, prefix, excluded, out) {
  for (const name of readdirSync(base).sort()) {
    const rel = prefix === '' ? name : prefix + '/' + name;
    if (prefix === '' && excluded.has(name)) continue;
    const full = join(base, name);
    const st = statSync(full);
    if (st.isDirectory()) walk(full, rel, excluded, out);
    else out.push(rel);
  }
  return out;
}

function slug(rel) {
  return rel.split('/').join('-');
}

// git diff --no-index names the two files it was given, so the headers carry the scratch path and
// the vendored path. They are rewritten to a/REL and b/REL, which is what makes a patch readable
// as a change to the upstream file rather than to two directories on one machine.
function diffOne(pristineRel, vendoredRel, rel) {
  const r = spawnSync(
    'git',
    ['diff', '--no-index', '--no-color', '--', pristineRel, vendoredRel],
    { cwd: ROOT, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
  if (r.error) die(2, 'git could not be run: ' + r.error.message);
  if (r.status === 0) return null;
  const text = r.stdout ?? '';
  if (text === '') return null;
  return text
    .split('\n')
    .map((line) => {
      if (line.startsWith('diff --git ')) return 'diff --git a/' + rel + ' b/' + rel;
      if (line.startsWith('--- a/') || line === '--- ' + pristineRel) return '--- a/' + rel;
      if (line.startsWith('+++ b/') || line === '+++ ' + vendoredRel) return '+++ b/' + rel;
      if (line === '--- /dev/null') return line;
      if (line === '+++ /dev/null') return line;
      return line;
    })
    .join('\n');
}

const results = [];
let outOfSync = 0;

for (const u of upstreams) {
  const pristineDir = join(PRISTINE, u.name);
  if (!existsSync(pristineDir)) {
    die(2, 'no pristine copy of ' + u.name + '. Run: node scripts/common/vendor-sync.mjs pristine');
  }
  const head = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: pristineDir, encoding: 'utf8' });
  const at = (head.stdout ?? '').trim();
  if (at !== u.base) {
    die(2, 'the pristine copy of ' + u.name + ' is at ' + at + ' and the manifest records ' + u.base);
  }

  const vendoredDir = join(ROOT, u.directory);
  if (!existsSync(vendoredDir)) die(2, 'no vendored tree at ' + u.directory);

  const excluded = new Set([...(u.exclude ?? []), '.git']);
  const left = walk(pristineDir, '', excluded, []);
  const right = walk(vendoredDir, '', excluded, []);
  const all = [...new Set([...left, ...right])].sort();

  const generated = [];
  for (const rel of all) {
    const lp = posix.join(relative(ROOT, pristineDir).split(sep).join('/'), rel);
    const rp = posix.join(u.directory, rel);
    const body = diffOne(lp, rp, rel);
    if (body === null) continue;
    generated.push({ rel, body });
  }

  const dir = join(PATCHES, u.name);
  const wanted = new Map();
  generated.forEach((g, i) => {
    const n = String(i + 1).padStart(4, '0');
    const header = [
      '# ' + u.name + ': ' + g.rel,
      '#',
      '# Against ' + u.repository + ' at ' + u.base + '.',
      '# Generated by scripts/common/vendor-diff.mjs from ' + u.directory + '. Do not edit:',
      '# the vendored tree is the truth and this is derived from it. What the change',
      '# is for, and how to tell whether a release retires it, is patches/README.md.',
      '',
    ].join('\n');
    wanted.set(n + '-' + slug(g.rel) + '.patch', header + g.body);
  });

  const onDisk = existsSync(dir)
    ? readdirSync(dir).filter((f) => f.endsWith('.patch')).sort()
    : [];

  const differences = [];
  for (const [name, body] of wanted) {
    const path = join(dir, name);
    if (!existsSync(path)) { differences.push('missing: ' + name); continue; }
    if (readFileSync(path, 'utf8') !== body) differences.push('differs: ' + name);
  }
  for (const name of onDisk) {
    if (!wanted.has(name)) differences.push('stale: ' + name);
  }

  results.push({ name: u.name, patches: wanted.size, differences });
  if (check) {
    outOfSync += differences.length;
    if (!json) {
      process.stdout.write(u.name + ': ' + wanted.size + ' patch(es), '
        + differences.length + ' difference(s)\n');
      for (const d of differences) process.stdout.write('  ' + d + '\n');
    }
    continue;
  }

  // ⛔ The directory is rebuilt rather than merged into. A patch whose file stopped differing has
  // to disappear, and a stale one left behind is a claim about the tree that is not true.
  rmSync(dir, { recursive: true, force: true });
  if (wanted.size > 0) {
    mkdirSync(dir, { recursive: true });
    for (const [name, body] of wanted) writeFileSync(join(dir, name), body, 'utf8');
  }
  if (!json) process.stdout.write(u.name + ': wrote ' + wanted.size + ' patch(es) to patches/' + u.name + '\n');
}

if (json) {
  process.stdout.write(JSON.stringify({
    schema: 'vendor-diff/1',
    mode: check ? 'check' : 'write',
    results,
  }) + '\n');
}
process.exit(check && outOfSync > 0 ? 1 : 0);
