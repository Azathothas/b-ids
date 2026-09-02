# INDEX.md

Every entry, one line each, sorted by id. This is a **list**, not a log and not
an order. ⛔ The work order lives in [`PROGRESS.md`](PROGRESS.md) and nowhere
else.

⛔ **The counts below are checked, not typed.** `scripts/common/check-record.sh`
asserts that they agree with the rows, that every row has an entry and every
entry a row, and that no status disagrees between the two. It runs as a gate.

---

## Counts

```text
total 98  open 30  blocked 0  done 68
```

| priority | open | blocked | done | total |
| --- | --- | --- | --- | --- |
| P0 | 0 | 0 | 11 | 11 |
| P1 | 7 | 0 | 36 | 43 |
| P2 | 20 | 0 | 19 | 39 |
| P3 | 3 | 0 | 2 | 5 |
| **all** | **30** | **0** | **68** | **98** |

---

## Entries

| id | pri | eff | status | title | file |
| --- | --- | --- | --- | --- | --- |
| CI-01 | P1 | M | done | Every push: validate, with no network and no browser | [`ci.md`](ci.md) |
| CI-02 | P1 | M | done | Staleness is a schedule, not a push trigger | [`ci.md`](ci.md) |
| CI-03 | P1 | L | done | The capture matrix, fanned out, with every lane allowed to fail alone | [`ci.md`](ci.md) |
| CI-04 | P1 | M | open | A scheduled run that finds a change opens a pull request, not an issue | [`ci.md`](ci.md) |
| CI-05 | P2 | M | open | The cold-start job, which is the only thing that catches rot | [`ci.md`](ci.md) |
| CI-06 | P2 | M | done | No single source of any fact | [`ci.md`](ci.md) |
| CI-07 | P2 | S | done | Exit 2 means could not run, and it is not a failure | [`ci.md`](ci.md) |
| CI-08 | P2 | S | done | A documented manual path, for the day the provider is not there | [`ci.md`](ci.md) |
| CORPUS-01 | P1 | M | done | Content-addressed, append-only, never edited in place | [`corpus.md`](corpus.md) |
| CORPUS-02 | P1 | L | open | The capture matrix: browsers, channels and hosts | [`corpus.md`](corpus.md) |
| CORPUS-03 | P2 | S | done | `latest` means stable, and beta is how the project gets ahead | [`corpus.md`](corpus.md) |
| CORPUS-04 | P2 | M | open | Per-build trust-anchor lists, and a recommendation | [`corpus.md`](corpus.md) |
| CORPUS-05 | P3 | S | done | Name the unidentified extension | [`corpus.md`](corpus.md) |
| DOC-01 | P2 | S | done | There is no technical reference, and one document is pretending not to notice | [`docs.md`](docs.md) |
| DOC-02 | P2 | S | open | The operator's side is unwritten because there is nothing to operate | [`docs.md`](docs.md) |
| DOC-03 | P2 | S | open | There is no threat model, and publishing a corpus will need one | [`docs.md`](docs.md) |
| DOC-04 | P2 | S | done | The founding brief is retired, and this entry records what replaced it | [`docs.md`](docs.md) |
| DRIVER-01 | P1 | M | done | Resolve a browser, and drive it at a URL | [`driver.md`](driver.md) |
| DRIVER-02 | P1 | M | done | Read the version that is serving, not the one that is published | [`driver.md`](driver.md) |
| DRIVER-03 | P1 | S | done | Headless changes the User-Agent, and normalising it is reported | [`driver.md`](driver.md) |
| DRIVER-04 | P2 | S | done | The root store a browser actually reads | [`driver.md`](driver.md) |
| DRIVER-05 | P2 | M | done | Acquisition, with more than one way to get a build | [`driver.md`](driver.md) |
| DRIVER-06 | P2 | M | open | Branded and unbranded builds are different products | [`driver.md`](driver.md) |
| DRIVER-07 | P2 | S | done | The browser's own output is discarded, so a lane that captured nothing says nothing | [`driver.md`](driver.md) |
| DRIVER-08 | P0 | L | done | Purge the machine's browsers, install the build the cell names | [`driver.md`](driver.md) |
| DRIVER-09 | P1 | M | done | The most dangerous script in the tree is the one with no twin | [`driver.md`](driver.md) |
| DRIVER-10 | P1 | L | done | Provisioning is written for one family and the matrix names four | [`driver.md`](driver.md) |
| EMIT-01 | P2 | L | open | The support matrix, with the holes left in | [`emitters.md`](emitters.md) |
| EMIT-02 | P2 | L | open | The escape hatch, and where it has to live | [`emitters.md`](emitters.md) |
| EMIT-03 | P2 | S | open | The priority block patch, if the measurement says it is needed | [`emitters.md`](emitters.md) |
| EMIT-04 | P3 | M | open | Emitters for the stacks a consumer already uses | [`emitters.md`](emitters.md) |
| HARNESS-01 | P0 | L | done | The oracle is a server, not a client | [`harness.md`](harness.md) |
| HARNESS-02 | P0 | M | done | The switches, each of which exists because something went wrong without it | [`harness.md`](harness.md) |
| HARNESS-03 | P1 | M | done | Read HTTP/2 settings, the window update and the priority block | [`harness.md`](harness.md) |
| HARNESS-04 | P1 | M | done | Decode HPACK Huffman, because header order is behind it | [`harness.md`](harness.md) |
| HARNESS-05 | P1 | S | done | Settle the priority block, and do it first | [`harness.md`](harness.md) |
| HARNESS-06 | P1 | S | done | Parse permissively, emit exactly | [`harness.md`](harness.md) |
| HARNESS-07 | P1 | S | done | A browser opens sockets it abandons, and it resumes | [`harness.md`](harness.md) |
| HARNESS-08 | P1 | S | done | One handshake is not a sample | [`harness.md`](harness.md) |
| HARNESS-09 | P1 | M | done | Fuzz the parsers. A panic here is unacceptable | [`harness.md`](harness.md) |
| HARNESS-10 | P2 | S | done | Check whether measuring changed what was measured | [`harness.md`](harness.md) |
| HARNESS-11 | P2 | M | open | The p0f layer, which is free once the listener is ours | [`harness.md`](harness.md) |
| HARNESS-12 | P2 | L | open | A public capture oracle | [`harness.md`](harness.md) |
| HARNESS-13 | P1 | L | done | Terminate the handshake, and mint the authority that lets a browser complete it | [`harness.md`](harness.md) |
| HARNESS-14 | P2 | M | done | The pin against a real trust anchor, on a machine that is thrown away | [`harness.md`](harness.md) |
| HARNESS-15 | P0 | M | done | A cold hello is thrown away because its own connection carried no HTTP/2 | [`harness.md`](harness.md) |
| HARNESS-16 | P2 | S | open | The trust store a Windows runner can be made to use without a person | [`harness.md`](harness.md) |
| LIB-01 | P2 | M | open | A crate that hands a program a profile | [`library.md`](library.md) |
| LIB-02 | P2 | M | open | The smallest client that proves a profile is usable | [`library.md`](library.md) |
| LIB-03 | P3 | L | open | Bindings for the ecosystems that will ask | [`library.md`](library.md) |
| PUB-01 | P1 | M | open | Releases, tagged and versioned and immutable | [`publish.md`](publish.md) |
| PUB-02 | P1 | M | open | The data branch, over raw file serving | [`publish.md`](publish.md) |
| PUB-03 | P1 | M | open | Routes a program with nothing but `curl` can read | [`publish.md`](publish.md) |
| PUB-04 | P2 | M | open | The formats that are not data files | [`publish.md`](publish.md) |
| PUB-05 | P2 | L | open | Language packages that embed the corpus | [`publish.md`](publish.md) |
| PUB-06 | P3 | M | open | A packet capture per profile | [`publish.md`](publish.md) |
| PUB-07 | P1 | S | open | The licence stated in three places | [`publish.md`](publish.md) |
| PUB-08 | P2 | S | open | One generator for the release body and the changelog | [`publish.md`](publish.md) |
| PUB-09 | P2 | M | open | Signed and attested captures | [`publish.md`](publish.md) |
| SCHEMA-01 | P0 | M | done | The profile: one browser, one build, one platform, one channel, one instant | [`schema.md`](schema.md) |
| SCHEMA-02 | P0 | M | done | The TLS half, in wire order, with unknown codepoints kept | [`schema.md`](schema.md) |
| SCHEMA-03 | P0 | S | done | The HTTP/2 half, as an ordered frame sequence | [`schema.md`](schema.md) |
| SCHEMA-04 | P0 | S | done | The HTTP half, its variants, and the one privacy rule | [`schema.md`](schema.md) |
| SCHEMA-05 | P0 | S | done | Provenance is per field, with four kinds and no more | [`schema.md`](schema.md) |
| SCHEMA-06 | P1 | M | done | Record everything the wire carried, from the first commit | [`schema.md`](schema.md) |
| SCHEMA-07 | P1 | S | done | What must never be in the model | [`schema.md`](schema.md) |
| SCHEMA-08 | P1 | L | open | Every generated format, from one generator, round-tripped | [`schema.md`](schema.md) |
| SCHEMA-09 | P1 | S | done | Name every field for the wire, because three quantities have two units | [`schema.md`](schema.md) |
| SCHEMA-10 | P2 | M | done | Record the shuffle as a property, and consider recording its seed | [`schema.md`](schema.md) |
| SCHEMA-11 | P2 | S | done | The multipart boundary, which is a per-browser surface nobody listed | [`schema.md`](schema.md) |
| SCHEMA-12 | P2 | L | open | The six formats that need a decoder as well as an encoder | [`schema.md`](schema.md) |
| SCHEMA-13 | P1 | S | done | The published schema accepts 999 for a byte | [`schema.md`](schema.md) |
| SCHEMA-14 | P1 | M | done | A credential's presence is a fingerprint, and it is currently a hole | [`schema.md`](schema.md) |
| TOOL-01 | P1 | S | done | There is no toolchain, and the minimum version is measured rather than chosen | [`tooling.md`](tooling.md) |
| TOOL-02 | P1 | S | done | The gate has no suite in it, and says so in a comment | [`tooling.md`](tooling.md) |
| TOOL-03 | P1 | S | done | The secret check will refuse the raw captures | [`tooling.md`](tooling.md) |
| TOOL-04 | P2 | S | done | The reference fetcher stops when one of its two routes is down | [`tooling.md`](tooling.md) |
| TOOL-05 | P2 | S | done | One twin pair has no comparison at all | [`tooling.md`](tooling.md) |
| TOOL-06 | P2 | S | done | The route check does not exist and it is three lines | [`tooling.md`](tooling.md) |
| TOOL-07 | P3 | S | done | The gate's cost on a real host has never been measured | [`tooling.md`](tooling.md) |
| TOOL-08 | P1 | S | done | The gate's strict mode was documented and did not exist | [`tooling.md`](tooling.md) |
| TOOL-09 | P1 | S | done | The licence filler was documented and absent, and the documentation was the defect | [`tooling.md`](tooling.md) |
| TOOL-10 | P1 | S | done | A cited path is not checked, and that is how this tree broke | [`tooling.md`](tooling.md) |
| TOOL-11 | P1 | S | done | The banned-vocabulary rule was documented and unenforced | [`tooling.md`](tooling.md) |
| TOOL-12 | P0 | S | done | A mined tree brings its own ignore rules, and 92 files of the corpus were never committed | [`tooling.md`](tooling.md) |
| TOOL-13 | P1 | S | done | The Windows job skipped a lint and the workflow counted it as allowed | [`tooling.md`](tooling.md) |
| TOOL-14 | P1 | S | done | The changelog check read a heading level this repository does not use | [`tooling.md`](tooling.md) |
| TOOL-15 | P2 | M | done | The twin comparison costs a thousand seconds, and half of it is one row | [`tooling.md`](tooling.md) |
| TOOL-16 | P1 | S | done | A tree that moved under the comparison reads as a drift | [`tooling.md`](tooling.md) |
| TOOL-17 | P1 | S | done | The gate's line-endings filter cannot see the working tree | [`tooling.md`](tooling.md) |
| TOOL-18 | P1 | M | done | The gate is slow because of how it reads files, not because of what it reads | [`tooling.md`](tooling.md) |
| VALID-01 | P0 | M | done | The coherence checks, as a library and a command and a schema | [`validator.md`](validator.md) |
| VALID-02 | P1 | S | done | Run it over the prior art, and publish what it finds | [`validator.md`](validator.md) |
| VALID-03 | P2 | S | done | A family the resolver cannot produce is data nobody can reach | [`validator.md`](validator.md) |
| VALID-04 | P2 | M | open | Reference digest implementations, with published test vectors | [`validator.md`](validator.md) |
| VALID-05 | P2 | L | open | A conformance suite for impersonating clients | [`validator.md`](validator.md) |
| VALID-06 | P2 | S | done | Diffs between adjacent versions | [`validator.md`](validator.md) |
| VENDOR-01 | P1 | L | done | The vendored tree, and the four things that keep it honest | [`vendor.md`](vendor.md) |

