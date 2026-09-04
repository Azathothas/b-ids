// The JavaScript side of LIB-03's comparison.
//
// ⛔ IT ASKS EXACTLY WHAT crates/b-ids/examples/answers.rs ASKS, in the same
// order, and prints the same document. scripts/common/check-bindings compares
// the two byte for byte after normalising key order, so a package that answered
// a different set of questions would produce a different document and be caught
// rather than counted as agreeing.
//
// ⭐ THE COMPARISON IS OVER THE ANSWERS RATHER THAN OVER THE INTERFACES, which
// is the entry's own wording: two implementations can expose the same names and
// disagree about what they mean.
//
// ⚠ RUN FROM THE PACKAGE DIRECTORY. The import below is relative, because a
// generated package is imported by its own path rather than installed.

import * as b from './index.mjs';

const id = (profile) => (profile === undefined ? null : profile.id);

const r = b.release();
const answers = {
  release: {
    identifier: r.identifier,
    layout: r.layout,
    profiles: r.profiles,
    newestCapture: r.newestCapture,
  },
  paths: b.paths(),
  ids: b.profiles().map((p) => p.id),
  at_first: id(b.at(b.paths()[0])),
  // ⛔ THE ABSENT CASES, which LIB-03 names. Two implementations agree easily
  // on what exists.
  at_missing: id(b.at('corpus/v1/nothing/here/at/all.json')),
  latest_chrome_linux64: id(b.latestStable('chrome', 'linux64')),
  latest_chrome_win64: id(b.latestStable('chrome', 'win64')),
  latest_firefox_linux64: id(b.latestStable('firefox', 'linux64')),
  latest_chromium_linux64: id(b.latestStable('chromium', 'linux64')),
  latest_chrome_macos: id(b.latestStable('chrome', 'macos-arm64')),
  latest_safari_linux64: id(b.latestStable('safari', 'linux64')),
  latest_upper_case: id(b.latestStable('CHROME', 'LINUX64')),
  select_for_testing_linux64: id(b.latestForChannel('chrome', 'for-testing', 'linux64')),
  hello_bytes: (b.clientHelloHex(b.latestStable('chrome', 'linux64')) ?? '').length / 2,
};

console.log(JSON.stringify(answers, null, 2));
