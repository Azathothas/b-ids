# harness

The capture oracle. A listener a browser is pointed at, the parsers that read
what arrives, and the traps that cost somebody a day each.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## HARNESS-01. The oracle is a server, not a client

**Source** the founding brief; the harness shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 14
**Category** harness, **Priority** P0, **Effort** L, **Status** done

### Problem

There is no way to read a fingerprint in this repository. Asking a client what
it sent returns what it intended, which is a different thing and is the commonest
way a whole set of numbers turns out to describe nothing.

### Premise

Believed, and it is the design decision everything else rests on: the only
honest reading of a handshake is off the wire, from outside the client, so the
harness is a listener rather than a probe inside a browser.

### Approach

A TLS listener that accepts a connection, reads the record, parses the
`ClientHello`, and writes one JSON object per connection to standard output
after one line carrying the base URL a caller should point a browser at.

⭐ **Design it multi-protocol from the first commit even while implementing
one.** A TLS listener, an HTTP/1.1 listener, an HTTP/2 listener and later a QUIC
listener are four capture surfaces, and the model in `SCHEMA-01` already has
room for all four. Retrofitting the fourth into a TLS-shaped harness is a
rewrite rather than an addition.

The record-everything rule from `SCHEMA-06` applies from the first commit, not
from a later pass. Retrofitting completeness means re-capturing, and by then the
build is gone.

Must not: complete a handshake in order to read one, by default. Completing it
can change what the client offers, which is `HARNESS-11`.

### Prove

```bash
cargo test -p b-ids-harness listener -- --nocapture
```

Passing means: a committed fixture of raw bytes is fed to the listener over a
loopback socket and produces the profile committed beside it, byte for byte, with
no browser involved.

### Closing

**Closed 2026-08-31.** The oracle binds, accepts, reads and records, and the
acceptance runs over a real loopback socket with no browser involved.

```text
$ cargo test -p b-ids-harness listener -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
exit=0
```

### ⭐ The fixture is generated, not pasted

⛔ **The committed bytes are CONSTRUCTED and are not a capture**, and the file
that builds them says so in its own header. They are shaped like a Chromium
hello so the parser meets the shapes a real one has: GREASE at both ends, a
codepoint with no name, a trailing GREASE carrying one zero byte, a
supported-versions extension whose list length is one byte rather than two, and
an extension order that is not sorted.

```text
cargo run -p b-ids-harness --example make-fixture > crates/b-ids-harness/fixtures/client-hello.hex
```

⭐ **A reader can re-derive it**, and a change to the fixture is a change to a
readable file rather than to an opaque blob. ⛔ No value in it is a measurement
and none of it may enter the corpus.

### What the seven tests cover

| test | what it would catch |
| --- | --- |
| the golden comparison | any change to what a capture records, byte for byte |
| the raw bytes are kept | a parser defect taking the one artefact that survives every parser defect |
| a socket opened and abandoned | a run that drops the connections a browser abandons, and so under-reports what a navigation does |
| ⭐ a record split across two reads | a truncation report for a truncation that never happened |
| more than one connection, in order | a harness that can only ever see one, when one navigation has produced thirteen |
| a cleartext request | the multi-protocol seam, on the second surface |
| the base URL names the bound port | a caller with no way to learn the port before the accept blocks |

### ⛔ The parser cannot panic on input

Every read goes through a bounds-checked cursor that returns `None` at the end
rather than slicing past it. ⚠ A panic here is a denial of service in the one
component that faces the network, and `HARNESS-09` is the entry that fuzzes it
rather than trusting this sentence.

**Two rules the parser follows and a test holds:**

- ⛔ **Count what arrived; do not trust what was declared.** A declared record
  length that disagrees with the bytes becomes a NOTE on the capture, never a
  repair. Padding to match a declared length would record bytes nobody sent.
- ⛔ **Parse permissively, emit exactly.** Everything unreadable inside a
  well-framed hello becomes a note; only bytes that are not a `ClientHello`
  record at all are an error. A parser that refused a hello it did not
  recognise would have thrown the capture away.

### ⚠ The shuffle is UNKNOWN from one hello, never Fixed

One handshake is not a sample, so the parser records `Shuffle::Unknown` and
never claims a fixed order. `HARNESS-08` is what takes more than one draw.
⭐ `VALID-01` refuses a profile that claims a shuffle state from fewer than two
draws, so the rule is held at both ends.

### Mutation-proved

```text
=== HARNESS-01: a record split across two reads is reassembled ===
test listener_reassembles_a_record_split_across_two_reads ... FAILED
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

The completeness test was replaced with one that stops at the first read. Six
tests still passed and one refused, which is what says the failure is the
reassembly rather than the harness.

### ⚠ What is NOT here, named rather than left to be discovered

- ⛔ **HTTP/2 is not read**, because reaching it needs the handshake terminated
  and terminating one can change what the client offers. `HARNESS-03`.

  ⛔ **The reason above is half wrong, and `HARNESS-03` measured it.** Reaching
  HTTP/2 from a BROWSER needs the handshake terminated. Reaching HTTP/2 at all
  does not: a client with prior knowledge opens a cleartext connection with the
  preface, and every frame that carries the fingerprint arrives before the
  first response. The premise keeps its wording and the correction is written
  underneath it.
- **The digest expectations are not shipped.** The sweep asks for
  `--expect-ja4`, `--expect-ja3` and `--expect-akamai` in this change;
  computing them needs MD5 and SHA-256 implementations with published test
  vectors, which is `VALID-04`. ⭐ What DID ship is the half that needs no
  hashing and carries the same property: `--write-golden` and `--expect-file`
  compare the whole capture and exit 1 on a difference, so the harness is
  already a regression check rather than only a probe.

---

## HARNESS-02. The switches, each of which exists because something went wrong without it

**Source** the founding brief; the harness shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 14
**Category** harness, **Priority** P0, **Effort** M, **Status** done

### Problem

A harness with one mode can capture one thing. Every switch below is a capture
that is otherwise impossible or a value that is otherwise wrong.

### Premise

Inherited from a working implementation in another repository, described but
never read here.

### Approach

| switch | why it exists |
| --- | --- |
| `--raw` | do not terminate TLS. Completing a handshake can change what the client offers, so a digest read through a terminated handshake is not the digest it ships. |
| `--plain` | cleartext HTTP/1.1, header order only. The capture that works when a client cannot be told to trust anything. |
| `--ca-out PATH` | mint a certificate authority per run and write it, so a client can complete a **verified** handshake. Nothing has to disable verification to reach the HTTP/2 half. |
| `--bind ADDR` | reach a browser that is not on this machine. Refuses a hostname, and refuses the unspecified address by name. |
| `--hello-out PATH` | the raw `ClientHello` as hex. The one artefact that survives every hashing scheme and every parser defect. |
| `--header-values` | record values, not only names. Gated because it is the one switch that can log a credential, per `SCHEMA-04`. |
| `--until-h2` | stop at the first connection that reached HTTP/2, per `HARNESS-07`. |
| `--json` | one object per connection on standard output, after one line carrying the base URL |
| `--handshakes N` | how many connections to accept before exiting, per `HARNESS-08` |

Must not: make `--header-values` the default, or let `--bind` accept the
unspecified address without saying so, since that accepts the local network as
well.

### Prove

```bash
cargo test -p b-ids-harness switches -- --nocapture
```

Passing means: every switch is exercised, `--bind 0.0.0.0` is refused with a
message naming the reason, and the default run's output contains no header value.

### Closing

**Closed 2026-09-01T05:25:00Z.** All nine switches are implemented, exercised
and mutation-proved. ⚠ It was `partial` for one day with `--ca-out` blocked on
a TLS server this tree did not have; `VENDOR-01` vendored one and `HARNESS-13`
wired it, and the sixteenth test is the switch driving a real handshake.

```text
$ cargo test -p b-ids-harness switches -- --nocapture
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s
exit=0
```

| switch | state |
| --- | --- |
| `--raw` | done, and it is the DEFAULT rather than a mode somebody opts into |
| `--plain` | done, and it reads whichever cleartext protocol the peer spoke rather than HTTP/1.1 alone. `HARNESS-03`. |
| `--bind ADDR` | done, refusing a hostname and the unspecified address, each by name |
| `--hello-out PATH` | done |
| `--header-values` | done, off by default, and it does not lift the credential rule |
| `--json` | done, base URL first, then one object per connection |
| `--handshakes N` | done, with `--once` beside it |
| `--until-h2` | done 2026-09-01. `HARNESS-03` reached HTTP/2 over cleartext with prior knowledge, so there is one to stop at. |
| ⭐ `--ca-out PATH` | done 2026-09-01 by `HARNESS-13`, over the rustls `VENDOR-01` vendored. It mints an authority, writes it, and selects the terminated surface, which is the only one that reaches a browser HTTP/2. |

⭐ **While it was blocked the switch was ABSENT rather than present and inert**,
and the command refused it by name. That shape is worth keeping in the record
because it is the one that cost nothing to be wrong about: a flag that parsed
and did nothing would be the "setting or flag that no code reads" row in
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md),
and a session reading the usage would have believed the capability existed.

⚠ **The refusal is what turned into a hang when it stopped refusing.** Its test
asserted exit 2, and the working switch binds and waits instead.
`HARNESS-13`'s closing has that measurement and what replaced the test.

### ⭐ The tests drive the COMMAND, not the library

⛔ **A switch is a property of the command.** A test that called the library
would prove the library, and the shape this project has already been bitten by
is a flag documented, parsed, and never passed through. So the switch tests
spawn the compiled binary, read its base URL line, connect to the port it
printed, and read its exit code.

⚠ **They block on that first line rather than sleeping.** The port is not
knowable until the command says so, and a sleep long enough to be safe is a
sleep that makes the suite slow for no reason.

### The two halves of the acceptance, both asserted

```text
$ b-ids-harness --bind 0.0.0.0
b-ids-harness: --bind 0.0.0.0: is the unspecified address, which accepts the local network as well. Name the interface
```

And the default run over a request carrying a value and a credential records
neither the value nor the credential.

### ⛔ A defect in this entry's own test, found by mutating the guard

**The command-level refusal test would have HUNG rather than failed.** With the
unspecified-address refusal removed, the command bound successfully and blocked
on the accept, and the test waited on a process that was never going to exit.

⚠ **A guard whose test cannot fail loudly is a guard nobody knows works**, and a
hang is worse than a failure: it produces no message and no exit code, and in
continuous integration it consumes the job's whole timeout. The test now polls
with a deadline and kills the child, so a regression reports instead of waiting.

### Mutation-proved

```text
=== HARNESS-02: --bind refuses the unspecified address ===
test switches_bind_refuses_the_unspecified_address_by_name ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

