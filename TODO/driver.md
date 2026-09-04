# driver

Getting a browser onto a machine, working out which build it is, and pointing it
at the harness. Two jobs kept separate: **resolve** a browser, and **drive** it.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## DRIVER-01. Resolve a browser, and drive it at a URL

**Source** the founding brief; the driver shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 15
**Category** driver, **Priority** P1, **Effort** M, **Status** done

### Problem

There is no way to run a browser from this repository. Without it the harness
listens and nothing connects.

### Premise

Inherited from a working implementation described but not read here. The flag
set below was reported as measured against two browser versions.

```text
--headless=new --no-sandbox --user-data-dir=THROWAWAY
--no-first-run --no-default-browser-check
--disable-search-engine-choice-screen --disable-gpu
--test-type --ignore-certificate-errors
--dump-dom URL
```

### Approach

Two separable pieces, and keeping them separable is the entry:

- **resolve**: find a browser on this machine, or acquire one, and report its
  exact build, its channel and where it came from;
- **drive**: launch it into a throwaway profile directory, point it at a URL,
  and wait for it to exit.

Two traps, both inherited:

- the dump-DOM flag is a **mode** and the URL is its positional argument. A URL
  passed as a bare argument makes the browser navigate and then sit there;
- wrap the launch in a hard time limit. A browser that cannot complete a
  handshake does not exit.

Must not: reuse a profile directory between runs, or leave one behind. The
throwaway profile is what makes a capture belong to nobody.

### Prove

```bash
cargo test -p b-ids-driver resolve_and_drive -- --nocapture
```

Passing means: on a host with a browser, the driver reports its exact build and
completes a capture against the harness; on a host without one it exits 2 rather
than 1, because "could not run" and "failed" are different facts.


### Closing

**Closed 2026-09-01T06:35:00Z.** ⭐ **The whole path runs from one command:**
the driver resolves Chrome, launches it into a profile nobody keeps, and the
harness captures a terminated handshake from it.

```text
$ cargo test -p b-ids-driver resolve_and_drive -- --nocapture
running 2 tests
resolve_and_drive: chrome 151.0.7922.76 from sibling-directory
resolve_and_drive: edge 152.0.4191.53 from sibling-directory
test resolve_and_drive_reports_a_build_from_a_source_it_names ... ok
resolve_and_drive: chrome 151.0.7922.76 produced 2 connection(s), 1 terminated
test resolve_and_drive_completes_a_capture_against_the_harness ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.24s
exit=0
```

```text
$ b-ids-driver resolve
chrome 151.0.7922.76
  sibling-directory: 151.0.7922.76
edge 152.0.4191.53
  sibling-directory: 152.0.4191.53
exit=0
```

### ⛔ Reading the version by running the browser opened the operator browser

The resolver asked two sources: a version-shaped directory beside the
executable, and the executable own `--version`. ⛔ **On Windows the second one
is not a query.** Measured 2026-09-01: `chrome.exe --version` launched Chrome
into the person own profile and never returned, so the resolver hung and the
browser was open.

⚠ **A time limit would have fixed the hang and not the side effect**, which is
the part that matters: a resolver must not touch the machine it is describing.
That source is now skipped on Windows by platform rather than bounded by a
timeout, and the sibling directory answers there anyway. ⭐ The rule the entry
stated for the LAUNCH turned out to apply to the RESOLVE as well, and reading a
version is exactly where nobody expects a launch.

### ⭐ The certificate flag is narrower than the one this entry inherited

The inherited flag set carried `--test-type --ignore-certificate-errors`, which
switches verification off for everything. ⛔ **This driver passes
`--ignore-certificate-errors-spki-list` instead**, carrying the base64 SHA-256
of the one subject public key the harness minted for that run. Verification
still runs, a certificate from any other key is still refused, and no trust
store is touched.

⚠ **It is still a condition of every capture taken through it**, and the switch
list is recorded on the result for that reason. `HARNESS-10` measures whether
the difference changes the answer and `DRIVER-04` is the platform detail.

⭐ **The pin is produced by the harness rather than by the driver**, because the
harness holds the key pair. `Authority::spki_pin` is the seam, and the base64
under it is checked against the specification vectors rather than against
itself.

⚠ **The two commands could not compose until the pin was printed.** The library
hands it over in one process, which is what the end-to-end test uses, and
outside a test the driver had no way to get it: the usage text said the harness
printed it and the harness did not. `--ca-out` now prints `pin: VALUE` on
stderr, so the stdout contract is untouched, and the false line in the usage is
gone.

### What the tests cover, and the one they cannot

| the test | what it would catch |
| --- | --- |
| a build is reported with the source that answered | a version nobody checked, and an executable no source could version being reported as a browser with an unknown build |
| a capture completes against the harness | the whole path, in one process, with the pin and the served key minted together so they cannot drift |
| the throwaway profile is removed | a capture that belongs to a profile history rather than to the build |
| the URL is the last argument | the mode trap this entry inherited: a URL passed as a switch value makes the browser navigate and then sit |

⛔ **On a host with no browser both tests print a SKIP and return.** A test that
passed vacuously there would make the suite green on the one machine that could
not have run it. ⚠ That is why the skip is printed rather than silent.

### ⛔ A browser being INSTALLED is not a browser that can complete a capture

The capture test asserted the first and claimed the second, and the remote
checks refused it on both runners. ⚠ **Both of them ship a browser**, so the
no-browser skip never fired.

| runner | what happened |
| --- | --- |
| Linux | the launch exited on its own after 3.0s, having connected to nothing |
| Windows | the browser connected and the handshake aborted with `os error 10053` |

⭐ **Neither is a defect in this tree.** Both are a headful browser on a machine
with nobody at it, which is a different environment rather than a broken one.

⛔ **So the capture half is gate part (b) and it is opt-in**, behind
`B_IDS_DRIVE=1`, with a printed skip that says why. The resolve half stays in
part (a) and passed on both runners.
[`../docs/methodology/gate.md`](../docs/methodology/gate.md) already put the
driven pass in the agent's hands rather than in the suite; this entry had it in
the wrong part of the gate, and the output above is the run that did it.

⚠ **The suite is faster for it**: the capture test was 30s of every `cargo
test`, and on the Linux runner it was 97s.

### ⚠ What is NOT here

- **No version is fetched from anywhere.** `DRIVER-02` reads what is serving
  rather than what is installed, and it is a different question.
- **Headless is a flag and nothing normalises its User-Agent.**
  `DRIVER-03` is that entry, and the default is headful precisely so nothing
  is normalised by accident.
- **Two families are looked for and one install of each is taken.**
  `DRIVER-06` is branded against unbranded builds and `DRIVER-05` is
  acquisition.
- ⚠ **The end-to-end test takes about thirty seconds**, because the browser
  does not exit once the harness stops listening and the launch runs to its
  ceiling before it is killed.

---

## DRIVER-02. Read the version that is serving, not the one that is published

**Source** the founding brief; the measurement is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 7
**Category** driver, **Priority** P1, **Effort** M, **Status** done

### Problem

A version-history endpoint queried for the newest version on a channel answers
with the newest version **known**, which during a staged rollout is a build
almost nobody has. Capturing it produces a correct fingerprint of a browser that
does not exist.

### Premise

Measured elsewhere and inherited. The table is in
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 7: the
highest known build had a rollout fraction of 0.005, the build at fraction 1 was
two majors behind it, and the automation-build index disagreed with that by one
patch component.

### Approach

Read the releases endpoint and the **rollout fraction**, cross-check against the
automation-build index, and print the highest published version and its fraction
beside the answer, so a reader can check the choice rather than take it.

Trap every fetch separately, so one dead endpoint degrades the run rather than
failing it, and report which sources answered.

⚠ **Treat a disagreement between two first-party sources as a finding rather
than an error.** That is how the defect above was found in the first place.

Must not: take the first answer, and must not silently prefer one source when
two disagree.

### Prove

```bash
cargo run -p b-ids-driver -- versions --channel stable --json
```

Passing means: the output names every source it asked, what each answered, the
chosen build with its fraction, and any disagreement, and exits 0 when at least
one source answered.

### Closing

**Closed 2026-09-01.** ⭐ **The inherited defect reproduced here, to the digit,
on a different day.**

```text
$ cargo run -p b-ids-driver -- versions --channel stable --json
{"answers":[{"source":"releases","version":"152.0.7977.65","error":null},{"source":"chrome-for-testing","version":"152.0.7977.64","error":null}],"chosen":{"version":"152.0.7977.65","fraction":1.0,"highest_known":"153.0.8010.12","highest_fraction":0.005},"disagreement":true}
rc=0
```

```text
$ cargo run -p b-ids-driver -- versions --channel stable
  releases: 152.0.7977.65
  chrome-for-testing: 152.0.7977.64
chosen 152.0.7977.65 fraction Some(1.0)
highest known 153.0.8010.12 fraction Some(0.005)
the sources disagree, and neither is preferred
```

```text
$ cargo test -p b-ids-driver versions
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

#### ⭐ Every number the premise inherited came back the same

| | inherited, 2026-08-29 | measured here, 2026-09-01 |
| --- | --- | --- |
| the highest build the endpoint knows | `153.0.8010.12` | `153.0.8010.12` |
| its rollout fraction | `0.005` | `0.005` |
| the build at fraction 1 | `152.0.7977.65` | `152.0.7977.65` |
| the automation index's stable | `152.0.7977.64` | `152.0.7977.64` |
| the two first-party sources | disagree by one patch component | disagree by one patch component |

⛔ **A run that took the naive answer would be capturing `153.0.8010.12`, a
build being served to one user in two hundred.** That is the entry's whole
premise and it is now a measurement rather than an inherited claim.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 7 carries
the confirmation beside the reading it inherited.

#### ⭐ The control, and it is what makes the disagreement mean something

```text
$ cargo run -p b-ids-driver -- versions --channel beta
  releases: 153.0.8010.12
  chrome-for-testing: 153.0.8010.12
