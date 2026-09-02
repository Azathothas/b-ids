# scripts

The probe, the checks, and the helpers.

| directory | what is in it |
| --- | --- |
| [`doctor/`](doctor/) | ⭐ the environment probe. Two implementations, one schema. Every session runs it. |
| [`common/`](common/) | the checks and the helpers. ⛔ Every CHECK has a POSIX sh implementation AND a PowerShell twin. |

⚠ **There is no platform-specific directory here and shipping an empty one
would be shipping a phantom**: git does not track an empty directory, so a fresh
clone would not have what a table described. The one job in this project with no
POSIX form is running a throwaway Linux machine from a Windows host, and the
tool for it lives upstream rather than here.
[`../docs/containers.md`](../docs/containers.md) is the procedure and
[`../docs/agent-tooling.md`](../docs/agent-tooling.md) is where it lives.

⚠ **`common/mine-repo` is here and has no section of its own below.** It fetches
what a reference sweep needs and keeps it;
[`../docs/methodology/references.md`](../docs/methodology/references.md) is the
procedure it serves, and `TOOL-04` in [`../TODO/tooling.md`](../TODO/tooling.md)
is an open defect in it found by running it.

---

## ⭐ Everything in `common/` has two implementations, and here is what that cost

⛔ **A POSIX sh check cannot be assumed to run on Windows.** This was the
template's original position and it was wrong. The reasoning was that `sh`
would be present because Git Bash ships with git, so one implementation was
enough. Measured on one Windows 11 machine, 2026-08-25, from a native
PowerShell session with Git Bash NOT on `PATH`:

| tool the checks need | native PowerShell resolves it to |
| --- | --- |
| `sed` | ⛔ nothing. Not installed. |
| `sort` | ⚠ PowerShell's own `Sort-Object` alias, not the coreutils binary |
| `awk`, `grep`, `tr`, `comm`, `xargs` | present here only because scoop and a coreutils package happen to be installed |

⚠ **The second row is the dangerous one.** A missing tool fails loudly and
somebody fixes it. An ALIASED one succeeds and returns a DIFFERENT ANSWER.
`Sort-Object` even accepts `-u`, which is what makes it convincing. Measured on
the same machine, same day, over the five values `b A a B a`:

| | result |
| --- | --- |
| `LC_ALL=C sort -u` | `A B a b` |
| `Sort-Object -u` | ⛔ `A b` |

⛔ **It dropped two of the four distinct values**, because it compares
case-insensitively and keeps whichever it saw first. A check that deduplicates
a file list that way does not crash and does not warn. It reports on a smaller
set than it was asked about, and reports success.

⭐ **What did NOT reproduce, and is worth writing down so nobody re-derives
it:** git and `gh` behaved identically from both shells on this machine. Same
`git.exe` 2.55.0.windows.3, same `credential.helper manager` from the same
system config, same authenticated `gh`. So the argument for twins here is the
TOOLCHAIN, not credential scoping. A machine that installs git differently per
shell would add a second reason; this one did not have it.

### ⛔ Wherever a twin exists, `check-twins.sh` covers it

That is not advice, it is the rule that keeps two implementations from becoming
two behaviours. [`common/check-twins.sh`](common/) runs BOTH halves of every
pair on one tree and compares the `--json` answer and the exit code.

⚠ **It compares ANSWERS on the tree it is run against, not the rules.** A scope
difference with nothing in the tree to exercise it is invisible: dropping `.py`
from one twin's extension list changed no number here, because this repository
has no `.py` file. Dropping `.md` was caught instantly. ⭐ Prove a scope rule
with a fixture, not by trusting the comparison to notice.

### The things that do NOT have twins, and why

