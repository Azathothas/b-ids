# publish

Three delivery surfaces, because each fails differently, and the routes a
program with nothing but `curl` can read.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## PUB-01. Releases, tagged and versioned and immutable

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P1, **Effort** M, **Status** done

### Problem

A consumer that pins a release and gets different bytes later has been broken
silently. A consumer that has to list releases before it can fetch one has been
given an API instead of a file.

### Premise

Believed. Nothing has been released.

### Approach

- **Tag shape**: the schema major, the date, and a counter, for example
  `v1.2026.08.30.1`. The first component says which schema a consumer is getting
  without opening anything, the date says how fresh the data is, the counter
  allows more than one release a day, and the whole thing sorts correctly and is
  a valid tag.
- **Moving tags beside it**: one per schema major, and one overall, so a script
  can fetch without listing.
- **Assets**: one archive per format from `SCHEMA-08`, a checksums file, and a
  build provenance attestation. Never an unversioned archive as the only asset.
- ⛔ **A published release is immutable.** Never re-upload an asset. Cut a new
  release that supersedes it and say so in the body. Consumers pin releases, and
  a mutated asset breaks them silently.
- **Reproducible**: the same inputs produce byte-identical archives. Sort
  everything, zero the timestamps, pin the generator.

Must not: put a token in the fetch path. A release asset is public and a
consumer should need nothing.

### Prove

```bash
sh scripts/common/check-release.sh --dry-run
```

Passing means: two runs of the release build over the same corpus produce
byte-identical archives and identical checksums, and a run that would overwrite
an existing tag exits non-zero.

---

### ⭐ Closed 2026-09-03. One assembler, and a build that is the same twice

⛔ **`PUB-01` and `PUB-02` publish the SAME BYTES**, and that is the design
decision this entry actually turned on. A release archive and a data branch
built by two assemblers is two answers to what this project publishes, and the
day they differ nobody finds out from either. `b_ids_corpus::publish::build` is
the one assembler; this entry archives what it produced and `PUB-02` pushes it.

#### The acceptance

```text
$ sh scripts/common/check-release.sh --dry-run
release ok: 197 artefact(s), 668384 byte(s), identical over two builds.
  built from corpus 91d175e84340...
  v1.2026.09.03.1 would be free, over 0 existing tag(s). archive: ok
  ⛔ Nothing was tagged, uploaded or pushed.
rc=0
```

⚠ **That byte count no longer reproduces, and two later entries say why.**
`PUB-10` found that the build it came from was given an absolute root, which the
route manifest then recorded; `VALID-04` then added a published artefact. ⛔ The
figure is left as it was measured rather than edited to today's, because a
pasted output is a measurement and re-running the command is what produces the
current one.

⚠ **The digest above is abbreviated at the ellipsis**, for the same reason
`TOOL-03`'s and `CORPUS-01`'s blocks are: written out it is a 64-character hex
run in a tracked file, which is what `check-no-secrets --public` refuses. ⛔ It
is a CONTENT ADDRESS rather than a credential, and abbreviating it was chosen
over widening a security rule for a cosmetic reason.

⛔ **`--dry-run` is REQUIRED**, for the reason `latest` requires
`--assert-stable`: this check publishes nothing, and a run with no argument
would read as though it had cut a release. It exits **2** without it.

Both halves agree, and each compares the two builds itself: the POSIX half in
`diff -r`, the PowerShell half by hashing every file on both sides.

```text
{"schema":"check-release/1","files":197,"bytes":668384,"cases":9,"tags":0,"archive":"ok","problems":0}
```

#### ⛔ Nothing in the assembler reads a clock

⚠ **A build stamped with the time it ran produces a different archive every
run**, and a consumer then cannot tell a real change from a rebuild. The
manifest's `generated_from` is a digest over the corpus artefacts alone, so a
change to a GENERATOR moves the files and not the stamp, and a change to the
CORPUS moves the stamp. Two builds are byte-identical, file for file, and so are
two archives of them.

#### The tag, and the immutability rule

| | |
| --- | --- |
| shape | `v1.2026.09.03.1`: the schema major, the date, and a counter. The first says which schema a consumer is getting without opening anything, the date says how fresh the data is, the counter allows more than one release a day, and the whole thing sorts. |
| ⛔ a tag that exists | **refused.** A published release is immutable: cut a new one that supersedes it rather than re-uploading an asset. Consumers pin releases, and a mutated asset breaks them silently. |
| ⛔ a date that is not one | refused. A tag built from a malformed date sorts wrongly forever, and the tag is the thing consumers pin. |
| ⛔ an empty build | refused. A release with no artefact is the "step that exits 0 having done nothing" row wearing a version number. |
| moving tags | `v1` and `latest`, one per schema major and one overall, so a script fetches without listing. ⚠ They MOVE, which is why the dated tag exists beside them. |

⛔ **The existing tags are the CALLER'S**, read from the repository by whoever
asks. `plan_release` takes them as an argument, so the naming rule is testable
and is not tied to a working directory. The check reads them with `git tag` and
asserts the tag this build would take is free.

#### ⛔ Two tars, two spellings, one date format

⚠ **The two halves of this check resolve DIFFERENT tar binaries on this
machine**, and the archive leg is where that showed. Measured 2026-09-03:

| | |
| --- | --- |
| GNU tar 1.35, which Git Bash resolves | wants `--owner=0 --group=0` and **refuses** `--uid`. And it reads a Windows path as a REMOTE HOST SPEC: `-cf C:/x.tar` produced `Cannot connect to C: resolve failed` until `--force-local` was added |
| the bsdtar Windows ships, which PowerShell resolves | wants `--uid 0 --gid 0` and **refuses** `--force-local` |
| ⛔ the date | `2026-01-01T00:00:00Z` is `Invalid argument to --mtime (bad date string)` to bsdtar. `2026-01-01 00:00:00` is accepted by both |

⭐ **The first version of this leg skipped on every host**, reporting
`archive: skipped` and a line saying a skip is not a pass, which is honest and
useless. Both halves probe the two spellings now and the leg runs.

#### The guard mutation, each exit code read unpiped

| planted | what went red |
| --- | --- |
| no `--dry-run` | exit **2**, in both halves, with the reason |
| a tag that already exists, passed to `plan_release` | `publish_a_tag_that_already_exists_is_refused` |
| a malformed date | `publish_a_date_that_is_not_one_is_refused`, over four shapes |
| a build with no artefact | `publish_a_build_with_no_artefact_is_not_releasable` |
| ⛔ the archive leg's first flag set | **it skipped rather than failing**, on every host, and a leg that always skips is a leg nobody knows works. Both spellings are probed now and the leg reports `ok` on this machine |

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a release | ⛔ None was cut. No tag was created, no asset uploaded and no remote written to. The acceptance is a dry run and says so in its own output. |
| a workflow that releases on a schedule | ⚠ Deliberately absent. A workflow that cut a release would be an outward-facing action taken by a session rather than by the operator, and the trigger is theirs to add. |
| a build provenance attestation | Named in the approach and not built. It needs a signing identity, which is `PUB-09`. |

---

## PUB-02. The data branch, over raw file serving

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P1, **Effort** M, **Status** done

### Problem

Releases alone are not enough, and the reason is practical rather than
theoretical. A releases API is rate-limited, needs authentication for serious
polling, has its own downtime, and returns a document a consumer has to walk
before it can fetch a file. Raw file serving takes a path, is fronted by a
cache, needs no token, and is what every install script already uses.

### Premise

Believed, and one reference project's practice supports it: a dataset published
as flat files over a content delivery network, refreshed weekly by its own
automation, with no index in the fetch path.

### Approach

Publish **both**, from the same build, with the same content.

- an **orphan branch** carrying only generated artefacts and no source history,
  so it stays small and clones fast;
- **dated snapshots and a `latest` pointer**, both. The snapshot is what a
  reproducible build pins; the pointer is what a script follows;
- ⛔ **append-only, never force-pushed.** A consumer pinning a commit on this
  branch keeps working forever, and that property is free;
- **an index and a checksums file**, so a consumer can verify what it fetched
  without any API;
- **document the cache** next to the URL. A raw file service caches for some
  minutes, and a consumer polling for a change must not be surprised.

⛔ The source tree, any vendored dependency and the reference corpus stay off
this branch. A consumer of the data never has to reason about somebody else's
licence, because none of it is in what they downloaded.

Must not: generate the branch from anything but the canonical corpus, and must
not hand-edit a file on it.

### Prove

```bash
sh scripts/common/check-data-branch.sh
```

Passing means: the branch's content is regenerated and compares byte-identical
to what is published, every file has a checksum in the index, and a push that
would rewrite history is refused.

---

### ⭐ Closed 2026-09-03. The branch's content is decided, and the leg that cannot run says so

⛔ **The same assembler as `PUB-01`.** Publishing both surfaces from one build
is what makes "with the same content" a property rather than a promise, and
`b_ids_corpus::publish::build` is that build.

#### The acceptance

```text
$ sh scripts/common/check-data-branch.sh
data branch ok: 199 file(s) regenerated, identical over two builds,
  197 of them with a checksum in the manifest and in SHA256SUMS, and no
  source, vendored dependency or reference corpus among them.
  ⚠ A SKIP IS NOT A PASS: the data branch is absent, so the regenerated tree was
  compared against nothing. Push it once and this leg starts running.
  ⛔ Nothing was pushed and no branch was created.
rc=0
```

```text
{"schema":"check-data-branch/1","files":197,"present":199,"recorded":197,"cases":9,"published":"absent","problems":0}
```

⚠ **199 files against 197 recorded**, and the two that are not is the point: a
file cannot carry its own digest, so `MANIFEST.json` and `SHA256SUMS` are the
two that every other file is checked against.

#### ⛔ The leg that cannot run, reported as a skip