=== HARNESS-02: the credential filter on the capture path ===
test switches_header_values_still_drops_a_credential ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.15s
```

⭐ **The second is the one worth noting.** The credential filter exists in the
MODEL, in `b-ids-schema`, and the capture path is a different door into the
same rule. Removing it from the listener alone was enough to leak a credential
into the command's output, which is the "control gated on one of several paths"
shape. Both doors are gated and both are tested.

### What closed this entry

A TLS server that can terminate a handshake, which `VENDOR-01` vendored and
`HARNESS-13` wired, at which point `--ca-out` mints the authority a client
verifies against.

⚠ **`--until-h2` was closed by `HARNESS-03` rather than by a terminated
handshake**, because reaching HTTP/2 at all did not need one. ⭐ Two of the
nine switches turned out not to need the thing they were thought to need, which
is the argument for taking the cheap route first and finding out.

---

## HARNESS-03. Read HTTP/2 settings, the window update and the priority block

**Source** the founding brief; the block is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5
**Category** harness, **Priority** P1, **Effort** M, **Status** done

### Problem

Half the fingerprint is above TLS. Without the HTTP/2 read there is no settings
order, no window increment, no pseudo-header order and no priority block.

### Premise

Read rather than measured here. Values and their sources are in
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 4.

### Approach

Read and record, in arrival order: the connection preface, every SETTINGS frame
with its entries in order, the connection-level WINDOW_UPDATE increment, any
standalone PRIORITY frames, and the first HEADERS frame including its flags
byte.

Derive the Akamai string from those, and store it in the derived block rather
than in the measured one.

Must not: record only the rendered Akamai string. It cannot distinguish an
absent priority block from an unread one, which is why two published readings of
that field are unusable and `HARNESS-05` reads the frame instead.

### Prove

```bash
cargo test -p b-ids-harness http2 -- --nocapture
```

Passing means: a fixture of frame bytes produces a profile whose settings list
is in arrival order, and a fixture that omits one settings key produces a
profile that records it as absent rather than as its default.

### Closing

**Closed 2026-09-01.** The reader takes the preface and every frame behind it,
and the cleartext surface reaches HTTP/2 with prior knowledge, so the acceptance
runs over a real loopback socket as well as over the frame bytes.

```text
$ cargo test -p b-ids-harness http2 -- --nocapture
running 15 tests
test http2_refuses_bytes_that_are_not_a_connection_preface ... ok
test http2_skips_the_pad_length_byte_before_the_priority_block ... ok
test http2_notes_a_truncated_frame_rather_than_padding_it ... ok
test http2_keeps_a_frame_type_it_has_no_name_for ... ok
test http2_records_a_standalone_priority_frame_separately_from_the_block ... ok
test http2_the_frame_list_is_the_arrival_sequence ... ok
test http2_reads_the_window_update_increment_and_never_the_window ... ok
test http2_reads_the_priority_block_as_bytes_and_reports_the_raw_five ... ok
test http2_records_an_absent_setting_as_absent_rather_than_as_its_default ... ok
test http2_reads_the_settings_in_the_order_they_arrived ... ok
test http2_the_akamai_string_is_derived_from_the_frames ... ok
test http2_a_cleartext_http1_request_is_still_read_as_http1 ... ok
test http2_reaches_the_listener_over_a_cleartext_socket ... ok
test http2_until_h2_stops_at_the_first_connection_that_reached_it ... ok
test http2_reassembles_a_connection_split_across_two_reads ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
exit=0
```

### ⭐ HTTP/2 was reached WITHOUT terminating TLS, and that is the finding

The entry sat behind `--ca-out` because reaching HTTP/2 from a browser needs a
terminated handshake. ⭐ **It does not need one to reach HTTP/2 at all.** A
client with prior knowledge opens a cleartext connection with the preface, and
every frame that carries the fingerprint arrives before the first response.

So the cleartext surface reads whichever protocol the peer actually spoke, and
⛔ **the bytes decide rather than a flag the operator passed.** A run does not
get to declare what its peer will send, and a capture that recorded an HTTP/2
connection as an unparseable HTTP/1.1 request would be recording the harness.

⚠ **What this does NOT reach is a browser.** No browser speaks cleartext
HTTP/2, so the browser half still needs the handshake terminated. That is what
`--ca-out` is for and it stays absent.

### The capture record is `harness-capture/2`

Two changes, and a version is part of the data rather than implied by the
reader:

- the `http2` half is a new sibling of `tls`;
- `plain_http1` became `cleartext`, because the surface is cleartext and the
  peer picks the protocol. A surface named for one protocol that can produce
  the other is a field that lies.

⚠ **It is `harness-capture/3` now, and `HARNESS-13` moved it.** Terminating a
handshake adds what that handshake negotiated, and a field added without a
version bump is a positional format that mis-reads silently.

### ⚠ The pad length byte comes before the priority block

⛔ **A reader that takes the five bytes straight after the frame head is right
on every unpadded frame and silently wrong on a padded one**, reporting a
dependency assembled from the pad length and four bytes of the real block.
RFC 9113 puts `Pad Length` first when `PADDED` is set, and the block after it.

⚠ The instrument this project inherited its predicted answer from reads the five
bytes at a fixed offset. That is correct for the browsers it was pointed at and
it is not a general rule, so this reader skips the pad byte and a test proves it
by sending a padded frame.

### What the fifteen tests cover

| test | what it would catch |
| --- | --- |
| the settings arrive in order | a reader that sorted, or that used a map. ⭐ The fixture sends 6, 1, 4, 2 because nothing sorts to that. |
| ⭐ an absent setting is absent | a default filled in for a key nobody sent, which is the one substitution the model exists to make impossible |
| the increment, never the window | the two units of one quantity, which one shipped database holds in a single field |
| the priority block as bytes, and the raw five | a rendered string standing in for a measurement |
| ⚠ the pad length byte is skipped | a dependency read out of the padding |
| a frame type with no name is kept | a sequence nobody can compare |
| a standalone PRIORITY frame is separate | two seams merged into one field |
| the frame list is the arrival sequence | any reordering, and a reserved bit nobody looked at |
| bytes that are not a preface are refused | the surface detection picking the wrong reader |
| a truncated frame is noted, not padded | a truncated frame recorded as complete |
| ⭐ a connection split across two reads | a completeness rule that stops at the preface's own blank line |
| the listener reads it over a socket | a library that works and a listener that does not |
| an HTTP/1.1 request is still HTTP/1.1 | the second reader taking the first one away |
| `--until-h2` stops at the first one | a run that keeps the abandoned preconnect |
| the Akamai string is derived | a digest stored beside the measurement it is derived from |

### Mutation-proved

⛔ Four guards, each mutated on its own and the failure read.

```text
=== HARNESS-03: the settings arrive in order, mutated to sort ===
assertion `left == right` failed: the arrival order is not preserved
  left: [1, 2, 4, 6]
 right: [6, 1, 4, 2]
test result: FAILED. 12 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

=== HARNESS-03: an absent setting is absent, mutated to fill in the default ===
test result: FAILED. 10 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

=== HARNESS-03: the pad length byte is skipped, mutated to ignore it ===
the pad byte was read as the block
test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

=== HARNESS-03: the preface is asked about FIRST, mutated to ask the blank line first ===
assertion `left == right` failed: the read stopped early
  left: 24
 right: 111
test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

### ⛔ The fourth mutation found a missing test rather than proving one

**The first attempt at it reported fifteen passing tests, and the mutation had
not been applied.** A quoting difference between the shell and the patch tool
left the file untouched, so the green result was the unmutated tree.

⭐ **Applying it properly showed the guard was genuinely untested**, for a
reason worth carrying: every existing test wrote its whole connection in one
call, and a message that arrives complete on the first read is complete
whatever the rule says. A completeness rule that stops too early passes every
single-write test and truncates every real client.
`http2_reassembles_a_connection_split_across_two_reads` was added for it, and
the mutation then reported 24 bytes read against 111.

⚠ **The same shape had already been paid for once**, on the TLS side, in
`HARNESS-01`. It was not carried across to the second protocol because the
second protocol's tests were written from the parser's side rather than the
socket's.

### What is NOT read here

- ⛔ **The header block is not decoded**, so `pseudo_header_order` is empty and
  the capture carries a note saying which entry decodes it. An empty list with
  no note would read as a client that sent no pseudo-headers. `HARNESS-04`.
- **A stream-level WINDOW_UPDATE is kept as the frame it is**, with its
  payload, rather than answering for the connection-level increment.
- **A SETTINGS acknowledgement is kept as the frame it is.** Recording it as a
  SETTINGS frame with an empty entry list would read as a client that sent no
  settings.

---

## HARNESS-04. Decode HPACK Huffman, because header order is behind it

**Source** the founding brief; the harness shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 14
**Category** harness, **Priority** P1, **Effort** M, **Status** done

### Problem

HTTP/2 header names arrive Huffman-coded. Without a decoder the harness cannot
read the header order, which is a first-class part of the fingerprint.

### Premise

Structural.

### Approach

A decoder for the static Huffman table, plus the dynamic table size updates,
enough to read names and, when the value switch is on, values.

Record **whether each header was Huffman coded**, per `SCHEMA-06`. That is a
choice the encoder made and it is part of the fingerprint.

⚠ **Test vectors exist and are not in this tree.** The reference corpus of HPACK
test cases is a separate upstream project, and it was deliberately deleted from
[`../references/hyperium__h2/`](../references/hyperium__h2/) during the sweep;
that project's `PROVENANCE.md` records where it came from. Fetching it is part
of this entry.

Must not: write a decoder without vectors and call it done. A Huffman decoder
that is subtly wrong produces plausible header names.

### Prove

```bash
cargo test -p b-ids-harness hpack -- --nocapture
```

Passing means: every case in the fetched vector corpus decodes to its expected
output, and the count of cases run is asserted, so a table that stopped early
cannot report green over a smaller suite.

### Closing

**Closed 2026-09-01.** The corpus was fetched, the decoder was written against
it, and header order is readable off a captured connection.

