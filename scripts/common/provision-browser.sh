#!/bin/sh
# provision-browser.sh - purge every browser of one family from this machine,
# install the build that was asked for, and prove both.
#
# ⛔ ON A MACHINE THIS PROJECT CONTROLS COMPLETELY, IT MEASURED WHATEVER
# SOMEBODY ELSE'S IMAGE INSTALLED. A capture lane called `b-ids-driver resolve`,
# which by design finds what is already there, so the corpus recorded a build
# nobody chose from a source nobody named, and every profile carried
# `captured.acquisition: null`. TODO/driver.md, DRIVER-08.
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
#   for-testing  an exact build, any version, every platform, from the
#                automation-build index. ⛔ UNBRANDED: a different brand list
#                and a different sec-ch-ua, and a profile taken through it
#                records `branded: false`. DRIVER-06 measures the difference.
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
#   B_IDS_DISPOSABLE=1 sh scripts/common/provision-browser.sh --browser chrome --route for-testing --version 151.0.7922.173
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
  printf '  is UNBRANDED. TODO/driver.md, DRIVER-08.\n' >&2
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
plan_for() {
  case "$OS/$ROUTE" in
    linux/vendor)
      printf 'purge   apt-get remove --purge, then /opt/google/chrome and the /usr/bin links\n'
      printf 'fetch   https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb\n'
      printf 'install dpkg -i, then apt-get -f install for anything it needs\n'
      ;;
    windows/vendor)
      printf 'purge   the vendor uninstaller for every install found, then the program directory\n'
      printf 'fetch   https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi\n'
      printf 'install msiexec /qn, which is the silent unattended mode\n'
      ;;
    */for-testing)
      printf 'purge   as for the vendor route on this platform\n'
      printf 'fetch   the zip the automation-build index names for this exact build\n'
      printf 'install unzip into a directory this project owns, and link it where resolve looks\n'
      ;;
    *)
      printf 'no plan is recorded for %s on this platform\n' "$ROUTE"
      ;;
  esac
  printf 'confirm resolve exits 2 after the purge, and reports the version after the install\n'
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
  printf '  Run with --plan to read what it would do. TODO/driver.md, DRIVER-08.\n' >&2
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
DRIVER="$ROOT/target/debug/b-ids-driver"
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

purge_linux() {
  for pkg in google-chrome-stable google-chrome-beta google-chrome-unstable microsoft-edge-stable; do
    sudo apt-get remove --purge -y "$pkg" >/dev/null 2>&1
  done
  sudo rm -rf /opt/google/chrome /opt/google/chrome-beta /opt/google/chrome-unstable \
              /opt/microsoft/msedge >/dev/null 2>&1
  sudo rm -f /usr/bin/google-chrome /usr/bin/google-chrome-stable \
             /usr/bin/microsoft-edge /usr/bin/microsoft-edge-stable >/dev/null 2>&1
}

purge_windows() {
  # ⚠ EVERY INSTALL, not the first one found: an image may carry a machine-wide
  # install and a per-user one, and removing one leaves the other for `resolve`.
  # shellcheck disable=SC2016 # the payload is PowerShell and the shell must not
  # expand anything in it. docs/conventions/shell.md section 1.
  powershell -NoProfile -Command '
    $ErrorActionPreference = "SilentlyContinue"
    $roots = @(
      "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
      "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
      "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )
    foreach ($root in $roots) {
      foreach ($key in (Get-ItemProperty $root)) {
        if ($key.DisplayName -match "^Google Chrome" -or $key.DisplayName -match "^Microsoft Edge$") {
          $u = $key.UninstallString
          if ($u -match "setup.exe") {
            $exe = ($u -split "`" ")[0].Trim(@("`""))
            & $exe --uninstall --system-level --force-uninstall 2>$null
            & $exe --uninstall --force-uninstall 2>$null
          }
        }
      }
    }
    Remove-Item -Recurse -Force "$env:ProgramFiles\Google\Chrome" 2>$null
    Remove-Item -Recurse -Force "${env:ProgramFiles(x86)}\Google\Chrome" 2>$null
    Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Google\Chrome" 2>$null
  ' >/dev/null 2>&1
}

case "$OS" in
  linux) purge_linux ;;
  windows) purge_windows ;;
  mac) sudo rm -rf "/Applications/Google Chrome.app" >/dev/null 2>&1 ;;
esac

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

case "$OS/$ROUTE" in
  linux/vendor)
    URL="https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb"
    ARCHIVE="$OUT/google-chrome-stable_current_amd64.deb"
    curl -fsSL -o "$ARCHIVE" "$URL" || { printf 'provision-browser: fetch failed\n' >&2; exit 1; }
    sudo apt-get install -y "$ARCHIVE" >/dev/null 2>&1 \
      || { sudo dpkg -i "$ARCHIVE" >/dev/null 2>&1; sudo apt-get -f install -y >/dev/null 2>&1; }
    ;;
  windows/vendor)
    URL="https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi"
    ARCHIVE="$OUT/googlechromestandaloneenterprise64.msi"
    curl -fsSL -o "$ARCHIVE" "$URL" || { printf 'provision-browser: fetch failed\n' >&2; exit 1; }
    native=$(cygpath -w "$ARCHIVE" 2>/dev/null || printf '%s' "$ARCHIVE")
    msiexec //i "$native" //qn //norestart >/dev/null 2>&1
    ;;
  *)
    printf 'provision-browser: the %s route on %s is not implemented yet\n' "$ROUTE" "$OS" >&2
    printf '  Run with --plan to read what it would do. TODO/driver.md, DRIVER-08.\n' >&2
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