⚠ **"the branch's content compares byte-identical to what is published" has
nothing to compare against**, because the branch does not exist. The check reads
git for it, reports `published: absent`, and says in its own output that a skip
is not a pass and what would make the leg start running. ⛔ Reporting a pass
over a branch nobody has made is the "step that exits 0 having done nothing" row
of
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md),
and this entry would have been the easiest place in the tree to write one.

#### What the branch carries, and what it must never

| on it | why |
| --- | --- |
| `corpus/v1/` and `raw/v1/` | ⛔ **copied verbatim**, never re-serialised. A published profile is immutable and its content address is the file's own bytes. |
| `formats/` | every generated format and the support matrix, each read back before it is written |
| `routes/` | the flat tree a program reads with `curl`, and its manifest |
| `anchors/` | one file per build that carries the root-store extension |
| `vectors/` | ⭐ the published JA4 test vectors, copied verbatim, so an implementation in any language has something to check itself against. `VALID-04`. |
| `LICENSE` | ⛔ copied from the repository's own file. A build that wrote its own text would be a second copy of a legal document. |
| `MANIFEST.json`, `SHA256SUMS` | what a program reads, and what `sha256sum -c` reads |

⛔ **Not on it: `crates/`, `vendor/`, `references/`, `scripts/`, `docs/`,
`TODO/` or `target/`.** A consumer of the data never has to reason about
somebody else's licence, because none of it is in what they downloaded, and both
halves of the check assert each of those seven is absent.

#### ⛔ Append-only, and what "would rewrite" means

`publish::would_rewrite` takes what the remote holds and what the new commit was
built on, and answers in four cases:

| head | parent | verdict |
| --- | --- | --- |
| absent | anything | ⭐ not a rewrite. The first push creates the branch. |
| a commit | the same commit | not a rewrite. This is an append. |
| a commit | a different commit | ⛔ a rewrite. The new commit was built on something the branch has moved past. |
| a commit | none | ⛔ a rewrite, and the one worth naming: an ORPHAN commit pushed over an existing branch discards every commit on it, which is exactly how a branch built with `--orphan` destroys itself on the second run. |

#### The guard mutation, each exit code read unpiped

| planted | what went red |
| --- | --- |
| each of the four rewrite cases | `publish_a_push_that_would_rewrite_the_data_branch_is_refused` |
| a file written into the tree with no manifest entry | the loop over what is on disk, which reads the FILES rather than the manifest, so an unrecorded file is a finding rather than an absence nobody counted |
| the suite's cases renamed | both halves name the five they expect |

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| the branch | ⛔ It was not created and nothing was pushed. Creating an orphan branch on this repository's remote is an outward-facing action, and it is the operator's. |
| a workflow that pushes it | ⚠ Deliberately absent, for the same reason. The assembler and the checks are here; the trigger is theirs. |
| the cache documented next to the URL | Named in the approach. There is no URL yet, and a documented cache duration for a service nobody is serving from would be a number nobody measured. |

### ⭐ 2026-09-03, later the same day: the branch exists, and the leg that could not run does

⛔ **`PUB-10` armed the trigger and the push that landed it created the
branch.** The first run of `publish.yml` on commit `8361ee7`:

```text
$ gh run view 33723601879 --json jobs --jq '.jobs[] | "\(.name)\t\(.conclusion)"'
assemble, and check both surfaces        success
the data branch, appended to             success
the release, cut from a pushed tag       skipped
```

⚠ **The release job skipped and that is correct**: no tag was pushed, and a tag
is the only thing that cuts a release.

#### ⭐ Verified byte for byte, which is the operator's own first step

```text
$ git ls-tree -r --name-only refs/remotes/origin/data | wc -l
200
$ git log -1 --format='%an %s' refs/remotes/origin/data
github-actions[bot] corpus: 198 artefact(s), built from 91d175e84340...
```

⛔ **The comparison is between two git TREE OBJECTS**, which is what "byte for
byte" means for a branch: one tree object is one set of bytes, over every path
and every mode. The regenerated tree and the published one are the same object.

#### ⛔ The skip that had stopped being honest

⚠ **This check reported `published: remote` and still printed "push it once and
this leg starts running".** The branch was there; nothing compared against it.
⛔ A skip whose own condition has been met is worse than a failure, because it
reads as a pass with a caveat nobody re-checks.

Both halves compare now, and the answer is in the JSON as `matched`, so
`check-twins` can see whether both did it:

```text
$ sh scripts/common/check-data-branch.sh
data branch ok: 200 file(s) regenerated, identical over two builds,
  198 of them with a checksum in the manifest and in SHA256SUMS, and no
  source, vendored dependency or reference corpus among them.
  ⭐ The data branch is remote and its tree is 6d0b4c1703f3…, which is what this
  corpus derives to. One tree object is one set of bytes.
  ⛔ Nothing was pushed and no branch was created.
rc=0
```

⚠ **The tree object is abbreviated at the ellipsis**, for the reason `PUB-01`s
digest and `TOOL-03`s are: written out it is a 40-character hex run in a
tracked file, and `check-no-secrets --public` excludes one inside a markdown
code span rather than inside a fenced block. ⛔ It is a CONTENT ADDRESS rather
than a credential, and abbreviating it was chosen over widening a security rule.

```text
{"schema":"check-data-branch/2","files":198,"present":200,"recorded":198,"cases":11,"published":"remote","matched":true,"problems":0}
```

⚠ **The schema moved to `check-data-branch/2`** because the object gained a
field, and a reader that was told the shape is told the shape changed.

#### ⛔ Two PowerShell traps, in one line, found by the twin disagreeing

⚠ **The PowerShell half staged nothing and produced the EMPTY tree**, which
compares unequal to everything, so it reported a difference that did not exist.
Two separate causes in one command:

| written | what PowerShell did |
| --- | --- |
| `--git-dir=(Join-Path $root '.git')` | ⛔ passed `--git-dir=` and the path as TWO arguments. git then had no directory, warned `unable to access '/config'` and staged nothing. Build the string first. |
| `add --all --force -- .` | ⚠ consumed the bare `--` before git saw it, so the pathspec was lost |

⭐ **Neither is visible by reading**, and the sh half is correct with the same
words. The twin comparison is what said so.

#### ⭐ And the no-op rule fired on a real run rather than in a test

The SECOND push to the default branch found the branch already holding those
bytes and pushed nothing:

```text
the branch already holds these bytes, so this run pushes nothing
```

⚠ **That is what stops the branch growing an empty commit on every push**, and
it is the same rule `CI-04` holds about a change that moved nothing. ⛔ It was
designed rather than measured until this run; now it is measured.

---

## PUB-03. Routes a program with nothing but `curl` can read

**Source** the operator; [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 9
**Category** publish, **Priority** P1, **Effort** M, **Status** done

### Problem

A consumer that wants one value should fetch one path and get one value. Today
that consumer would have to fetch a profile, parse it, and extract a field, which
is three dependencies for a string.

### Premise

⭐ **Measured, and the model has the defect.** The reference the operator named
uses exactly this naming shape, and its single-value files end with a newline:
`od -c` over two of them shows a trailing `\n`. So a consumer has to strip it,
which is the burden this entry exists to remove.
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md)
has the measurement.

### Approach

Generate a flat route tree from the canonical corpus, one file per value, at
every useful permutation of the axes the corpus carries.

```text
user-agent/chrome/stable/latest/windows.txt
user-agent/chrome/stable/152.0.7977.64/windows.txt
user-agent/chrome/beta/latest/linux64.txt
ja4/chrome/stable/latest/linux64.txt
akamai/chrome/stable/latest/linux64.txt
sec-ch-ua/chrome/stable/latest/windows.txt
accept-language/firefox/release/latest/macos-arm64.txt
header-order/chrome/stable/latest/windows/navigate.txt
client-hello-hex/chrome/stable/latest/linux64.txt
```

Rules the generator holds:

- ⛔ **a single-value file contains the value and nothing else.** No trailing
  newline, no leading whitespace, no quotes. A consumer must never need to strip
  anything.
- **a multi-value file is newline-delimited and says so by its extension**, so
  the two are distinguishable without fetching. A list file is `.list.txt`.
- **every axis is a directory**, so a path is constructible without an index:
  property, browser, channel, version-or-`latest`, platform, and the request
  variant where a property has one.
- **`latest` is a real file, not a redirect**, so one fetch is one round trip.
- ⭐ **an `index.txt` per directory** listing what is beside it, for a consumer
  that wants to discover rather than construct.
- the axes come from the corpus, so a property this project stops measuring
  stops having routes, rather than serving a stale value forever.

⚠ **Decide the permutation policy explicitly**, because the cross product grows
fast: every property times every browser times every channel times every version
times every platform times every variant. Generating all of them is cheap in
bytes and expensive in file count.

Must not: generate a route for a value that does not exist in the corpus, and
must not fall back to a neighbouring platform's value when one is missing. A
missing route is a 404, which is a fact; a substituted value is a lie.

### Decision

Whether to generate every permutation or only those with a value.

Recommendation: only those with a value, and publish the `index.txt` so a
consumer can see what exists. A route that resolves to a plausible-looking wrong
value is worse than one that 404s, and this is the same rule as the corpus's.

### Prove

```bash
sh scripts/common/check-routes.sh
```

Passing means: for every generated single-value route the last byte is not a
newline; every route resolves to a value present in the corpus; `latest`
resolves to a stable channel; and a fixture route deliberately given a trailing
newline fails the check.

---

### ⭐ Closed 2026-09-03. Fifty-four routes, and a check that reads the corpus rather than the generator

⛔ **The decision this entry left open is taken as its own recommendation**, per
the operator's standing instruction for this session: **only the permutations
the corpus HOLDS a value for**, with an `index.txt` per directory so a consumer
can discover rather than construct. A route that resolves to a plausible-looking
wrong value is worse than one that 404s, and that is the corpus's own rule.