```text
$ cargo test -p b-ids-harness hpack -- --nocapture
running 15 tests
test hpack_records_a_dynamic_table_size_update ... ok
test hpack_evicts_from_the_dynamic_table_by_size_and_not_by_count ... ok
test hpack_refuses_a_size_update_above_what_the_peer_was_allowed ... ok
test hpack_records_which_indexing_form_the_encoder_chose ... ok
test hpack_reads_the_header_order_off_a_captured_connection ... ok
test hpack_fills_the_pseudo_header_order_from_the_recorded_fields ... ok
test hpack_refuses_an_index_that_names_nothing ... ok
test hpack_records_the_indexing_form_of_every_captured_field ... ok
test hpack_drops_a_credential_the_dynamic_table_had_to_see ... ok
test hpack_refuses_padding_longer_than_seven_bits ... ok
test hpack_records_whether_each_field_was_huffman_coded ... ok
test hpack_the_transcribed_table_is_a_canonical_huffman_code ... ok
test hpack_refuses_padding_that_is_not_the_end_of_string_code ... ok
test hpack_records_no_header_value_by_default ... ok
hpack: 47142 case(s) across 446 file(s)
test hpack_decodes_every_case_in_the_fetched_vector_corpus ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.69s
exit=0
```

### The corpus was fetched, and it is tracked

```text
sh scripts/common/mine-repo.sh http2jp/hpack-test-case --out references
```

[`../references/http2jp__hpack-test-case/`](../references/http2jp__hpack-test-case/)
at commit `8a1406e7d14bfcb6c046021f13cc15cfb162726d`, fetched 2026-09-01 through
`gh`, with no gaps reported. ⭐ **47,142 cases across 446 story files**, and
every one decodes.

⚠ **One directory of the corpus is skipped and the skip is asserted rather than
inferred.** It holds the header lists with no wire bytes, which is the
encoders' input rather than a decoder's. 478 files minus those 32 is the 446 the
test pins.

**The measurement that decided keeping it whole.** It adds 66 MB on disk and
6.0 MiB packed, taking the reference corpus from 22.7 MiB packed to 26.9 MiB.
[`RULES.md`](RULES.md) sets the threshold for re-taking that decision at about
100 MiB packed, so nothing was trimmed and no citation was invalidated.

### ⛔ The decoder does NOT filter credentials, and a test proves why

**The dynamic table has to see every field the encoder inserted or every later
index is wrong.** A decoder that dropped `cookie` before storing it would
resolve every subsequent indexed field against a table one entry short of the
encoder's, and produce a header set that is plausible and wrong.

⭐ **So the credential rule is applied where the capture is BUILT, which is the
third door into it**, beside the model's and the HTTP/1.1 reader's. The
committed fixture is built to separate the two requirements: it adds a `cookie`
field to the dynamic table and then sends an indexed field at the slot behind
it, so index 63 names the marker header if and only if the credential took slot
62.

⚠ **Mutating the decoder to filter, which is the tempting defence-in-depth fix,
produces a wrong capture rather than a safer one.** The proof is below.

### What is recorded that a header list alone would not carry

| what | why it is not droppable |
| --- | --- |
| ⭐ whether each name was Huffman-coded | a choice the encoder made, and it differs between clients |
| whether each value was Huffman-coded | the same, and the two are independent |
| ⚠ an ABSENT flag where a field came from an index | it was not coded at all, and `false` would say the encoder chose plain text |
| which of the four indexing forms was used | indexed, incremental, without indexing, never indexed are four visible choices |
| every dynamic table size update, in order | a choice about a table the encoder owns, and one a settings value does not predict |

### ⛔ A mutation found that the transcribed code table was not load-bearing

**The first attempt at mutating one row of the Huffman table changed nothing:
all 47,142 cases still decoded.**

⭐ **The reason is worth carrying.** Canonical decoding needs only the bit
length of each symbol and the order of the symbols within each length; the code
column is then derived. The construction here derived it from the counts, so a
transcription error that preserved the sort order was invisible, and the column
a reader would check against the specification was decoration.

Two changes came out of it:

- the first code at each bit length is now read from the **transcribed** table
  rather than derived from the counts, so a wrong row moves every symbol behind
  it;
- ⭐ `check_table_is_canonical` states the assumption the decoder rests on, out
  of the table rather than out of a comment, and a test asserts it. Canonical
  decoding is correct only where the codes at each length are consecutive and
  ascending in symbol order, and nothing in the tree said so.

⚠ **The mutation that reported nothing is the one that produced the finding.**
A guard that cannot fail is not evidence, and neither is a table nothing reads.

### Mutation-proved

```text
=== HARNESS-04: the Huffman table, mutated by one row ===
failures:
    hpack_the_transcribed_table_is_a_canonical_huffman_code
    hpack_records_whether_each_field_was_huffman_coded
    hpack_decodes_every_case_in_the_fetched_vector_corpus
the table is canonical: "symbol 58 is transcribed as 0x5d over 7 bit(s), and the canonical construction puts it at 0x5c over 7"
47142 case(s) ran, and these failed:
test result: FAILED. 12 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s

=== HARNESS-04: the credential filter at the HTTP/2 door, mutated away ===
failures:
    hpack_drops_a_credential_the_dynamic_table_had_to_see
    hpack_reads_the_header_order_off_a_captured_connection
    hpack_records_the_indexing_form_of_every_captured_field
test result: FAILED. 12 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.78s

=== HARNESS-04: the decoder filtering a credential out of the dynamic table ===
  left: [":method", ":authority", ":scheme", ":path", "x-fixture-marker", "user-agent", "x-fixture-marker", ":authority"]
 right: [":method", ":authority", ":scheme", ":path", "x-fixture-marker", "user-agent", "x-fixture-marker"]
  left: ":authority"
 right: "x-fixture-marker"
```

⭐ **The third is the one worth reading.** Filtering inside the decoder does not
merely fail a test: it shifts the dynamic table by one and turns the last field
of the request into a different header. That is a capture that is wrong rather
than a capture that is missing something.

### ⚠ What is NOT here

- **Encoding.** This decodes. An emitter that has to reproduce a captured
  header block is `EMIT-01`, and it needs the Huffman table in the other
  direction.
- **More than the first request.** A connection carries several, and the reader
  records the first header block and notes when more arrived. `SCHEMA-04`'s
  variants are how a navigation and a subresource fetch are told apart, and
  that is a driver question rather than a decoder one.
- **A fuzz target.** `HARNESS-09` owns it, and this decoder is now the fourth
  parser it has to cover.

---

## HARNESS-05. Settle the priority block, and do it first