chosen 153.0.8010.12 fraction Some(1.0)
highest known 153.0.8010.12 fraction Some(1.0)
```

⚠ **Beta's two sources agree**, so the stable disagreement is not this command
answering wrongly. ⭐ **And beta at full rollout is the same build stable lists
at `0.005`**, which is the mechanism itself: what stable "knows about" during a
staged rollout is the next channel's build arriving.

#### ⚠ What it says about this project's own corpus

The one profile in the corpus is Chrome `151.0.7922.76`, and stable is serving
`152.0.7977.65`. ⛔ **The corpus is one major behind the browser most people
run**, and nothing said so before this command existed. `DRIVER-05` is
acquisition and `CORPUS-02` is the matrix; between them they are what closes the
gap. This entry's contribution is that the gap is now visible.

#### The design, and the three things it refuses

| | |
| --- | --- |
| ⛔ never the highest build | The highest at full rollout wins, and only where no release states a fraction of 1 does the highest fraction win. |
| ⛔ never a preferred source | Both answers are printed, `disagreement` is a field, and neither is dropped. One source silent is a DEGRADED run rather than a dispute, and the tests hold that apart. |
| ⛔ never a fabricated fraction | A release with no stated fraction is `None` rather than zero. Absent and "served to nobody" are different facts, and a release the endpoint said nothing about can still be the answer through the fallback. |

#### ⚠ Why it shells out to a fetcher, with the two routes that lost

⛔ **The obvious route was refused for a reason worth keeping.**

| route | why it lost |
| --- | --- |
| an HTTP client crate | It brings its own TLS stack into a workspace that vendors one. [`../Cargo.toml`](../Cargo.toml) already names two builds of the same primitives as a cost to refuse, and version discovery is not worth paying it. |
| a client on the vendored rustls | It needs a root store, so a dependency arrives anyway, and an HTTP/1.1 client this project would then own. ⛔ This project has enough parsers to keep correct. |
| ⭐ a fetcher the host already has | No new dependency, no second TLS stack, and trapping each fetch separately falls out of one process per request. |

⚠ **Its cost is that a host with neither `curl` nor `wget` cannot run this**,
which the command reports as a per-source error rather than as a wrong answer.
`mine-repo` already fetches this way and `TOOL-04` is the open entry about that
fetcher stopping when one of its two routes is down; the same shape applies
here and is worth watching.

#### ⛔ The suite touches no network, and that is the design

A test that fetched would fail during somebody else's outage and pass whenever
the live answer happened to match, which is the opposite of what a test is for.
The whole decision is pure and the ten tests are over fixtures. ⭐ **And the
fixture is the inherited measurement itself**, so a test written against
invented numbers could not have covered the defect this entry is about.

⚠ **The driven half is the command**, run above against the live endpoints,
which is what proves the fetching and the parsing that the fixtures cannot.

#### ⚠ What is not covered

| | |
| --- | --- |
| the platform | The releases endpoint is asked about `win`, because a rollout fraction is per platform and reading one platform's while capturing on another compares two questions. `CORPUS-02` is where more arrive. |
| Firefox and Edge | Their endpoints are in [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 7 and neither is called. This entry is the Chrome family, which is what the corpus holds. |
| a staleness schedule | `CI-02` is the entry that runs this on a cron and turns a moved answer into a red check. |

---

## DRIVER-03. Headless changes the User-Agent, and normalising it is reported

**Source** the founding brief; the trap is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8
**Category** driver, **Priority** P1, **Effort** S, **Status** done

### Problem

A headless capture reports a different product token from the browser a person
runs, and on some builds the substitution reaches the brand list too. Pasting a
headless capture verbatim ships a User-Agent that announces automation.

### Premise

Measured elsewhere and inherited.

### Approach

Detect the headless token, normalise it, and record the normalisation in the
profile's provenance map as a `substituted` field with its reason. The value
changes and the fact that it changed is published beside it.

⭐ **Silently rewriting a captured value is the failure this whole project is
about**, so the reporting is the entry rather than a nicety.

Must not: normalise without recording, and must not normalise a field the
detection is not sure about. An uncertain case is `unreproducible` with a
reason.

### Prove

```bash
cargo test -p b-ids-driver headless_normalisation -- --nocapture
```

Passing means: a headless capture fixture produces a profile whose User-Agent
carries the normal product token and whose provenance map marks that field
`substituted` with a reason naming headless mode.


### Closing

**Closed 2026-09-01T06:55:00Z.** ⭐ **The trap was measured here rather than
inherited**, by driving Chrome `151.0.7922.76` at this project's own harness
twice in one session, once with a window and once without, and reading the
header off the decrypted HTTP/2 stream.

| header | headful | headless |
| --- | --- | --- |
| `user-agent` | ends `Chrome/151.0.0.0 Safari/537.36` | ends `HeadlessChrome/151.0.0.0 Safari/537.36` |
| `sec-ch-ua` | `"Not=A?Brand";v="99", "Google Chrome";v="151", "Chromium";v="151"` | ⭐ byte-identical |

```text
$ cargo test -p b-ids-driver headless_normalisation -- --nocapture
running 5 tests
test headless_normalisation_leaves_a_windowed_capture_alone ... ok
test headless_normalisation_measured_that_the_brand_list_does_not_change ... ok
test headless_normalisation_marks_the_field_substituted_with_a_reason ... ok
test headless_normalisation_restores_the_product_token_and_nothing_else ... ok
test headless_normalisation_records_an_unfamiliar_marker_rather_than_guessing ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### ⛔ The premise about the brand list is refuted on the build measured here

The problem statement says the substitution reaches the brand list on some
builds. ⛔ **On Chrome `151.0.7922.76` on Windows it does not**: the two runs
produced a byte-identical `sec-ch-ua`, and the only difference between them is
the product token in the User-Agent.

⚠ The title stays and the premise stays, because "some builds" is a claim
about builds this session did not measure. What changed is that nothing is
rewritten in a field nothing was seen to change, and the measurement is a test
so it cannot be lost.

⭐ **A field carrying a headless marker that this module does not normalise is
recorded `unreproducible` with a reason rather than guessed at.** That is the
entry's own rule, and it is what makes the next build's change visible instead
of silently wrong.

### What the five tests cover

| the test | what it would catch |
| --- | --- |
| the product token is restored and nothing else is | a normalisation that rewrote the version, the platform block or any other token |
| a windowed capture is left alone | a normalisation that fired where it was not needed, marking a MEASURED field as substituted |
| the substitution is recorded with its reason | ⛔ the failure this whole project is about: a rewritten capture with nothing beside it saying so |
| an unfamiliar marker is recorded rather than rewritten | a guess at a field nothing was seen to change |
| the brand list does not change | the inherited claim, kept as a measurement so a future session does not re-derive it wrongly |

### ⚠ What is NOT here

- **Headless is not the default anywhere.** The driver runs with a window
  unless asked, so nothing is normalised by accident.
- **One build was measured.** `CORPUS-02` is the matrix, and a claim about
  "some builds" needs more than one machine.
- ⚠ **The normalisation is available and nothing calls it in a capture path**,
  because no capture path writes a profile yet. `CORPUS-01` is where a capture
  becomes a profile, and that is where this belongs.

---

## DRIVER-04. The root store a browser actually reads

**Source** the founding brief; the trap is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8
**Category** driver, **Priority** P2, **Effort** S, **Status** done

### Problem

Adding the harness certificate authority to the user's certificate database, and
confirming it is there, still produces an unknown-certificate failure, because
the browser uses its own root store for server authentication on that platform.

### Premise

Measured elsewhere and inherited.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8 carries
it.

### Approach

Document, per platform, what the browser actually trusts and how the harness
gets there. The way through on the affected platform is a browser flag that
changes what it **accepts** after the handshake rather than what it **sends**,
so the captured hello is unaffected.

⛔ Record loudly that such a flag is a capture tool and never something to ship
in a client.

Must not: recommend disabling verification generally. The `--ca-out` path exists
so that a client can complete a **verified** handshake, and it is the preferred
route.

### Prove

```bash
sh experiments/40-trust-paths.sh
```

Passing means: the script reports, per platform available on this host, which
trust route completed a handshake, and the profile records which route produced
each capture.

### ⚠ The acceptance names `40-` rather than `10-`, and the reason is a rule

⛔ **A number is never reused and `10-` was taken** by
[`../experiments/10-first-profile.sh`](../experiments/10-first-profile.sh),
which was written after this entry was authored.
[`../docs/methodology/experiments.md`](../docs/methodology/experiments.md) says
a citation of `10-` has to keep meaning what it meant, so this entry's script is
[`../experiments/40-trust-paths.sh`](../experiments/40-trust-paths.sh) and the
Prove block above is corrected rather than the file being misnumbered to match
it.

### Closing

**Closed 2026-09-02T03:15:00Z.** The script reports, per route, whether a
browser completed a handshake and reached HTTP/2 on the platform it ran on, and
names the one route it deliberately does not run.