#### The acceptance

```text
$ sh scripts/common/check-routes.sh
routes ok: 42 single-value file(s), none ends with a line ending,
  and 54 generated route(s) each carry the value the corpus holds
rc=0
```

```text
$ b-ids-corpus routes --root . --out .tmp/routes
corpus=routes files:170 single:36 profiles:6
```

⚠ **42 single-value files against 36 routes**: the other six are the committed
raw captures, which this check has always walked.

#### ⭐ The check reads the CORPUS, not the generator

⛔ **Comparing a generated file with the generator's own manifest asks only
whether the generator agrees with itself**, which it always does. The manifest
names the profile and the property behind every route, and each half goes and
reads the value out of the profile: the POSIX half in `jq`, the PowerShell half
in `ConvertFrom-Json`. ⭐ **Two readings of the corpus rather than two wrappers
over one**, which is what makes the twin row worth having.

```text
$ sh scripts/common/check-routes.sh --json
{"schema":"check-routes/2","files":42,"verified":54,"problems":0,"routes":true}
$ pwsh -NoProfile -File scripts/common/check-routes.ps1 -Json
{"schema":"check-routes/2","files":42,"verified":54,"problems":0,"routes":true}
```

#### What the routes are, and what has none

| property | file | what it carries |
| --- | --- | --- |
| `user-agent`, `sec-ch-ua`, `accept-language` | `.txt` | one header value of one request variant |
| `header-order` | `.list.txt` | the header names in wire order |
| `alpn` | `.list.txt` | the protocols offered, in order |
| `client-hello-hex` | `.txt` | the bytes this project read off the wire |
| ⛔ `ja3`, `ja4`, `akamai` | none | **null in every published profile.** Nothing here computes one, so nothing is published, and `VALID-04` is the entry that changes that. |

⚠ **The platform is the corpus's own token**, `win64` rather than `windows`, so
a consumer that knows the corpus route knows this one. The entry's sketch used
the other spelling and a second spelling would be a value in two places with
nothing checking that they agree.

⭐ **`latest` is a real file** rather than a redirect, so one fetch is one round
trip, and it is derived from the routes rather than from a second walk of the
corpus. Both halves assert that every `latest` route names a **stable** profile,
read from the profile rather than from the path.

#### ⛔ Three defects, and two of them were in the check rather than the generator

| what | how it showed |
| --- | --- |
| ⛔ **jq on Windows writes CRLF** | Every one of the 54 comparisons failed while both sides were correct: the carriage return lands on the last field of every `@tsv` line AND on the end of every value. ⚠ This is the SECOND time it has bitten here; `CORPUS-02` carries the first. Every jq read in this check is stripped now. |
| ⛔ **the generated tree is under `.tmp`, which is ignored** | `git ls-files --others --exclude-standard` answers with NOTHING for an ignored path, so the walk reported a clean tree it had never opened: `files:6`, which is the committed raw captures alone. ⚠ Same class as the fixture defect this check's own header already describes, arriving from the other direction. |
| ⛔ **a list file and a single-value file both end in `txt`** | The classifier read the last dot, so `navigate.list.txt` would have been refused for the newline a list needs. Both halves read the whole suffix now, and `index.txt` is a listing rather than a route. |

#### The guard mutation, each exit code read unpiped

| planted | what went red |
| --- | --- |
| the generator reads the `user-agent` header for the `accept-language` property | ⛔ 9 problems and exit **1**, in BOTH halves, with identical messages |
| a fixture route given a trailing newline | `ends with a line ending, and it carries exactly one value`, exit **1** |
| a `.list.txt` fixture with a trailing newline beside it | ⭐ **not** flagged, which is what proves the classifier distinguishes the two rather than refusing everything |

⚠ **And one exit code in this session's own measurement was read through a
pipe** while proving the first row, which reported `rc=0` over a check that had
plainly failed. It is the same defect
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md)
carries a row for, made by the session doing the proving, and the row above is
the re-run without the pipe.

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| the routes on a published surface | The tree is generated into `.tmp` and checked there. `PUB-02` is the data branch that serves it and `PUB-01` the release that ships it. |
| every permutation | ⛔ Only those the corpus holds a value for, which is this entry's own recommendation taken as the ruling. |
| a `beta` or `canary` route | The corpus holds one channel. The generator is keyed on the channel a profile carries, so a beta profile produces beta routes on the day one lands. |

---

## PUB-04. The formats that are not data files

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P2, **Effort** M, **Status** done

### Problem

Most of the day-to-day value of a corpus is in the artefact somebody can paste
into their own tool, and none of the generated data formats is that.

### Premise

Believed.

### Approach

Generate, from the canonical corpus, alongside the data formats in `SCHEMA-08`:

- **ready-to-use client configuration**: a flag line for a general-purpose HTTP
  client, an invocation for an impersonating one, a snippet for each TLS library
  this project targets. A consumer copies one line and is done.
- **detection-side artefacts**: digest allowlists, and rule snippets for the
  common reverse proxies and web application firewalls. ⭐ Same data, other
  direction, and it is half the reason the project is acceptable to everybody.

Must not: generate a configuration for a stack that cannot actually emit the
profile. That is what the support matrix in `EMIT-01` is for, and a snippet that
silently approximates is worse than no snippet.

### Prove

```bash
sh scripts/common/check-generated-configs.sh
```

Passing means: every generated snippet is for a (profile, stack) pair the
support matrix marks as emittable, and a pair marked as a hole generates a
comment naming the hole instead of a snippet.

### ⭐ Closed 2026-09-04. Thirty-seven generated files, and twenty-four of them are refusals

⛔ **The gate is the entry.** A snippet exists only where the support matrix
records the pair as emittable; every stack the matrix records a hole against
gets a file naming the hole, its file and its line, and no snippet at all.

```text
$ sh scripts/common/check-generated-configs.sh
generated configs ok: 6 snippet(s) over 6 profile(s), each for a pair the
  matrix marks emittable, and 24 refusal(s) naming a hole at a file and a line.
  6 detection rule(s), none naming a digest the corpus does not hold.
```

```text
$ sh scripts/common/check-generated-configs.sh --json
{"schema":"check-generated-configs/1","snippets":6,"refusals":24,"detection":6,"profiles":6,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-generated-configs.ps1 -Json
{"schema":"check-generated-configs/1","snippets":6,"refusals":24,"detection":6,"profiles":6,"problems":0}
```

**What is generated**, per profile, under `configs/` in the published tree:

| file | what it is |
| --- | --- |
| `b-ids-emit.rs` | the snippet, for the one stack this tree can RUN. Its byte count comes from the run that produced the file. |
| `rustls.txt`, `h2.txt`, `impit.txt`, `utls.txt` | a refusal each, naming what that stack cannot emit, the path under `references/` it was read at, and the line |
| `detect.conf` | the detection side: the navigation header order and the ALPN list, in the order the wire carried them |

⭐ **Twenty-four refusals to six snippets is the ratio the entry wanted.** A
tree of six snippets and nothing else would be one where every stack was assumed
able.

#### ⚠ Three things the generator does that are worth stating

⛔ **The random is zeroed and says so.** The 32 random bytes are the one part of
a `ClientHello` that differs on every connection, so a snippet that pinned a
captured draw would teach a reader to send a replay.

⛔ **The detection rule is not matrix-gated and IS checked for the opposite
property.** It emits nothing, so no stack has to be able to emit it; the check
refuses it if it names a digest, because the corpus holds none. ⚠ It also says
in its own text that it matches a BUILD rather than a browser, because a rule
pinned this tightly refuses real traffic as a fleet updates.

⛔ **The pair is checked, not the stack.** A cell for one profile does not
license a snippet for another, so the check reads the profile out of the file
and requires an emitting cell for exactly that pair.

#### ⛔ The guard mutation

⛔ **Planted against a copy under the ignored scratch directory, the live file
restored from that copy, and the restored file compared byte for byte.**

| planted | red |
| --- | --- |
| the generator writes a `.rs` snippet for every hole stack instead of a `.txt` refusal | exit 1, 24 problems, each naming the stack and the path: `h2 has a hole in the matrix and a snippet in the tree` |

⚠ **And the exit code was read twice**, because the first reading was through a
pipe into `head` and reported `0`, which is the pipeline's status and the trap
[`../docs/conventions/shell.md`](../docs/conventions/shell.md) section 2 names.
Unpiped it is 1.

#### ⚠ What this entry does NOT generate, and why it is a question rather than a gap

⛔ **No digest allowlist.** The Approach names "digest allowlists" among the
detection-side artefacts, and the operator's ruling on `PUB-03` declined digest
ROUTES on the ground that a route resolves to a value the corpus HOLDS and the
corpus holds no digest. ⚠ Those two are in tension and the tension is a finding
rather than something to resolve by judgement: an allowlist is the same computed
value on a different surface.

⭐ **The question is recorded in [`PROGRESS.md`](PROGRESS.md) with a
recommendation.** Everything in the Approach that does not depend on that answer
is built.

⚠ **And one thing found by building it.** Adding an artefact class to the
assembler made `check-data-branch` fail, because the published branch predates
the new files. Nothing published was wrong; the check could not tell "behind"
from "diverged". `PUB-14` is that entry.

---

## PUB-05. Language packages that embed the corpus

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P2, **Effort** L, **Status** open

### Problem

Fetching and parsing a corpus is work. A dependency line is not.

### Premise

Believed. ⭐ Stated in the design brief as the single biggest quality-of-life
win in the project.

### Approach

One package per major ecosystem, each embedding the corpus at a pinned release
and published on the same schedule as the release. The Rust one is `LIB-01` and
is the reference implementation; the others follow its shape.

Each package states which corpus release it embeds, in a field a program can
read, so a consumer can tell how old their data is without leaving their own
language.