**Source** [`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md) finding 1
**Category** harness, **Priority** P1, **Effort** S, **Status** done

### Problem

Whether a browser opens its first stream with a PRIORITY block inside the
HEADERS frame decides a whole emitter patch, and this project has not read a
byte of it. Three published readings disagree, and until one is taken here no
profile may carry the field.

### Premise

⭐ **Measured elsewhere, off frame bytes, on two versions.** [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5 has
the table. The origin repository's probe reads the HEADERS flags byte for
`0x20` and decodes the five bytes behind it, and reports exclusive, dependency
zero, weight 255 for both Chrome 151 and Chrome 152.

⚠ **The two sources reporting zero read a rendered Akamai string**, not frame
bytes, and one of them is a tool that could not write the block at the time it
recorded its own data. A zero in that field is what an unpatched stack emits.

⛔ **That made this a confirmation with a predicted answer, not an open
question, and it did not make the value publishable.** It was measured
somewhere else, which is `vendor` provenance, and the first rule is why the
capture still had to be taken here. ⭐ It has been: the closing has the reading
and the conditions.

### Approach

Read the HEADERS frame's flags byte for the `0x20` bit and, if it is set, the
five bytes after the frame head: one exclusive bit, a 31-bit dependency, and one
weight byte. Report the raw five bytes as well as the parsed values.

⭐ **Run a positive control in the same session.** A probe that finds nothing may
have been looking in the wrong place, and only a control separates the two. The
control is any client known to write a block; the patch that makes one is cited
in [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 4.

Run it against at least two browsers and two versions, so a disagreement with
the predicted answer is localisable to a version rather than to the probe.

⭐ **This is the first capture the harness should take**, because it is one
measurement, it is cheap, and it decides whether `EMIT-03` is work at all.

Must not: report the Akamai string as the answer. That is the artefact whose
ambiguity created the disagreement. ⛔ And must not skip the control because the
answer is predicted: a probe that agrees with an expectation it was told is the
one result that proves nothing.

### Prove

```bash
cargo run -p b-ids-harness -- --plain --json --handshakes 8
```

Passing means: the output records, per connection, whether the priority flag was
set and the five raw bytes when it was; the positive control shows the flag set;
and the result is written into [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5 with the browser, the build and
the date, beside the inherited reading and saying whether the two agree.

### Closing

**Closed 2026-09-01T05:35:00Z.** ⭐ **The block is there, on both browsers, on
every terminated connection, and the raw five bytes are `80000000ff`.** That is
the first quantity this project has read off a browser's own wire.

```text
$ ./target/debug/b-ids-harness --ca-out CA --handshakes 4 --run-timeout-ms 90000 --header-values
connection 2 from 127.0.0.1:56470, 1803 byte(s), TlsTerminated
  tls: 16 cipher suite(s), 17 extension(s), 2 GREASE slot(s)
  terminated: alpn Some("h2"), version Some("TLSv1_3"), suite Some("TLS13_AES_128_GCM_SHA256"), 534 plaintext byte(s)
  http2: 3 frame(s), settings [SettingEntry { id: 1, value: 65536 }, SettingEntry { id: 2, value: 0 }, SettingEntry { id: 4, value: 6291456 }, SettingEntry { id: 6, value: 262144 }], window increment Some(15663105)
  http2 priority block: Some(StreamPriority { exclusive: true, stream_dependency: 0, weight_wire: 255 }), raw Some("80000000ff")
sampling: 3 of 4 handshake(s) completed, 3 distinct GREASE draw(s), 3 distinct extension order(s)
exit=1
```

⚠ **Exit 1 is correct and it is not a failure of the measurement.** Three of
four handshakes completed, so the run reported a shortfall rather than success.
The fourth connection is a preconnect the browser abandoned.

### ⛔ The acceptance command in this entry was wrong, and the correction is here

It says `--plain`. That was written before this tree could terminate a
handshake, when the cleartext surface was the only one that reached HTTP/2 at
all. ⛔ **No browser speaks cleartext HTTP/2**, so that command can never take
this measurement from a browser. The command actually run is the one above,
and `--ca-out` is what makes it possible.

⚠ The title stays. The premise was right, the predicted answer was right, and
only the route to it changed.

### The two browsers, and they agree with each other and with the prediction

| browser | build | terminated | the five raw bytes |
| --- | --- | --- | --- |
| Chrome | `151.0.7922.76` | 6 of 7 accepted | `80000000ff` on every one |
| Edge | `152.0.4191.53` | 7 of 8 accepted | `80000000ff` on every one |

⭐ **Two browsers and two versions, which is what the approach asked for**, so a
disagreement would have been localisable to a version rather than to the probe.
There was none. The reading is written into
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5 with the
browser, the build, the date and the conditions, beside the inherited reading,
and it says the two agree.

### ⭐ The positive control, which is why a negative result would have meant something

The committed HTTP/2 fixture sets the flag and carries a block reading
exclusive, dependency zero, weight 255 on the wire, and
`http2_reads_the_priority_block_as_bytes_and_reports_the_raw_five` asserts it. A
probe that reported nothing over that fixture would be a probe looking in the
wrong place, and the suite refuses that.

⚠ **The control was NOT skipped because the answer was predicted.** A probe that
agrees with an expectation it was told is the one result that proves nothing,
and the approach says so in as many words.

### ⚠ What this does NOT settle

- **Whether `EMIT-03` is work.** It is: a client that cannot write the block
  cannot reproduce either browser measured here. That entry stays open with a
  measurement behind it now instead of a prediction.
- ⚠ **Whether the conditions changed the answer.** The authority was trusted
  through a per-launch flag rather than a root store. `HARNESS-10` is the
  entry that measures the difference, and it is now takeable.
- **Anything about a browser this host does not have.** Two browsers on one
  Windows 11 machine is what was measured, and `CORPUS-02` is the matrix.

### What was tried before it worked, kept because it saves the next session the walk

⛔ **The probe half was done a day before the browser half**, and separating
them is what let this close in one command once termination existed. The reader
takes the HEADERS flags byte, tests `0x20`, skips the pad length byte where
`PADDED` is set, and decodes the five bytes behind it into an exclusive bit, a
31-bit dependency and a wire weight. ⭐ It reports the parsed block AND the five
raw bytes, so nothing rests on a rendered string.

**Done, by `HARNESS-03`:** the reader takes the HEADERS flags byte, tests
`0x20`, skips the pad length byte where `PADDED` is set, and decodes the five
bytes behind it into an exclusive bit, a 31-bit dependency and a wire weight.
⭐ It reports the parsed block AND the five raw bytes, so nothing rests on a
rendered string. `Http2Capture::priority_block_hex` is the raw half and
`Http2Half::stream_priority` is the parsed one.

**The positive control exists**, as the committed HTTP/2 fixture: it sets the
flag and carries a block reading exclusive, dependency zero, weight 255 on the
wire. A probe that reported nothing over it would be a probe looking in the
wrong place, and the test refuses that.

**What opened it**, in the order it happened: `VENDOR-01` put a TLS server in
the tree, `HARNESS-13` wired it so `--ca-out` mints an authority and
terminates, and the browsers were launched by hand rather than by `DRIVER-01`,
which is still open and is what makes this repeatable rather than driven once.

---

## HARNESS-06. Parse permissively, emit exactly

**Source** the founding brief; the trap is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8
**Category** harness, **Priority** P1, **Effort** S, **Status** done

### Problem

A parser that maps a GREASE codepoint to a typed field with an empty body
rejects the GREASE extension that carries a byte, which is what a browser sends
at the end of its list. It passes locally, fails once in continuous integration,
and passes the next run.

### Premise

Measured elsewhere and inherited: three of the sixteen GREASE values failed that
way, so about one handshake in five.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8 carries
it.

### Approach

Two separate types, deliberately: the parser's model accepts any codepoint with
any body, and the emitter's model refuses anything it cannot reproduce exactly.
A codebase that uses one type for both gets one of them wrong.

Any field on a GREASE codepoint takes an arbitrary body, including a body that
is not empty.

Must not: share one type between the two. That is the whole entry.

### Prove

```bash
cargo test -p b-ids-harness grease_bodies -- --nocapture
```

Passing means: a fixture per GREASE value, each with a one-byte body, all parse;
and the emitter refuses a profile whose extension body it cannot reproduce,
rather than emitting an approximation.

### Closing

**Closed 2026-09-01.** Two types, in two crates, and the conversion between them
is fallible.

```text
$ cargo test -p b-ids-harness grease_bodies -- --nocapture
running 5 tests
test grease_bodies_the_emitter_refuses_a_body_it_cannot_reproduce ... ok
test grease_bodies_the_emitter_reports_every_refusal_and_not_only_the_first ... ok
test grease_bodies_the_parser_keeps_what_the_emitter_refuses ... ok
test grease_bodies_every_reserved_value_parses_with_a_one_byte_body ... ok
test grease_bodies_the_emitter_reproduces_every_body_byte_for_byte ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### ⭐ The two types, and what each one can represent that the other cannot

| | `b-ids-schema`'s `Extension` | `b-ids-emit`'s `EmittableExtension` |
| --- | --- | --- |
| the codepoint | any `u16`, including one learned at run time | the same |
| the body | any bytes, hex-encoded | any bytes |
| the length | ⭐ **a separate field**, recorded as the wire DECLARED it | ⛔ **absent**, derived from the body when it is written |
| a declared length that disagrees with the body | representable, and it is a finding worth keeping | ⛔ not representable at all |

⭐ **The difference is one field and it is the whole entry.** A parser has to
keep a disagreement between a declared length and a body, because that
disagreement is what a truncated or padded hello looks like and the capture
cannot be retaken. An emitter cannot put both numbers on the wire, so it has to
refuse. One shared type would satisfy exactly one of those two requirements.

⛔ **The emitter's length is derived at the moment of writing and nowhere else.**
A length that came from any other source is a length that can disagree with the
body it describes.

### The refusals, and why each is a real capture shape

| refusal | what produces it |
| --- | --- |
| the declared length disagrees with the body | a truncated or padded hello, which the parser records rather than repairs |
| the recorded body is not hex | a capture written by something other than this parser |
| the body is longer than a two-byte length | a capture whose body cannot be framed at all |

⚠ **Every refusal is reported, not the first one.** An emitter that stopped at
the first sends its author back for one more run per defect, and a capture
cannot be retaken.

### What the five tests cover

| test | what it would catch |
| --- | --- |
| ⭐ all sixteen reserved values with a one-byte body | the measured defect: a parser mapping GREASE to a typed field with an empty body rejected three of the sixteen, so about one handshake in five, and a test over ONE value would have passed four times in five |
| the emitter reproduces every body byte for byte | an approximation reaching the wire |
| the emitter refuses what it cannot reproduce | a lossy path back into an emitter |
| every refusal is reported | a run that names one defect per invocation |
| ⭐ the parser keeps what the emitter refuses | the two models collapsing into one, which is the entry itself |

### Mutation-proved

⛔ Both directions, because a shared type gets one of them wrong and the
question is which.

```text
=== HARNESS-06: the emitter refuses rather than approximates, mutated to approximate ===
failures:
    grease_bodies_the_emitter_refuses_a_body_it_cannot_reproduce
    grease_bodies_the_emitter_reports_every_refusal_and_not_only_the_first
    grease_bodies_the_parser_keeps_what_the_emitter_refuses
test result: FAILED. 2 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

=== HARNESS-06: a GREASE codepoint takes an arbitrary body, mutated to force it empty ===
failures:
    grease_bodies_every_reserved_value_parses_with_a_one_byte_body
    grease_bodies_the_emitter_reproduces_every_body_byte_for_byte
test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### ⚠ The dependency runs one way and it is a DEV dependency

`b-ids-harness` dev-depends on `b-ids-emit`, because this entry's acceptance
needs both models in one test. ⛔ The harness itself does not depend on the
emitter and must not: a capture tool that imported an emitter would be one
component with two jobs, and the schema is the seam between them.

---

## HARNESS-07. A browser opens sockets it abandons, and it resumes

**Source** the founding brief; the trap is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8
**Category** harness, **Priority** P1, **Effort** S, **Status** done

### Problem

One navigation produces many connections, and neither the first nor the last is
the one to keep. A harness that takes either is recording something other than a
cold handshake.

### Premise

Measured elsewhere and inherited: driving one browser at a probe produced
thirteen connections from one navigation. The first carried no HTTP/2 at all, a
preconnect the browser abandoned. Every one after the second offered a
pre-shared key instead of a session ticket, because the session resumed, and the
two produce different digests.

### Approach

Keep the **first connection that completed HTTP/2**. That is the cold handshake
and it is what a fresh client sends. Record a resumed connection separately, as
its own profile with its own provenance, and never average the two or assert
they are equal.

`--until-h2` is the switch, per `HARNESS-02`.

Must not: deduplicate connections by digest. Two connections that differ are the
data.

### Prove

```bash
cargo test -p b-ids-harness connection_selection -- --nocapture
```

Passing means: a fixture of a thirteen-connection navigation selects connection
two, and the resumed connections are emitted as a separate labelled set whose
digest differs from the cold one.

### Closing

**Closed 2026-09-01.** The selection rule is code, not a sentence, and the
thirteen-connection shape is a committed fixture the rule is exercised over.

```text
$ cargo test -p b-ids-harness connection_selection -- --nocapture
running 6 tests
test connection_selection_classifies_a_connection_that_sent_nothing_as_abandoned ... ok
test connection_selection_the_resumed_handshake_differs_from_the_cold_one ... ok
test connection_selection_never_deduplicates_two_connections_that_agree ... ok
test connection_selection_keeps_the_first_connection_that_reached_http2 ... ok
test connection_selection_records_the_resumed_ones_as_their_own_set ... ok
test connection_selection_says_when_resumption_is_not_observable ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
exit=0
```

### Four sets, not two

| set | what it holds |
| --- | --- |
| `cold` | ⭐ the FIRST connection that reached HTTP/2, which is the one a profile is built from |
| `resumed` | every connection that reached HTTP/2 and offered a pre-shared key |
| `additional_cold` | ⚠ a LATER connection that reached HTTP/2 without resuming |
| `abandoned` | every connection that never reached HTTP/2 |

⚠ **The third set exists because folding it into either of the others would be
a label the bytes contradict.** A second cold handshake is not a resumed one,
and it is not the one the profile is built from either. Two sets would have
forced a wrong answer for it.

### ⛔ A capture with no hello cannot be asked whether it resumed

`resumption_observable` is a field on the selection rather than an assumption in
the code. A cleartext capture carries no `ClientHello`, so the cold-against-
resumed split says nothing about resumption on such a run and the selection
says so. ⛔ An unavailable field is absent with a reason, never assumed to be
the safe value.

### ⚠ "Digest" in the acceptance is read as the RAW HELLO, and that is stronger

No digest implementation exists yet: `VALID-04` owns MD5 and SHA-256 with
published test vectors. The comparison the acceptance asks for is made over the
raw hello bytes instead.

⭐ **That is a stronger discriminator rather than a weaker one.** Every digest
strips GREASE and most sort before hashing, so two hellos that genuinely differ
can share a digest. `T-263` in the origin tree records one client and one
browser producing the SAME JA4 over two different wire orders. No two differing
hellos share their bytes.

### The fixture is CONSTRUCTED and it needs termination to arise for real

⛔ **No value in it is a measurement.** It rebuilds the SHAPE of a reading that
is inherited: thirteen connections from one navigation, the first carrying no
HTTP/2 at all, every one after the second offering a pre-shared key.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8 carries
that reading and its source.

⚠ **Only a terminated connection carries both a `ClientHello` and HTTP/2
frames**, so this shape cannot arise from a browser until `--ca-out` works. The
shape is representable today and the rule is written against it now, so the
first terminated capture is selected by a tested rule rather than by eye.

### Mutation-proved

```text
=== HARNESS-07: the cold handshake is the first that reached HTTP/2, mutated to the first socket ===
failures:
    connection_selection_classifies_a_connection_that_sent_nothing_as_abandoned
    connection_selection_keeps_the_first_connection_that_reached_http2
    connection_selection_records_the_resumed_ones_as_their_own_set
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

=== HARNESS-07: two connections that differ are the data, mutated to deduplicate ===
failures:
    connection_selection_keeps_the_first_connection_that_reached_http2
    connection_selection_never_deduplicates_two_connections_that_agree
    connection_selection_records_the_resumed_ones_as_their_own_set
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

⭐ **The second is the one worth reading.** Deduplicating by digest is the
tempting tidy-up and it destroys a measurement: how many connections a
navigation opens is itself data, and eleven identical resumed handshakes are
eleven observations rather than one.

---

## HARNESS-08. One handshake is not a sample

**Source** the founding brief; the trap is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8
**Category** harness, **Priority** P1, **Effort** S, **Status** done

### Problem

Anything drawn per connection means a single handshake tests a single draw. A
defect that fires on three values in sixteen reaches a one-handshake check four
times in five, and passes.

### Premise

Arithmetic over the measured GREASE behaviour in
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 2.

### Approach

Make the handshake count a parameter, default it to eight, and assert that every
one of them completed. A run where six of eight completed is a run that reports
six, not a run that reports success.

Report the per-draw variation: which values were drawn, and how many distinct
orders were seen. That is what `SCHEMA-10` records.

Must not: default to one, and must not report a pass when fewer handshakes
completed than were asked for.

### Prove

```bash
cargo test -p b-ids-harness sampling -- --nocapture
```

Passing means: a run configured for eight handshakes where the fixture supplies
six exits non-zero with a message naming both numbers.

### Closing

**Closed 2026-09-01.** The default is eight, a run reports what it drew, and a
run that did not get what it asked for exits 1 naming both numbers.

```text
$ cargo test -p b-ids-harness sampling -- --nocapture
running 6 tests
test sampling_the_default_is_eight_handshakes_and_never_one ... ok
test sampling_an_accepted_connection_is_not_a_completed_one ... ok
test sampling_reports_the_per_draw_variation ... ok
test sampling_a_run_that_completed_every_handshake_exits_zero ... ok
test sampling_a_run_with_a_deadline_ends_rather_than_waiting_forever ... ok
test sampling_a_run_that_completed_six_of_eight_says_six_and_eight ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.57s
exit=0
```

### ⭐ Accepted is not completed, and the two are counted separately

A browser opens sockets it abandons. They are accepted, they are recorded, and
they are useless as a draw. A run that counted them would report a sample larger
than the one it took, which is the "hardcoded or synthetic metric" defect in the
component whose whole job is measuring.

```text
b-ids-harness: 6 of 8 handshake(s) completed, from 6 accepted connection(s)
```

⚠ **Both numbers, and the accepted count beside them.** "Some handshakes
failed" is a sentence nobody can act on, and six-completed-of-six-accepted is a
different diagnosis from six-completed-of-eight-accepted: the first says the
subject failed and the second says the run ran out of time.

### ⛔ A run needed a deadline before six of eight could even be reported

**Without one, a run configured for eight that gets six does not report six: it
blocks on the seventh accept forever.** A hang has no message and no exit code,
and in continuous integration it consumes the job's whole timeout and reports
nothing about what went wrong. That is the same shape `HARNESS-02` already paid
for in a refusal test.

`--run-timeout-ms` is the switch. ⛔ It is absent by default, because a run
driving a browser wants to block until the browser connects, and a deadline
somebody has to remember to turn off is a deadline that will one day truncate a
real capture.

⚠ **A stream accepted from a non-blocking listener inherits that mode on some
platforms**, and a non-blocking read returns immediately, which the reader would
record as a peer that sent nothing. The accept puts it back to blocking, once,
rather than each reader remembering to.

### What the run reports about the draw itself

```text
sampling: 3 of 3 handshake(s) completed, 3 distinct GREASE draw(s), 3 distinct extension order(s)
```

⭐ **Two consecutive captures of one binary must produce two different draws, or
the capture is wrong.** A run that reported only a count could not say whether
it had seen one behaviour eight times or eight behaviours once, and that is the
difference between a sample and a repetition.

⚠ **The values drawn and the order they arrive in are counted separately.** One
distinct order across eight handshakes is not proof of a fixed order: it is also
what a shuffle whose input list is exhaustive produces, which is a defect
measured elsewhere and one that reports nothing on its own. `SCHEMA-10` is where
the property is recorded.

### ⚠ Every test that feeds fewer than eight connections now says so

The default moved from one to eight, so twenty call sites that passed
`Config::default()` and fed one connection would have waited for seven more.
They say `one_connection()` now, in one place, which is what stops the next test
from quietly waiting on connections nobody is going to make.

### Mutation-proved

```text
=== HARNESS-08: a short run exits non-zero, mutated to report success ===
failures:
    sampling_a_run_that_completed_six_of_eight_says_six_and_eight
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.64s

=== HARNESS-08: the default is eight, mutated back to one ===
failures:
    sampling_the_default_is_eight_handshakes_and_never_one
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.64s

=== HARNESS-08: accepted is not completed, mutated to count accepts ===
failures:
    sampling_an_accepted_connection_is_not_a_completed_one
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.65s
```

---

## HARNESS-09. Fuzz the parsers. A panic here is unacceptable

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** harness, **Priority** P1, **Effort** M, **Status** done

### Problem

The harness reads bytes chosen by whoever connects to it. A parser that panics
on a truncated or hostile record is a parser that can be stopped by anybody who
can reach the listener.

### Premise

Structural.

### Approach

A fuzz target per parser: the record layer, the `ClientHello`, the HTTP/2 frame
reader, and the Huffman decoder. Each must return an error and never panic, on
any input.

Every parser returns an option or a result rather than panicking, including on a
length field that claims more bytes than arrived. Trusting a declared length
instead of counting what arrived is its own defect class.

Must not: treat a fuzz finding as a corpus entry. A crash input is a test case;
it is not a capture.

### Prove

```bash
cargo fuzz run client_hello -- -runs=1000000
```

Passing means: every target runs its budget with no crash and no timeout, and
each corpus seed is committed so the run is reproducible.

### Closing

**Closed 2026-09-01.** ⭐ **One million runs, no crash and no timeout**, over
every parser this crate exposes to the network.

```text
$ RUSTUP_TOOLCHAIN=nightly cargo fuzz run parsers -- -runs=1000000
rustc 1.100.0-nightly (0dfb098f3 2026-08-31)
###### Recommended dictionary. ######
"PRI * HTTP/2.0\015\012\015\012S\000\000\000\000\000" # Uses: 108
"\357\000\000c\026\001\000/\377\377\377\000\000\377\377\377\377\377\377\377\377\377\377\377" # Uses: 32
"P\377\021\015\012\015\012SM0\012\004\000\034\000\004\015\012\012\000\000\006\000\001" # Uses: 35
###### End of recommended dictionary. ######
Done 1000000 runs in 295 second(s)
```

⭐ **The dictionary is the evidence that it was exploring rather than bouncing
off a length check.** libFuzzer discovered the HTTP/2 connection preface on its
own, which it can only do by reaching the frame reader behind it.

```text
$ cargo test -p b-ids-harness hostile
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
```

#### ⭐ Two halves, because a fuzz target nobody can run is a target nobody runs

| | |
| --- | --- |
| `fuzz/fuzz_targets/parsers.rs` | coverage-guided, one million runs, ⚠ needs a nightly toolchain and a platform that can link libFuzzer |
| `crates/b-ids-harness/tests/hostile.rs` | a deterministic corpus of **6767** mutations, ⭐ on every host and every push, in 0.44s |

⛔ **Both call the same function.** `b_ids_harness::fuzz::drive_every_parser` is
the list of parsers, and it lives in the library rather than in either caller. A
target per parser would be four lists to keep in step with a crate that grows
one, and the day a fifth parser lands the four would keep passing while covering
less.

⚠ **The corpus is mutations of the committed captures, not random bytes.**
Random input almost never survives the first length check. Every prefix of every
committed fixture is in it, which is the highest-value mutation available here:
every parser in this crate reads a declared length and then takes bytes behind
it, and a truncation is what puts those two in conflict.

#### ⛔ Mutation-proved, and the exit code read unpiped

A panic planted in the Huffman decoder, on an input length nothing should care
about:

```text
$ cargo test -p b-ids-harness --test hostile
thread 'hostile_no_parser_panics_on_any_mutation_of_a_real_capture' panicked at crates\b-ids-harness\src\hpack.rs:473:5:
PLANTED: a length the parser should not care about
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
rc=101
```

⭐ **Five tests still passed and one refused**, which is what says the failure is
the planted panic rather than the harness. Reverted, the suite is green again at
6 passed.

#### ⛔ Windows cannot run the coverage-guided half, and three routes were measured

⚠ **Each failed differently, and the reasons are kept so the next session does
not walk them again.** [`../fuzz/README.md`](../fuzz/README.md) carries the
table; the short form:

| route | what stopped it |
| --- | --- |
| MSVC, default sanitizer | `LNK1104: cannot open file 'clang_rt.asan_dynamic_runtime_thunk-x86_64.lib'`. ⭐ The one route with a named, operator-actionable fix: the AddressSanitizer runtime is an optional Visual Studio Build Tools component. |
| MSVC, `-s none` | `LNK2001: unresolved external symbol __stop___sancov_pcs`. `link.exe` provides no section-boundary symbols for the sections libFuzzer reads its counters from. |
| GNU nightly | libFuzzer's own `FuzzerExtFunctionsWindows.cpp` does not compile under mingw `g++`. Its Windows support is written for MSVC. |

⭐ **The fourth route is a Linux container and it is what produced the run
above.** [`../docs/containers.md`](../docs/containers.md) is the procedure, and
it was followed: the platform was named on the pull, the image was removed
afterwards, and the engine was left stopped exactly as it was found.

#### ⚠ The trap that cost a whole run, and it will cost a CI job the same way

[`../rust-toolchain.toml`](../rust-toolchain.toml) pins this tree to an exact
stable compiler, and that file applies to every directory under the repository
root. A nightly IMAGE is not enough: rustup reads the toolchain file, downloads
the pinned stable, and then `-Z sanitizer` is refused as a nightly-only option.

⛔ **Anything that runs this overrides the toolchain explicitly**, which is why
the command above carries `RUSTUP_TOOLCHAIN=nightly`. `CI-03` is the entry that
will hit this on a runner.

#### The design, and the three refusals in it

| | |
| --- | --- |
| ⛔ nothing is asserted about the result | The property is the absence of a panic. An assertion about the value would make this a test of the parse rather than of the process surviving what arrives on a socket. |
| ⛔ a fresh decoder per input | The HPACK dynamic table is connection state. A decoder carried across inputs would make every case depend on the ones before it, so a crash would not be reproducible from its own input. |
| ⛔ the corpus is not committed | One run produced 856 files, a different set on every host, regenerated in minutes. The SEEDS are committed and they are the harness's own captures. A crash input is a regression test rather than a file to keep in a fuzz directory, which is this entry's own "must not". |

#### ⚠ What this does not cover

| | |
| --- | --- |
| the certificate minter and the TLS terminator | They are `rcgen` and the vendored rustls, whose own suites cover them. This crate's parsers are what face an unknown client before any of that runs. |
| a hang | libFuzzer's `-timeout` catches one and none fired, but a decoder walked into a slow loop by a padding run is a shape a crash-watcher can miss. `hostile_a_huffman_literal_of_all_ones_terminates` is the bounded test that would see it. |
| a run in continuous integration | `CI-01` and `CI-03`. The command and the toolchain trap are written down for them. |

---

## HARNESS-10. Check whether measuring changed what was measured

**Source** the founding brief; [`../docs/methodology/experiments.md`](../docs/methodology/experiments.md)
**Category** harness, **Priority** P2, **Effort** S, **Status** done

### Problem

An instrument that has to relax something in order to see anything may have
changed what it is watching. A value captured that way is not the value the
subject ships.

### Premise

Inherited: in a related sweep, disabling certificate verification so a probe
could terminate a connection also changed the client's advertised algorithms.
The answer there was to capture that one field passively instead.

### Approach

For every field the harness records, establish which capture mode produced it,
and compare the raw mode against the terminating mode on the same browser and
build. Where they differ, the field is captured in the mode that does not
perturb it, and the reason is recorded beside the field rather than in a commit
message.

⭐ Ask this of every instrument this project builds. It is not an exotic case.

Must not: publish a field measured in a mode that changes it, without saying so.

### Prove

```bash
sh experiments/20-compare-capture-modes.sh
```

Passing means: the script captures the same browser in raw and terminating modes
and prints a field-level diff, and every differing field is named in the profile
schema's documentation with the mode that owns it.

### Closing

**Closed 2026-09-01.** ⭐ **Terminating the handshake did not change what the
browser offered before it.** Seventeen of nineteen compared fields agree exactly
across the two surfaces; none differ; two carry a per-connection draw and are
reported as not comparable rather than as findings.

```text
$ sh experiments/20-compare-capture-modes.sh
raw:        0 cold, 0 resumed, 18 abandoned
terminated: 4 cold, 11 resumed, 3 abandoned
⚠ the two surfaces produced different numbers of resumed connections, which is a mode effect on the RUN even where every field of the cold hello agrees

comparing 18 raw hello(s) against 7 terminated, resumed connections excluded
  agrees        tls.record_version  0x0301
  agrees        tls.legacy_version  0x0303
  agrees        tls.session_id_len  32
  not comparable tls.cipher_suites
      raw         13 distinct value(s) within the run
      terminated  6 distinct value(s) within the run
  agrees        tls.cipher_suites.no_grease  0x1301,0x1302,0x1303,0xc02b,0xc02f,0xc02c,0xc030,0xcca9,0xcca8,0xc013,0xc014,0x009c,0x009d,0x002f,0x0035
  agrees        tls.compression_methods  0
  not comparable tls.extensions.order
      raw         18 distinct value(s) within the run
      terminated  7 distinct value(s) within the run
  agrees        tls.extensions.set.no_grease  0x0005,0x000a,0x000b,0x000d,0x0010,0x0012,0x0017,0x001b,0x0023,0x002b,0x002d,0x0033,0x44cd,0xfe0d,0xff01
  agrees        tls.extensions.count  17
  agrees        tls.key_exchange_groups.no_grease  0x11ec,0x001d,0x0017,0x0018
  agrees        tls.key_shares.groups.no_grease  0x11ec,0x001d
  agrees        tls.key_shares.lengths  1216,32
  agrees        tls.signature_algorithms  0x0904,0x0905,0x0906,0x0403,0x0804,0x0401,0x0503,0x0805,0x0501,0x0806,0x0601
  agrees        tls.signature_algorithms_cert  absent
  agrees        tls.alpn  h2,http/1.1
  agrees        tls.ech  0/0x0001
  agrees        tls.padding_len  absent
  agrees        tls.grease.count  2
  agrees        tls.grease.positions  0,16

only the terminating surface can see: http2, http, raw.http2_frames_hex, raw.connection_hex
modes=agree differing:0 not_comparable:2 fields:19
```

```text
$ cargo test -p b-ids-harness modes
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

⚠ **The conditions, because a measurement carries them or it is not a
measurement.** One Windows 11 host (`10.0.26200.9168`, `x86_64`), Chrome
`151.0.7922.76` headful, 2026-09-01, three rounds of one raw run and one
terminating run each, six connections requested per run, every launch into a
fresh throwaway profile. ⛔ One machine on one day, one browser, one build.

#### ⛔ The two fields that do not differ and cannot be shown to agree either

`tls.cipher_suites` and `tls.extensions.order` both carry a value the browser
draws per connection: a GREASE codepoint at the head of the cipher list, and a
shuffled extension order. Eighteen raw connections produced eighteen distinct
orders.

⭐ **A field with no single value per mode has nothing for a mode to have
changed**, so more connections do not fix it. What does is comparing the same
quantity with the drawn part removed, which is why `tls.cipher_suites.no_grease`
and `tls.extensions.set.no_grease` are separate fields: both are stable in both
modes and both agree.

⚠ **So the honest scope of the result is: every TLS field that has a stable
value at all is unchanged by terminating the handshake.** The two that do not
have one are stated rather than counted as agreement.

#### ⭐ The second finding, which is a mode effect and is not a field

**Only a surface that completes a handshake can produce a resumption.** The raw
run resumed nothing, of eighteen connections; the terminating run resumed eleven
of eighteen. That is not a difference in what the browser offers on a cold
hello, and it is not noise either: it changes the *mix* of connections a run
produces, and a corpus built by averaging them would publish a handshake nothing
sent.

⛔ **It is why `b_ids_harness::comparable` exists.** The corpus already keeps the
cold connection and records a resumed one separately; this comparison now
follows the same rule, and the counts are printed above the field list rather
than folded into it.

#### ⚠ Two defects the driven pass found in the comparison itself

⛔ **Neither would have been found by the suite**, and both were found by running
the real thing, which is what gate part (b) is for.

| what happened | the fix |
| --- | --- |
| The first run handed every connection of each side to the comparison and reported the extension SET as not comparable. That was resumption, not a mode effect: the terminating run held two different sets because some of its connections resumed. | The caller selects. `comparable` keeps the connections that offered no pre-shared key, and `resumption_split` reports the counts beside the comparison. |
| A single terminating run produced **0 cold connections and 5 resumed**, so the comparison rested on one hello and said so through its own `thin` warning. More connections cannot fix that: the first connection of a navigation gets a ticket and every later one resumes. | The experiment runs several rounds, each with a fresh throwaway profile, because a cold hello is sampled per RUN rather than per connection. Three rounds produced seven comparable terminated hellos. |

⚠ **And both helpers were reading the wrong thing at first.** They routed
through `select`, which answers a question about ONE navigation, so across
concatenated runs the repeating connection numbers would have excluded the wrong
captures. They ask each capture directly now, through `select::kind` and
`select::offers_pre_shared_key`, which have no navigation assumption.

#### ⚠ What this did NOT measure, and the entry's own framing was wider than this

⛔ **The record's work order said this entry would measure a per-launch key pin
against a real trust anchor**, and the entry's own Approach and acceptance ask
for something narrower: the raw mode against the terminating mode. This closed
against the entry, which is the specification.

The trust-store question is real and remains open, and it is a different
measurement:

- ⛔ **It needs a root certificate installed into the machine's trust store**,
  which is a change to that machine's security configuration and is the
  operator's action rather than an agent's.
- ⭐ **What this result does establish about it:** the pin is not what makes the
  terminating surface trustworthy or otherwise, because the surface itself
  changes nothing the raw surface can see. The remaining question is narrower
  than it was.
- `DRIVER-04`, the root store a browser actually reads, is the entry that has to
  land before a trust-store capture is even meaningful on Windows.

⚠ It is written into `TODO/PROGRESS.md` as an open question with that
recommendation rather than left as a sentence in a closed entry.

#### What the suite proves that the driven run cannot

⛔ **A driven run cannot show that its own comparison could have failed.** The
seven tests plant each case: a field stable and equal, a field stable and
different, a field drawn per connection, the same field with GREASE stripped, a
one-connection run reported as thin, an empty side reported as not comparable
rather than agreeing, and every named field rendering on a real hello.

⭐ **The third of those is the mutation proof of the whole design.** Two runs
whose GREASE differs on every connection produce `NotComparable` and zero
differing fields, which is the result a naive diff would have got wrong.

---

## HARNESS-11. The p0f layer, which is free once the listener is ours

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** harness, **Priority** P2, **Effort** M, **Status** open

### Problem

A whole second fingerprint sits below TLS and nothing in the plan reads it. It
is thrown away by every project examined, and it costs almost nothing when the
listener already belongs to this project.

### Premise

Believed. Not attempted.

### Approach

Record the source port, the maximum segment size, the window size and scale, the
time to live, and the order of the TCP options. That is operating-system-level
fingerprinting and it belongs in the same profile as everything else.

⚠ It needs a raw socket or an equivalent, which is a capability question rather
than a code question, and it may be unavailable on a hosted runner. Establish
that first and record the answer, because it decides whether this is a lane in
the capture matrix or a local-only extra.

Must not: fabricate a value the platform did not give. An unavailable field is
absent, with a reason.

### Prove

```bash
cargo test -p b-ids-harness tcp_layer -- --nocapture
```

Passing means: a capture on a host where the capability exists records all five
fields, and a capture on a host where it does not records them as absent with a
reason rather than as zero.

---

## HARNESS-12. A public capture oracle

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** harness, **Priority** P2, **Effort** L, **Status** open

### Problem

Every existing hosted fingerprint service returns a subset and no raw hello.
Somebody who wants to know what their own browser sends has to trust a page.

### Premise

Believed, and it is a service question as much as a code one.

### Approach

A hosted endpoint anybody can point a browser at, returning the full model from
`SCHEMA-01`, including the raw hello, rather than a hash and a marketing page.
The same harness binary, run as a service.

⚠ **It changes this project's obligations.** A hosted endpoint receives traffic
from people, which is the one thing the scope boundary says this project does
not do. Settle before building: what is logged, for how long, and whether
anything is retained at all. The default that keeps the boundary intact is to
retain nothing and return the result to the caller only.

Must not: retain a capture from a visitor's browser as a corpus entry. The
corpus is captures the harness took of browsers it launched itself.

### Decision

Whether to run this at all.

Recommendation: build the mode, and do not host it until the retention question
has an answer written down and a human has approved it.

### Prove

```bash
cargo run -p b-ids-harness -- --serve --no-retain
```

Passing means: the endpoint returns a full profile to a browser pointed at it,
nothing is written to disk, and a test asserts the no-retain default by checking
that the process created no file.

---

## HARNESS-13. Terminate the handshake, and mint the authority that lets a browser complete it

**Source** `HARNESS-02`, whose `--ca-out` switch is blocked, and the operator's vendoring ruling of 2026-09-01
**Category** harness, **Priority** P1, **Effort** L, **Status** done

### Problem

`b-ids-harness --ca-out unused` exits 2 and says the switch needs a TLS server
this tree does not have. Everything above TLS in a browser's fingerprint is
therefore unreachable: the settings frame, the window update, the header block
and the priority block are all behind an encrypted record layer, and no browser
speaks cleartext HTTP/2.

### Premise

⭐ **Measured on 2026-09-01.** `crates/b-ids-harness/src/listener.rs` reads the
first TLS record, records its bytes, parses the `ClientHello` and closes.
`Protocol` has two variants and its own comment names itself as the seam a
third goes through. `crates/b-ids-harness/src/h2.rs` already reads a connection
preface and its frames from a byte slice, and `crates/b-ids-harness/src/main.rs`
refuses `--ca-out` by name.

⚠ **Read rather than measured:** that Chrome accepts a leaf certificate whose
subject alternative name is an IP address, and that it skips certificate
transparency enforcement for a chain ending at a locally installed root. Both
are checked by the driven pass rather than assumed, and a failure of either is
recorded as a finding under this entry rather than as a defect in the code.

### Approach

`VENDOR-01` puts `rustls` in the tree. This wires it to the listener.

**The surface.** A third `Protocol` variant beside `TlsRaw` and `Cleartext`,
selected by `--ca-out PATH` rather than by a mode flag of its own, because
minting the authority and terminating the handshake are one capability and two
switches for it would be two ways to reach one path.

**The order of reads, and it is the part that matters.** The listener reads the
first record and records its bytes exactly as it does today, and the existing
`parse_record` stays the oracle for the hello. Only then are those bytes and
the socket handed to the terminator, which replays what was already read and
continues the handshake. ⛔ **The raw hello is never read back out of the TLS
library.** A hello reported by the implementation that consumed it is a hello
filtered through somebody else's parser, and this project's whole contribution
is that the bytes are kept.

**The authority.** One certificate authority minted per run, written to the
path `--ca-out` names in PEM, and one leaf under it carrying the bound address
as a subject alternative name. `parse_bind` already refuses a hostname and says
in its own message that a leaf certificate needs a literal address, so the two
halves already agree.

**Above the handshake.** ALPN offers `h2` and `http/1.1`, in that order, and
whichever the peer selected is recorded on the capture beside the negotiated
protocol version and cipher suite. Those three are properties of this server
rather than of the browser, so they are recorded as conditions of the capture,
which is what `HARNESS-10` will compare against.

The decrypted stream then feeds the reader that already exists: the same
`read_cleartext` path, chosen by the bytes rather than by the flag, so one
capture path serves both surfaces.

⛔ **What it must not do.** It must not disable certificate verification in the
browser, which changes the subject and is what the authority exists to avoid.
It must not edit or re-encode the recorded bytes. It must not make termination
the default, because completing a handshake can change what a client offers and
the raw surface is the one that answers the narrower question. It must not
build a second HTTP/2 reader.

### Decision

**Where the terminator lives.**

⭐ **Recommendation: a module inside `b-ids-harness`, not a crate of its own.**
There is one consumer, and `../docs/conventions/code.md` refuses an abstraction
built beyond one real seam. The alternative, a `b-ids-tls` crate, was rejected
because the argument for it is a consumer that does not exist: `HARNESS-12`
runs the same binary as a service and needs no second crate to do it.

### Prove

⛔ **The acceptance, and it is a command.**

```bash
cargo test -p b-ids-harness termination -- --nocapture
```

Passing means: a client completes a verified handshake against a run started
with `--ca-out`, trusting only the authority that run wrote; the capture
records the raw `ClientHello` bytes for that connection and they are byte
identical to what the client sent; the negotiated protocol is recorded; and a
run started without `--ca-out` still terminates nothing.

The driven pass is a real browser, and it is this entry's own obligation rather
than the operator's:

```bash
cargo run -p b-ids-harness -- --ca-out .tmp/ca.pem --json --handshakes 8
```

Passing means a browser on this host, pointed at the printed base URL with that
authority trusted, produces at least one capture whose HTTP/2 half carries a
settings frame.

### Closing

**Closed 2026-09-01T05:20:00Z.** `--ca-out` mints an authority, writes it, and
terminates the handshake behind it. ⭐ **Two real browsers completed verified
handshakes against it and the harness read their HTTP/2**, which is the first
time anything in this repository has read a byte a browser put on a wire.

```text
$ cargo test -p b-ids-harness termination -- --nocapture
running 4 tests
test termination_refuses_a_terminated_surface_with_no_server_configuration ... ok
test termination_is_absent_on_the_surfaces_that_do_not_terminate ... ok
test termination_records_the_authority_and_never_its_key ... ok
test termination_completes_a_verified_handshake_and_reads_http2 ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
exit=0
```

### ⭐ The driven pass, which is this entry's own obligation

Two browsers on this host, each pointed at the printed base URL, each in a
throwaway profile directory. Chrome `151.0.7922.76` produced **7 connections, 6
of them terminated**; Edge `152.0.4191.53` produced **8 connections, 7 of them
terminated**. Every terminated one negotiated `h2` over TLS 1.3 with
`TLS13_AES_128_GCM_SHA256`, and every one carried three frames: SETTINGS, a
connection WINDOW_UPDATE, and a HEADERS frame with the priority bit set.

One run of four, printed by the command itself rather than as JSON:

```text
connection 2 from 127.0.0.1:56470, 1803 byte(s), TlsTerminated
  tls: 16 cipher suite(s), 17 extension(s), 2 GREASE slot(s)
  terminated: alpn Some("h2"), version Some("TLSv1_3"), suite Some("TLS13_AES_128_GCM_SHA256"), 534 plaintext byte(s)
  http2: 3 frame(s), settings [SettingEntry { id: 1, value: 65536 }, SettingEntry { id: 2, value: 0 }, SettingEntry { id: 4, value: 6291456 }, SettingEntry { id: 6, value: 262144 }], window increment Some(15663105)
  http2 priority block: Some(StreamPriority { exclusive: true, stream_dependency: 0, weight_wire: 255 }), raw Some("80000000ff")
sampling: 3 of 4 handshake(s) completed, 3 distinct GREASE draw(s), 3 distinct extension order(s)
```

⚠ **The first connection of each run terminated nothing**, and that is the
browser rather than a defect: it is the preconnect `HARNESS-07` describes,
opened and abandoned before a handshake completes. It is recorded with its note
rather than dropped.

### ⛔ The authority was not installed into the machine's trust store

Installing a root certificate changes a machine's security configuration and it
is the operator's to make. This session used the narrowest thing that avoids
it: both browsers were launched with `--ignore-certificate-errors-spki-list`
carrying the base64 SHA-256 of the run's own authority key, so exactly one key
was trusted, for one launch, with no store touched.

⚠ **That is a condition of every capture taken this way and it is not the same
as a trusted root.** ⛔ It is also not `--ignore-certificate-errors`:
verification still runs and a certificate from any other key is still refused.
`HARNESS-10` is the entry that measures whether the difference changed what was
measured, and `DRIVER-04` is where the root store a browser actually reads
belongs.

### ⚠ A test that had been a refusal became a HANG

`switches_ca_out_is_absent_rather_than_inert` asserted that the command exits 2.
Once `--ca-out` worked, the command bound a socket and blocked on an accept
nobody was going to make, and the test waited on a process that was never going
to exit. ⛔ **A hang has no message and no exit code**, and killing it left the
test binary locked, so the next build failed on a file the linker could not
open.

⭐ **The same hazard is already recorded in `HARNESS-02`'s closing, from the
other direction**: there, removing a guard made its test hang. The rule is now
applied twice: a test that drives this command passes a deadline, so a change in
behaviour reports rather than waits. The replacement asserts what the switch
does now and exits on a 1500ms run timeout.

### What the four tests cover

| the test | what it would catch |
| --- | --- |
| a verified handshake completes and HTTP/2 is read | the whole path, driven through the COMPILED COMMAND rather than the library, with a client that trusts only the authority the run wrote |
| ⭐ the raw block is the client's own first record, byte for byte | a hello re-encoded by the TLS library rather than kept. The client socket is wrapped in a recorder, so the assertion compares bytes rather than shapes. |
| the authority is written and its key is not | a private key on disk, which `docs/security/secrets.md` refuses whatever it is for |
| a terminated surface with no configuration is refused at the bind | a mode that parsed and silently did nothing |

⚠ **The HTTP/2 payload the test sends is the committed cleartext fixture.** A
second copy of a connection would let the two surfaces drift apart while both
tests stayed green.

### Mutation-proved

⛔ **Each of these was planted, run, and read.**

| what was planted | what happened |
| --- | --- |
| `raw_hex` set from the decrypted plaintext instead of the first record | `termination_completes_a_verified_handshake_and_reads_http2` FAILED on the byte comparison, printing the HTTP/2 preface where the record should be |
| the bind-time refusal disabled | `termination_refuses_a_terminated_surface_with_no_server_configuration` FAILED, because the combination bound successfully |
| the golden capture left at the old schema version | `listener_reads_the_committed_fixture_and_produces_the_committed_capture` FAILED and printed the whole diff, which is how the version bump was reviewed rather than assumed |

### ⚠ What is NOT here, named rather than left to be discovered

- **The handshake bytes after the first record are not recorded.** They are
  ciphertext on the wire and the terminator consumes them. What the peer sent
  inside them is in `Termination::plaintext_hex`, and ⚠ that is not the wire:
  it is what the peer sent, recovered by holding the key.
- **No profile is written.** This is a capture surface, not a corpus entry.
  `CORPUS-01` is where a capture becomes something published.
- ⚠ **The certificate's validity window is the minter's default**, which is
  wide. Both browsers accepted it. Narrowing it needs another direct
  dependency and buys nothing measurable, because the key is minted per run
  and never written to disk.
- **A hostname is still refused by `--bind`.** The leaf names a literal
  address, and `parse_bind` and the minter are the two halves of one rule.
- ⚠ **Three distinct extension orders arrived in three completed connections.**
  The shuffle is real and it is visible now. `SCHEMA-10` is the entry that
  records it as a property rather than as noise.

---

## HARNESS-14. The pin against a real trust anchor, on a machine that is thrown away

**Source** ruled by the operator 2026-09-01. It is the half `HARNESS-10` measured around.
**Category** harness, **Priority** P2, **Effort** M, **Status** done

### Problem

Every capture this project has taken went through a **per-launch key pin**
rather than a root in the browser's trust store, and `captured.trust` records
which. ⛔ Nothing has ever measured whether that choice changes what the browser
puts on the wire.

⚠ **`HARNESS-10` measured the adjacent question and it is not this one.** It
compared the raw surface against the terminating surface and found the two agree
on every TLS field with a stable value. That says the act of terminating changes
nothing; it says nothing about the pin.

### Premise

⭐ **Measured, and the measurement is what narrowed this entry to its remaining
half.** `HARNESS-10`'s comparison ran three rounds and reported 17 of 19 TLS
fields agreeing, none differing, and two carrying a per-connection draw. One
browser, one build, one host.

⚠ **The reason this stayed unmeasured until now was a machine**, not a
difficulty: installing a root into a trust store is a change to that machine's
security configuration. ⭐ The operator's 2026-09-01 ruling removes it: a runner
is disposable, so the install is free and undoes itself when the job ends.

### Approach

⭐ **A job, on a runner, that captures the same build twice**: once through the
standing per-launch pin, and once with this project's own root installed into
the store the browser actually reads. Compare the two profiles field by field
with the comparison `b_ids_harness::modes` already owns.

⚠ **`DRIVER-04` lands first, and that ordering is not a preference.** On Windows
the store a browser reads is not obviously the one `certutil` writes to, and a
comparison run against the wrong store produces a confident wrong answer that
looks exactly like a real result.

⛔ **The root is generated for the run and never committed.** It is a capture
tool. Nothing about it may resemble something to ship in a client, and
[`../docs/security/secrets.md`](../docs/security/secrets.md) is the rule.

⛔ **Both profiles record their own `captured.trust`**, so the comparison is
readable from the corpus afterwards rather than only from a job log.

Must not: change the standing capture method on the strength of one host. ⚠ If
the two disagree, that is a finding about a platform and it needs the matrix,
not a switch flip.

### Prove

```bash
sh experiments/50-trust-anchor.sh --json
```

Passing means: the script reports which trust route each capture used, the count
of TLS fields that agree, differ and could not be compared, and refuses to
report a comparison at all when only one of the two routes completed a
handshake.

### ⚠ The acceptance names `50-` rather than `30-`

⛔ **`30-` was taken** by
[`../experiments/30-resumption-control.sh`](../experiments/30-resumption-control.sh).
[`corpus.md`](corpus.md), `CORPUS-05`, states the renumbering rule and why the
Prove block is corrected rather than the file renamed; the script here is
[`../experiments/50-trust-anchor.sh`](../experiments/50-trust-anchor.sh).

### Closing

**Closed 2026-09-02T05:30:00Z.** The comparison ran on a machine that was thrown
away, and the pin and a real trust anchor produced hellos that agree on every
comparable TLS field.

```text
$ sh experiments/50-trust-anchor.sh --headless --browser chrome --rounds 2
   (.github/workflows/trust-anchor.yml, run 33592736694, ubuntu-latest)

== round 1 of 2 ==
route=pin handshakes=3 h2=3 connections=4
route=trust-store handshakes=2 h2=2 connections=4

== round 2 of 2 ==
route=pin handshakes=1 h2=1 connections=4
route=trust-store handshakes=0 h2=0 connections=0

roots left in the store afterwards: 0

pin: 4 cold, 0 resumed, 4 abandoned
trust-store: 2 cold, 0 resumed, 2 abandoned

comparing 8 pin hello(s) against 4 trust-store, resumed connections excluded
  agrees        tls.record_version  0x0301
  agrees        tls.legacy_version  0x0303
  agrees        tls.session_id_len  32
  not comparable tls.cipher_suites
      pin  5 distinct value(s) within the run
      trust-store  4 distinct value(s) within the run
  agrees        tls.cipher_suites.no_grease  0x1301,0x1302,0x1303,0xc02b,0xc02f,0xc02c,0xc030,0xcca9,0xcca8,0xc013,0xc014,0x009c,0x009d,0x002f,0x0035
  agrees        tls.compression_methods  0
  not comparable tls.extensions.order
      pin  8 distinct value(s) within the run
      trust-store  4 distinct value(s) within the run
  agrees        tls.extensions.set.no_grease  0x0005,0x000a,0x000b,0x000d,0x0010,0x0012,0x0017,0x001b,0x0023,0x002b,0x002d,0x0033,0x44cd,0xfe0d,0xff01
  agrees        tls.extensions.count  17
  agrees        tls.key_exchange_groups.no_grease  0x11ec,0x001d,0x0017,0x0018
  agrees        tls.key_shares.groups.no_grease  0x11ec,0x001d
  agrees        tls.key_shares.lengths  1216,32
  agrees        tls.signature_algorithms  0x0904,0x0905,0x0906,0x0403,0x0804,0x0401,0x0503,0x0805,0x0501,0x0806,0x0601
  agrees        tls.signature_algorithms_cert  absent
  agrees        tls.alpn  h2,http/1.1
  agrees        tls.ech  0/0x0001
  agrees        tls.padding_len  absent
  agrees        tls.grease.count  2
  agrees        tls.grease.positions  0,16

modes=agree differing:0 not_comparable:2 fields:19

conditions
  host      Linux 6.17.0-1022-azure
  browser   Chrome 151.0.7922.173
  taken     2026-09-02T05:00:42Z
  headless  --headless
  rounds    2, each one pinned run then one trust-store run
  teardown  0 root(s) left in the store
exit=0
```

### ⭐ The answer, and what it is worth

⭐ **19 TLS fields compared, 0 differing, 2 not comparable.** The two not
comparable are `tls.cipher_suites` and `tls.extensions.order`, both of which
carry a per-connection GREASE draw or shuffle, and `b_ids_harness::modes` reports
those as not comparable rather than as findings.

⛔ **So on this platform, with this build, the per-launch key pin did not change
what the browser put on the wire against a root in the store it reads.** ⚠ That
is one platform, one build, one day. It does not generalise and the entry's own
rule says so: if the two ever disagree, that is a finding about a platform and it
needs the matrix rather than a switch flip.

### ⛔ An inherited claim is refuted here, and the wording is kept

[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8 carried:
**"Chrome on Linux does not read the user's NSS database for server
authentication"**, with `certutil -t "C,,"` reported as still producing
`CertificateUnknown`.

⭐ **Measured otherwise.** The trust-store route completed **2 handshakes and 2
HTTP/2 connections** on `ubuntu-latest` with Chrome `151.0.7922.173`, through a
root added with exactly that command into `~/.pki/nssdb`.

⚠ **And it is not reliable.** Round 2's trust-store leg accepted **no connection
at all**, so the route works sometimes and not always on this runner. ⛔ The
honest statement is that the claim as written does not hold here, not that the
NSS route is a good way to capture. [`../docs/HISTORY/README.md`](../docs/HISTORY/README.md)
carries the original wording with this measurement under it.

### ⛔ The teardown is counted, not remembered

`roots left in the store afterwards: 0`, read back with `certutil -L` rather
than assumed, and a root left behind fails the run whatever the comparison said.
⭐ The removal is why this can run at all: the operator's 2026-09-01 ruling is
that installing a root belongs on a machine that is reclaimed, and the script
refuses to install one unless `B_IDS_DISPOSABLE=1` says the machine is one.

### ⛔ Two defects in this session's own new code, both found by running it

| what | how it showed |
| --- | --- |
| **every store command could hang** | the first dispatch, run `33590621046`, was cancelled at its 25-minute limit on BOTH platforms with the measurement already taken and the comparison never printed. A certificate tool that asks for a password reads a terminal that is not there. ⭐ Every store command goes through one bounded helper now: `timeout 60` and stdin closed. |
| **a native tool got an msys path** | run `33592736694`'s Windows lane handed `certutil.exe` a path like `/d/a/b-ids/...`. ⭐ The path is converted with `cygpath` now. |

⚠ **And the first run printed nothing between the round header and the
comparison**, so the report it uploaded said only which round it had reached.
The script prints one fixed line per route as it goes.


### ⛔ Windows is UNMEASURED, and the script says so rather than inventing a comparison

Run `33594293802`, `windows-latest`, Chrome `151.0.7922.174`: the pinned route
completed 2 handshakes of 4, the trust-store route could not be set up at all,
and the script **exited 2**.

```text
== round 1 of 2 ==
route=pin handshakes=2 h2=2 connections=4
b-ids-harness: 2 of 4 handshake(s) completed, from 4 accepted connection(s)
exit=2
```

⭐ **That refusal is the entry's own requirement working.** The acceptance asks
the script to refuse a comparison when only one route completed a handshake, and
it refused rather than reporting one side against nothing.

⚠ **The remaining cause is named and not guessed at.** `certutil -addstore
-user Root` returned non-zero under the bounded helper with its stdin closed,
once, in run `33594293802`, which is the only Windows run taken after the path
was corrected. ⛔ What this project has MEASURED is that the command did not
succeed non-interactively on that runner in that run; why it did not is a
reading nobody here has taken, and writing one would be the guess this project
refuses.

⛔ **So the answer is recorded for Linux and left open for Windows**, which is
what `DRIVER-04` warned about in as many words: on Windows the store a browser
reads is not obviously the one `certutil` writes to, and measuring against the
wrong store gives a confident wrong answer.