| | |
| --- | --- |
| [`common/set-record.mjs`](common/) | ⛔ **It does not need one**, and for the same reason as `write-file.mjs` below: it is node. ⚠ What it would cost to give it one is the thing to notice: a twin here means a second implementation of table arithmetic, which is a second place for that arithmetic to be wrong, in the one file whose whole job is that the arithmetic is right. |
| [`common/write-file.mjs`](common/) | ⛔ **It does not need one.** It is node, and node is the same program on every host: no `sed`, no `sort`, no shell built-ins, no aliases. The reason the sh checks needed twins does not apply to it. ⚠ What it needs instead is node itself, which is the one dependency anything under `scripts/` has, and the reason a project may decline this helper rather than inherit it. |
| [`common/check-twins.sh`](common/) | ⛔ **It cannot have one.** It works by running both halves of every pair, so it needs a POSIX shell to run the sh half no matter what language it is written in. A PowerShell twin would still require `sh`, which is the exact dependency a twin exists to remove. It is a maintainer's tool and it runs where both implementations do: this machine, and the CI job that has `pwsh` on an Ubuntu runner. |
| [`common/mine-repo.sh`](common/) | ⚠ **It has a twin and the pair is NOT compared**, which is a defect rather than a decision. It needs the network, which is why it was left off the list. ⭐ Its offline self-test does not, and comparing that is `TOOL-05`. |

⭐ **The question to ask is whether the JOB exists on the other platform, not
whether the language does.** Every check in `common/` passes that test, which is
why every one of them has two halves.

---

## ⚠ What a vendored tree costs the checks that read the whole tree

Five of them failed the moment the first vendored tree landed: the four prose
checks and the secret scan. ⭐ **That is the cost
[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) names in
advance**, where it says warnings become yours.

The exemption covers each vendored tree and the patch series derived from it,
and ⛔ it is never the whole vendor directory, because
[`../vendor/upstream.json`](../vendor/upstream.json) and
[`../patches/README.md`](../patches/README.md) are this project's own writing.
Each half of each check carries its reason, and the secret scan carries the
reading that was done before the exemption was taken.

⭐ **Keeping the manifest in scope paid immediately.**
[`../TODO/vendor.md`](../TODO/vendor.md) records what the boundary caught on the
day it was drawn.

---
## The check contract

⛔ **Every check in this repository, and every check a project inherits from it,
satisfies all five.** A script that does not is not a check; it is a script
somebody has to remember to interpret.

1. **A header comment saying what defect it exists to catch.** Not what it
   does: what goes wrong without it. ⭐ This is the field that decides whether a
   future session keeps it, deletes it, or writes a second one that overlaps.
2. **Exit 0 pass, 1 fail, 2 could not run.** ⚠ Those are three different facts.
   "The check failed" and "the check could not run" mean opposite things about
   whether you can ship, and a script that returns 1 for both hides the
   difference.
3. **A json switch**, so a gate runner can consume it.
4. **No dependence on the directory it is run from.** Resolve paths from the
   script's own location.
5. **Read only, unless a fix flag is passed.** A check that repairs things by
   default is a check nobody can use to find out whether something is wrong.

⚠ **A check that measures an open defect must not fail the build for that
defect alone.** Record the count and judge it only past a stated ceiling.
⭐ The other half of that rule is that the exemption comes off when the item
closes. An exemption nobody removes is a check that stopped checking.

---

## ⛔ An exit code is read from the process that produced it, unpiped

```bash
sh scripts/common/check-no-secrets.sh
```

Not `check | grep`, not `check | Select-String`, not `check | tee`. A pipeline
reports the **last** command's status, so a check that failed reads as green.

⚠ This has caught the author of this sentence, in the session that wrote it.

---

## What is here

### `doctor/`

The environment probe. Read [`doctor/README.md`](doctor/README.md) for what it
answers, the schema, and the measured runtimes.

⭐ It is a **probe, not a gate**: a missing tool is data, so it exits 0 whether
or not anything is missing. Nothing here belongs in a gate chain.

### `common/check-no-secrets.sh`

Does any file in this tree carry something that must not be published.

⚠ **Tracked plus untracked-but-not-ignored, not tracked alone.** A file that
has never been staged is exactly when a new file is likeliest to carry a
credential, and exactly what the next `git add -A` would take.