```text
$ sh experiments/40-trust-paths.sh --headless

resolving a browser
{"family":"chrome","name":"Chrome","path":"C:\\Program Files\\Google/Chrome/Application/chrome.exe","version":"151.0.7922.76","answers":[["sibling-directory","151.0.7922.76"]],"disagreement":false}

-- routes this host can exercise without changing the machine --
route=pin handshakes=3 h2=3 connections=4
route=none handshakes=0 h2=0 connections=4
route=verification-disabled handshakes=3 h2=3 connections=4

-- the route this host does NOT exercise --
route=trust-store handshakes=- h2=- connections=- not-attempted
  ⛔ Installing a root is a change to this machine, and the operator ruled
     2026-09-01 that it belongs on a runner that is thrown away. HARNESS-14.
  would be: certutil -addstore -user Root <ca.pem>
  ⚠ and whether a browser reads THAT store is the question DRIVER-04 asks

conditions
  host      MINGW64_NT-10.0-26200 3.6.9-b4195d69.x86_64
  browser   Chrome 151.0.7922.76
  rustc     rustc 1.98.0 (88d9e12ae 2026-08-18)
  taken     2026-09-02T03:11:25Z
  headless  --headless
  handshakes 4 per route, resumption refused, one throwaway profile each

the standing route completed 3 connection(s) to HTTP/2
exit=0
```

#### ⭐ The negative control is the finding

⛔ **`route=none` reached 0 handshakes over 4 connections.** The browser
connected four times and completed nothing, which is what says the pin is doing
the work. ⚠ Without that row the other two rows would prove nothing: a harness
whose certificate the browser accepted anyway would give the same `pin` line.

#### What each route means, and what is recorded

| route | `captured.trust` | this host, Chrome `151.0.7922.76`, headless |
| --- | --- | --- |
| `pin` | `spki-pin` | ⭐ 3 handshakes, 3 to HTTP/2. The standing route, and no trust store is changed. |
| `none` | n/a | ⛔ 0 of 4. The control. |
| `verification-disabled` | `verification-disabled` | 3 handshakes, 3 to HTTP/2. ⛔ A CAPTURE TOOL AND NEVER SOMETHING TO SHIP IN A CLIENT. |
| `trust-store` | `trust-store` | ⚠ not attempted here. It changes the machine, and `HARNESS-14` runs it on one that is thrown away. |

⭐ **The recording half already existed and is unchanged.** `Trust` names all
four states and `Profile::check` refuses `not-applicable` on a profile carrying
both a hello and HTTP/2 frames, so a capture cannot claim no handshake was
completed while publishing one.

#### ⛔ What is NOT concluded, and this is the entry's own rule

⚠ **This is one machine on one day.** The inherited claim in
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8 is about
**Chrome on Linux** not reading the user's NSS database for server
authentication. ⛔ Nothing here measures that: the script ran on Windows, and it
reports the Linux command it would use rather than claiming a result for it.
`CORPUS-02`'s Linux lane is where that gets measured.

⚠ **And "verification-disabled works" is not a recommendation.** It is recorded
because a route that completes a handshake by removing every check the subject
makes has to be labelled honestly rather than left looking equivalent to the pin.
The `--ca-out` path is preferred and the driver refuses the two together: a
launch trusts one key or it verifies nothing, never both.

#### The driver gained one switch, and it is refused beside a pin

`b-ids-driver drive --disable-verification` passes
`--ignore-certificate-errors --test-type`. ⚠ **Both flags, and the second is not
decoration**: a branded Chromium ignores the first unless the run is marked as a
test run. ⛔ Passing it together with `--pin` is refused, because two trust
configurations at once is a capture whose condition nobody can name.

---

## DRIVER-05. Acquisition, with more than one way to get a build

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** driver, **Priority** P2, **Effort** M, **Status** done

### Problem

Every download URL will one day 404. A capture pipeline with one acquisition
route stops working on that day, and by then the build is gone.

### Premise

Believed, and the design rule behind it is that no fact has a single source.

### Approach

For "where do I get that build", know several routes and try them in order:
the vendor's automation-build archive, the vendor's own package repository, a
distribution's packages, the browser bundles that test frameworks pin and
mirror, a prebuilt container image, and a copy this project already cached.

**Cache every artefact fetched, keyed by digest**, and keep the last known-good
build per channel. That turns "upstream removed the artefact" from an outage
into a note.

**Pin by digest, never by tag**, for every image, tool and archive.

Must not: redistribute a browser binary. Publish measurements, versions, digests
and the URL a build was fetched from. Never the artefact.

### Prove

```bash
cargo test -p b-ids-driver acquisition -- --nocapture
```

Passing means: with the primary route made to fail, the resolver falls back,
reports which route answered, and the resulting profile records the route and
the digest of what it fetched.


### Closing

**Closed 2026-09-01T14:40:00Z.** A build has more than one way to be got, the
routes are tried in order, and what answered is recorded on the profile with the
digest of what arrived.

```text
$ cargo test -p b-ids-driver acquisition -- --nocapture
     Running tests/acquisition.rs
running 5 tests
test acquisition_treats_an_empty_answer_as_a_refusal ... ok
test acquisition_reports_every_refusal_when_no_route_answers ... ok
test acquisition_leaves_out_the_exact_build_route_when_no_build_was_named ... ok
test acquisition_falls_back_when_the_primary_route_is_down ... ok
test acquisition_plans_the_installed_route_first_and_the_index_last ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
exit=0
```

### ⭐ The door sweep found nothing read `captured.acquisition`

⛔ **The published schema constrained it and this side did not.** Its route is an
enum and its object has four required fields in
[`../crates/b-ids-schema/schema/browser-profile-1.schema.json`](../crates/b-ids-schema/schema/browser-profile-1.schema.json),
so a consumer validating a profile would refuse a bad one. `Profile::check`
would not: a profile could claim a route no driver can produce and a digest that
is not one, and every check in this tree would have passed it.

⭐ **Fixed in the same change, and mutation-proved.** The route is checked
against `ACQUISITION_ROUTES` and the digest against the 64-lower-case-hex shape
the corpus index already uses for every published file. Disabling the route
check with `if false &&` takes exactly one test red:

```text
$ cargo test -p b-ids-schema --test acquisition
thread 'acquisition_a_route_no_driver_can_produce_is_refused' panicked at crates\b-ids-schema\tests\acquisition.rs:46:5
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

⚠ **Absent is correct and is not checked.** A build already installed on the
machine was not obtained by this project and has no route.

### ⛔ The design changed during implementation, and the boundary is why

The module was written to hash the bytes itself with the harness's `sha256`. It
does not, and cannot:
[`../crates/b-ids-driver/Cargo.toml`](../crates/b-ids-driver/Cargo.toml) keeps
`b-ids-harness` as a DEV dependency on purpose, because a driver that imported
the harness would be one component with two jobs. The compiler said so on the
first build.

⭐ **So the digest is injected too**, beside the fetcher. The bytes are hashed by
whoever asked for them, the driver stays free of the harness, and the test
supplies the harness's own digest because a test may.

⚠ **That is a larger change than the entry described** and the gate was
re-passed against it, per
[`../docs/methodology/gate.md`](../docs/methodology/gate.md).

### ⭐ The fetcher is a parameter, and that is what makes this testable at all

⛔ **The failure case is the whole point of the entry**, and it cannot be
arranged against a live network: a pipeline with one route works right up until
the day the URL 404s, so what has to be tested is the day it does.
[`acquire_with`](../crates/b-ids-driver/src/acquire.rs) takes the fetch as a
closure, and the suite hands it one that refuses.

⚠ **A function that reached the network itself could only be tested on a day the
network agreed**, which is a test that reports the weather.

### The routes, and why the order is the design

| route | when it answers | why it sits there |
| --- | --- | --- |
| `installed` | a build is already on this machine | ⭐ it cannot fail for a network reason. ⚠ And it is last in VALUE: it answers with whatever is installed rather than with what was asked for, so a caller wanting an exact build checks the version it gets. |
| `cache` | this project already fetched that build | turns "upstream removed the artefact" from an outage into a note |
| `chrome-for-testing` | the vendor's automation index has that exact build | the one first-party route keyed by BUILD rather than by "current" |

⛔ **A route that cannot answer is left out of the plan rather than offered.**
The automation index is keyed by build, so a plan with no version does not carry
it: offering a URL that must 404 is a route that reports an outage where there
was never an answer.

⚠ **Edge and the rest are not here**, and that is stated rather than implied:
they have their own indexes and `DRIVER-06` is the entry that adds them. The
plan says so by returning two routes rather than three.

### ⛔ What is recorded, and what is never redistributed

The profile's `captured.acquisition` carries the route, the URL and the digest.
⛔ **The artefact never appears anywhere**: this project publishes measurements,
versions, digests and where a build was fetched from, and the binary is the
vendor's to serve.

⭐ **The digest is what makes an acquisition reproducible after the artefact
stops being served.** Every download URL will one day 404, and a later reader
still has to be able to say whether two captures used the same bytes.

⚠ **The field is omitted from the serialised form when absent**, which nothing
else in the model does. The reason is the corpus rather than taste: it is
append-only, so a profile published before the field existed has to keep
serialising exactly as it was published. The one profile in the corpus today
carries no acquisition, because it was captured from a build that was already
installed.

### ⚠ What this entry does NOT do

- **it does not fetch.** Every route above is a plan and a digest contract; the
  fetch itself is the caller's, and `CI-03` is the caller that has a network.
  ⛔ Building a downloader here with no lane to use it would be machinery with
  one imagined consumer.
- **it does not cache.** `Route::Cache` is in the plan and there is no cache
  directory yet. ⚠ Saying so is the point: the route exists so that the day a
  cache lands, nothing above it has to change.

---

## DRIVER-06. Branded and unbranded builds are different products

**Source** the founding brief; the header sets are [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 6
**Category** driver, **Priority** P2, **Effort** M, **Status** done

### Problem

The build that is addressable by channel is unbranded, and the build that
carries the real brand list reaches one channel only. A profile that does not
say which it came from is a profile whose brand list cannot be trusted.

### Premise

Measured elsewhere and inherited.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 6 carries
the brand string and the table.

### Approach

Record `browser.branded` and enforce it in the validator, per `VALID-01` check
3: a branded profile has a vendor entry in its brand list and an unbranded one
does not.

Capture both where both are reachable, as separate profiles rather than as one
profile with a substituted field, because they are different builds.

⚠ Both give the **host** platform's platform token and operating-system token,
so a profile claiming one platform captured on another needs those substituted
and marked, per `SCHEMA-05`.

Must not: synthesise a brand list for an unbranded build. That is the derivable
value the first rule refuses.

### Prove

```bash
cargo test -p b-ids-driver branded -- --nocapture
```

Passing means: an unbranded capture whose profile claims `branded: true` is
rejected by the validator with a message naming the brand list.

---

### ⭐ Closed 2026-09-03. The vocabulary, then the route that decides it, then the pass that joins the two

⭐ **The blocker named in [`PROGRESS.md`](PROGRESS.md) is gone.** The open
question was where an unbranded build lives in the corpus, and the answer taken
is this entry's own recommendation: **`for-testing` is a `Channel`**, and
`branded: false` follows from it rather than becoming a fifth path component.

| what landed | |
| --- | --- |
| `b_ids_schema::Channel::ForTesting` | ⛔ renamed explicitly to `for-testing`, because the container's `rename_all = "snake_case"` would have written `for_testing`, which is not what `as_str`, the matrix, the published schema or the route spell |
| the published JSON Schema | its channel enum carries the seventh value, with the sentence saying a build on it is unbranded |
| ⭐ nothing about the layout moved | the channel is already part of the route and of the `latest` key, so no consumer's pin moves |
| ⚠ `latest` is unaffected | the pointer map is built from stable profiles alone, so an automation build cannot be mistaken for one. `CORPUS-03` is the rule |

#### The acceptance

```text
$ cargo test -p b-ids-driver branded
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.01s
```

⚠ **Seven targets are compiled and five report zero**, so the block above is the
two that ran, with the empty ones dropped and this line saying so. ⭐ The four
are this entry's new file; the fifth is an acquisition case whose name already
contained the filter word, and it is counted here rather than hidden.

⚠ **This command selected NOTHING until now**, and exiting 0 having run nothing
is the "step that exits 0 having done nothing it was asked to do" row of
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md)
wearing an acceptance's name. `crates/b-ids-driver/tests/branded.rs` is the file
that makes it select something, and every case in it is named for the filter.

#### ⭐ The driver half was a route that could not answer the question

The validator has refused an incoherent claim since `VALID-01`. What the driver
could not do was say what a build obtained through a given route IS, so nothing
connected an acquisition to the claim a profile makes about it.
`Route::branded` is that answer, and its shape is the entry:

| route | branded | why |
| --- | --- | --- |
| `chrome-for-testing` | `Some(false)` | the automation index serves unbranded builds, and they are the ones automation reaches for first |
| `vendor`, `edge-enterprise` | `Some(true)` | each serves the vendor's own product |
| `installed`, `cache` | ⛔ `None` | whatever somebody installed. Returning `true` here would be the synthesised brand list this entry forbids |

#### ⛔ The finding: the rule does not run on a profile that keeps names only

⚠ **Check 3 answered `NotCheckable` over the fixture**, and the fixture is the
default shape: a profile records header NAMES and not values, which is
`SCHEMA-04`'s privacy rule. So the enforcement half this entry called "already
existing" fires only on a profile whose capture turned values on deliberately.

⛔ **That is recorded rather than repaired.** Widening the privacy rule to make
a check convenient is the wrong direction, and the corpus captures already turn
values on where the field matters. The suite asserts the `NotCheckable` answer
first, so the limitation is stated by a test rather than by a sentence.

#### The pass, and what each case is for

| case | what it drives |
| --- | --- |
| `branded_the_automation_index_serves_an_unbranded_build` | a real acquisition with the first two routes arranged to fail, so the one that answers is the unbranded index and its two refusals are kept |
| `branded_a_route_that_does_not_decide_it_says_so` | the three answers above, including the two that are `None` |
| `branded_an_unbranded_capture_claiming_a_brand_is_refused_by_the_validator` | ⭐ the acceptance: one field flipped on an otherwise unchanged profile, and the refusal names `Google Chrome` and the brand list |
| `branded_a_branded_build_with_no_vendor_entry_is_refused_from_the_other_side` | the same rule the other way, because a check that only ever fires one way is half a check |

⚠ **`b-ids-validator` is a DEV dependency of this crate now**, and the two test
graphs point at each other: the validator already takes this crate as a dev
dependency for `VALID-03`. Neither build graph does, which is why cargo accepts
it.

#### ⭐ Both `for-testing` matrix cells are enabled

⛔ **Ruled by the operator 2026-09-03.** The vocabulary no longer blocks them, so
`chrome/for-testing/linux64` and `chrome/for-testing/win64` are `enabled: true`
in [`../.github/capture-matrix.json`](../.github/capture-matrix.json) and the
coverage report moves them from `not-attempted` to `absent`, which is the
difference between planned and tried:

```text
$ sh scripts/common/check-coverage.sh
coverage over 8 planned cell(s):

  captured       chrome/stable/linux64              2 profile(s) required
  captured       chrome/stable/win64                3 profile(s) required
  captured       edge/stable/linux64                1 profile(s) required
  absent         chrome/for-testing/linux64         0 profile(s)
  absent         chrome/for-testing/win64           0 profile(s)
  not-attempted  chrome/stable/macos-arm64          0 profile(s)
  not-attempted  chrome/beta/linux64                0 profile(s)
  not-attempted  firefox/stable/linux64             0 profile(s)

3 captured, 2 absent, 3 not attempted.
rc=0
```

⚠ **Nothing here has exercised that lane's capture path.** `capture.yml` allows
every lane to fail alone, so the first scheduled run reports what the lane could
not do rather than taking the run down with it. ⛔ Neither cell is `required`,
so `check-coverage --require-rows` does not fail over them either.

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a capture through the unbranded lane | ⛔ None was taken. The lane is enabled and the next scheduled run is the first attempt; this machine has no browser purge to run and would not run one if it had. |
| `browser.branded` written from the acquisition | ⚠ The capture path still takes it from the identity file. `Route::branded` is the value it should read, and wiring it is `CORPUS-02`'s lane work rather than this entry's rule. |
| a widened privacy rule | ⛔ Declined above, with the reason. |
---

## DRIVER-07. The browser's own output is discarded, so a lane that captured nothing says nothing

**Source** found while working `CORPUS-02`, 2026-09-02; authored on the operator's ruling the same day
**Category** driver, **Priority** P2, **Effort** S, **Status** done

### Problem

`b-ids-driver drive` gave the browser `Stdio::null()`, so a lane that captured
nothing carried no word from the browser about why. A person reading the
artefact learns that the process exited and nothing else.

### Premise

⚠ **Measured rather than anticipated.** On 2026-09-02 the `edge` capture lane
launched Edge on a hosted runner, the browser exited after **1402 ms** having
opened no connection, and what it said went to a null sink. `capture.yml` run
`33584060390`.

### Approach

`Launch.log: Option<PathBuf>`, and `--log PATH` on the command. Both streams go
to the named file.

⚠ **A FILE rather than a pipe.** A pipe nobody drains fills, and a browser that
filled it would block on a write while this process waits for the browser to
exit.

⛔ **Opened before the spawn**, so a path that cannot be written is a refusal
rather than a launch whose output went nowhere while a caller believed it was
being recorded.

Must not: capture the streams and discard them on a successful run. The empty
file IS the answer on a healthy launch.

### Consumers

None: no route or generated file carries a browser log. It is written under the
ignored scratch directory and uploaded as a lane artefact.

### Prove

```bash
cargo test -p b-ids-driver resolve_and_drive -- --nocapture
```

Passing means: `--log` with no value is refused; a path that cannot be opened is
refused before the browser starts, naming the path; and a launch with a log
path writes the file.

### Closing

**Closed 2026-09-02T07:20:00Z**, and the work landed under `CORPUS-02` before
this entry existed, which is why the entry is authored and closed in one change:
the operator ruled on 2026-09-02 that it should have been its own unit of work.

```text
$ cargo test -p b-ids-driver resolve_and_drive -- --nocapture
running 10 tests
test resolve_and_drive_a_family_name_round_trips ... ok
test resolve_and_drive_completes_a_capture_against_the_harness ... ok
test resolve_and_drive_the_vendor_name_is_what_the_corpus_routes_by ... ok
test resolve_and_drive_reports_a_build_from_a_source_it_names ... ok
test resolve_and_drive_log_refuses_a_path_it_cannot_write ... ok
test resolve_and_drive_browser_refuses_a_family_the_resolver_cannot_produce ... ok
test resolve_and_drive_log_with_no_value_is_refused ... ok
test resolve_and_drive_browser_with_no_value_is_refused ... ok
test resolve_and_drive_browser_reports_only_the_family_it_names ... ok
test resolve_and_drive_log_records_what_the_browser_said ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.38s
exit=0
```

⚠ **What is asserted is the FILE, not its content.** A browser that says nothing
on a healthy run is normal, and a test demanding output would fail for the wrong
reason. The file existing is what a later diagnosis needs.

⛔ **And the diagnosis it was written for has not been taken yet.** The `edge`
lane still exits having opened no connection; `--log` means the next run of it
carries what Edge said. `CORPUS-02` is where that gets read.

---

## DRIVER-08. Purge the machine's browsers, install the build the cell names

**Source** the operator, 2026-09-02, on reading that the corpus records builds nobody chose
**Category** driver, **Priority** P0, **Effort** L, **Status** done

### Problem

⛔ **On a machine this project controls completely, it measures whatever
somebody else's image happened to install.** A capture lane calls
`b-ids-driver resolve`, which by design finds what is already there, so the
corpus records a build nobody chose from a source nobody named.

### Premise

⛔ **Measured, and the cost is already paid.** On 2026-09-02, `ubuntu-latest`
served Chrome `151.0.7922.173` and `windows-latest` served `151.0.7922.174`, so
the single highest-value capture available, one build on two platforms, was
unobtainable. All three published profiles carry `captured.acquisition: null`:
the corpus records "it was already here" rather than a URL and a digest, which
is the weakest provenance the artefact half can have in a project whose product
is provenance.

⚠ **And the apparatus exists without a caller.** `b_ids_driver::acquire` has a
route plan and an injected fetcher, `DRIVER-05` closed on it, and nothing
outside its own tests calls it: there is no real fetcher, no unpack, no install,
no purge, and no `acquire` subcommand.

### Approach

A provisioning step, before the resolve step, in every capture lane:

1. ⛔ **Purge every browser of the target family from the machine**, by every
   route an image might have installed one: the package manager, the vendor's
   own uninstaller, and the paths `b_ids_driver::resolve` searches.
2. ⛔ **Confirm the purge**, by running `resolve` and requiring it to exit **2**.
   A purge that reported success while a browser remained is the "reporting a
   result the code never read" row of
   [`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md).