Must not: publish a package that fetches at runtime by default. A package that
needs the network to answer is a package that fails in the environment its
consumers care most about.

### Prove

```bash
sh scripts/common/check-packages.sh
```

Passing means: each package builds offline, reports the corpus release it
embeds, and a test asserts that release matches the one the build was cut from.

---

## PUB-06. A packet capture per profile

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P3, **Effort** M, **Status** open

### Problem

Every network engineer already has a tool for one format, and this project does
not produce it.

### Premise

Believed, and it depends on whether the harness can synthesise one from the
recorded bytes rather than needing to have captured one.

### Approach

Synthesise a capture file per profile from the raw bytes the profile already
carries, so it is a generated format like any other rather than a second
capture path.

⚠ Establish first whether a synthesised capture is faithful enough to be worth
publishing. A capture with invented timing and invented sequence numbers may
mislead a reader who opens it expecting a real one, and if so the answer is to
label it as synthesised in the file itself.

Must not: publish a synthesised capture that is indistinguishable from a real
one.

### Prove

```bash
sh scripts/common/check-pcap.sh
```

Passing means: a synthesised capture parses in a standard tool, its
`ClientHello` bytes compare equal to the profile's raw hex, and the file carries
a comment identifying it as synthesised.

---


### ⛔ Ruled by the operator 2026-09-04: vendor a raw-socket route, and close this with `HARNESS-11`'s residue

⛔ **`HARNESS-11` measured the capability and it is one field of six.** The
source port is readable from safe std; the maximum segment size, the window size
and scale, the option order and the peer's hop limit are not, and
`unsafe_code = "deny"` at the workspace root makes reading them a dependency
question rather than a code one.

⭐ **The ruling: take the dependency, then add the whole TCP half at once.** No
schema version is spent on one weak field, and this entry and `HARNESS-11`'s
follow-on close together because they want the same bytes.

⚠ **`b_ids_harness::tcp` already carries the model**, with every field an option
and every absence carrying its reason, so the work is a route that fills them
rather than a shape to design.

## PUB-07. The licence stated in three places

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P1, **Effort** S, **Status** done

### Problem

A file that travels alone still has to say what it is. A consumer who downloads
one profile should not have to find this repository to learn they may use it.

### Premise

Believed. The licence choice itself is settled and its reasoning is in
[`../README.md`](../README.md).

### Approach

State it in three places, and have a check assert all three agree:

- a `LICENSE` file on the data branch;
- a `"license"` field in every profile and in the index;
- a line in every release body.

⛔ **Separate the trees.** The harness, the emitters and any vendored upstream
live in the source repository under their own terms. The data branch and the
release assets carry only generated data. A consumer of the data never has to
reason about a vendored dependency's licence because none of it is in what they
downloaded.

⚠ **One caveat that is not settled and must not be published as though it
were.** A digest specification can carry its own terms. Nothing in this project
emits a restricted digest variant until `VALID-04`'s licence question has a
written answer.

Must not: put a licence identifier in a profile that disagrees with the branch's
`LICENSE` file. A check holds that.

### Prove

```bash
sh scripts/common/check-license-consistency.sh
```

Passing means: the three statements agree, and a fixture where one disagrees
fails with a message naming all three.

---

### ⭐ Closed 2026-09-03. Seven statements, one home, and the six that predate the field

⛔ **The identifier has ONE home**, `b_ids_schema::LICENSE`, and every other
statement is generated from it or checked against it. A check carrying its own
copy would be an eighth place for it to disagree, so both halves read the
constant out of the source file.

#### The acceptance

```text
$ sh scripts/common/check-license-consistency.sh
licence ok: every statement says 0BSD.

  crates/b-ids-schema/src/lib.rs: 0BSD
  Cargo.toml: 0BSD
  crates/b-ids-schema/schema/browser-profile-1.schema.json: 0BSD
  corpus/v1/index.json: 0BSD
  corpus profiles: 0 carrying it, 6 published before the field existed
  crates/b-ids-corpus/tests/notes.rs: asserted by notes_the_release_body_states_the_licence
  crates/b-ids-schema/tests/profile.rs: asserted by profile_a_freshly_written_one_carries_the_licence

⚠ 6 profile(s) were published before the field existed and do not carry it.
  The corpus is append-only, so they never will.
rc=0
```

Both halves agree, and each reads the seven statements itself:

```text
{"schema":"check-license-consistency/1","license":"0BSD","stated":7,"profiles":6,"carrying":0,"predating":6,"problems":0}
```

#### ⛔ The six published profiles do not carry the field, and never will

⚠ **This is the entry's honest limit and it is reported rather than
repaired.** The corpus is append-only, so adding `license` to a published
profile would be an edit of a published file, which
[`../docs/AGENTS.md`](../docs/AGENTS.md) forbids and `check-corpus`'s history
leg refuses. The field is therefore:

| | |
| --- | --- |
| ⭐ **defaulted on the model** | reading a profile written before the field fills in the project licence, so a consumer using this crate always gets an answer |
| ⛔ **OPTIONAL in the published JSON Schema** | a schema requiring it would refuse every profile in the corpus. The check asserts that it is NOT in `required`, so nobody tightens it by accident |
| ⭐ **written literally by every profile from today** | and that is the leg that can actually fail now |

⛔ **A loop over the published corpus therefore proves nothing today**, and
saying so is the point rather than a caveat. The rule that CAN be broken is what
the writer emits, and `profile_a_freshly_written_one_carries_the_licence`
asserts both directions: a fresh profile carries the identifier, and a profile
with the field removed still reads back.

#### Where the licence is stated, and why each place

| place | who sees it |
| --- | --- |
| `LICENSE` at the repository root | somebody who opens the repository |
| `Cargo.toml` | a builder of the code |
| ⭐ `b_ids_schema::LICENSE` | **the one home.** Everything generated reads it |
| the published JSON Schema | a consumer validating a profile |
| `corpus/v1/index.json` | a consumer who fetches only the index |
| every profile written from today | ⭐ a consumer who fetches ONE file, which is what the entry is about |
| the release body | a consumer who downloads an asset |

⛔ **The data branch is NOT checked, because it does not exist.** `PUB-02` is
the entry that creates it, and reporting a pass over a branch nobody has made is
the "step that exits 0 having done nothing" row of
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md).
The check says so in its own header rather than counting it.

#### The guard mutation, each exit code read unpiped

| planted | what went red |
| --- | --- |
| `Cargo.toml` states `MIT` | `Cargo.toml says MIT and crates/b-ids-schema/src/lib.rs says 0BSD`, exit **1** |
| a fixture schema stating `MIT`, run through the same comparison | the `--fixture` leg, which asserts the comparison REFUSES it rather than only that the fixture differs |
| the release-body case removed from its suite | the check names the case, so a deleted test is caught rather than passed over |

⚠ **The mutation on `Cargo.toml` was a copy-and-restore on this machine**, and
the file was compared byte for byte with its pre-mutation copy afterwards.

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| the `LICENSE` on the data branch | `PUB-02`. The check's header names it as the leg it will grow. |
| a restricted digest variant | ⛔ Nothing emits one, and `VALID-04`'s licence question comes first. That caveat was in this entry and it is still open. |
| a licence on the vendored tree | Each vendored tree is under its own terms, and the data this project publishes carries none of it. |

---

## PUB-08. One generator for the release body and the changelog

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P2, **Effort** S, **Status** done

### Problem

Release notes and a changelog that are written separately drift, and the reader
who trusts the wrong one is the one who was doing something careful.

### Premise

Believed.

### Approach

One generator, two outputs, so the two cannot disagree by construction rather
than by discipline. The same code writes the automated pull request body in
`CI-04`.

What the body must contain, so a reader can answer "what changed and how do I
know" without leaving the page: what changed per browser, channel and platform,
**field-level** rather than "updated a browser"; new and superseded profiles;
the validator's summary; capture provenance for every profile that moved; and
the run identifier.

Must not: write a release body by hand, and must not let the changelog be
generated from a different query than the body.

### Prove

```bash
sh scripts/common/check-notes-generator.sh
```

Passing means: the generator is run twice over one corpus change and produces a
release body and a changelog entry that agree field for field, and a fixture
where they are generated from different inputs fails.

---

### ⭐ Closed 2026-09-02. One model, two renderers, and the comparison can fail

⛔ **The two cannot disagree by construction rather than by discipline.**
`b_ids_corpus::notes::model` computes what changed between two corpus states,
and `release_body` and `changelog_entry` are the only things that turn it into
text. Neither computes anything of its own.

```text
$ sh scripts/common/check-notes-generator.sh
notes generator ok: 6 case(s). The release body and the changelog entry
  are rendered from one model, they carry every fact it holds, a no-op
  change renders nothing, and two outputs from different inputs are
  asserted NOT to agree.
exit=0

$ sh scripts/common/check-notes-generator.sh --json
{"schema":"check-notes-generator/1","cases":6,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-notes-generator.ps1 -Json
{"schema":"check-notes-generator/1","cases":6,"problems":0}
```

#### What makes "agree" checkable rather than a sentence

⭐ **`notes::facts` returns every fact the model holds**, and the suite asserts
both renderings contain every one of them. ⛔ That is a function rather than a
paragraph, so a renderer that quietly dropped a movement fails a test rather
than passing a review.

| assertion | why it is there |
| --- | --- |
| both outputs carry every fact | the entry's requirement, made mechanical |
| a no-op renders **nothing** | ⛔ silence is the correct output for "the browser did not change". A bot that writes on a schedule trains people to ignore it, and `CI-04` states the same rule |
| a version move is **field-level** | "updated a browser" is what a reader cannot act on. ⚠ The fields come from `b_ids_validator::diff` rather than a second comparison here |
| a first profile is an **addition**, not a diff | listing every field of a first profile as changed would be a diff against nothing |
| two runs produce identical text | a body that diffs on every run is one nobody can review |
| ⛔ two outputs from **different inputs** do NOT agree | the negative case the acceptance names. A check that only ever sees agreement is one nobody knows works |