---

## Priorities and effort

Defined once, here, and meant.

| priority | means |
| --- | --- |
| P0 | breaks correctness, loses data, or takes the process down |
| P1 | a documented capability does not work, or a flag does nothing |
| P2 | worth doing; nothing is wrong without it |
| P3 | worth recording so it is not rediscovered |

| effort | means |
| --- | --- |
| S | under a day |
| M | a few days |
| L | a week |
| XL | ⚠ almost always two entries pretending to be one |

## Status

`open`, `partial`, `blocked`, `done`. ⛔ There is no `wontfix` and no
`deferred`: a blocked entry stays open with the blocker named and what would
unblock it.

⚠ **Almost every entry here is `open`**, and that is honest rather than
alarming: no component has been built. The closed ones record where the founding
brief's content went and two defects in this repository's own tooling that were
found and fixed while it was being set up.

---

## ⭐ The argument behind the current ordering

Written down so a later session can re-derive it rather than re-argue it. ⚠ The
order itself lives in [`PROGRESS.md`](PROGRESS.md); this is the reasoning.

### Why the schema is first, and why it is P0 rather than P1

Every other component reads or writes a profile. A harness with no schema
produces something nobody can validate, and a validator with no schema has
nothing to check. Five schema entries are P0 because starting anything else
first means writing it twice.