⛔ **It finds the shapes it knows, and a green run is not a clearance.** It
cannot find a password that looks like a word or a page of correct-looking
examples that happens to describe a real system.

`--public` adds the rules that only matter for a repository that will be
public: emails, absolute home paths, long hex identifiers. In a private project
those are legitimate content, which is why they are not the default.

### `common/check-placeholders.sh`

Did a template placeholder survive into a real file. Run at the end of a
bootstrap, and as a gate afterwards.

### `common/check-docs.sh`

Do the documents still resolve, and are they written the way this repository
writes documents. Every relative link resolves, ⭐ **every cited path in a code
span resolves**, every fenced shell block parses, no block carries an
angle-bracket placeholder that a shell reads as a redirect, and no page is
linked from nowhere. Plus the **banned vocabulary**, over the fourteen of its
eighteen words that are always a quality assertion.

⛔ **The cited-path half is scoped to the top-level directories this repository
owns**, and the scope was measured rather than guessed: without it the check
reported 30 spans and every one was legitimate, because the sweep documents cite
paths inside the reference trees as shorthand. ⚠ A path this tree deliberately
does not have is written as plain text rather than in a code span, and
[`../docs/conventions/prose.md`](../docs/conventions/prose.md) carries that
rule.

⛔ **No file is exempt from the link check**, and the exemption that used to be
here was removed rather than emptied: it covered a template directory this
repository does not have.

⚠ **Two prose rules are deliberately elsewhere.** The character set and the
marker density belong to [`common/check-markers.sh`](common/) and control bytes
to [`common/check-control-bytes.sh`](common/), because both read every tracked
text file rather than markdown alone. Two checks holding one rule is two places
for it to be wrong.

### `common/check-markers.sh`

Are the only characters outside ASCII in this tree the five this repository
defines, and does any one page carry so many of them that they have stopped
meaning anything.

⛔ **It covers every tracked text file, not markdown alone**, which is the whole
reason it exists beside `check-docs.sh` rather than inside it: that one reads
markdown, and in every tree where this check has been armed, the findings were
in scripts rather than in documents. Its own header carries the numbers.

⭐ **The density ceiling is 30 markers per 100 non-blank lines**, and it is a
constant rather than a flag: a ceiling anybody can raise from a command line is
a ceiling that gets raised instead of met.

⚠ **The reference corpus is out of scope**, because it is somebody else's
writing. The header says which two checks deliberately do not exempt it.

⚠ **A specimen inside a code span or a fenced block is permitted in markdown.**
Without that, a page that bans a character cannot show a reader which one.

### `common/check-one-home.sh`

Does any sentence of twelve words or more appear in two documents.

⭐ [`../docs/conventions/prose.md`](../docs/conventions/prose.md) owns the rule
that one fact lives in one document; this is what enforces it.

⛔ **It carries no router exemption, and that is deliberate.** Where a project
has two routers each stating the absolutes in full, the pair has to be exempt
from each other by name. ⭐ This project has one router, so nothing is
duplicated and no exemption exists to grant itself to whatever lands at a path
later.

⚠ **`docs/HISTORY/` and the reference corpus are both out of scope**, for two
different reasons: a retired page states things the live pages now state
differently, and the corpus is somebody else's writing.

⚠ **It compares sentences**, so a fact restated in different words passes here
and fails a review instead. That is the same split every other prose rule has.

### `common/check-twins.sh`

Do the two probe implementations still answer the same way. It runs both on
one machine and compares the schema, the section keys, and the host and repo
facts that describe that machine.

⚠ It compares the SHAPE and the FACTS, not the tool-by-tool verdicts. Each
twin reports what its own host can reach, and on a Windows machine with msys
installed `bash`, `tar` and `zsh` genuinely differ between them.

⭐ **It also compares the CLI surface, which the schema cannot show.** Every
comparison above reads what the probes OUTPUT; none of them reads what the
probes ACCEPT. `doctor.sh --text` exited 0 while `doctor.ps1 -Text` exited 1
with a parameter-binding error, and every other comparison in the file passed
the whole time that was true.