#### ⛔ The pair earned itself before it was registered

⚠ **The two halves disagreed the first time they were run against each other**,
and the cause is one this tree's own conventions warn about:

```text
$ sh scripts/common/check-notes-generator.sh --json
{"schema":"check-notes-generator/1","cases":6,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-notes-generator.ps1 -Json
{"schema":"check-notes-generator/1","cases":0,"problems":0}
```

⛔ **A backslash was lost crossing a shell**, so the PowerShell half's
`'^running (\d+) tests'` arrived as `'^running (d+) tests'`, matched nothing and
reported zero cases.
[`../docs/conventions/shell.md`](../docs/conventions/shell.md) section 1 names
exactly this: "A backslash escape that survives one hop loses a backslash on the
next." ⚠ It was written by passing a payload through a shell, which is what that
section says not to do, and it was found by running both halves rather than by
reading them.

⭐ **Neither half's answer was wrong about the suite**; they were wrong about
each other, which is the drift a twin comparison exists to catch and the reason
`check-notes-generator` has a row in `check-twins` rather than only a gate line.

#### ⚠ What this does not do

⛔ **It writes no release and no changelog entry into the tree.** There is no
release to write notes for: `PUB-01` is that surface and it does not exist.
⭐ What exists is the generator both it and `CI-04` will call, proved before
either of them can be wrong about it.

⚠ **And the model compares two corpus STATES rather than reading git.** A caller
holding a before and an after gets an answer; deriving those two states from a
revision range is the caller's job and `CI-04` is where it lands.

---

## PUB-09. Signed and attested captures

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P2, **Effort** M, **Status** open

### Problem

A consumer currently has to take this project's word that a value was measured
rather than typed. Provenance in a file is a claim by whoever wrote the file.

### Premise

Believed, and there is a caution worth carrying: a checksums file published in
the same release as the artefact proves transport rather than authorship,
because whoever could replace one could replace the other. A signature the
consumer verifies independently is the check that proves authorship, and it
applies on top.

### Approach

Sign the index and attest the build, so a consumer can distinguish a capture
from an assertion without trusting a file that travelled with the artefact.

Publish what verifying looks like, as a command a consumer runs, next to the
route it verifies.

Must not: describe a checksums file as proof of authorship. It is not, and
saying so is the kind of claim this project exists to stop making.

### Prove

```bash
sh scripts/common/check-signing.sh
```

Passing means: a published index verifies against the project's key, a corpus
file altered after signing fails verification, and the published verification
command is the one the check runs.

---


### ⛔ Ruled by the operator 2026-09-04: keyless attestation, and no signing key

⭐ **The runner's own OIDC identity signs**, so no long-lived key exists and no
workflow names a secret. ⛔ That preserves the property the record states about
this tree: nothing in it needs a credential.

⚠ **And it changes what `DOC-02` is waiting for.** That entry's trigger is a
workflow needing a secret, a release needing a signing key, or a capture lane
needing a machine somebody sets up. ⛔ Keyless attestation makes none of the
first two true, so `DOC-02` is now waiting on the third alone, which the
vendored `certutil` in `DRIVER-11` and the raw-socket route in `PUB-06` are the
candidates for.

## PUB-10. Nothing triggers the two surfaces that were built to publish

**Source** the operator, ruled 2026-09-03 after the previous session wrote its record
**Category** publish, **Priority** P1, **Effort** L, **Status** done

### Problem

`PUB-01` assembles a release and checks it. `PUB-02` assembles the data branch
and checks it. Both closed with the same sentence: nothing cuts a tag, uploads
an asset or creates a branch. So a consumer who reads this project's own
documentation about its published routes finds no published route, and the
whole publishing half of the project is a directory this repository can build
and has never handed to anybody.

### Premise

⭐ **Measured, and the numbers are in the two closings.** `b-ids-corpus publish`
assembles 197 artefacts and two builds are byte-identical; both checks are green
and in the gate. What is missing is an event.

### Approach

One workflow, [`../.github/workflows/publish.yml`](../.github/workflows/publish.yml),
triggered the three ways the operator ruled: `workflow_dispatch`, a push to the
default branch, and a pushed tag. One job assembles and runs both existing
checks with `contents: read`; two jobs publish, and only those two declare
`contents: write`.

- ⛔ **The data branch is append-only and never force-pushed.** Two conditions
  from two sources: `b_ids_corpus::publish::would_rewrite`, reached through a
  new `b-ids-corpus data-branch` command, and the remote's own refusal of a
  push that is not a fast-forward.
- ⛔ **The release rules stay in the crate.** A new `b-ids-corpus release`
  parses the pushed tag, plans it against the tags that already carry a
  release, rebuilds it from its own parts, and writes the body with `PUB-08`'s
  renderer.
- ⭐ **A check for the workflow itself**, twinned, so a file that grew a force
  push or moved its write to the top fails a gate rather than a branch.

Must not: hold a personal access token, force-push anything, or restate a rule
the crate already holds.

### Decision

**Whether the release job moves the `v1` and `latest` git tags.**
`publish::moving_tags` names them and `PUB-01`'s approach asks for them, so a
script can fetch without listing.

⛔ **Recommendation, and the one taken: it does not.** A moving tag is a
force-update of a ref, and this session was told in the same breath that no
session force-pushes this remote. `gh release create --latest` gives a consumer
the same property through a `releases/latest/download/` path, with no ref moved
and no listing. The alternative lost because it buys nothing a consumer can use
that the release pointer does not already give them, at the price of the one
operation this project has ruled against.

### Consumers

⚠ **Nothing is fetched from this repository yet**, which is the state this entry
ends. After the first run there are two: whoever clones or fetches the `data`
branch, and whoever downloads a release asset. ⛔ Both contracts are established
by the first push and are expensive to change afterwards, which is why the
routes, the manifest and the checksums were built and checked before any of this
was triggered.

### Prove

```bash
sh scripts/common/check-publish.sh
```

Passing means: the workflow declares all three triggers, grants `contents: read`
at the top and `contents: write` on exactly the two publishing jobs, names no
secret, carries no force push, consults the rewrite rule before the push, runs
both existing checks in a job the publishing jobs need, and reads the archive
epoch from `check-release.sh`; and the ten refusal paths driven against the
built binary each exit as expected.

---

### ⭐ Closed 2026-09-03. The trigger exists, and the two conditions in front of it are not one condition twice

#### The acceptance

```text
$ sh scripts/common/check-publish.sh
publish ok: 3 trigger(s) over 3 job(s), 2 job-scoped write(s),
  no force push and no named secret, and 10 refusal(s) driven against the binary.
  ⛔ Nothing was tagged, uploaded or pushed.
rc=0
```

Both halves agree, and each reads the workflow itself rather than wrapping one
reading: the POSIX half in `awk`, `sed` and `grep`, the PowerShell half in
`-match` over the file's lines.

```text
$ sh scripts/common/check-publish.sh --json
{"schema":"check-publish/1","triggers":3,"jobs":3,"writes":2,"cases":10,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-publish.ps1 -Json
{"schema":"check-publish/1","triggers":3,"jobs":3,"writes":2,"cases":10,"problems":0}
```

#### ⛔ A defect in the assembler, found by running it two ways

⚠ **The published route manifest carried the absolute path of whoever built
it.** `publish::build` handed `routes()` the path string it got from the
caller's `--root`, so a build under a relative root and a build under an
absolute one produced different bytes for one corpus:

| built with | what `routes/routes.json` recorded | total |
| --- | --- | --- |
| a relative root | `corpus/v1/chrome/stable/linux64/151.0.7922.173.json` | 666710 bytes |
| an absolute root | that same path with the whole checkout prefix in front of it | 668384 bytes |

⛔ **Two consequences, and the second is worse than the first.** The build is
not reproducible across machines, which is the property `PUB-01` asserts; and a
public data branch and a public release asset would have carried the operator's
own home directory. ⭐ `check-release` could not see it, because it builds twice
under **one** root and compares those two.

Fixed by giving the route generator the same `relative_to` every artefact is
placed with, and `publish_the_tree_names_no_path_outside_itself` is the case
that keeps it fixed. ⚠ The 668384 in `PUB-01`'s closing is the absolute build;
666710 is what the corpus derives to now, and the difference is the path prefix
alone.

#### The two conditions, and why they are two

⛔ **A guard on something irreversible is two conditions from two sources.** A
force push over the data branch discards every commit a consumer pinned.

| whose condition | what it is |
| --- | --- |
| this project's | `b-ids-corpus data-branch --head H --parent P`, which is `would_rewrite` with a command around it. Exit 1 refuses. |
| the environment's | the push carries no `--force`, no `--force-with-lease` and no `+`, so the remote rejects anything that is not a fast-forward |
| ⭐ and one that is neither | the commit that landed is read BACK from the remote, and its tree object compared with the one that was built. One tree object is one set of bytes. |

⚠ **The commit is built with plumbing over a temporary index**, so the checkout
is never touched and a step that fails leaves the working tree as it was. A run
whose tree object matches what the branch already holds pushes nothing, which is
the same rule `CI-04` holds about a no-op change.

#### The guard mutation, each exit code read unpiped

⛔ **Every mutation was made against a copy under the ignored scratch
directory, and the live file was compared byte for byte with that copy
afterwards.**

