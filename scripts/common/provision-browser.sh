#!/bin/sh
# provision-browser.sh - purge every browser of one family from this machine,
# install the build that was asked for, and prove both.
#
# ⛔ ON A MACHINE THIS PROJECT CONTROLS COMPLETELY, IT MEASURED WHATEVER
# SOMEBODY ELSE'S IMAGE INSTALLED. A capture lane called `b-ids-driver resolve`,
# which by design finds what is already there, so the corpus recorded a build
# nobody chose from a source nobody named, and every profile carried
# `captured.acquisition: null`. docs/history/todo/driver.md, DRIVER-08.
#
# ⛔ MEASURED, AND THE COST WAS ALREADY PAID: on 2026-09-02 ubuntu-latest served
# Chrome 151.0.7922.173 and windows-latest served 151.0.7922.174, so the single
# highest-value capture available, one build on two platforms, was unobtainable.
#
# -- ⛔ FOUR STEPS, AND EVERY ONE OF THEM IS CONFIRMED -----------------------
#
#   1. purge, by every route an image might have used;
#   2. CONFIRM the purge: `resolve` must exit 2, meaning it found nothing. A
#      purge that reported success while a browser remained is the "reporting a
#      result the code never read" row of
#      docs/conventions/forbidden-patterns.md;
#   3. install the build asked for, from the route asked for;
#   4. CONFIRM the install: `resolve` must report exactly the version asked for.
#      A lane that installed one build and captured another is the same defect
#      one step along.
#
# -- ⭐ TWO ROUTES, BECAUSE THEY ARE TWO PRODUCTS ----------------------------
#
#   vendor       branded Chrome from the vendor's own channel. CURRENT BUILD
#                ONLY: the channel serves what is current and nothing else.
#                ⭐ Both platforms provisioned on one day therefore get the SAME
#                build, which is the whole point.
#   for-testing  AN EXACT BUILD, from the family's own first-party index. The
#                index and the artefact kind are per family and the driver's
#                route table owns both.
#
#                ⛔ WHETHER THAT BUILD IS BRANDED DEPENDS ON THE FAMILY, and
#                the route name does not say. Chrome's automation index serves
#                UNBRANDED builds: a different brand list and a different
#                sec-ch-ua, and a profile taken through it records
#                `branded: false`. Edge's enterprise index serves the vendor's
#                own branded product, so a profile taken through it records
#                `branded: true`. ⚠ The matrix cell records which, because
#                this flag cannot. DRIVER-06 measures the difference.
#
# ⛔ THIS NEVER REDISTRIBUTES A BROWSER. It prints the URL it fetched and the
# sha256 of what arrived; the artefact is the vendor's to serve.
#
# -- ⛔ IT REFUSES TO RUN ON A MACHINE THAT IS NOT DISPOSABLE ----------------
#
# Purging a browser is a change to somebody's machine. B_IDS_DISPOSABLE=1 says
# the machine is thrown away afterwards, and only a workflow sets it. A
# developer's laptop must not lose its browser to a capture.
#
# Usage:
#   B_IDS_DISPOSABLE=1 sh scripts/common/provision-browser.sh --browser chrome --route vendor
#   B_IDS_DISPOSABLE=1 sh scripts/common/provision-browser.sh --browser chrome --route for-testing --version 151.0.7922.76
#   sh scripts/common/provision-browser.sh --plan --browser chrome --route vendor
#
# Exit codes: 0 provisioned and confirmed,
#             1 a step ran and failed, or the version is not the one asked for,
#             2 could not run, which includes a machine that is not disposable.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

BROWSER=""
ROUTE="vendor"
VERSION=""
PLAN=0

while [ $# -gt 0 ]; do
  case "$1" in
    --browser) shift; BROWSER="${1:-}" ;;
    --route) shift; ROUTE="${1:-}" ;;
    --version) shift; VERSION="${1:-}" ;;
    --plan) PLAN=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'provision-browser: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

[ -n "$BROWSER" ] || { printf 'provision-browser: --browser is required\n' >&2; exit 2; }
case "$ROUTE" in
  vendor|for-testing) ;;
  *) printf 'provision-browser: --route is vendor or for-testing, not %s\n' "$ROUTE" >&2; exit 2 ;;
esac
# ⛔ THE ROUTE DECIDES WHETHER A VERSION MEANS ANYTHING, and saying so is better
# than accepting one and ignoring it. The vendor channel serves what is current;
# asking it for a build is asking for something it cannot answer.
if [ "$ROUTE" = "vendor" ] && [ -n "$VERSION" ]; then
  printf 'provision-browser: --route vendor serves the CURRENT build only, so --version %s\n' "$VERSION" >&2
  printf '  cannot be honoured. Use --route for-testing for an exact build, and read that it\n' >&2
  printf '  is UNBRANDED. docs/history/todo/driver.md, DRIVER-08.\n' >&2
  exit 2
fi
if [ "$ROUTE" = "for-testing" ] && [ -z "$VERSION" ]; then
  printf 'provision-browser: --route for-testing needs --version\n' >&2
  exit 2
fi

# ⛔ Resolved from the script's own location, never from the working directory.
HERE=$(cd -- "$(dirname -- "$0")" && pwd) || exit 2
ROOT=$(cd -- "$HERE/../.." && pwd) || exit 2
cd "$ROOT" || exit 2