### `common/check-remote-items.sh`

What is open against the repository, and does it say anything that survives
being checked. For every pinned action a pull request proposes: the commit
exists in the repository the ref names, the tag comment resolves to that same
commit, and ⭐ the runtime it DECLARES is not one the platform has deprecated.

⛔ **It never merges, closes, comments or approves.** It reports, and deciding
is the operator's.

⚠ It cannot tell you whether a change is a good idea. It checks the facts an
item asserts about the world; whether you want the change is a reading.

⭐ It exists because this repository was pinned to an action targeting a Node
runtime GitHub had deprecated, and the warning sat in a log nobody read. A
dependency bot is right almost every time, and that is precisely what makes
the wrong one expensive.

### `common/check-control-bytes.sh`

Is there a literal control byte in any text file in the tree.

⭐ **It covers every text file, not only markdown.** The rule used to live in
`check-docs.sh` and scanned `.md` alone, which left every `.ts`, `.py`, `.rs`,
`.sh` and `.yml` unchecked for the one defect that makes a file invisible to
both review tools at once: `grep` calls it binary and skips it, and `git diff`
prints "Binary files differ" so a code review shows no diff at all.

⚠ The runtime value is identical either way, so only reviewability is ever at
stake. That is exactly why it survives unnoticed.

### `common/check-msrv.sh`

Is the declared minimum supported Rust version derived from the dependency
graph, or is it a number somebody typed?

⭐ **Two legs, and they answer different questions.** The default leg reads
`cargo metadata` and takes the highest `rust-version` any package OUTSIDE this
workspace declares; a declared value under that floor is refused. `--verify`
compiles the workspace with the declared toolchain, which is the only leg that
can say the declared value is reachable at all.

⛔ **Workspace members are excluded from the floor, and that exclusion is why
the check can fail.** Every member inherits the field from the workspace, so a
floor taken over all packages would read back the value it is checking and
agree with itself forever.

⚠ **`--write` is the fix flag and it refuses when the graph imposes no floor.**
A version invented there would be the fabricated number the check exists to
find. It patches through [`common/write-file.mjs`](common/) rather than with
its own writer, so a substitution that matched the wrong number of times leaves
the file untouched.

⭐ **The graph imposed no floor at all until 2026-09-01**, when the vendored TLS
terminator brought a certificate minter with it. The check now reads a floor of
1.88 from that crate, which is exactly the value this tree had already declared
as an upper bound. ⚠ The two agreeing is a coincidence worth knowing about: the
declared value stopped being unconstrained without anybody choosing a number.

⚠ **Exit 2 means cargo or jq is absent, and the gate reports that as a SKIP
rather than a pass.** A host with no cargo has verified nothing about the
manifest. ⛔ That is a different fact from `check-changelog`'s 2, which is a
pass because a project with no changelog has satisfied its rules vacuously.

### `common/check-vendor.sh`

Does [`../vendor/upstream.json`](../vendor/upstream.json) still describe the
vendored trees, and has upstream moved past what it records.

⭐ **The defect it exists to catch is a vendored tree nobody can reconcile.** A
tree with no recorded commit is a fork whose base is lost: the next release
cannot be merged onto it, and no patch can be said to be a diff from anything.

⚠ **Two legs, and only one is in the gate.** The default leg reads the manifest
against the tree: directories exist, excluded paths are absent, every crate the
manifest names declares that name, every tree under the vendor directory has an
entry, and every patch names a file the tree still has and a section in
[`../patches/README.md`](../patches/README.md). `--upstream` fetches the
recorded ref from the remote and reports whether it still resolves to the
recorded base and which newer release tags exist. ⛔ A gate that needs the
network fails on a machine that has none, which is why only the first runs
there.

⚠ **Exit 2 is "could not run", and the gate reports it as a SKIP.** The sh half
needs `jq`; both halves treat an absent manifest as 2, because a tree that
vendors nothing has verified nothing.