| planted | what went red |
| --- | --- |
| the `workflow_dispatch` trigger removed | `does not declare workflow_dispatch`, exit **1** |
| `contents: write` moved to the top of the file | `the top-level permissions block does not grant contents: read`, exit **1** |
| the push given `--force` | `1 git push line(s) carry a force flag`, exit **1** |
| ⛔ the refspec given a leading `+` | ⛔ **nothing. The check passed.** Fixed, and then exit **1** in both halves |
| a secret named instead of the run's own token | `the workflow names a secret`, exit **1** |
| the rewrite rule removed | `no step calls b-ids-corpus data-branch`, exit **1** |
| `check-data-branch` no longer run | `a tree that fails it would still publish`, exit **1** |
| the tar epoch typed instead of read | `does not use the epoch it read`, exit **1** |
| one job no longer needing the assemble job | `1 job(s) need the assemble job where 2 ... are expected`, exit **1** |
| the workflow deleted | `there is no .github/workflows/publish.yml`, exit **1** |
| `would_rewrite` accepting an orphan over an existing branch | `exit 0 where 1 was expected`, from the drive rather than from the suite |
| the assembler handed the caller's path again | `routes/routes.json names the builder's own filesystem` |

#### ⛔ The finding: this session's own guard passed the mutation it was written for

⚠ **The force-push rule looked for `:+`, which is where a `+` is not.** A
forcing refspec is `+src:dst`, so the plus sits at the START of the token, and
the mutation that gave the push a leading `+` was reported clean by both halves.
⛔ The rule is now that any `+` on a `git push` line is a force, which is blunt
and correct: that line takes a remote and refspecs, and a `+` in any of them
means force. ⭐ It is the same class as the previous session's SQLite finding,
and it is the argument for planting the mutation rather than reading the code.

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a run | ⛔ Nothing was published from this machine. No tag was created, no asset uploaded and no branch pushed: this entry is the trigger, and the first run is whatever event reaches it. |
| the moving `v1` and `latest` git tags | ⛔ Declined above, with the reason. `moving_tags` stays in the crate with its case. |
| a build provenance attestation | `PUB-09`, which needs a signing identity. |
| the checks that read the corpus from the working tree | `PUB-11`, and the operator ruled the order: the branch is published and verified first. |

---

## PUB-11. Every check reads the corpus from the working tree, and the corpus is leaving it

**Source** the operator, ruled 2026-09-03
**Category** publish, **Priority** P2, **Effort** M, **Status** done

### Problem

`corpus/` and `raw/` are to leave the default branch once the data branch
carries them. Eleven checks read them from the working tree today, so removing
them turns eleven green checks into eleven that verify nothing or refuse to run,
and the tempting fix for each is to widen it until it passes.

### Premise

⭐ **Measured 2026-09-03**, over the POSIX halves, and each has a PowerShell
twin that follows it:

```bash
grep -lE '(corpus|raw)/v1|b-ids-corpus.*--root|\$BIN.*--root' scripts/common/check-*.sh
```

```text
check-corpus check-coverage check-data-branch check-formats
check-license-consistency check-publish check-release check-routes
check-staleness check-trust-anchors check-validate
```

⚠ **Eleven, not the fifteen the ruling names**, and the difference is where the
line is drawn rather than a disagreement: the prose checks and the secret scan
walk every tracked file and would see fewer of them rather than fail, and two
more read fixtures instead of the corpus. ⛔ The number is re-measured by the
command above rather than carried from here.

### Approach

Give each of them one way to reach a corpus that is not "the working tree", and
change them together rather than one at a time.

- ⭐ **One resolver, not eleven.** A single helper that answers "where is the
  corpus", preferring an explicit root, then a fetched copy of the data branch,
  then the working tree. `Store::at` is already the one read path in the crate;
  this is its equivalent for the scripts.
- ⚠ **A check that cannot reach a corpus exits 2**, which is "could not run",
  and never 0. That distinction is the whole reason this entry is not a
  one-line deletion.
- ⛔ **Nothing is deleted in this entry.** It ends with every check reading from
  the branch and the working tree still holding the corpus, so the step that
  removes it is separately reversible.

Must not: bake a URL into a check, and must not make a gate depend on the
network. A fetched copy is a local clone of the branch, and the absence of one
is exit 2.

### Decision

⛔ **Ruled by the operator 2026-09-03, and it is an ORDER rather than a fork.**
The data branch is pushed and verified byte for byte first; then this entry
moves the checks; then `corpus/` and `raw/` leave the default branch. Each step
is reversible until the last.

### Consumers

⚠ **This entry does not change what is published**, so it breaks nobody. ⛔ The
step AFTER it does: a consumer reading `corpus/v1/` from the default branch over
raw file serving would 404, which is the reason `PUB-02` exists and the reason
the branch is published before anything is removed.

### Prove

```bash
sh scripts/common/check-gate.sh --fast
```

Passing means: the gate is green with `corpus/` and `raw/` moved out of the
working tree and a local copy of the data branch present, and every one of the
eleven checks above reports on the profiles it found there rather than skipping.

### ⚠ Worked 2026-09-03 and NOT closed. What landed, and what is left

⛔ **This entry stays open**, with the residue measured and named at a file and
a line rather than described. What follows is the state the tree is in, not a
plan.

**The resolver exists and is the one answer.**
[`../scripts/common/corpus-root.sh`](../scripts/common/corpus-root.sh) and its
twin resolve `B_IDS_CORPUS_ROOT`, then the working tree, then a materialised
copy of the data branch, and print one path.

```text
$ sh scripts/common/corpus-root.sh --json
{"schema":"corpus-root/1","source":"working-tree","ref":"","profiles":6}

$ sh scripts/common/corpus-root.sh --fixture
corpus-root fixture ok: a tree with no corpus resolves to the data branch,
carrying 6 profile(s).
```

⛔ **The order in the Approach above is wrong and was not followed.** It puts the
branch before the working tree; that would have every check read the PUBLISHED
corpus while a session is adding a profile to the working one, and report green
over the corpus it is about to publish. The resolver prefers the working tree
for exactly as long as that holds a corpus. ⚠ The Approach keeps its wording and
this is the correction, by the rule in
[`../docs/methodology/authoring.md`](../docs/methodology/authoring.md).

**Twelve check pairs resolve rather than assume**, and the root is EXPORTED as
well as passed, because
[`../crates/b-ids/build.rs`](../crates/b-ids/build.rs) embeds the corpus at
build time from that same variable and already calls it the seam this entry
needs. A check that resolved a root and did not export it would build against
one corpus and report on another.

### ⭐ Driven with `corpus/` and `raw/` moved out of the working tree

⛔ **The measurement, not a prediction.** Both directories were moved to a
scratch copy, the checks were run, and both were restored and compared against
`HEAD` byte for byte afterwards.

| | |
| --- | --- |
| resolve off the branch and pass | `check-coverage`, `check-corpus`, `check-formats`, `check-routes`, `check-support-matrix`, `check-license-consistency`, `check-publish` |
| ⚠ still fail | `check-validate`, `check-release`, `check-trust-anchors` |
| ⛔ refuses by design | `check-data-branch`, which exits 2 rather than comparing the branch against itself |

### ⛔ What is left, and it is one shape rather than three

**The Rust side has the coupling the scripts just lost.** `check-release` and
`check-validate` fail on their suite and determinism legs, and
`check-trust-anchors` finds no carrier, because each of those legs reaches the
corpus through code that resolves the WORKSPACE root rather than
`B_IDS_CORPUS_ROOT`:

- [`../crates/b-ids-corpus/tests/publish.rs`](../crates/b-ids-corpus/tests/publish.rs),
  run by `check-release`, opens the corpus relative to the crate;
- the determinism leg of `check-validate` re-runs the generator inside a scratch
  root it copied, and the copy is now of the resolved root while the run is not;
- `check-trust-anchors` generates lists with `--root "$CORPUS_ROOT"` and then
  looks for carriers in a walk that the materialised copy does not satisfy.

⭐ **What would close this entry**: give those three legs the same resolved root
the rest of the check already has, then re-run the driven pass above and get ten
of ten. ⛔ It is not a blocker on anything outside this tree and needs no
ruling.

⚠ **And one thing the operator's sequence has to settle before the step AFTER
this one.** Once `corpus/` leaves the default branch, the data branch becomes
the canonical corpus rather than a derivation of it, and `check-data-branch`'s
comparison stops meaning anything. That is a question about where a NEW capture
is written, and it is recorded in [`PROGRESS.md`](PROGRESS.md) rather than
answered here.

### ⭐ Closed 2026-09-04. Ten of ten, and two checks that passed by comparing something to itself

⛔ **The three legs named above are fixed at their cause**, and the driven pass
that measured them now reports ten of ten. ⚠ Two further defects were found by
running it, both of the same shape and neither in the three.

**The three legs.**

| leg | what reached the working tree | what it reads now |
| --- | --- | --- |
| [`../crates/b-ids-corpus/tests/publish.rs`](../crates/b-ids-corpus/tests/publish.rs) | `repository_root()`, which walked up from `CARGO_MANIFEST_DIR` and was BOTH the build source and the scratch destination | `corpus_root()` for the source, reading `B_IDS_CORPUS_ROOT` in `b-ids/build.rs`'s own order, and `workspace_root()` for the destination |
| [`../scripts/common/check-validate.sh`](../scripts/common/check-validate.sh) determinism leg | `RAW_DIR` relative to the repository, so the scratch copy got a branch corpus and no raw bytes | `"$CORPUS_ROOT/$RAW_DIR"`, and its PowerShell twin the same |
| [`../scripts/common/check-trust-anchors.sh`](../scripts/common/check-trust-anchors.sh) carrier walk | `find "$REPO_ROOT/corpus"` while the publisher above it took `--root "$CORPUS_ROOT"` | `find "$CORPUS_ROOT/corpus"`. ⚠ The PowerShell twin already did, so the halves disagreed and only a run with the corpus moved out could show it |

### ⛔ The two the driven pass found, and they are worse than the three

⭐ **A check cannot pass by comparing something to itself, and two could.** Both
had the identical cause: the check asked the resolver a second question AFTER
exporting `B_IDS_CORPUS_ROOT`, and the resolver's first rule is that an explicit
root is never second guessed. The export disarmed the guard on the line below
it.

```text
$ sh scripts/common/corpus-root.sh --ref
refs/remotes/origin/data
$ B_IDS_CORPUS_ROOT="$(sh scripts/common/corpus-root.sh)" sh scripts/common/corpus-root.sh --json
{"schema":"corpus-root/1","source":"explicit","ref":"","profiles":6}
```

- ⛔ **`check-data-branch` compared the published branch against a materialised
  copy of that same branch and reported green.** Its own header says it must
  exit 2 rather than do that. Driven with `corpus/` moved out, before the fix:
  `data branch ok: 200 file(s) regenerated, identical over two builds`, exit 0.
- ⛔ **`check-corpus` asked THIS repository's history about files that are not
  in it.** Its one irreplaceable leg asks whether a published file was ever
  modified; with the ref emptied it read `main` instead of the data branch and
  reported `nothing edited after publication` for the wrong reason.

⭐ **`--ref` was the wrong question and a new one answers it.** An empty ref
means the working tree answered OR the caller named a root, which are different
facts. `corpus-root.sh --source` and its `-Source` twin print which of the three
rules answered, and `check-data-branch` now refuses anything that is not
`working-tree`. `check-corpus` keeps `--ref`, asked before the export.

### ⭐ Driven with `corpus/` and `raw/` moved out of the working tree

⛔ **The measurement, not a prediction.** Both directories were copied to the
ignored scratch directory and removed, the checks were run, and both were
restored and compared against `HEAD` afterwards.

```text
check-coverage               exit=0
check-corpus                 exit=0
check-formats                exit=0
check-routes                 exit=0
check-support-matrix         exit=0
check-license-consistency    exit=0
check-publish                exit=0
check-validate               exit=0
check-trust-anchors          exit=0
check-release                exit=0
---
ten checks: 10 passed, 0 failed
check-data-branch: exit=2 (2 = refuses by design, and now it does)
```

```text
$ git status --porcelain corpus raw
corpus/ and raw/ are byte-identical to HEAD
```

⚠ **And the guard was checked for over-refusing**, because one that always
refuses is as useless as one that never does. With the corpus restored,
`check-data-branch` compares and passes, naming the tree it compared:

```text
data branch ok: 200 file(s) regenerated, identical over two builds,
  198 of them with a checksum in the manifest and in SHA256SUMS, and no
  source, vendored dependency or reference corpus among them.
