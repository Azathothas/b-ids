# driver

Getting a browser onto a machine, working out which build it is, and pointing it
at the harness. Two jobs kept separate: **resolve** a browser, and **drive** it.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## DRIVER-01. Resolve a browser, and drive it at a URL

**Source** the founding brief; the driver shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 15
**Category** driver, **Priority** P1, **Effort** M, **Status** open

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
**Category** driver, **Priority** P1, **Effort** S, **Status** open

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