3. **Install the build the matrix cell names**, from the route that cell names,
   with the archive's digest recorded.
4. ⛔ **Confirm the install**, by running `resolve` and requiring the version it
   reports to be the version asked for. A lane that installed one build and
   captured another is the defect this entry exists to prevent, one step along.
5. **Set it up headless and unattended**, with no first-run state and no
   profile that outlives the run, which is what the driver already does.

⭐ **Two routes, because they are two products.** Ruled by the operator
2026-09-02, and the matrix carries both as separate cells:

| route | what it serves | what a profile records |
| --- | --- | --- |
| the vendor's own channel | branded Chrome, current build only | `branded: true`, and both platforms get the same build because both install on the same day from the same channel |
| the automation-build index | an exact build, any version, every platform | `branded: false`. ⚠ A different brand list and a different `sec-ch-ua`: `DRIVER-06` is the entry that measures the difference |

⛔ **Never redistribute the artefact.** The URL and the digest are published;
the binary is the vendor's to serve. That rule is `DRIVER-05`'s and it does not
move.

⛔ **A lane that cannot provision captures nothing and exits 2.** "This runner
has no browser" and "the capture failed" are different facts, and `CI-07` is the
rule.

Must not: install alongside the image's browser and hope the driver picks the
right one. Two browsers on one machine is a capture that is wrong in a way
nothing notices, which is exactly what this entry is replacing.

Must not: run the purge on a machine that is not disposable. `HARNESS-14`'s
`B_IDS_DISPOSABLE` guard is the shape; a developer's laptop must not lose its
browser to an experiment.

### Decision

**Ruled by the operator 2026-09-02.** Purge completely, confirm no leftovers,
install the build that is wanted, set it up headlessly, and carry both the
branded and the unbranded route as separate matrix cells. ⭐ Priority P0.

### Consumers

`corpus/v1/**` gains a populated `captured.acquisition` on every profile written
after this lands. ⚠ **Nothing is fetched from this repository yet**, so there
are no consumers to break: `PUB-01`, `PUB-02` and `PUB-03` are the three
surfaces and none exists. The field is already in the published schema and
already optional, so the change is additive whenever they do.

### Prove

```bash
sh scripts/common/check-provisioning.sh
```

Passing means: on a disposable machine, the script purges every browser of the
named family and `resolve` then exits 2; it installs the named build and
`resolve` then reports exactly that version; a purge that leaves a binary
behind, and an install that produces a different version than was asked for, are
both refused with a message naming what was found.

### ⭐ What exists, and what it was before the runner run

⚠ **The tool and its acceptance existed for a day before either had run
anywhere.** What follows is what landed first; the closing below is what a
runner then proved about it.

| what exists now | |
| --- | --- |
| ⭐ [`../scripts/common/provision-browser.sh`](../scripts/common/provision-browser.sh) | purge, confirm the purge by requiring `resolve` to exit 2, install, confirm the version. The vendor route is implemented for Linux and Windows. |
| ⭐ [`../scripts/common/check-provisioning.sh`](../scripts/common/check-provisioning.sh) | the acceptance: eight checks on any host, seven of them refusals, and the provisioning itself only where the machine is disposable |
| the `--plan` mode | prints what the tool would do, per platform and route, and runs nothing. It is what a person reads before letting this near a machine. |

```text
$ sh scripts/common/check-provisioning.sh
provisioning ok: 8 check(s), every refusal held, provisioning skipped
  SKIP the provisioning itself: this machine is not disposable, so nothing
  was purged. A workflow on a disposable runner is where that leg runs,
  and .github/workflows/provision.yml is what runs it.
  SKIP the exact-build route: it runs with --build on a disposable
  machine, and the build is named by the matrix cell rather than here.
exit=0
```

#### ⛔ The guard is two conditions, and the reason is an incident

⚠ **On 2026-09-02 this tool ran its purge path on the operator's own laptop.** A
session proving the guard could fail mutated the single condition and ran the
live tool on the machine the guard protects. Nothing was removed, because the
Windows uninstaller match did not fire, and the confirm step then refused
correctly at exit 1; the install step is after that confirm and was never
reached. ⛔ "It happened not to match" is not a safety margin.

⭐ **Two conditions from two sources now, and one edit cannot lift both**:
`B_IDS_DISPOSABLE=1`, which this project sets only inside a workflow, and `CI`,
which the platform sets on every hosted runner. All three refusal paths are
asserted:

```text
B_IDS_DISPOSABLE=unset and CI=unset, and BOTH are required.      exit=2
B_IDS_DISPOSABLE=unset and CI=true,  and BOTH are required.      exit=2
B_IDS_DISPOSABLE=1     and CI=unset, and BOTH are required.      exit=2
```

⛔ **And the rule that was missing is written down**: a test that has to bypass a
guard runs against a copy under the ignored scratch directory, never against the
file on a machine the guard protects.
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md)
carries the row and [`../docs/HISTORY/README.md`](../docs/HISTORY/README.md) the
incident.

### ⭐ Closed 2026-09-02. The purge and the install ran on hosted runners

⛔ **Until this run, everything about this tool was a claim.** Its seven
refusals were proved on a laptop and its success path had never executed
anywhere. `.github/workflows/provision.yml` run `33628209454` executed it on
`ubuntu-latest` and `windows-latest`, both routes on each, and every step
confirmed itself.

```text
$ cat .tmp/provisioned.txt          # ubuntu-latest, the vendor route
-- purging every chrome on this machine --
before  151.0.7922.173
after   nothing resolves, resolve exits 2

-- installing chrome via vendor --
url     https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
sha256  a0b7a64f768ffc0ff5ccc9260ad9ebb53fd16f7a131e2e36994da58b82d913df
bytes   140834800
version 152.0.7977.75

provisioned  chrome 152.0.7977.75 via vendor on linux
```

```text
$ cat .tmp/acquired.txt             # ubuntu-latest, the exact-build route
-- purging every chrome on this machine --
before  152.0.7977.75
after   nothing resolves, resolve exits 2

-- installing chrome via for-testing --
index   https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json
sandbox root:root 4755 /opt/google/chrome/chrome-sandbox
url     https://storage.googleapis.com/chrome-for-testing-public/151.0.7922.76/linux64/chrome-linux64.zip
sha256  d9e4c5916f77e737f22056cab47cd8abb7d39ebec559500ffc2d6f3e02106a7e
bytes   193284800
version 151.0.7922.76

provisioned  chrome 151.0.7922.76 via for-testing on linux
```

⭐ **And the Windows lane, which is where the new resolver source earns its
place.** The `after` line is read by `b-ids-driver resolve`, independently of
the tool's own confirm:

```text
$ cat after.jsonl                   # windows-latest, after the for-testing route
{"family":"chrome","name":"Chrome","path":"C:\\Program Files\\Google/Chrome/Application/chrome.exe","version":"151.0.7922.76","answers":[["manifest-file","151.0.7922.76"]],"disagreement":false}
```

`manifest-file` is the source that did not exist this morning. Without it the
`answers` list is empty, `resolve` skips an executable it cannot version, and
the confirm step refuses a correct install.

#### ⛔ The measured finding that refutes a premise of the ruling

⚠ **The operator ruled on 2026-09-02 that the vendor route gives both platforms
the same build, "because both install on the same day".** ⛔ Measured on one
day, one hour apart, from the vendor's own channel:

| platform | the vendor route served |
| --- | --- |
| `ubuntu-latest` | `152.0.7977.75` |
| `windows-latest` | `152.0.7977.76` |

⛔ **Two builds, not one.** The reasoning behind the ruling does not hold: the
vendor's channel does not serve one build across platforms on a given day.
⭐ **The exact-build route does**, and this is the same run:

| platform | the for-testing route served |
| --- | --- |
| `ubuntu-latest` | `151.0.7922.76` |
| `windows-latest` | `151.0.7922.76` |

⭐ **So one build on two platforms, which is the single highest-value capture
available and has been unobtainable since the first runner capture, is
obtainable only through the automation route.** ⚠ That route is UNBRANDED, so
the pair it produces answers "is the TLS half platform-independent" and not "is
branded Chrome platform-independent". `DRIVER-06` is the entry that measures the
difference, and `CORPUS-02` is where the pair lands.

⚠ **The ruling itself stands and is the operator's.** What is refuted is one
sentence of its reasoning, and it is written here rather than edited into the
ruling.

#### What the tool records now

⭐ **`captured.acquisition` is populated from what the tool wrote**, which was
the last of the six items. `provision-browser` writes
`.tmp/provision-browser/acquisition.json`; `experiments/10-first-profile.sh`
reads it into the identity; `b_ids_corpus::capture` copies it onto the profile.
⚠ Absent stays absent: a build already on the machine was not fetched by this
project and has no route or digest, which is a different fact from a fetch that
failed.

```text
$ target/debug/b-ids-corpus.exe add --captures CAPTURES --identity IDENTITY --root SCRATCH
8 connection(s): 1 cold, 0 resumed, 6 further cold, 1 abandoned
wrote SCRATCH\corpus\v1\chrome/stable/linux64\151.0.7922.173.json
chrome-151.0.7922.173-linux64-stable
$ node -e "console.log(require('SCRATCH/corpus/v1/chrome/stable/linux64/151.0.7922.173.json').captured.acquisition.route)"
chrome-for-testing
```

⛔ **The route vocabulary gained `vendor`, because it had no name for the route
the tool's own vendor path uses.** `b_ids_schema::ACQUISITION_ROUTES` would have
refused every profile taken through it. Three copies of that list exist, in the
driver's enum, in the schema's constant and in the published JSON schema, and
two tests now assert that all three agree.

#### The six items, and where each one landed

| | |
| --- | --- |
| 1. the `for-testing` route | `b-ids-driver acquire` reads the index, `provision-browser` fetches, unpacks and installs. Both platforms, proved on runners. |
| 2. the matrix carries the routes | every cell names a `route`, a `branded` and a `build`. ⚠ The two unbranded cells stay `enabled: false` on a question that is not this entry's: the corpus route carries no `branded` and `Channel` has no automation channel, so a branded and an unbranded build of one version publish at one path. [`PROGRESS.md`](PROGRESS.md) carries it with a recommendation. |
| 3. a provisioning workflow | [`../.github/workflows/provision.yml`](../.github/workflows/provision.yml), dispatch only, each platform running the acceptance in its own language |
| 4. a run on a disposable runner | run `33628209454`, both platforms, both routes, green |
| 5. `captured.acquisition` populated | from the record the tool writes, never retyped |
| 6. `check-provisioning` into the gate | it is a gate check now, in both halves, and `check-twins` compares the pair as before |

#### ⚠ What this entry did NOT settle

⛔ **No capture has been taken through a provisioned browser yet.** The
provisioning leg and the capture leg are two workflows, and joining them is
moving the step into `capture.yml`, which is `CORPUS-02`'s business now rather
than this entry's. Every profile in the corpus today still records
`captured.acquisition: null` and will continue to: the corpus is append-only and
those profiles were taken through builds nobody chose.

⛔ **And the first run failed, for a reason worth keeping.** Run `33627230086`
failed both lanes before anything was purged: `.tmp` is ignored by git, so a
fresh checkout does not have it, and the redirect into `.tmp/provisioned.txt`
failed before the command on its left ever ran. Both lanes then reported that
the tool "did not purge and install cleanly" when the tool had never been
invoked. ⭐ It failed safe and it failed loudly, which is what the workflow is
for; the message was the part that was wrong, and both halves name the exit code
and the output file now.

#### ⛔ Two findings that changed what this entry can promise

⚠ **Both were measured on 2026-09-02 while the route was being written, and
each one moves a claim the entry made before them.**

| the finding | what it changes |
| --- | --- |
| ⛔ **The automation index publishes a SUBSET of builds.** It carried 2497 builds in all and 67 of Chrome `151`, and **neither `151.0.7922.173` nor `151.0.7922.174`**, which are the two the runner images served. | "an exact build, any version" is wrong. The route serves an exact build **from the set the index publishes**, and the two builds the corpus captured on runners cannot be reproduced through it. ⭐ `151.0.7922.76`, which is the corpus's own first profile, IS published, on both platforms. |
| ⛔ **`resolve` could not version an automation build on Windows at all.** The archive is FLAT: read from the central directory of `chrome-win64.zip` for `151.0.7922.76`, `chrome.exe` sits beside `151.0.7922.76.manifest` and there is no version-shaped DIRECTORY. `Source::SiblingDirectory` answered nothing and `Source::VersionFlag` is not asked on Windows. | the confirm step in step 4 of the tool could not have passed on Windows. `Source::ManifestFile` is the third source and `b_ids_driver::sources_for` is the one reader all three go through. |

⛔ **And a third finding is not this entry's to fix.** The corpus route is
`browser/channel/platform/version` and carries no `branded`, and `Channel` is a
closed vocabulary of six that does not include the automation channel. So a
branded and an unbranded build of one version publish at one path. That is why
the two `for-testing` cells are planned and not attempted, and
[`PROGRESS.md`](PROGRESS.md) carries the question with a recommendation.
`DRIVER-06` owns the difference between the two products.

```text
$ target/debug/b-ids-driver.exe acquire --browser chrome --version 151.0.7922.76 --platform linux64 --index .tmp/cft-known-good.json --json
{"browser":"chrome","index":"https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json","platform":"linux64","route":"chrome-for-testing","schema":"acquire/1","url":"https://storage.googleapis.com/chrome-for-testing-public/151.0.7922.76/linux64/chrome-linux64.zip","version":"151.0.7922.76"}
exit=0
$ target/debug/b-ids-driver.exe acquire --browser chrome --version 151.0.7922.173 --platform linux64 --index .tmp/cft-known-good.json
b-ids-driver: the automation index does not publish 151.0.7922.173. It carries 2497 build(s), and it publishes a subset rather than every build the vendor ships. Nearest in the same line: 151.0.7922.2, 151.0.7922.3, 151.0.7922.4, 151.0.7922.5, 151.0.7922.10, 151.0.7922.19, 151.0.7922.34, 151.0.7922.47, 151.0.7922.71, 151.0.7922.76, 151.0.7922.77, 151.0.7922.138
exit=1
$ target/debug/b-ids-driver.exe acquire --browser edge --version 151.0.4129.101 --index .tmp/cft-known-good.json
b-ids-driver: there is no automation-build route for edge. It is the one route that serves an exact build, and it is Chrome only.
exit=2
```

⭐ **The acceptance grew from seven refusals to eight checks, and both halves
answer identically.** The eighth is that the `for-testing` plan names an index
step where the vendor plan does not: two routes described identically is a plan
nobody could use to tell them apart.

```text
$ sh scripts/common/check-provisioning.sh --json
{"schema":"check-provisioning/2","checks":8,"problems":0,"provisioned":"skipped","acquired":"skipped"}
$ pwsh -NoProfile -File scripts/common/check-provisioning.ps1 -Json
{"schema":"check-provisioning/2","checks":8,"problems":0,"provisioned":"skipped","acquired":"skipped"}
```
---

## DRIVER-09. The most dangerous script in the tree is the one with no twin

**Source** found while writing `DRIVER-08`'s tool, 2026-09-02
**Category** driver, **Priority** P1, **Effort** M, **Status** done

### Problem

⛔ **`scripts/common/provision-browser.sh` purges browsers on Windows and it is
a POSIX shell script.** So does its acceptance,
`scripts/common/check-provisioning.sh`. Every other check in `common/` has two
halves for a reason this one does not escape: a native PowerShell session on
Windows has no `sh`, and a runner that has to install a POSIX layer before it
can purge a browser has already changed the machine the capture is about to
measure.

⚠ **And the shape is the one this project rejects everywhere else.** The tool
was a single script with per-platform branches, which is the arrangement
[`../scripts/README.md`](../scripts/README.md) argues against at length in the
section listing what does not have a twin and why. It was entered there as a
debt, which was honest and was not a fix.

### Premise

⭐ **Measured.** The tool and the check exist and were run on this Windows host
through `sh`; `check-provisioning` reports seven refusals held. Neither has a
`.ps1`, so `check-twins` compares nothing for either, and `check-exit-codes`
sees one script where every other check gives it two.

### Approach

Write `provision-browser.ps1` and `check-provisioning.ps1`, then register the
pairs in `check-twins.sh` so the `--json` answer and the exit code of each half
are compared on one tree, exactly as the other twenty-six pairs are.

⛔ **A per-platform split was planned here and is NOT what was built.** The
plan was for the sh half to keep the Linux routes and the PowerShell half the
Windows ones, so that neither carried a branch for a platform it will never run
on. ⚠ Both halves carry both platforms instead: `sh` runs under a POSIX layer on
Windows and `pwsh` runs on Linux, so a half that dropped a platform would refuse
on a host where the other half is the one that is missing. ⭐ The plan stays on
the page because the comparison is only meaningful while both halves answer the
same question on the same host, and splitting them would end that.