PLATFORM=$(uname -s 2>/dev/null || printf 'unknown')
case "$PLATFORM" in
  Linux*) OS=linux ;;
  Darwin*) OS=mac ;;
  MINGW*|MSYS*|CYGWIN*|Windows*) OS=windows ;;
  *) OS=unknown ;;
esac

# -- what each route and platform would do, printed rather than assumed -------
#
# ⭐ --plan RUNS NOTHING. It is what a person reads before letting this near a
# machine, and it is what the acceptance check can assert on a host that is not
# disposable.
# ⭐ THE ROUTE TABLE IS PER FAMILY AND IT IS DATA, and it is defined HERE, above
# the plan, because --plan reads it too. A table defined below its first caller
# is a table the plan cannot see: measured 2026-09-02, --plan for an edge
# request printed an empty purge line and a command-not-found. docs/history/todo/driver.md,
# DRIVER-10.
#
# ⛔ IT PURGES THE NAMED FAMILY AND NOT EVERY BROWSER. Before 2026-09-02 the
# Linux purge removed Chrome AND Edge whatever --browser said. The rule is
# every browser of the target family, and this is that.

packages_for() {
  case "$1" in
    chrome) printf 'google-chrome-stable google-chrome-beta google-chrome-unstable\n' ;;
    edge) printf 'microsoft-edge-stable microsoft-edge-beta microsoft-edge-dev\n' ;;
    # ⛔ chromium-browser IS THE SHIM, AND REMOVING IT IS THE WHOLE POINT. On
    # ubuntu-24.04 that package installs a snap, and capture.yml run
    # 33854002345 measured what that costs: the resolver finds
    # /usr/bin/chromium, reads 151.0.7922.0 from it, and the launch aborts on
    # signal 6 inside content::ZygoteHostImpl::Init. ⚠ The snap itself is
    # removed by purge_linux's own snap step, because apt does not own it.
    chromium) printf 'chromium chromium-browser chromium-common chromium-sandbox chromium-l10n\n' ;;
    *) printf '\n' ;;
  esac
}

paths_for() {
  case "$1" in
    chrome) printf '/opt/google/chrome /opt/google/chrome-beta /opt/google/chrome-unstable\n' ;;
    edge) printf '/opt/microsoft/msedge /opt/microsoft/msedge-beta /opt/microsoft/msedge-dev\n' ;;
    chromium) printf '/usr/lib/chromium /usr/lib/chromium-browser /snap/chromium\n' ;;
    *) printf '\n' ;;
  esac
}

links_for() {
  case "$1" in
    chrome) printf '/usr/bin/google-chrome /usr/bin/google-chrome-stable\n' ;;
    edge) printf '/usr/bin/microsoft-edge /usr/bin/microsoft-edge-stable\n' ;;
    chromium) printf '/usr/bin/chromium /usr/bin/chromium-browser\n' ;;
    *) printf '\n' ;;
  esac
}

sandbox_for() {
  case "$1" in
    chrome) printf '/opt/google/chrome/chrome-sandbox\n' ;;
    edge) printf '/opt/microsoft/msedge/msedge-sandbox\n' ;;
    # ⚠ The distributor lays it out under /usr/lib rather than /opt, which is
    # what a Debian-policy package does. confirm_sandbox_linux reports what it
    # found rather than assuming either.
    chromium) printf '/usr/lib/chromium/chrome-sandbox\n' ;;
    *) printf '\n' ;;
  esac
}

# ⛔ SNAPS ARE A SECOND PACKAGE MANAGER AND apt CANNOT SEE THEM. Empty for every
# family the vendor ships as a deb, which is why this is a table rather than a
# flag: the one family that needs it is the one whose distribution ships it that
# way. docs/history/todo/corpus.md, CORPUS-02.
snaps_for() {
  case "$1" in
    chromium) printf 'chromium\n' ;;
    *) printf '\n' ;;
  esac
}

uninstall_match_for() {
  case "$1" in
    chrome) printf '^Google Chrome\n' ;;
    edge) printf '^Microsoft Edge$\n' ;;
    *) printf '^$\n' ;;
  esac
}

vendor_dir_for() {
  case "$1" in
    chrome) printf 'Google\\Chrome\n' ;;
    edge) printf 'Microsoft\\Edge\n' ;;
    *) printf '\n' ;;
  esac
}

