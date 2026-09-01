# SUMMARY.md

⚠ **The last session's table, and a snapshot rather than an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record and [`INDEX.md`](INDEX.md) is the
list; where this file disagrees with either, this one is stale.

⛔ Overwritten every session. Every cell is grounded in something a reader can
point at, including the cells that say nothing moved.

---

## 2026-09-01

Session ran 2026-09-01T00:50:02Z to its operator interrupt.

| entry | eff | what closed | acceptance |
| --- | --- | --- | --- |
| `HARNESS-03` | M | the HTTP/2 frame reader: preface, SETTINGS in arrival order, the WINDOW_UPDATE increment, PRIORITY frames, and the priority block as BYTES | `cargo test -p b-ids-harness http2`, 15 passed |
| `HARNESS-04` | M | HPACK, checked against a fetched corpus rather than against itself | `cargo test -p b-ids-harness hpack`, 15 passed over 47,142 cases |
| `HARNESS-06` | S | two types, in two crates: the parser keeps what the emitter refuses | `cargo test -p b-ids-harness grease_bodies`, 5 passed |
| `HARNESS-07` | S | the connection selection rule, over a thirteen-connection fixture | `cargo test -p b-ids-harness connection_selection`, 6 passed |
| `HARNESS-08` | S | eight handshakes by default, and a short run reports six rather than success | `cargo test -p b-ids-harness sampling`, 6 passed |
| `SCHEMA-06` | M | a raw block a profile can be rebuilt from, asserted rather than intended | `cargo test -p b-ids-schema raw_backstop`, 11 passed |
| **total** | **9 points** | six entries closed, one moved from two blocked switches to one | |

⚠ **Nine of the twenty points the quota asks for.** The session ended on an
operator interrupt, which [`RULES.md`](RULES.md) section 10 names as the other
way a session ends.

### What moved that is not an entry

| | |
| --- | --- |
| `HARNESS-02` | ⚠ still `partial`, and now eight of nine rather than seven. `--until-h2` is implemented; `--ca-out` is absent rather than inert. |
| `HARNESS-05` | still open, with the probe half done and the browser half's blocker named at file and step |
| `check-no-secrets` | gained `--scope`, which its own exemption had instructed sessions to use for one session before it existed. Both halves, with a `check-twins` row. |
| `references/` | a nineteenth tree: the HPACK vector corpus, at a named commit, measured at 26.9 MiB packed against a 100 MiB threshold |

### The three review lenses, and what each found that the others did not

| lens | swept | found |
| --- | --- | --- |
| ⭐ **door sweep** | every path into the credential rule, and every construction path that produces a recorded header | ⛔ **the fourth door, and it was open.** A capture drops `cookie` from its parsed fields and keeps it hex-encoded in the bytes beside them. `SCHEMA-06` had just routed those bytes into a published profile field. |
| **guard mutation** | seventeen guards, planted one at a time, each exit code read unpiped | ⭐ **two mutations reported NOTHING**, and both produced better findings than the fifteen that failed: a transcribed table nothing read, and a comparison never seen to fire |
| **claim audit** | every number in this table and in the record, re-derived from the tree | a test count written as 140 that measured 166, and ⚠ a probe process this session left running on the capture host |

⛔ **The door sweep is the one that mattered**, and it is the second session
running that this lens has found a credential path nobody enumerated.

### What the passes would have had to see to fire differently

The guard mutation reported on every guard, so it owes no such sentence. The
claim audit's two findings were both arithmetic against the tree; had the tree
agreed with the record it would have said so and named what it re-derived.

### The gate

```text
gate ok: 18 passed, but 1 SKIPPED on this host: check-twins
```

⚠ `check-twins` is skipped by `--fast` by definition and was run separately:
every pair agrees, including the new scoped row.