⛔ **The two-condition guard is duplicated deliberately, and both halves are
asserted.** A guard implemented once and called from two places is one guard;
this project's twin rule accepts that duplication everywhere else, and what is
being duplicated here is the thing standing between a machine and losing its
browser. `check-provisioning` asserts all three refusal paths in each half.

Must not: have the PowerShell half shell out to `sh`. That reports a green half
of a pair as the whole pair on the hosts that most need the other half, which is
the check contract in [`../scripts/README.md`](../scripts/README.md).

⛔ **This entry was written saying the twin should wait for `DRIVER-08`'s
success path, and the gate disproved that within the hour.** The sh half cannot
land alone: `check-exit-codes` counts the scripts of its own language, the two
counts had been equal only by coincidence, and one untwinned script broke the
tie. ⭐ The correction stays here rather than being edited away, because a
plan a check refused is worth more on the page than a plan nobody tested.

### Consumers

Nothing is published yet, so there are no consumers to break. The capture
workflow is the only caller either script will have.

### Prove

⛔ **The acceptance, and it is a command.**

```bash
sh scripts/common/check-twins.sh
```

Passing means: exit 0, and `check-provisioning` is compared among the pairs.
⚠ Run it on a host with `pwsh`, and do not edit the tree while it runs.

### ⭐ Closed 2026-09-02

⛔ **The gate refused the sh half on its own, and that is why this closed the
day it was written.** `check-exit-codes` reported 27 scripts against 25 the
minute the provisioning tool landed untwinned, and `check-twins` called it a
drift. ⭐ The rule about two halves had never had teeth before, because the
two counts had always been equal; one sh script with no twin and one PowerShell
script with no twin had been cancelling each other out.

| what landed | |
| --- | --- |
| [`../scripts/common/provision-browser.ps1`](../scripts/common/provision-browser.ps1) | the same four steps, the same two-condition guard, the same argument refusals, and the Windows purge as native PowerShell rather than as a payload handed to `powershell -Command` from sh |
| [`../scripts/common/check-provisioning.ps1`](../scripts/common/check-provisioning.ps1) | ⚠ it drives the PowerShell tool and nothing else, which is the check contract: a half that shelled out to `sh` would report a green half of a pair as the whole pair on the host that most needs this one |
| the pair in `check-twins.sh` | with the reason it exists written above the row |

```text
$ pwsh -NoProfile -File scripts/common/check-provisioning.ps1 -Json
{"schema":"check-provisioning/1","checks":7,"problems":0,"provisioned":"skipped"}
$ sh scripts/common/check-provisioning.sh --json
{"schema":"check-provisioning/1","checks":7,"problems":0,"provisioned":"skipped"}
```

#### ⛔ Mutation-proved, on copies, and never on the live file

⭐ **This is the first guard in this repository mutated under the rule the
incident produced.** The tool and its check were copied into the ignored scratch
directory, the copy of the check was pointed at the copy of the tool, and the
copy of the tool was mutated twice. ⚠ Both mutations leave the tool refusing
every case, so neither could reach a purge even if it had been run in the tree:

| planted, in the copy | what the check said |
| --- | --- |
| the refusal stops naming both conditions | 3 problems, `refused without saying 'BOTH are required'`, exit 1 |
| the guard exits 0 instead of 2 | 3 problems, `exit 0, expected 2`, exit 1 |

⛔ **What is still not proved is the success path**, in either half. No runner
has purged or installed anything, and `DRIVER-08` is where that stays recorded.

---

## DRIVER-10. Provisioning is written for one family and the matrix names four

**Source** the operator, 2026-09-02: install and set up any browser version we
want, headless and unattended, on any runner
**Category** driver, **Priority** P1, **Effort** L, **Status** done

### Problem

⛔ **`provision-browser.sh` knows Chrome.**
[`../.github/capture-matrix.json`](../.github/capture-matrix.json) names four
families, `check-coverage --require-rows` can be asked to fail on all four, and
`CORPUS-02` cannot close until `edge`, `chromium` and `firefox` have profiles. A
provisioning step serving one family leaves the other three where they are
today: whatever the image installed, or nothing at all.

### Premise

⭐ **Measured, per family, and they are not variations of one job:**

| family | what purging and installing it actually needs |
| --- | --- |
| `chrome` | the implemented route. The vendor channel on both platforms. |
| `edge` | ⚠ present on the `ubuntu-latest` image at `/usr/bin/microsoft-edge`, measured in `capture.yml` run 33579619515, and the lane is enabled. Its own vendor channel and its own uninstaller; the resolver finds it already. |
| `chromium` | ⛔ no vendor channel with a stable download URL by build. On Linux the distribution package is a snap on the runner image, which is a different install mechanism and a different sandbox. |
| `firefox` | ⛔ a different vendor, a different index, a different archive layout and a different headless switch. The resolver does not know the family at all, which is why the matrix cell is `enabled: false`. |

### Approach

One family per change, in matrix order, and each lands with a profile in the
corpus rather than with a code path:

1. **`edge`**, which is cheapest: the resolver finds it, the lane is enabled,
   and only the purge and install routes are missing.
2. **`firefox`**, the highest-value non-Chrome lane because the TLS stack is
   genuinely different, and which needs `resolve` to know the family before
   anything else can be attempted.
3. **`chromium`**, last, and ⚠ it may end as a recorded refusal rather than a
   lane: a family with no build addressable by version cannot satisfy what this
   entry is for. A refusal written down with its reason is a complete outcome.

⭐ **The route table is per family and it is DATA, not branches.** Each family
names its purge routes, its download index and its headless switch in one
place, so adding a family is a table row and a fixture rather than a fifth arm
of a case statement.

Must not: report a family as provisioned because the purge found nothing to
remove. `DRIVER-08`'s confirm step is the rule and it applies per family.

Must not: fall back to the image's browser when the install route fails. That is
the defect `DRIVER-08` exists to remove, arriving one family later.

### Consumers

Nothing is published yet, so there are no consumers to break. ⚠ Each family
this closes changes what `check-coverage --require-rows` reports, which is a
report rather than a contract.

### Prove

⛔ **The acceptance, and it is a command.**

```bash
sh scripts/common/check-coverage.sh --require-rows chrome,edge
```

Passing means: exit 0 with an `edge` row carrying at least one profile whose
`captured.acquisition` names a URL and a sha256. ⚠ The command is written with
`edge` because that is step 1; each later family widens the list, and the
four-family form in `CORPUS-02` is what closes that entry rather than this one.

---

### ⭐ Closed 2026-09-02 on step 1. The `edge` lane captures, and its build was chosen

⛔ **Every profile before this one records `captured.acquisition: null`.** This
is the first the project chose the build for, and the first whose artefact was
checked against the publisher rather than only recorded.

```text
$ sh scripts/common/check-coverage.sh --require-rows chrome,edge
coverage over 8 planned cell(s):

  captured       chrome/stable/linux64              2 profile(s) required
  captured       chrome/stable/win64                3 profile(s) required
  captured       edge/stable/linux64                1 profile(s) required
  not-attempted  chrome/for-testing/linux64         0 profile(s)
  not-attempted  chrome/for-testing/win64           0 profile(s)
  not-attempted  chrome/stable/macos-arm64          0 profile(s)
  not-attempted  chrome/beta/linux64                0 profile(s)
  not-attempted  firefox/stable/linux64             0 profile(s)

3 captured, 0 absent, 5 not attempted.
exit=0
```

```text
Chrome/stable/linux64/151.0.7922.173           acquisition: null
Chrome/stable/win64/151.0.7922.174             acquisition: null
Chrome/stable/win64/151.0.7922.76              acquisition: null
Edge/stable/linux64/151.0.4129.101             acquisition: edge-enterprise, sha256 bd7604025424...
```

#### ⭐ Edge has a first-party index, and it is better than Chrome's

⚠ **The premise said Edge needed "its own vendor channel and its own
uninstaller".** Measured 2026-09-02: it needs an index, and the one the vendor
publishes carries something the Chrome automation index does not.

| | |
| --- | --- |
| ⭐ **it states a SHA-256 and a byte count per artefact** | so what arrived is compared with what the publisher said it published, and a mismatch is a refusal. The automation index for Chrome states neither, and the tool reports that absence rather than passing over it |
| ⭐ **it carries the build the runner images ship** | `151.0.4129.101` is in it, so the lane installs the build it means to measure rather than the nearest one |
| ⚠ **it is keyed by build and serves no current-build URL** | so `--route vendor` for this family is a refusal with its reason, which is a complete outcome. `--route for-testing` is the route |
| ⛔ **and the route name does not say whether a build is branded** | Chrome's automation index serves UNBRANDED builds and Edge's enterprise index serves the vendor's own branded product. The flag cannot say which and the matrix cell does. `DRIVER-06` is the entry that measures the difference |

```text
$ target/debug/b-ids-driver.exe acquire --browser edge --version 151.0.4129.101 --platform linux64 --index INDEX --json
{"browser":"edge","index":"https://edgeupdates.microsoft.com/api/products?view=enterprise","platform":"linux64","published_bytes":194950834,"published_sha256":"bd7604025424914a61c06293cb6bf269141a29d8c54cf1997110bc96d3365d60","route":"edge-enterprise","schema":"acquire/2","url":"https://packages.microsoft.com/repos/edge/pool/main/m/microsoft-edge-stable/microsoft-edge-stable_151.0.4129.101-1_amd64.deb","version":"151.0.4129.101"}
exit=0
```