plan_for() {
  # ⛔ KEYED ON THE FAMILY TOO. A plan that described Chrome's archive for an
  # `edge` request is a plan nobody could act on, and --plan is what a person
  # reads before letting this near a machine. docs/history/todo/driver.md, DRIVER-10.
  printf 'purge   %s\n' "$(purge_line)"
  case "$BROWSER/$OS/$ROUTE" in
    chrome/linux/vendor)
      printf 'fetch   https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb\n'
      printf 'install dpkg -i, then apt-get -f install for anything it needs\n'
      ;;
    chrome/windows/vendor)
      printf 'fetch   https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi\n'
      printf 'install msiexec /qn, which is the silent unattended mode\n'
      ;;
    chrome/linux/for-testing)
      printf 'index   the first-party index for this family, whose URL b_ids_driver::acquire owns.\n'
      printf '        it publishes a SUBSET of builds, so an exact build may not be in it\n'
      printf 'fetch   the chrome-linux64.zip that index names for the build asked for\n'
      printf 'install unzip into /opt/google/chrome and link /usr/bin/google-chrome at it\n'
      ;;
    chrome/windows/for-testing)
      printf 'index   the first-party index for this family, whose URL b_ids_driver::acquire owns.\n'
      printf '        it publishes a SUBSET of builds, so an exact build may not be in it\n'
      printf 'fetch   the chrome-win64.zip that index names for the build asked for\n'
      printf 'install expand into the Chrome Application directory. The archive is FLAT:\n'
      printf '        chrome.exe sits beside the manifest resolve reads the build from\n'
      ;;
    edge/linux/for-testing)
      printf 'index   the enterprise update index, whose URL b_ids_driver::acquire owns.\n'
      printf '        it publishes a SHA-256 per artefact, which this run checks what arrived against\n'
      printf 'fetch   the microsoft-edge-stable deb that index names for the build asked for\n'
      printf 'install apt-get install of the deb, whose own post-install sets the sandbox up\n'
      ;;
    edge/windows/for-testing)
      printf 'index   the enterprise update index, whose URL b_ids_driver::acquire owns.\n'
      printf '        it publishes a SHA-256 per artefact, which this run checks what arrived against\n'
      printf 'fetch   the MicrosoftEdgeEnterpriseX64.msi that index names for the build asked for\n'
      printf 'install msiexec /qn, which is the silent unattended mode\n'
      ;;
    chromium/linux/for-testing)
      printf 'index   the APT archive b_ids_driver::acquire owns the URL of. ⛔ NOT A VENDOR\n'
      printf '        index: Chromium is source rather than a product with channels, so what\n'
      printf '        serves it by version is a distributor. The archive publishes a SHA-256\n'
      printf '        and a Size per artefact, which this run checks what arrived against\n'
      printf 'fetch   the chromium deb that index names for the build asked for. ⚠ The index\n'
      printf '        is gzipped and there is no uncompressed Packages, so it is decompressed\n'
      printf 'install apt-get install of the deb, whose own post-install sets the sandbox up\n'
      ;;
    chromium/*/*)
      printf 'fetch   nothing. This project reads one archive for this family and it serves\n'
      printf '        linux64 only, so no other platform or route is implemented\n'
      ;;
    edge/*/vendor)
      printf 'fetch   nothing. The vendor publishes no current-build URL for this family,\n'
      printf '        so this route is refused and --route for-testing is the one to use\n'
      ;;
    *)
      printf 'no plan is recorded for %s via %s on %s\n' "$BROWSER" "$ROUTE" "$OS"
      ;;
  esac
  if [ "$OS" = linux ]; then
    printf 'sandbox %s, which is set to root ownership and mode 4755 after the install\n' \
      "$(sandbox_for "$BROWSER")"
  fi
  printf 'confirm resolve exits 2 after the purge, and reports the version after the install\n'
}

# ⚠ One sentence, per platform, naming what the purge actually removes for the
# family that was asked for. It reads the same table the purge itself reads.
purge_line() {
  if [ "$OS" = linux ]; then
    printf 'apt-get remove --purge of %s, ' "$(packages_for "$BROWSER")"
    _snaps=$(snaps_for "$BROWSER")
    [ -z "$_snaps" ] || printf 'snap remove --purge of %s, ' "$_snaps"
    printf 'then %s and the /usr/bin links' "$(paths_for "$BROWSER")"
  else
    printf 'the vendor uninstaller for every install matching %s, then the program directories' \
      "$(uninstall_match_for "$BROWSER")"
  fi
}

if [ "$PLAN" = 1 ]; then
  printf 'provision-browser plan: %s via %s on %s\n\n' "$BROWSER" "$ROUTE" "$OS"
  plan_for
  exit 0
fi

# ⛔ TWO INDEPENDENT REFUSALS, AND THEY ARE THE FIRST THING AFTER PARSING.
#
# ⚠ MEASURED, ON THIS PROJECT'S OWN OPERATOR MACHINE, 2026-09-02. A session
# testing that the guard could fail mutated the single condition and ran the
# tool on a developer laptop. The purge path executed. Nothing was removed,
# because the Windows uninstaller match did not fire, and the confirm step then
# refused correctly. ⛔ It should not have been reachable at all, and "it
# happened not to match" is not a safety margin.
#
# ⭐ SO THERE ARE TWO CONDITIONS FROM TWO SOURCES, and one edit cannot lift
# both: a variable this project sets only inside a workflow, and the marker the
# platform sets on every hosted runner. A person who genuinely wants this on a
# disposable machine of their own sets both, deliberately.
#
# ⛔ AND A TEST THAT HAS TO BYPASS A GUARD RUNS AGAINST A COPY, never against
# this file on a machine the guard protects.
DISPOSABLE="${B_IDS_DISPOSABLE:-}"
ON_A_RUNNER="${CI:-}"
if [ "$DISPOSABLE" != "1" ] || [ -z "$ON_A_RUNNER" ]; then
  printf 'provision-browser: this machine is not marked disposable, so nothing was purged.\n' >&2
  printf '  B_IDS_DISPOSABLE=%s and CI=%s, and BOTH are required.\n' \
    "${DISPOSABLE:-unset}" "${ON_A_RUNNER:-unset}" >&2
  printf '  Set them only on a machine that is thrown away afterwards.\n' >&2
  printf '  Run with --plan to read what it would do. docs/history/todo/driver.md, DRIVER-08.\n' >&2
  exit 2
fi

command -v cargo >/dev/null 2>&1 || { printf 'provision-browser: cargo not found\n' >&2; exit 2; }
[ "$OS" != "unknown" ] || { printf 'provision-browser: no plan for %s\n' "$PLATFORM" >&2; exit 2; }