⛔ **A moved ref is reported, never followed.** Reconciling a release is a
reading and
[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) says what
it owes.

### `common/check-corpus.sh`

Is the published corpus still append-only, and does every profile in it still
agree with itself?

⭐ **Two legs, and only one of them is a question this tree can answer.** The
working tree cannot say whether a file was edited after it was published,
because an edited file and a file that was always that way are identical on
disk. That leg asks git, over the whole history, with `--diff-filter=MDR`: a
modification breaks immutability, a deletion breaks "never delete a superseded
profile", and a rename is a published route changing under a consumer who
pinned it.

⛔ **The second leg is delegated, not re-implemented.** Every profile validating,
sitting at the route its own keys derive, publishing the bytes it says it
publishes, and being listed in an index the tree derives to is the question
`b-ids-corpus verify` answers. A second implementation of the layout rule in
shell would be a second answer to where a profile lives.

⚠ **The numbers come from a fixed status line.** That command prints
`corpus=profiles:N problems:N` as its last line and its usage says so, which is
the same discipline [`common/check-powershell.ps1`](common/) already follows.
Parsing the prose above it would make every wording change a silent behaviour
change.

⚠ **Exit 2 is "could not run", three ways over**: there is no corpus at all,
the per-profile leg needed cargo and did not get it, or ⛔ **the clone is
SHALLOW.** The gate reports all three as a SKIP. ⛔ The git leg still decides a
failure: a published file edited after its first commit is exit 1 whether or not
cargo was there.

⛔ **The shallow-clone refusal was a measured defect, not a precaution.**
`actions/checkout` fetches one commit by default, so `git log` over the corpus
paths saw a single commit and `--diff-filter=MDR` found nothing. This check ran
inside the gate on both CI jobs from the day it was written and its history leg
verified nothing on either, while reporting `corpus ok: 1 profile(s), nothing
edited after publication`. That is the "step that exits 0 having done nothing it
was asked to do" row in
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md),
in the check whose whole job is reading the history. ⭐ Both workflows carry
`fetch-depth: 0` now, and the refusal is what makes losing that line fail rather
than go quiet. `TODO/ci.md`, `CI-01`.

⛔ **The derived `index.json` and `latest.json` are excluded from the history
leg**, and that was a defect rather than a design: they are regenerated whenever
a profile is added, so a rule refusing their modification would refuse the second
profile the corpus ever gets. ⚠ Nothing goes unchecked, because their content is
asserted by the other leg against what the tree derives to. It fired on exactly
that, on the first commit after the corpus had one.

### `common/check-validate.sh`

Is every PUBLISHED profile coherent, and does the generator answer the same way
twice?

⭐ **The defect it exists to catch is a corpus that is structurally intact and
incoherent.** [`common/check-corpus.sh`](common/) asks whether every profile
sits at the route its keys derive, publishes the bytes it claims and was never
edited after publication. Every one of those can be true of a profile whose
User-Agent says 151 and whose brand list says 152. ⚠ Nothing in this tree ran
the coherence checks over what is published until this did: `b-ids-validator`
takes the paths a caller names, so it answered about whatever somebody
remembered to list.

⛔ **Leg one is delegated to `b-ids-corpus validate`**, which is the one place
that knows both the layout and the checks, and it reads that command's fixed
`corpus=validate profiles:N findings:N notcheckable:N` line rather than the
prose above it. ⭐ That command also runs the CROSS-profile form of check 4,
`shared_handshakes`, which no per-profile invocation can reach: two profiles
claiming different majors and carrying a byte-identical TLS half, of which at
most one was measured. ⚠ It is structurally silent on a corpus of one, and
`CORPUS-02` is what ends that.

⭐ **Leg two is a class `b-ids-corpus verify` cannot see.** Verify compares the
committed index against ONE derivation, so a generator that answered differently
on alternate runs would fail it intermittently and read as a flake. This runs
the generator twice over a throwaway copy and compares the bytes. ⚠ A release
nobody can reproduce is a release whose every run looks like a change.