⭐ **The provenance map is P0 for a different reason.** It is the one field that
cannot be retrofitted: adding it later means every profile captured before it
has no provenance and can never get one, because the capture is gone.

### Why the validator can be finished before any capture exists

It is pure logic over the model. ⭐ And `VALID-02` makes it publishable on day
one: running it over the prior art's own tables produces a real result with no
browser, and three violations are already located at file and line by the
sweep. That is a contribution the project can make before it has captured
anything.

### Why one measurement outranks a whole component

`HARNESS-05` is one capture. It confirms, here, a reading taken off frame bytes
in the origin repository, and until it is taken `SCHEMA-03` cannot say what the
priority field means and `EMIT-03` may not be built. ⭐ A measurement that
unblocks two entries is worth more than finishing either of them, and a
confirmation with a predicted answer is the cheapest kind there is.

### Why the tooling entries are P1 despite being small

`TOOL-01` blocks every acceptance command in the tree, because they all name a
workspace that does not exist. `TOOL-03` is worse than it looks: the first raw
capture committed will fail the gate, and the tempting fix removes a security
rule. Both are cheap and both are in the way.

### Why publishing is P1 before the corpus has more than one profile

Publishing is a contract. Contracts are cheap to establish and expensive to
change, and a consumer who pinned a route in week one has to keep working.
`PUB-03` in particular is the operator's own requirement and it has a measured
defect to avoid, so writing the check with the generator costs nothing.

### Why the emitters and the library are P2

They are the payoff and they depend on everything above. ⚠ `LIB-02` is worth
starting earlier than its priority suggests, because it is the only thing that
proves the corpus is **usable** rather than merely accurate, and the honest
expectation is that it will not match on the first attempt.