OUT="$ROOT/.tmp/provision-browser"
mkdir -p "$OUT" || exit 2

printf 'building the driver\n'
cargo build -q -p b-ids-driver || {
  printf 'provision-browser: the driver did not build\n' >&2
  exit 2
}
TARGET_DIR=${CARGO_TARGET_DIR:-"$ROOT/target"}
DRIVER="$TARGET_DIR/debug/b-ids-driver"
[ -x "$DRIVER" ] || DRIVER="$DRIVER.exe"
[ -x "$DRIVER" ] || { printf 'provision-browser: %s is not executable\n' "$DRIVER" >&2; exit 2; }

# ⛔ ONE READER OF "WHAT IS INSTALLED", and it is the driver rather than a second
# search written here. A script that looked for a browser its own way would be a
# second answer to the question `resolve` exists to answer, and the two would
# disagree the first time a path moved.
resolved_version() {
  "$DRIVER" resolve --browser "$BROWSER" --json 2>/dev/null \
    | awk -F'"version":"' 'NR==1 { split($2, a, /"/); print a[1] }'
}
resolve_rc() {
  "$DRIVER" resolve --browser "$BROWSER" --json >/dev/null 2>&1
  printf '%s' "$?"
}

# -- 1. purge -----------------------------------------------------------------
printf '\n-- purging every %s on this machine --\n' "$BROWSER"
before=$(resolved_version)
printf 'before  %s\n' "${before:-nothing resolved}"

# ⭐ THE ROUTE TABLE IS PER FAMILY AND IT IS DATA. Adding a family is a row in
# each of these three functions and a fixture, rather than a fifth arm of a case
# statement threaded through the script. docs/history/todo/driver.md, DRIVER-10.
#
# ⛔ IT PURGES THE NAMED FAMILY AND NOT EVERY BROWSER. Before 2026-09-02 the
# Linux purge removed Chrome AND Edge whatever --browser said, so a lane
# provisioning one family destroyed the other family's lane on the same runner
# if they ever shared one. The entry's rule is "every browser of the target
# family", and this is that.



# ⛔ THE SUID SANDBOX HELPER, WHICH IS THE DIFFERENCE BETWEEN INSTALLED AND
# ABLE TO CAPTURE. Measured 2026-09-02 in capture.yml run 33615327503: the edge
# lane on ubuntu-latest exited after 2.4 seconds having opened no connection,
# and its own log named this file and the mode it needs. DRIVER-07 is why that
# log was kept at all.

purge_linux() {
  for pkg in $(packages_for "$BROWSER"); do
    sudo apt-get remove --purge -y "$pkg" >/dev/null 2>&1
  done
  # ⛔ THE SNAP, WHICH apt DOES NOT OWN. On ubuntu-24.04 the chromium the image
  # serves is a snap, and `apt-get remove --purge chromium-browser` removes the
  # shim while leaving the snap mounted and /usr/bin/chromium resolving to it.
  # ⚠ The confirm step after the purge would then refuse correctly and nobody
  # would know why. ⭐ It is scoped to the family's own snap names, for the same
  # reason the apt list is: a lane provisioning one family must not remove
  # another family's browser off the same runner.
  for snap in $(snaps_for "$BROWSER"); do
    sudo snap remove --purge "$snap" >/dev/null 2>&1
  done
  for path in $(paths_for "$BROWSER"); do
    sudo rm -rf "$path" >/dev/null 2>&1
  done
  for link in $(links_for "$BROWSER"); do
    sudo rm -f "$link" >/dev/null 2>&1
  done
}

# ⛔ CHECKED AFTER EVERY LINUX INSTALL, and reported rather than assumed. A
# vendor package sets this up in its own post-install step and an unpacked
# archive does not, so the one path that needs the fix is the one that would
# otherwise install a browser that cannot open a socket.
confirm_sandbox_linux() {
  helper=$(sandbox_for "$BROWSER")
  [ -n "$helper" ] || return 0
  if [ ! -f "$helper" ]; then
    printf 'sandbox no %s on this machine after the install\n' "$helper"
    return 0
  fi
  sudo chown root:root "$helper" >/dev/null 2>&1
  sudo chmod 4755 "$helper" >/dev/null 2>&1
  printf 'sandbox %s\n' "$(stat -c '%U:%G %a %n' "$helper")"
}

# ⛔ THE FAMILY IS PASSED IN, so a run provisioning one family does not uninstall
# the other. The uninstaller pattern and the directory list are per family, which
# is the same table the Linux half reads. docs/history/todo/driver.md, DRIVER-10.


