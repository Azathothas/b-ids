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
**Category** driver, **Priority** P1, **Effort** M, **Status** open

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
**Category** driver, **Priority** P2, **Effort** S, **Status** open

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
sh experiments/10-trust-paths.sh
```

Passing means: the script reports, per platform available on this host, which
trust route completed a handshake, and the profile records which route produced
each capture.

---

## DRIVER-05. Acquisition, with more than one way to get a build

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** driver, **Priority** P2, **Effort** M, **Status** open

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

---

## DRIVER-06. Branded and unbranded builds are different products

**Source** the founding brief; the header sets are [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 6
**Category** driver, **Priority** P2, **Effort** M, **Status** open

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