⚠ **The copy is outside the repository** and the generator writes only into it,
so the check stays read-only over the tree it measures.

⚠ **What it does not assert yet, said rather than implied**: the round trip of
the generated formats. There is one generator in this tree today and it writes
the index and the pointer file; `SCHEMA-08` is what adds the rest, and it adds
them to leg two in the same change.

⛔ **Nothing here reaches the network or resolves a browser.** That is the
requirement in `CI-01` rather than a property it happens to have, and
[`../.github/workflows/validate.yml`](../.github/workflows/validate.yml) is
where it is enforced with `CARGO_NET_OFFLINE`.

### `common/check-routes.sh`

Does any published route file that carries exactly one value end with a
newline?

⭐ **The requirement came from a measurement in somebody else's dataset**, not
from taste: `od -c` over two single-value files published by
`pkgforge-security/Wordlists` shows a trailing newline on each, so every
consumer of one has to strip something and the ones that forget compare a value
against a value-plus-newline.
[`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md)
section 9.

⛔ **What counts as single-valued is decided by extension**, because a file
carries one value when this project says its type does. `.hex` is defined here
as one raw capture on one line and nothing else, which is the same definition
[`common/check-no-secrets.sh`](common/) uses to exempt one from the credential
rule. A check that guessed from content would call a one-line JSON file
single-valued.

⚠ **A route tree that yielded no single-value file is exit 2, not exit 0.** A
check reporting clean over nothing is how it would quietly stop applying the day
a route type is renamed, and that shape was found here by the fixture written to
prove the check could refuse: `git ls-files` answers a path outside the
repository with an empty list, so both halves reported ok over zero files.
⭐ `--fixtures` walks the filesystem for that reason.

⛔ **It reports, it does not strip.** The generator is what gets fixed.

### `common/check-staleness.sh`

Is the corpus behind the build the vendor is serving, and what would replace it?

⛔ **A SCHEDULE, NEVER A PUSH TRIGGER, and it is not in the gate.** A browser
shipping a new version is not a defect in a commit, and without `--versions`
this reaches the network, which gate part (a) must not.
[`../.github/workflows/staleness.yml`](../.github/workflows/staleness.yml) is
where it runs. [`../TODO/ci.md`](../TODO/ci.md), `CI-02`.

⭐ **Exit 1 is the SIGNAL rather than a defect in this tree.** 0 means the
corpus holds the serving build, 1 means it is behind, and 2 means no source
answered, which is a fact about the vendor rather than about this repository.

⛔ **The ordering is numeric per component.** `151.0.7922.9` is behind
`151.0.7922.76`, and a string comparison says the opposite. ⚠ Both halves
implement it themselves, so the pair is a genuine two-implementation comparison.

⛔ **It never fetches anything itself.** `b-ids-driver versions` asks each
source separately and reports which answered; a second fetcher here would be a
second answer to what is current. `--versions FILE` takes the same JSON from a
file, which is how the twin comparison runs with no network, against
[`fixtures/staleness-versions.json`](fixtures/staleness-versions.json).

### `common/check-exit-codes.sh`

Does every script in this tree report "could not run" as **2**, on both halves
of every pair?

⛔ **1 is "it ran and the thing failed" and 2 is "it could not run."** A check
that returned 1 for the second is a check somebody disables the day a runner has
no browser, and a capture job on a machine without one must not fail the build.
[`../TODO/ci.md`](../TODO/ci.md), `CI-07`.

⭐ **The input is an argument no script accepts**, because that is the one
state every script can be put into from outside with no missing tool, no missing
browser and no network. ⚠ A check needing a real unrunnable condition per
script would have as many special cases as scripts, and the ones it could not
construct would go unchecked.

⛔ **0 is refused as well as 1.** A script that ignored an argument it does not
understand and ran anyway did something other than what it was asked to do and
reported success.

⚠ **Each half checks its own language**, and that is the contract above rather
than a shortcut: a POSIX half shelling out to `pwsh` would report a green half
of a pair as the whole pair on a host without it.

⭐ **It carries a fixture leg on every run**: it plants a script that exits 1,
invokes it the same way, and refuses to report a result at all if that reads as
2. A check that could not tell the two apart would pass over everything.

### `common/check-gate.sh`

⭐ **Run every local gate this host can run, in one command.** Part (a) of
[`../docs/methodology/gate.md`](../docs/methodology/gate.md) is a list, and a
list run by hand is run in the order somebody recalls it, missing whichever
entry was added last.

```bash
sh scripts/common/check-gate.sh --fast
```

⛔ **It is not a second set of rules.** Every line delegates to a check that
already exists and reads that check's own exit code. When it and
`.github/workflows/ci.yml` disagree about what runs, CI gates the push and this
one is the defect.

⚠ **A skipped check is not a passed check.** `shellcheck`, `jq`, `pwsh` and
PSScriptAnalyzer are not on every machine. A missing one is reported as `SKIP`,
counted separately, named in the summary and carried in `--json` as
`skipped`. The exit code is still 0, because "this host cannot run that one" is
not a failure of the tree.

⛔ **The analyzer and the parse are scored separately**, because they can have
different answers and `check-powershell` exits 0 either way. One verdict for
both is how a skipped analyzer reads as a passed check, which is what it did
here once.

⚠ **`--fast` skips `check-twins` and nothing else.** Measured on one Windows 11
machine, 2026-08-27: the full run took 208s and `check-twins` was 171s of it.
That is the right price before a push and the wrong one before each of eleven
commits.

⛔ **It runs `check-twins`, which runs it.** A recursion guard breaks the cycle;
without it the pair hung for ten minutes and left twenty stray shells holding
their own files open.

### `common/check-powershell.ps1`

Does every tracked `.ps1` parse, and is PSScriptAnalyzer clean over `scripts/`
at Error and Warning.

⚠ **The analyzer is a module, not part of PowerShell.** Without it this reports
`SKIPPED` and exits 0. ⛔ **It never installs it**: a check that installs
software changes the machine it is measuring, and this one runs before a commit.
CI installs it explicitly and then asserts it was not skipped.

⭐ Its last line is a fixed `analyzer=clean|skipped|issues:N`, which is what
`check-gate` reads. ⛔ Parse that, never the prose above it.

### `common/check-changelog.sh`

Does `CHANGELOG.md` still obey the four rules a machine can hold: newest first,
every heading dated, every entry naming its record, every entry saying whether
it deployed.

⭐ It exists because [`../docs/conventions/docs.md`](../docs/conventions/docs.md)
stated those four rules, said in as many words that each was mechanical enough
to check, and nothing checked them.

⚠ **No `CHANGELOG.md` is exit 2, not exit 0.** A project without one has
neither broken these rules nor satisfied them, and reporting green over an
absent file is how a check quietly stops applying.

---

## The helpers, which are not checks

⚠ **A helper writes; a check reports.** The five-point contract above is for
checks. The ones below are held to the header rule and the exit-code rule, and
deliberately not to "read only": writing is what they are for.

⛔ **A helper refuses rather than guessing.** Both of the ones below stop and
write nothing when what they were asked for does not match what they found, and
that is the property that matters more than which list they appear in.

### `common/write-file.mjs`

Write, append to, or patch a file without the shell touching the payload.

⭐ **The payload channel is base64**, which is the one encoding no shell
interprets: not bash, not PowerShell, not `cmd`. A quote, a backtick, a dollar
sign, a percent and an emoji all survive it unchanged.

⛔ **A substitution whose match count differs from the number you declared is
REFUSED and the file is left untouched.** A silent no-op reporting success is
the failure this exists to remove. It fired twice while this template was
being maintained, once on a CRLF file whose LF search string matched nothing.

⚠ It needs `node`. That is the only thing under `scripts/` that does, and it
is the reason this is a helper a project may decline rather than a check every
project inherits. [`../docs/conventions/shell.md`](../docs/conventions/shell.md)
section 1 is the reasoning, measured.

### `common/set-record.mjs`

Move an entry's status and re-derive every count from the rows.
[`../docs/methodology/work-todo.md`](../docs/methodology/work-todo.md) calls the
counts the model's one mechanical hazard and says to automate **both** halves;
`check-record` is the reader and this is the writer it names.

```bash
node scripts/common/set-record.mjs status WSL-06 done
```

Closing one entry moves seven numbers: the index count line, the priority
table's four figures for that priority, that table's **all** row, and the
record's own count line.

⛔ **It does not run `check-record` and report green.** A writer that grades its
own work is one bug away from hiding the bug, and the reader has to assert
independently. It prints the command; `check-gate` runs it.

⚠ **It needs `node`, and has no PowerShell twin for the same reason
`write-file.mjs` has none.** A second implementation of table arithmetic is a
second place for that arithmetic to be wrong.

### `common/vendor-sync.mjs`

Fetch a pristine copy of a vendored upstream at the recorded commit, and
materialise a tree from it the first time.

⛔ **It refuses to overwrite a tree that already has content.** The tree carries
local patches and a refresh that took upstream's copy would delete them with no
diff to notice it by. `--force` is the deliberate spelling.

⚠ **A ref that has MOVED is reported and not followed.** The recorded base is
what the series was generated against, so following a moved tag silently
changes what every patch is a diff from.

### `common/vendor-diff.mjs`

Regenerate the patch series from the vendored tree, or assert with `--check`
that the series on disk still matches what the tree produces.

⛔ **The patches are output, not input.** Nothing applies them, so editing one
changes nothing about what is built.

⚠ **It needs the pristine copy, so it needs the network, so it is not a gate
check.** The offline half of the same question belongs to `common/check-vendor`.

### `common/git-sync.sh`

Commit and push with the rules in
[`../docs/conventions/git.md`](../docs/conventions/git.md) enforced rather than
remembered.

⭐ **It arrived as a 674-line PowerShell script and now exists as both**: a
POSIX sh implementation so every Linux and macOS project can run it, and a
PowerShell twin because on Windows the sh one needs a POSIX layer that a native
session may not have. ⚠ On Windows prefer the `.ps1`: it drives the native
`git.exe` rather than one inside an msys layer.

⛔ **An AI-attribution line is refused, never stripped.** Rewriting somebody's
commit message is worse than declining it: the author never learns the rule.

⛔ **A CI-skip marker is refused unless the flag was passed.** A message that
merely mentions one skips CI, because the platform does not read the sentence
around it.

⚠ **It knows nothing about who you are.** Identity comes from the flags or from
git config, and if neither has one it refuses rather than guessing.

## Adding one

1. **Name the defect first.** If you cannot say what goes wrong without this
   script, it is not a check.
2. **Follow the contract**, all five points.
3. ⭐ **Mutation-prove it.** Plant the defect it exists to catch, run it, and
   read the exit code unpiped. **A guard that has never been seen to refuse is
   a guard nobody knows works.**

   This is not optional advice, and it has fired here. ⛔ **The
   banned-vocabulary rule was documented in four files and enforced in
   none.** `check-docs`'s own header claimed it, `docs/conventions/prose.md`
   claimed it, the tool catalogue claimed it, and the success line read "Links
   and prose clean". A sentence carrying two banned words was appended to a
   document, the check passed, and that is how it was found. `TOOL-11`.

   ⭐ **And the measurement changed the design.** Four of the eighteen banned
   words matched nineteen times in this tree, every match legitimate, so the
   check holds the other fourteen and the review holds those four. A guard is
   scoped to what it can decide, not to what the rule says.

4. **Wire it into the gate**, if it can fail.
5. **Document it**: here, and in the project's own tool table.

⚠ **A script that lives only in a transcript is re-derived every session.**
When a scratch helper does something a future session will also need, promote
it: write it into `scripts/` with the contract above, document it where agents
are told to look, and wire it into the gate if it is a check rather than a
one-off.