purge_windows() {
  pw_match=$(uninstall_match_for "$BROWSER")
  pw_dir=$(vendor_dir_for "$BROWSER")
  # ⚠ EVERY INSTALL, not the first one found: an image may carry a machine-wide
  # install and a per-user one, and removing one leaves the other for `resolve`.
  #
  # ⛔ THE TWO VALUES CROSS THROUGH THE ENVIRONMENT, and this is the second
  # spelling because the first one did not work. `powershell -Command` does NOT
  # bind trailing arguments to a `param()` block: it appends them to the command
  # TEXT. So the pattern arrived null, an empty pattern matched every uninstall
  # entry, and the directory removal was skipped entirely because `if ($Dir)`
  # was false.
  #
  # ⚠ MEASURED, capture.yml run 33639884645: the win64 lane reported "the purge
  # left a browser behind. resolve exited 0 and reports 151.0.7922.174" and went
  # red. ⭐ That is the confirm step doing its job. A purge that reported success
  # over a browser still on the machine is the "reporting a result the code never
  # read" row of docs/conventions/forbidden-patterns.md, and the lane would have
  # captured a build nobody chose.
  #
  # shellcheck disable=SC2016 # the payload is PowerShell and the shell must not
  # expand anything in it. docs/conventions/shell.md section 1.
  B_IDS_PURGE_MATCH="$pw_match" B_IDS_PURGE_DIR="$pw_dir" powershell -NoProfile -Command '
    $Match = $env:B_IDS_PURGE_MATCH
    $Dir = $env:B_IDS_PURGE_DIR
    if (-not $Match) { exit 3 }
    $ErrorActionPreference = "SilentlyContinue"
    $roots = @(
      "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
      "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
      "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )
    foreach ($root in $roots) {
      foreach ($key in (Get-ItemProperty $root)) {
        if ($key.DisplayName -match $Match) {
          $u = $key.UninstallString
          if ($u -match "setup.exe") {
            $exe = ($u -split "`" ")[0].Trim(@("`""))
            & $exe --uninstall --system-level --force-uninstall 2>$null
            & $exe --uninstall --force-uninstall 2>$null
          }
        }
      }
    }
    if ($Dir) {
      Remove-Item -Recurse -Force (Join-Path $env:ProgramFiles $Dir) 2>$null
      Remove-Item -Recurse -Force (Join-Path ${env:ProgramFiles(x86)} $Dir) 2>$null
      Remove-Item -Recurse -Force (Join-Path $env:LOCALAPPDATA $Dir) 2>$null
    }
  ' >/dev/null 2>&1
}

case "$OS" in
  linux) purge_linux ;;
  windows) purge_windows ;;
  mac) sudo rm -rf "/Applications/Google Chrome.app" >/dev/null 2>&1 ;;
esac

# -- the automation-build route ----------------------------------------------
#
# ⛔ THE INDEX IS READ BY THE DRIVER AND SO IS THE URL IT LIVES AT. A second
# spelling in this file would 404 on its own the day the vendor moves the file,
# and nothing would compare the two. b_ids_driver::acquire owns both, and
# `acquire --index-url` is how this asks.
#
# ⚠ THE INDEX PUBLISHES A SUBSET OF BUILDS. Measured 2026-09-02: it carried 67
# builds of Chrome 151, and neither 151.0.7922.173 nor 151.0.7922.174, which are
# the two the hosted runner images served. The driver's refusal names the
# nearest builds in the same line, so a caller that asked for an unpublished
# build is told what it could have asked for instead.
#
# Sets URL and ARCHIVE for the digest step. Exits nonzero rather than setting
# a flag: a fetch that half worked must not reach the install.
index_fetch() {
  command -v curl >/dev/null 2>&1 || {
    printf 'provision-browser: curl not found, and the index has to be fetched\n' >&2
    return 2
  }

  index_url=$("$DRIVER" acquire --index-url --browser "$BROWSER")
  rc=$?
  if [ "$rc" != 0 ] || [ -z "$index_url" ]; then
    printf 'provision-browser: the driver named no automation index for %s\n' "$BROWSER" >&2
    return 2
  fi

  printf 'index   %s\n' "$index_url"
  curl -fsSL -o "$OUT/index.raw" "$index_url" || {
    printf 'provision-browser: the automation index did not fetch\n' >&2
    return 1
  }
  # ⛔ DECOMPRESSED WHERE THE INDEX IS COMPRESSED, decided by the URL the driver
  # named rather than by the family. Measured 2026-09-04: the APT archive this
  # project reads for Chromium serves Packages.gz at 200 and Packages at 404, so
  # there is no uncompressed form to prefer. ⚠ The file keeps the name the rest
  # of this function uses, so nothing downstream has to know which arm ran.
  case "$index_url" in
    *.gz)
      gzip -dc "$OUT/index.raw" > "$OUT/index.json" || {
        printf 'provision-browser: the index did not decompress\n' >&2
        return 1
      }
      printf 'index   decompressed, %s bytes\n' "$(wc -c < "$OUT/index.json" | tr -d ' ')"
      ;;
    *) mv "$OUT/index.raw" "$OUT/index.json" || return 1 ;;
  esac

  # ⛔ READ UNPIPED. A guard on the left of a pipe reports the pipeline's
  # status, and this one distinguishes "the index does not publish that build"
  # at 1 from "could not ask" at 2.
  URL=$("$DRIVER" acquire --browser "$BROWSER" --version "$VERSION" --index "$OUT/index.json")
  rc=$?
  if [ "$rc" != 0 ]; then
    printf 'provision-browser: the index named no archive for %s %s\n' "$BROWSER" "$VERSION" >&2
    "$DRIVER" acquire --browser "$BROWSER" --version "$VERSION" --index "$OUT/index.json" >/dev/null
    return "$rc"
  fi

  ARCHIVE="$OUT/$(basename "$URL")"
  curl -fsSL -o "$ARCHIVE" "$URL" || {
    printf 'provision-browser: the archive did not fetch from %s\n' "$URL" >&2
    return 1
  }

  # ⭐ THE DRIVER'S ANSWER IS READ ONCE AND BOTH VALUES COME OUT OF IT: the
  # route a profile records, and the digest the publisher states.
  #
  # ⛔ THE ROUTE NAME COMES FROM THE DRIVER'S ROUTE TABLE, never from a mapping
  # here. ⚠ Measured 2026-09-02, capture.yml run 33637307031: a `case` in this
  # script mapped `for-testing` to `chrome-for-testing` whatever the family
  # was, so the first Edge profile recorded a Chrome route. The driver knows
  # which index answered and this reads that.
  ACQUIRED_JSON=$("$DRIVER" acquire --browser "$BROWSER" --version "$VERSION" \
    --index "$OUT/index.json" --json)
  RECORDED_ROUTE=$(printf '%s' "$ACQUIRED_JSON" |
    awk -F'"route":"' 'NR==1 { split($2, a, /"/); print a[1] }')
  [ -n "$RECORDED_ROUTE" ] || {
    printf 'provision-browser: the driver named no route for this acquisition\n' >&2
    return 1
  }
  # ⭐ AND WHERE THE PUBLISHER STATES A DIGEST, WHAT ARRIVED IS COMPARED WITH
  # IT. The Edge index states a SHA-256 for every artefact and the automation
  # index for Chrome states none, so this fires on one route and reports the
  # absence on the other. ⛔ A mismatch is a refusal: an artefact that is not
  # the one the publisher named is not the one a profile would be describing.
  PUBLISHED=$(printf '%s' "$ACQUIRED_JSON" |
    awk -F'"published_sha256":"' 'NR==1 { split($2, a, /"/); print a[1] }')
  if [ -n "$PUBLISHED" ]; then
    ARRIVED=$(sha256sum "$ARCHIVE" 2>/dev/null | awk '{ print $1 }')
    if [ "$ARRIVED" != "$PUBLISHED" ]; then
      printf 'provision-browser: the archive is not what the index published.\n' >&2
      printf '  published %s\n  arrived   %s\n' "$PUBLISHED" "$ARRIVED" >&2
      return 1
    fi
    printf 'verified %s matches the digest the index publishes\n' "$ARRIVED"
  else
    printf 'verified no, this index publishes no digest to compare against\n'
  fi

  # ⛔ THE SIBLINGS THE INDEX NAMES, WHICH AN ARCHIVE-PER-BUILD INDEX HAS NONE
  # OF. An APT archive splits one build across several binary packages: the one
  # carrying the browser Depends on a sibling at the same exact version, and
  # Recommends the SUID sandbox helper without which it launches and opens
  # nothing. ⚠ The driver reads which ones from the index's own Depends and
  # Recommends fields; this fetches what it named and nothing else.
  COMPANIONS=""
  for extra in $(printf '%s' "$ACQUIRED_JSON" | node -e '
    let s = ""; process.stdin.on("data", d => s += d).on("end", () => {
      const j = JSON.parse(s);
      for (const u of (j.companions || [])) console.log(u);
    });
  '); do
    dest="$OUT/$(basename "$extra")"
    curl -fsSL -o "$dest" "$extra" || {
      printf 'provision-browser: a companion artefact did not fetch from %s\n' "$extra" >&2
      return 1
    }
    COMPANIONS="$COMPANIONS $dest"
    printf 'sibling %s\n' "$(basename "$extra")"
  done
  return 0
}

# ⚠ ONLY THE ZIP ROUTE UNPACKS. A deb and an msi are installed by the platform's
# own tool rather than unpacked into a directory, so unpacking is the Chrome
# route's step and not a step every family takes.
unpack_archive() {
  rm -rf "$OUT/unpacked"
  mkdir -p "$OUT/unpacked" || return 1
  if command -v unzip >/dev/null 2>&1; then
    unzip -q -o "$ARCHIVE" -d "$OUT/unpacked" || {
      printf 'provision-browser: the archive did not unpack\n' >&2
      return 1
    }
  elif [ "$OS" = windows ]; then
    # ⚠ Expand-Archive is built in, and unzip is not on a Windows runner. Both
    # arguments are converted to native paths first: Git Bash hands a POSIX path
    # to a Windows process unchanged, and PowerShell cannot open it.
    native_archive=$(cygpath -w "$ARCHIVE" 2>/dev/null || printf '%s' "$ARCHIVE")
    native_out=$(cygpath -w "$OUT/unpacked" 2>/dev/null || printf '%s' "$OUT/unpacked")
    powershell -NoProfile -Command \
      "Expand-Archive -LiteralPath '$native_archive' -DestinationPath '$native_out' -Force" || {
      printf 'provision-browser: Expand-Archive refused the archive\n' >&2
      return 1
    }
  else
    printf 'provision-browser: no unzip on this machine and no built-in to fall back to\n' >&2
    return 2
  fi
  return 0
}

# ⚠ THE PLATFORM'S OWN INSTALLER, for a family the vendor ships as a package
# rather than as an archive. The package's own post-install step is what puts
# the SUID sandbox helper in place, which is why `confirm_sandbox_linux` reports
# rather than assumes.
install_package_linux() {
  # ⛔ EVERY ARTEFACT IN ONE apt INVOCATION, and that is not tidiness. A build
  # split across binary packages has a dependency on a sibling AT AN EXACT
  # VERSION, which is on no configured apt source: installing them one at a time
  # fails on the first, and installing the main one alone leaves a browser that
  # cannot start. ⚠ COMPANIONS is empty for every index that serves one archive
  # per build, so this is the same command it was for those.
  # shellcheck disable=SC2086 # COMPANIONS is a deliberate list of paths this
  # script itself wrote under .tmp, and quoting it would pass one argument.
  sudo apt-get install -y "$ARCHIVE" $COMPANIONS >/dev/null 2>&1 ||
    { sudo dpkg -i "$ARCHIVE" $COMPANIONS >/dev/null 2>&1; sudo apt-get -f install -y >/dev/null 2>&1; }
  return 0
}

install_package_windows() {
  native=$(cygpath -w "$ARCHIVE" 2>/dev/null || printf '%s' "$ARCHIVE")
  msiexec //i "$native" //qn //norestart >/dev/null 2>&1
  return 0
}

# ⛔ THE SUID SANDBOX HELPER IS SET UP, AND SKIPPING IT IS A LANE THAT CAPTURES
# NOTHING. Measured 2026-09-02 in capture.yml run 33615327503: the edge lane on
# ubuntu-latest exited after 2.4 seconds having opened no connection, and its
# own log said the helper "was found, but is not configured correctly" and named
# the ownership and mode it needs. An unpacked archive carries neither.
#
# ⚠ TWO NAMES, because the official build's compiled-in path uses a hyphen and
# the archive ships an underscore. Installing both costs a copy and removes a
# whole class of "it launched and opened nothing".
install_for_testing_linux() {
  src="$OUT/unpacked/chrome-linux64"
  [ -x "$src/chrome" ] || {
    printf 'provision-browser: no chrome in %s after unpacking\n' "$src" >&2
    ls -la "$OUT/unpacked" >&2
    return 1
  }
  sudo rm -rf /opt/google/chrome || return 1
  sudo mkdir -p /opt/google/chrome || return 1
  sudo cp -a "$src/." /opt/google/chrome/ || return 1
  sudo ln -sf /opt/google/chrome/chrome /usr/bin/google-chrome || return 1
  # ⚠ TWO NAMES, and only this route needs the copy. The archive ships an
  # underscore and the official build's compiled-in path uses a hyphen, so an
  # unpacked build is laid out under both. ⛔ The ownership and the mode are
  # `confirm_sandbox_linux`'s job for every route, and doing them here as well
  # would be the same rule in two places.
  if [ -f /opt/google/chrome/chrome_sandbox ]; then
    sudo cp -a /opt/google/chrome/chrome_sandbox /opt/google/chrome/chrome-sandbox || return 1
    sudo chown root:root /opt/google/chrome/chrome_sandbox || return 1
    sudo chmod 4755 /opt/google/chrome/chrome_sandbox || return 1
  else
    printf 'provision-browser: the archive carried no chrome_sandbox\n' >&2
  fi
  return 0
}

# ⚠ THE ARCHIVE IS FLAT AND THAT IS WHY THIS WORKS AT ALL. Read from the
# archive's central directory 2026-09-02: chrome.exe sits beside a
# VERSION.manifest file and there is no version-shaped DIRECTORY, so
# b_ids_driver::resolve reads the build from the manifest. Before that source
# existed the confirm step below could not name what it had installed.
install_for_testing_windows() {
  src="$OUT/unpacked/chrome-win64"
  [ -f "$src/chrome.exe" ] || {
    printf 'provision-browser: no chrome.exe in %s after unpacking\n' "$src" >&2
    ls -la "$OUT/unpacked" >&2
    return 1
  }
  # ⚠ Read from the environment. A program files directory is not the same
  # string on every Windows install and nobody's to hardcode.
  root=$(cygpath -u "${PROGRAMFILES:-C:\\Program Files}" 2>/dev/null || printf '/c/Program Files')
  dest="$root/Google/Chrome/Application"
  rm -rf "$dest"
  mkdir -p "$dest" || return 1
  cp -a "$src/." "$dest/" || return 1
  printf 'installed %s\n' "$dest/chrome.exe"
  return 0
}

# -- 2. confirm the purge -----------------------------------------------------
#
# ⛔ READ FROM THE RESOLVER, not from the exit code of a package manager. A
# remove that reported success over a browser somewhere else on the machine is
# exactly what this step is for. 2 is "resolve found nothing", which is CI-07's
# meaning of could-not-run and here is the success condition.
rc=$(resolve_rc)
if [ "$rc" != "2" ]; then
  printf 'provision-browser: the purge left a browser behind. resolve exited %s and reports %s\n' \
    "$rc" "$(resolved_version)" >&2
  exit 1
fi
printf 'after   nothing resolves, resolve exits 2\n'

# -- 3. install ---------------------------------------------------------------
printf '\n-- installing %s via %s --\n' "$BROWSER" "$ROUTE"
URL=""
ARCHIVE=""
COMPANIONS=""

# ⛔ KEYED ON THE FAMILY AS WELL AS THE PLATFORM AND THE ROUTE, which is what
# DRIVER-10 needed: two families do not install the same way, and a case that
# keyed on the platform alone installed Chrome whatever --browser said.
case "$BROWSER/$OS/$ROUTE" in
  chrome/linux/vendor)
    URL="https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb"
    ARCHIVE="$OUT/google-chrome-stable_current_amd64.deb"
    curl -fsSL -o "$ARCHIVE" "$URL" || { printf 'provision-browser: fetch failed\n' >&2; exit 1; }
    install_package_linux
    confirm_sandbox_linux
    ;;
  chrome/windows/vendor)
    URL="https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi"
    ARCHIVE="$OUT/googlechromestandaloneenterprise64.msi"
    curl -fsSL -o "$ARCHIVE" "$URL" || { printf 'provision-browser: fetch failed\n' >&2; exit 1; }
    install_package_windows
    ;;
  chrome/linux/for-testing)
    index_fetch || exit $?
    unpack_archive || exit 1
    install_for_testing_linux || exit 1
    confirm_sandbox_linux
    ;;
  chrome/windows/for-testing)
    index_fetch || exit $?
    unpack_archive || exit 1
    install_for_testing_windows || exit 1
    ;;
  # ⭐ EDGE IS A PACKAGE ON BOTH PLATFORMS, from the vendor's own enterprise
  # index, which publishes a digest this run compares what arrived against.
  edge/linux/for-testing)
    index_fetch || exit $?
    install_package_linux
    confirm_sandbox_linux
    ;;
  # ⭐ CHROMIUM IS A PACKAGE FROM A DISTRIBUTOR'S ARCHIVE, and it is the one
  # family here whose index is not the vendor's. Chromium is source rather than
  # a product with channels: the snapshot bucket is keyed by a trunk revision
  # that is not a version a profile records, and Ubuntu's own package is a shim
  # for a snap whose zygote will not start on a hosted runner. The archive that
  # remains publishes a SHA-256 per artefact, which is what makes it usable.
  # docs/history/todo/corpus.md, CORPUS-02.
  chromium/linux/for-testing)
    index_fetch || exit $?
    install_package_linux
    confirm_sandbox_linux
    ;;
  edge/windows/for-testing)
    index_fetch || exit $?
    install_package_windows
    ;;
  # ⛔ A REFUSAL WITH ITS REASON, which is a complete outcome. Measured
  # 2026-09-02: the vendor publishes no current-build URL for this family the
  # way it does for Chrome, and its enterprise index is keyed by build. Asking
  # for "whatever is current" of a family whose vendor does not serve that is
  # asking for something nobody can answer.
  edge/*/vendor)
    printf 'provision-browser: --route vendor serves a CURRENT-build URL, and the vendor\n' >&2
    printf '  publishes none for %s. Its index is keyed by build, so use --route for-testing\n' "$BROWSER" >&2
    printf '  with the build you want. docs/history/todo/driver.md, DRIVER-10.\n' >&2
    exit 2
    ;;
  *)
    printf 'provision-browser: the %s route for %s on %s is not implemented yet\n' \
      "$ROUTE" "$BROWSER" "$OS" >&2
    printf '  Run with --plan to read what it would do. docs/history/todo/driver.md, DRIVER-10.\n' >&2
    exit 2
    ;;
esac

# ⛔ THE DIGEST OF WHAT ARRIVED, printed rather than inferred. Every download URL
# will one day 404, and a later reader still has to be able to say whether two
# captures used the same bytes. DRIVER-05 is where that rule comes from.
if [ -f "$ARCHIVE" ]; then
  SHA=$(sha256sum "$ARCHIVE" 2>/dev/null | awk '{ print $1 }')
  BYTES=$(wc -c < "$ARCHIVE" | tr -d ' ')
  printf 'url     %s\n' "$URL"
  printf 'sha256  %s\n' "${SHA:-unknown}"
  printf 'bytes   %s\n' "${BYTES:-unknown}"

  # ⭐ WRITTEN WHERE A CAPTURE CAN READ IT, not only printed. Every profile this
  # project has published carries `captured.acquisition: null`, which is the
  # weakest provenance the artefact half can have in a project whose product is
  # provenance. experiments/10-first-profile.sh reads this file into the
  # identity, and b_ids_corpus::capture copies it onto the profile.
  # docs/history/todo/driver.md, DRIVER-08.
  #
  # ⛔ SERIALISED BY node, never by a format string. A URL carrying a character
  # that has to be escaped would otherwise emit JSON that does not parse, and it
  # would be indistinguishable from a surviving template placeholder to the
  # check that looks for one.
  #
  # ⚠ THE ROUTE NAME IS THE PROFILE'S VOCABULARY, not this script's flag. The
  # index routes set it from the driver's route table in `index_fetch`; the
  # vendor route has no index to ask and its flag and its recorded name are the
  # same word.
  RECORDED_ROUTE="${RECORDED_ROUTE:-$ROUTE}"
  node -e '
    const fs = require("fs");
    const [out, route, url, sha256, bytes] = process.argv.slice(1);
    fs.writeFileSync(out, JSON.stringify({ route, url, sha256, bytes: Number(bytes) }, null, 2) + "\n");
  ' "$OUT/acquisition.json" "$RECORDED_ROUTE" "$URL" "${SHA:-}" "${BYTES:-0}" || {
    printf 'provision-browser: could not write the acquisition record\n' >&2
    exit 1
  }
  printf 'record  %s\n' "$OUT/acquisition.json"
fi

# -- 4. confirm the install ---------------------------------------------------
got=$(resolved_version)
if [ -z "$got" ]; then
  printf 'provision-browser: nothing resolves after the install\n' >&2
  exit 1
fi
printf 'version %s\n' "$got"

if [ -n "$VERSION" ] && [ "$got" != "$VERSION" ]; then
  printf 'provision-browser: asked for %s and got %s\n' "$VERSION" "$got" >&2
  exit 1
fi

printf '\nprovisioned  %s %s via %s on %s\n' "$BROWSER" "$got" "$ROUTE" "$OS"
exit 0