```

⚠ **The line after those names the tree object both sides resolved to**, and it
is quoted here rather than in the block because a bare 40-hex run trips the
secret scan's public rules while one in a code span does not. The check reported
the remote data branch at tree `6d0b4c1703f3da2294ec3fb3f9654f6a042126c3`, which
is what this corpus derives to, and one tree object is one set of bytes.

### ⚠ And the count in the paragraph above has already moved

⛔ **"Twelve check pairs" was true the day it was written and is not now.**
`PUB-04` added a thirteenth, and the number will move again. ⭐ The claim audit
found it in [`../scripts/README.md`](../scripts/README.md) stating twelve in the
present tense about the tree, and that page reads it with a command now instead.

```bash
grep -l corpus-root.sh scripts/common/check-*.sh
```

### The acceptance

```text
$ sh scripts/common/check-gate.sh --fast
gate ok: 38 passed, but 1 SKIPPED on this host: check-twins
A skipped check is not a passed check. CI runs on two hosts that between
them have every tool; that is where the coverage for these comes from.
```

### ⚠ One residual, measured and named

⛔ **A root named explicitly still makes `check-corpus`'s history question
ambiguous.** With `B_IDS_CORPUS_ROOT` set by a caller,
[`../scripts/common/check-corpus.sh`](../scripts/common/check-corpus.sh) line 90
gets an empty ref and reads this repository's history, which is right when the
explicit root IS this working tree and unanswerable when it is not. ⚠ It is not
reachable from the gate, which never sets the variable before a check resolves.

⭐ **It is carried by `PUB-13`**, which is where the operator's 2026-09-04
ruling put the question of which branch holds the canonical corpus. The history
to read is a property of that answer rather than of this entry.

---

## PUB-12. The licence check declines the one surface a consumer actually fetches

**Source** found by this session's consolidation pass, 2026-09-03, reading the check headers against the tree
**Category** publish, **Priority** P2, **Effort** S, **Status** done

### Problem

[`../scripts/common/check-license-consistency.sh`](../scripts/common/check-license-consistency.sh)
compares six places that state this project's licence and skips the data
branch, saying in its own header that the branch does not exist. ⛔ **It does**,
and it carries a `LICENSE` file nothing compares against the one home.

### Premise

⭐ **Read from the branch, 2026-09-03.**

```bash
git ls-tree --name-only origin/data
```

The branch's root carries a `LICENSE`, and the check's header says the file "is
this file's to check on the day it does" exist. ⚠ That day was 2026-09-03, and
the sentence stayed. This is the same shape `check-data-branch` was corrected
for in the same session: a leg whose condition had been met still reporting a
skip.

### Approach

Add the branch to the places compared, under the rule the check already uses
for a subject it cannot reach.

- ⭐ **A local ref, never the network.** `refs/heads/data` then
  `refs/remotes/origin/data`, which is what `check-data-branch` already does,
  so a gate does not acquire a network dependency.
- ⚠ **No branch at all is a SKIP naming the branch**, not a pass and not a
  failure. A clone with no data branch fetched is a machine that cannot answer
  the question.
- ⛔ **Both halves**, and a row in `check-twins`, because a leg added to one
  half is a second behaviour.

⛔ Must not: fetch. A check that reaches the network is a check that is red when
somebody else's host is down.

### Prove

```bash
sh scripts/common/check-license-consistency.sh --json
```

Passing means: exit 0, and the JSON names the data branch among the places
compared with the licence it carries, rather than omitting it.

### Closing

**Closed 2026-09-03T13:07:00Z.** Two legs, in both halves, over a local ref.
The schema moves to `check-license-consistency/2` because the object gained a
field.

```text
$ sh scripts/common/check-license-consistency.sh --json
{"schema":"check-license-consistency/2","license":"0BSD","stated":8,"profiles":6,"carrying":0,"predating":6,"data_branch":"0BSD","problems":0}
exit=0
```

⭐ **Two legs rather than one, and the second is the one an identifier cannot
see.** The manifest's `license` field is compared against the one home, and the
`LICENSE` at the branch's root is compared as a git object against this tree's
own, so a branch naming `0BSD` over somebody else's licence text is a failure
rather than a pass.

### ⛔ It was driven against a branch that disagrees

**A local `data` branch was built off `origin/data` with its manifest rewritten
to `MIT`**, using a temporary index so nothing in this repository's own index
moved, and deleted afterwards. ⛔ Nothing was pushed and the published branch was
not touched.

```text
$ sh scripts/common/check-license-consistency.sh
licence check failed, 1 problem(s):

  the data branch manifest says MIT and crates/b-ids-schema/src/lib.rs says 0BSD