#### ⛔ The reason the lane captured nothing was never in this tree

⚠ **`capture.yml` run `33615327503` recorded it, in the browser's own log, which
is what `DRIVER-07` kept it for.** The record called it "Edge exits after 1.4
seconds having opened no connection", which is the symptom:

```text
FATAL:sandbox/linux/suid/client/setuid_sandbox_host.cc:166] The SUID sandbox
helper binary was found, but is not configured correctly. Rather than run
without sandboxing I'm aborting now. You need to make sure that
/opt/microsoft/msedge/msedge-sandbox is owned by root and has mode 4755.
```

⭐ **A vendor package sets that up in its own post-install step and the image's
Edge did not have it**, so the tool confirms it after every Linux install rather
than assuming it. The whole chain, on `ubuntu-latest`:

```text
-- purging every edge on this machine --
before  151.0.4129.101
after   nothing resolves, resolve exits 2

-- installing edge via for-testing --
index   https://edgeupdates.microsoft.com/api/products?view=enterprise
verified bd7604025424914a61c06293cb6bf269141a29d8c54cf1997110bc96d3365d60 matches the digest the index publishes
sandbox root:root 4755 /opt/microsoft/msedge/msedge-sandbox
url     https://packages.microsoft.com/repos/edge/pool/main/m/microsoft-edge-stable/microsoft-edge-stable_151.0.4129.101-1_amd64.deb
sha256  bd7604025424914a61c06293cb6bf269141a29d8c54cf1997110bc96d3365d60
bytes   194950834
version 151.0.4129.101

provisioned  edge 151.0.4129.101 via for-testing on linux
```

#### ⭐ And the lane that captured is the case `HARNESS-15` was written for

⛔ **With session tickets offered, this navigation has no connection carrying
both halves.** Its only cold hello arrived on connection 1, which reached no
HTTP/2, and every other connection resumed:

```text
8 connection(s): 0 cold, 7 resumed, 0 further cold, 1 with no http2
tls from connection 1, http2 from connection 2
edge-151.0.4129.101-linux64-stable
```

⚠ **`0 cold` is the whole point.** Under the rule this session replaced, that
navigation publishes nothing, and the `edge` row would still be absent.

#### The route table, which is what the entry asked for

⭐ **Per family and data rather than branches**, in both halves and in the
driver:

| where | what it holds |
| --- | --- |
| `b_ids_driver::acquire::index_url` and `index_route` | which index a family has and which route a profile records for it. ⛔ `plan` reads the table rather than branching on Chrome. |
| `provision-browser.sh`, six functions above the plan | packages, paths, links, the SUID sandbox helper, the uninstaller pattern and the program directory, per family |
| `provision-browser.ps1`, `$familyRoutes` | the same six, as a hashtable |

⛔ **And the purge removes the family that was asked for.** Before this it
removed Chrome AND Edge whatever `--browser` said, which is not what "purge
every browser of the target family" means and would have destroyed the other
family's lane on a shared runner.

#### ⛔ Two defects this entry's own work produced, both found by running it

| | how it showed |
| --- | --- |
| ⛔ **the recorded route came from the shell, not the driver** | a `case` mapped `for-testing` to `chrome-for-testing` whatever the family was, so the first Edge profile recorded a Chrome route. Found by reading the identity file capture.yml run 33637307031 produced. Both halves read the driver's answer now, out of the same JSON the digest comparison already reads. |
| ⛔ **the table was defined below the plan that reads it** | `--plan` for an `edge` request printed an empty purge line and a command-not-found. Found by running `--plan`, which is the one mode this tool has that runs nothing. |

#### ⚠ What remains, and it is steps 2 and 3

⛔ **`firefox` and `chromium` are untouched**, and the entry's approach is one
family per change. This closes on step 1, which is what its acceptance names.
⚠ `CORPUS-02` is where the four-family form of the same command closes, and
`firefox` needs `b_ids_driver::resolve` to know the family before anything else
can be attempted.

⚠ **And `captured.operator` is still typed.** The identity writer leaves it
empty and it was filled in by hand for this profile as it was for the three
before it. That is named in the writer's own comment and it is not this entry's
to fix.

---

## DRIVER-11. The launcher speaks Chromium, and the highest-value lane is not one

**Source** found while working `CORPUS-02`, 2026-09-04, once the resolver could name the family
**Category** driver, **Priority** P1, **Effort** L, **Status** open

### Problem

⛔ **`b_ids_driver::drive` passes Chromium switches and nothing else.** Firefox
resolves now, so `capture.yml` could name a `firefox` lane, and the lane would
launch Gecko with `--user-data-dir`, `--no-first-run` and `--headless=new`,
which Firefox reads as file names to open rather than as switches. The launcher
refuses the family instead, and that refusal is what stands between the corpus
and its first non-Chromium profile.

### Premise

⭐ **Measured on this host 2026-09-04, from `firefox --help` on 148.0.2**, not
carried from anywhere:

```text
  --profile <path>   Start with profile at <path>.
  --ProfileManager   Start with ProfileManager.
  --headless         Run without a GUI.
```

⛔ **And the whole of what the same output says about certificates:**

```text
Please read security guidelines at https://firefox-source-docs.mozilla.org/remote/Security.html
```

⚠ **That absence is the entry.** Two of the three switch groups map across
cleanly: `--user-data-dir=X` becomes `--profile X`, and `--headless=new` becomes
`--headless`. The third does not map at all. Chromium takes
`--ignore-certificate-errors` and
`--ignore-certificate-errors-spki-list=PIN` on the command line, and Firefox
offers no command-line equivalent for either, so a capture against this
project's own TLS terminator cannot be arranged by adding arguments.

⚠ **Believed**: that the trust has to be arranged in the profile instead,
through the NSS database Firefox creates there or through an enterprise policy
file beside the binary.

⛔ **One half of that is now measured, and it closes the obvious route.**
Seeding a profile's `cert9.db` needs NSS's `certutil`, and this host has none:
the `certutil` on `PATH` is Windows' own, a different program with a different
verb set, and Firefox ships `nss3.dll` with no `certutil` beside it. Measured
2026-09-04:

```text
$ command -v certutil
/c/Windows/system32/certutil
$ ls "/c/Program Files/Mozilla Firefox/" | grep -iE "certutil|nss"
nss3.dll
```

⚠ **So the three remaining routes each cost something, and none has been
tried**: an enterprise policy file beside the binary, which is a machine change
this project's trust-anchor ruling refuses; `security.enterprise_roots`, which
reads the operating system's own store and is the same machine change one step
away; and a `cert_override.txt` written into the throwaway profile, which is the
only one that stays inside the profile the driver already creates.
⭐ **The third is the one to try first**, and until it is tried this is a
belief. `docs/inherited-claims.md` is where a claim about any of them belongs.

### Approach

⭐ **A launch path per engine, chosen by `Family::is_chromium`**, which exists
and is what the current refusal reads.

- **The switch list becomes per engine.** ⛔ Data rather than a second arm of a
  case statement inside `drive`, for the reason `DRIVER-10` gives about the
  route table: adding an engine should be a table and a fixture.
- **The trust configuration is the entry's real work**, and it lands with the
  measurement that says which mechanism was used. ⛔ `captured.trust` records
  the configuration a profile was taken under, so a Gecko capture whose trust
  nobody can name is a profile this corpus must not publish.
  `TODO/harness.md`, `HARNESS-10`.
- ⚠ **A Gecko capture is compared against a Chromium one before it is
  believed.** The point of the lane is that the stack is different, so a run
  producing a hello that looks like Chromium's is a finding about the harness
  rather than a result.

⛔ Must not: reuse `--ignore-certificate-errors` by hoping Firefox ignores what
it does not know. Measured above: it takes the unknown argument as a file to
open, so the browser navigates somewhere nobody asked for and the capture is of
the wrong thing.

⛔ Must not: publish a profile from a lane whose trust configuration is not
recorded, which is the rule `HARNESS-10` exists to hold.

### Consumers

⚠ Nothing is published that this changes. A new family in the corpus adds
routes rather than moving existing ones, and `PUB-03` generates a route only
where the corpus holds a value.

### Prove

```bash
cargo test -p b-ids-driver --test resolve_and_drive
```

```bash
sh scripts/common/check-coverage.sh --require-rows chrome,edge,firefox
```

Passing means: the suite is green with a Gecko launch case that asserts the
switch list contains `--headless` and `--profile` and none of the Chromium
switches, and the coverage report carries a `firefox` row with at least one
profile whose `captured.trust` names the configuration it was taken under.
⛔ Read the exit code from the process that produced it, unpiped.

### ⛔ Ruled by the operator 2026-09-04: vendor an NSS `certutil`

⛔ **The trust is arranged the way Firefox itself expects**, by seeding the
throwaway profile's `cert9.db`, and the tooling to do it comes into the tree.
⚠ The recommendation attached to the question was `cert_override.txt`, and the
operator ruled against it: that file's format is version-dependent and
undocumented as a stable interface, so a lane built on it would break on a
Firefox release with nothing to point at.

⭐ **It also removes the machine change.** `policies.json` beside the binary and
`security.enterprise_roots` both need something installed on the host, which the
trust-anchor ruling refuses. A vendored `certutil` writes only into the profile
the driver already creates and removes.

⚠ **`captured.trust` still records which configuration the capture was taken
under**, and a Gecko capture whose trust nobody can name is a profile this
corpus must not publish.
