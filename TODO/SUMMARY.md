# SUMMARY.md

⛔ **Overwritten every session, and it is a snapshot rather than an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record; this is the table that session
printed in chat.

**Session 2026-09-02T01:14:00Z to 2026-09-02T07:00:00Z, unattended, ended by
operator interrupt.**

---

## What moved

| | before | after | read from |
| --- | --- | --- | --- |
| entries done | 48 | **62** | `sh scripts/common/check-record.sh` |
| entries open | 43 | **29** | the same |
| effort points closed | - | **19** of 20 | five `M` at two, nine `S` at one |
| profiles in the corpus | 1 | **3** | `corpus=profiles:3 problems:0` |
| profiles from a machine nobody owns | 0 | **2** | `captured.operator` on each |
| gate checks | 24 | **26** | `gate ok: 26 passed` |
| twin pairs compared | 22 | **26** | `every twin pair agrees on this tree` |
| tests | 256 | **309** | `cargo test --workspace` |
| experiments in the tree | 2 | **6** | [`../experiments/README.md`](../experiments/README.md) |
| workflows | 3 | **5** | `workflows ok: 5 file(s), 9 job(s)` |
| inherited claims refuted | 4 | **5** | [`../docs/HISTORY/README.md`](../docs/HISTORY/README.md) |

---

## The fourteen entries that closed

| entry | eff | what it now does |
| --- | --- | --- |
| `SCHEMA-10` | M | `Shuffle::Observed` carries `distinct_orders`; the seed is ruled out of the profile |
| `SCHEMA-11` | S | `http.multipart_boundary`, as a pattern rather than a drawn value |
| `SCHEMA-13` | S | every integer in the published schema is bounded by its Rust width |
| `SCHEMA-14` | M | a credential is present, in its wire position, with no value |
| `VALID-03` | S | `unreachable_dimensions` over every dimension the corpus carries |
| `VALID-06` | S | `b-ids-validator diff`, field by field, with an uncontrolled-conditions warning |
| `DRIVER-04` | S | `40-trust-paths.sh`: which trust route completes a handshake, per platform |
| `HARNESS-14` | M | the pin against a real trust anchor on a disposable runner |
| `CI-02` | M | `check-staleness` and a scheduled `staleness.yml` |
| `CI-06` | M | `check-sources`: isolation, degradation, and a flagged disagreement |
| `CI-07` | S | `check-exit-codes`: 2 means could not run, on both halves of every pair |
| `CI-08` | S | `check-manual-path`, and a `# manual:` line on all nine jobs |
| `CORPUS-05` | S | the extension search, recorded and re-runnable |
| `DOC-01` | S | [`../docs/architecture.md`](../docs/architecture.md), the technical reference |

---

## ⭐ The measurements this session took

| question | answer | where |
| --- | --- | --- |
| does the capture matrix work on hosted runners | yes, four runs, every job green | `capture.yml` |
| do two runner images serve the same Chrome build | ⛔ no: `151.0.7922.173` against `151.0.7922.174` | the two lane logs |
| does refusing session tickets change the cold hello | no: 19 fields, 0 differing | `30-resumption-control.sh` |
| which trust route completes a handshake on Windows | pin and verification-disabled; ⛔ none with no flag at all | `40-trust-paths.sh` |
| does a real trust anchor change the hello on Linux | no: 19 fields, 0 differing | `50-trust-anchor.sh`, run `33592736694` |
| does Chrome on Linux read the user's NSS database | ⛔ yes, sometimes: 2 handshakes of 4. The inherited claim is refuted | the same run |
| is `0x12e0` in Chrome 151 | no, in none of three profiles; the origin's `152` capture has it | `60-identify-extension.sh` |
| what changed between Chrome `151.0.7922.76` and `.174` on `win64` | ⭐ only the version string | `b-ids-validator diff` |

---

## ⛔ What did not get done, named rather than implied

- **`CORPUS-02` stays open.** Two of its four required rows are captured. The
  `edge` lane is enabled and wired and has not produced a profile: Edge exited
  after 1.4 seconds having opened no connection, and `--log` now records what it
  says. `chromium` and `firefox` need the resolver to know them at all.
- **The twentieth effort point.** Nineteen were closed when the session ended.
- **Windows cannot exercise the trust-store route**, and why has not been read.
- **`EMIT-03` was read and not started.** `HARNESS-05` has unblocked it.