exit=1
```

⭐ **The PowerShell half read the same branch and exited 1 with
`"data_branch":"MIT","problems":1`**, which is what says both halves grew the
leg rather than one of them.

A second run of the same fixture, with the branch's `LICENSE` replaced as well,
produced **two** problems: the identifier and the text are separate findings.

### ⚠ What the leg cannot answer, and it says so rather than passing

⛔ **A clone that has not fetched the branch gets a SKIP naming it**, not a pass
and not a failure. The check reads `refs/heads/data`, then
`refs/remotes/origin/data`, and never fetches: a gate that reaches the network is
red the day somebody else's host is down.

⚠ **So on a fresh CI checkout this leg reports `skipped`**, and that is visible
in the JSON rather than hidden. Making it fetch would be the wrong repair; the
right one is a checkout that fetched the branch, which is what
`check-data-branch` already needs for its own comparison.

---

## PUB-13. The corpus moves to a source branch, and the default branch carries neither

**Source** the operator, ruled 2026-09-04, answering `PROGRESS.md`'s open question 1
**Category** publish, **Priority** P2, **Effort** L, **Status** open

### Problem

`PUB-11` moved every check off the working tree, so `corpus/` and `raw/` can now
leave the default branch. ⛔ **The step after it has an unanswered question in
it, and removing the directories without answering it breaks two things.**

- `check-data-branch` compares the published branch against what the canonical
  corpus derives to. With no canonical corpus in the tree there is nothing to
  compare against, and the check refuses rather than comparing the branch
  against itself.
- The capture workflow adds a profile by writing `corpus/v1/...` in the working
  tree and committing it. With the directory gone there is nowhere to write.

### Premise

⚠ **Both halves were measured by `PUB-11`'s driven pass on 2026-09-04**, not
predicted. With `corpus/` and `raw/` moved out, ten checks resolve off the
branch and pass, and `check-data-branch` exits 2 naming `data-branch` as what it
resolved to.

⛔ **And the refusal only works because `PUB-11` fixed it.** Before that, the
same run reported `data branch ok: 200 file(s) regenerated, identical over two
builds` and exited 0, having compared the branch against a copy of itself.

### Decision

⛔ **Ruled by the operator 2026-09-04.** The corpus moves to a dedicated SOURCE
branch. The default branch carries neither `corpus/` nor `raw/`, a capture opens
its pull request against the source branch, and the data branch is derived from
the source branch rather than from the default one.

⚠ **The two alternatives and why they lost.** Keeping `corpus/` on the default
branch until a capture path writes to the data branch was the recommendation
attached to the question; it is the smallest change and it leaves the default
branch carrying data indefinitely, which is the thing the sequence exists to
end. Having the capture workflow commit straight to the data branch removes the
comparison entirely: the branch becomes canonical, and `check-data-branch` has
nothing independent left to check it against.

⭐ **What the ruling buys is that the comparison survives.** Three branches
means the data branch is still a DERIVATION of something, so the check that
asks whether it matches keeps a real question to ask.

### Approach

⛔ **Sequenced, and every step reversible until the last.** This entry is not a
single change.

1. **Name the source branch and create it** carrying `corpus/` and `raw/` as the
   default branch holds them today, verified tree for tree the way
   `PUB-02` verified the data branch.
2. **Teach the resolver a third rule.**
   [`../scripts/common/corpus-root.sh`](../scripts/common/corpus-root.sh)
   resolves an explicit root, then the working tree, then the data branch. The
   source branch belongs between the working tree and the data branch, and
   `--source` must name it, because `check-data-branch` refuses anything that is
   not the canonical corpus and the canonical corpus is about to stop being
   `working-tree`.
3. **Point `check-data-branch` at the source branch** as the thing the data
   branch is compared against.
4. **Move the capture lane's pull request** to open against the source branch.
   [`../.github/workflows/capture.yml`](../.github/workflows/capture.yml).
5. ⛔ **Only then remove `corpus/` and `raw/` from the default branch.**
6. ⚠ **And fix what step 5 does to CI, which is not obvious.** `PUB-14` made
   the gate record `check-data-branch`'s designed exit 2 as a SKIP rather than a
   failure, which is right. ⛔ But the ubuntu job runs `--strict`, which fails on
   any skip, and the windows job asserts `$j.skipped -gt 1` because only
   `check-twins` may be one. With `corpus/` gone from the default branch that
   check skips on both, so both jobs go red on a state that is correct.
   ⭐ Found by the door sweep on 2026-09-04, before it could bite.

⚠ **And one thing `PUB-11` left, which this entry owns.**
[`../scripts/common/check-corpus.sh`](../scripts/common/check-corpus.sh) line 90
asks the resolver which ref carries the corpus, so it can ask the right history
whether a published file was ever edited. An explicitly named root answers with
an empty ref, and the check then reads this repository's history, which is right
only when the explicit root is this working tree. ⛔ Once the canonical corpus is
a branch, "which history" is this entry's question rather than an ambiguity to
leave.

Must not: make any check depend on the network, and must not give the capture
workflow a write to the data branch. ⭐ The data branch stays derived and
append-only.

### Consumers

⛔ **This is the breaking one, and it is why `PUB-02` exists.** A consumer
reading `corpus/v1/` from the default branch over raw file serving gets a 404
after step 5. The data branch has carried the same tree since 2026-09-03 and is
the surface a consumer is meant to fetch. ⚠ Nothing is known to fetch either
today.

### Prove

```bash
sh scripts/common/check-gate.sh --fast
```

Passing means: the gate is green with `corpus/` and `raw/` absent from the
working tree and a local copy of BOTH branches present, `check-data-branch`
comparing the data branch against the source branch rather than refusing, and
`check-corpus` reading the source branch's history. ⛔ Read the exit code from
the process that produced it, unpiped.

---


### ⛔ Ruled by the operator 2026-09-04: a session may create and push the source branch

⭐ **All six steps are a session's to run**, including creating the branch on
this repository's own remote and, at the end, removing `corpus/` and `raw/` from
the default branch.

⚠ **What that does NOT license, and the distinction is the whole of absolute
6.** Creating a branch is not rewriting one. ⛔ The data branch stays
append-only and is never force-pushed, the history reset on `main` remains the
operator's own action, and nothing here is written to any other repository.

⛔ **Verify before removing.** The source branch is compared tree-for-tree
against a local build the way `PUB-02` verified the data branch, and every step
before the last stays reversible.

## PUB-14. The data branch check cannot tell a branch that is behind from one that is wrong

**Source** found by `PUB-04`, 2026-09-04, on the first change to the assembler since the branch was published
**Category** publish, **Priority** P1, **Effort** M, **Status** done

### Problem

⛔ **`check-data-branch` compares two git tree objects and fails on any
difference**, so adding an artefact class to the assembler takes the gate red.
Nothing published is wrong when that happens: the branch simply carries fewer
files than the generator now produces, and the publisher adds them on the next
push.

⚠ **A red nobody can act on is a red that gets ignored**, and this one cannot be
cleared from a working tree at all: only a workflow run fixes it, and that
workflow runs AFTER the gate on the same push.

⛔ **And a second defect in the same area.** The check exits 2 when the canonical
corpus is not in this tree, which its own header calls its honest state and
which `CI-07` rules is not a failure. Both halves of the gate ran it through a
runner that fails on any non-zero, so the designed refusal would have taken the
gate red on the day `PUB-13` removes `corpus/` from the default branch.

### Premise

⭐ **Measured 2026-09-04, on the first assembler change since 2026-09-03.**
`PUB-04` added 37 files under `configs/`, and the difference between the
published branch and the regenerated tree was exactly that:

```text
published: 200  regenerated: 237
--- paths on the branch that the regeneration does NOT produce ---
--- count added ---
37
```

⛔ **Every one of the 198 real artefacts was byte-identical.** Only the two
files DERIVED from the artefact list differed, and they differed because the
list grew.

⚠ **`generated_from` does not help**, and it is the field that looks as though
it would. It records the corpus digest, and that value was IDENTICAL on both
sides: the branch's `MANIFEST.json` and the local build report the same one. The
corpus did not change, the GENERATOR did, and the manifest records nothing about
the generator.

```bash
git show origin/data:MANIFEST.json | jq -r .generated_from
```

⛔ **The digest is read with that command rather than written out here.** A
64-character hex run in prose is refused by the public rules in
[`../scripts/common/check-no-secrets.sh`](../scripts/common/check-no-secrets.sh),
which exempt one only where an identifier names it as a digest. ⚠ Widening that
rule to make a document read better is how a secret scan stops working.

### Approach

⭐ **The two cases are distinguishable, so distinguish them.**

- **behind**: every path the branch carries is still produced, every one of them
  is byte-identical apart from the two derived files, every checksum line the
  branch publishes is still in the regenerated set, and every path the branch's
  own manifest lists is on the branch. Reported with the count, exit 0.
- **diverged**: anything else. Exit 1, with the count of each kind.

⛔ **The derived files are compared by CONTENT rather than exempted.** Every
artefact line the published `SHA256SUMS` carries has to appear unchanged in the
regenerated one, so a digest that moved is caught in the one file that lists
every digest.

⛔ **And the branch is checked against its own manifest**, because a path
DELETED from the branch leaves it a smaller subset, which the tree comparison
alone cannot tell from "not published yet".

Must not: exempt a file from comparison by name. Must not turn the 2 into a
pass; it is a skip, so `--strict` still refuses it in CI where the corpus is
present on purpose.

### Consumers

⚠ This changes a check, not what is published. The JSON gains a `pending` field
and the schema moves to `check-data-branch/3`; nothing outside this tree reads
it.

### Prove

```bash
sh scripts/common/check-data-branch.sh
```

Passing means exit 0, with the branch reported as behind by the number of
artefacts the assembler now produces and the branch does not carry, and no
problem raised. ⛔ Read the exit code from the process that produced it,
unpiped.

### ⭐ Closed 2026-09-04. Three failure modes, each planted and each seen to fail

```text
$ sh scripts/common/check-data-branch.sh
data branch ok: 237 file(s) regenerated, identical over two builds,
  235 of them with a checksum in the manifest and in SHA256SUMS, and no
  source, vendored dependency or reference corpus among them.
  ⚠ The data branch is remote and BEHIND by 37 artefact(s): every path it
  carries is still produced and still byte-identical, and the assembler
  now produces more. ⛔ Nothing published is wrong, so this is reported
  rather than failed. The publisher adds them on the next push.
  ⛔ Nothing was pushed and no branch was created.
exit=0
```

```text
$ sh scripts/common/check-data-branch.sh --json
{"schema":"check-data-branch/3","files":235,"present":237,"recorded":235,"cases":11,"published":"remote","matched":false,"pending":37,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-data-branch.ps1 -Json
{"schema":"check-data-branch/3","files":235,"present":237,"recorded":235,"cases":11,"published":"remote","matched":false,"pending":37,"problems":0}
```

#### ⛔ The guard mutation, three plants

⛔ **Each was made on a local branch built off `origin/data` in a throwaway
worktree, and the branch and the worktree were removed afterwards.** ⚠ Nothing
was pushed and `origin/data` was never written to.

| planted | red |
| --- | --- |
| a published artefact's bytes changed | exit 1: `1 published artefact(s) changed their bytes, and a published artefact is immutable` |
| a published path deleted from the branch | exit 1: `1 path(s) the branch's own manifest lists are not on the branch, so a consumer fetching one gets a 404` |
| the canonical corpus moved out of the working tree | the gate records `SKIP check-data-branch -- the canonical corpus is not in this tree, so the branch has nothing to be compared against`, where it previously recorded a failure |

⭐ **The second plant is the one that earned the manifest leg.** With only the
tree comparison, a DELETED published path read as `BEHIND`, because a branch
missing a file is a smaller subset and a subset was the signal for "not
published yet". ⛔ That would have turned a rewritten branch green, which is the
one thing this check exists to refuse. The branch's own manifest is what tells
the two apart.

#### ⚠ And the manifest leg was wrong on its first run, in a way this tree has seen before

⛔ **It reported all 198 artefacts missing.** `jq` on this Windows host writes
CRLF, so every path came back with a carriage return riding on it and matched
nothing. ⭐ `CORPUS-02` recorded that exact defect against that exact tool on
2026-09-02, and it bit again in a new script. The read strips it now, as
[`../scripts/common/check-coverage.sh`](../scripts/common/check-coverage.sh)
already did.

⚠ **A guard that fires on everything is as useless as one that fires on
nothing**, which is why the count is asserted rather than the failure: 198 was
wrong and 1 was right, and both are non-zero.
