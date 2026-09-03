# agent-tooling.md

⭐ **Read this before you install anything, write your own, or decide a job
cannot be done here.** It is a catalogue of tools that already exist, with
where each one lives.

⛔ **It carries names, links and one line each, and nothing else.** No flags, no
options, no exit codes, no worked invocations. Every one of those is upstream's
to change, and a page that copies them is a page that becomes wrong without
anybody editing it. ⚠ Read the tool's own documentation at the link for how to
call it.

---

## The three reflexes this page exists to stop

| the reflex | what it costs |
| --- | --- |
| **installing something** | a system change nobody asked for, on somebody else's machine, that outlives the session |
| **writing your own** | a second implementation of a solved problem, with its own defects, that nobody else will ever fix |
| ⛔ **refusing, because a tool "is not available"** | the most expensive of the three. [`methodology/sessions.md`](methodology/sessions.md) is the rule: a missing tool closes one route, not the question. |

⚠ **A tool being absent is a measurement, not a verdict.** Run the probe, say
what is missing, then find another route. Three routes considered and rejected
is a finding; one route tried is a stop.

---

## What this repository ships

⭐ Everything here runs with **no network**, which is why it is here rather than
upstream. A gate that has to fetch a check is a gate that is red when somebody
else's host is down, and a check fetched at gate time is code nobody reviewed
judging the tree.

| tool | what it does |
| --- | --- |
| [`../scripts/doctor/`](../scripts/doctor/) | the environment probe. What host, what shell, what tools, what the repository is. A probe, not a gate. |
| ⭐ [`../scripts/common/check-gate`](../scripts/common/) | runs every check below that this host can run, and prints one verdict |

### The checks over the documents and the record

| tool | what it does |
| --- | --- |
| [`../scripts/common/check-docs`](../scripts/common/) | links resolve, fenced blocks parse, no banned vocabulary, no orphan pages |
| [`../scripts/common/check-markers`](../scripts/common/) | only the five defined characters, and not too many of them |
| [`../scripts/common/check-one-home`](../scripts/common/) | one fact, one home: no long sentence in two documents |
| [`../scripts/common/check-placeholders`](../scripts/common/) | did a template placeholder survive into a real file |
| [`../scripts/common/check-control-bytes`](../scripts/common/) | a literal control byte in a tracked text file |
| [`../scripts/common/check-line-endings`](../scripts/common/) | the index and the working tree against what [`../.gitattributes`](../.gitattributes) declares |
| [`../scripts/common/check-changelog`](../scripts/common/) | the four changelog rules a machine can hold |
| ⭐ [`../scripts/common/check-record`](../scripts/common/) | do the record's counts still agree with its rows |
| [`../scripts/common/check-license-consistency`](../scripts/common/) | do the places stating the licence all state the same one |
| ⭐ [`../scripts/common/check-catalogues`](../scripts/common/) | is every script and every document named by the catalogue that lists it |

### The checks over the corpus and what it publishes

| tool | what it does |
| --- | --- |
| [`../scripts/common/check-corpus`](../scripts/common/) | is the store still append-only, and does each profile agree with itself |
| [`../scripts/common/check-validate`](../scripts/common/) | is the corpus coherent as well as intact, with no network and no browser |
| [`../scripts/common/check-coverage`](../scripts/common/) | which planned matrix cells have a profile and which have none |
| [`../scripts/common/check-formats`](../scripts/common/) | one generator, every format, round-tripped, byte-identical twice |
| [`../scripts/common/check-routes`](../scripts/common/) | a single-value route file carries the value and nothing else |
| [`../scripts/common/check-trust-anchors`](../scripts/common/) | a published anchor list per profile that carries the extension |
| [`../scripts/common/check-support-matrix`](../scripts/common/) | every cell produced by a run, and every hole still pointing somewhere |
| [`../scripts/common/check-release`](../scripts/common/) | the same bytes twice, and a refusal to overwrite a pinned tag |
| ⭐ [`../scripts/common/check-data-branch`](../scripts/common/) | is the published branch the tree this corpus derives to, and is a rewrite refused |
| [`../scripts/common/check-notes-generator`](../scripts/common/) | the release body and the changelog entry from one generator |

### The checks over the automation

| tool | what it does |
| --- | --- |
| [`../scripts/common/check-workflows`](../scripts/common/) | the four declarations that decide whether a run produces data or nothing |
| ⭐ [`../scripts/common/check-publish`](../scripts/common/) | what the publishing workflow declares, and whether its rules refuse |
| [`../scripts/common/check-cold-start`](../scripts/common/) | does everything a cold pipeline names still resolve here |
| [`../scripts/common/check-provisioning`](../scripts/common/) | does the browser-purging tool refuse what it must |
| [`../scripts/common/check-exit-codes`](../scripts/common/) | is "could not run" reported as 2 in both halves of every pair |
| [`../scripts/common/check-manual-path`](../scripts/common/) | does every automated job name the command a person runs instead |
| [`../scripts/common/check-pr-body`](../scripts/common/) | would a scheduled run open a pull request a reviewer can act on |
| [`../scripts/common/check-sources`](../scripts/common/) | is every external question asked more than one way |
| [`../scripts/common/check-staleness`](../scripts/common/) | is the corpus behind the build the vendor is serving |

### The checks over the code and the tree

| tool | what it does |
| --- | --- |
| [`../scripts/common/check-no-secrets`](../scripts/common/) | does anything in the tree carry something that must not be published |
| ⭐ [`../scripts/common/check-vendor`](../scripts/common/) | does [`../vendor/upstream.json`](../vendor/upstream.json) still describe the vendored trees, and with `--upstream`, has upstream moved past what it records |
| ⭐ [`../scripts/common/check-msrv`](../scripts/common/) | is the declared minimum Rust version derived from the dependency graph, or typed by somebody |
| [`../scripts/common/check-remote-items`](../scripts/common/) | do the open items against this repository say anything that survives being checked |
| [`../scripts/common/check-powershell`](../scripts/common/) | does every tracked PowerShell file parse, and is the analyzer clean |
| [`../scripts/common/check-twins`](../scripts/common/) | do both halves of every pair still answer the same way |

### The helpers, which are not checks

| tool | what it does |
| --- | --- |
| ⭐ [`../scripts/common/corpus-root`](../scripts/common/) | where is the corpus this run should read: an explicit root, then the working tree, then the data branch |
| ⭐ [`../scripts/common/set-record.mjs`](../scripts/common/) | move an entry's status and re-derive every count from the rows |
| [`../scripts/common/write-file.mjs`](../scripts/common/) | write or patch a file without the shell touching the payload |
| [`../scripts/common/vendor-sync.mjs`](../scripts/common/) | fetch a pristine copy of a vendored upstream at the recorded commit, and materialise a tree from it |
| [`../scripts/common/vendor-diff.mjs`](../scripts/common/) | regenerate the patch series from the vendored tree, or check that it is still in sync |
| ⛔ [`../scripts/common/provision-browser`](../scripts/common/) | purge every browser of one family from this machine and install the build asked for. ⚠ Run it with `--plan` on a machine you keep. |
| [`../scripts/common/git-sync`](../scripts/common/) | commit and push with [`conventions/git.md`](conventions/git.md)'s rules enforced rather than remembered |
| ⭐ [`../scripts/common/mine-repo`](../scripts/common/) | fetch everything a reference sweep needs, and keep it. [`methodology/references.md`](methodology/references.md) is the procedure. |

---

## What lives upstream

⛔ **A tool is not copied into this tree when it already has a home.** A tool
kept in two repositories acquires two sets of defects, and one of the two never
gets fixed.

⚠ **Fetch by a pinned commit or a release tag, never a branch.** A moving
reference runs code nobody reviewed. [`containers.md`](containers.md) has the
worked shape of a pinned wrapper and what it cost to get right.

| tool | upstream | what it does |
| --- | --- | --- |
| `wsl-toolkit` | [`Azathothas/ToolKit`](https://github.com/Azathothas/ToolKit) | creates a throwaway Linux distro on a Windows host, runs a command in it, and destroys it. [`containers.md`](containers.md) is the procedure, and it is how a browser gets captured on a machine that does not have that browser. |

---

## The general-purpose ones, which are somebody else's entirely

⚠ **Presence is not capability.** The probe reports what resolves on `PATH`,
and a name that resolves can still be the wrong program: measured on one
Windows 11 machine, `sort` resolved to PowerShell's own `Sort-Object` alias and
`python3` resolved to a Microsoft Store stub that exits 49 without running
anything. ⛔ Probe by RUNNING the tool, not by finding it.

| job | reach for | why not the obvious thing |
| --- | --- | --- |
| talk to a code host's API | [`gh`](https://cli.github.com/) | reads only, and never against somebody else's repository. [`security/remote-ops.md`](security/remote-ops.md). |
| fetch a URL | `curl`, or the host's own client | in Windows PowerShell 5.1 `curl` is an ALIAS for a cmdlet that takes different arguments. [`conventions/shell.md`](conventions/shell.md). |
| read or reshape JSON | [`jq`](https://jqlang.github.io/jq/) | ⛔ never a regular expression over JSON. A bracket inside a string value is how one page joiner lost an entire comment corpus. |
| read or reshape YAML | [`yq`](https://github.com/mikefarah/yq) | the same reason |
| lint POSIX shell | [`shellcheck`](https://www.shellcheck.net/) | it finds the quoting and exit-code traps [`conventions/shell.md`](conventions/shell.md) documents, before they ship |
| lint PowerShell | [`PSScriptAnalyzer`](https://github.com/PowerShell/PSScriptAnalyzer) | the same, on the half a POSIX linter cannot see |
| time a command honestly | [`hyperfine`](https://github.com/sharkdp/hyperfine) | a single `time` run is not a measurement. [`methodology/experiments.md`](methodology/experiments.md) says what one owes. |
| count lines of code | [`scc`](https://github.com/boyter/scc) or [`tokei`](https://github.com/XAMPPRocky/tokei) | ⚠ counters disagree about blank and comment lines, so name which one produced a number |
| search a tree | [`rg`](https://github.com/BurntSushi/ripgrep) | it locates; it does not confirm. Open the file. |
| run something on Linux from Windows | `wsl-toolkit`, above | never install a distro by hand and leave it registered. [`containers.md`](containers.md). |
| drive a browser without a display | the browser's own headless mode | ⚠ a headless build reports a DIFFERENT User-Agent. [`inherited-claims.md`](inherited-claims.md) carries the claim and the entry that re-measures it. |
| read bytes off a socket in Rust | the standard library, then a vendored parser | ⛔ a fingerprinting harness parses PERMISSIVELY and emits EXACTLY. A library built for one of those jobs gets the other wrong. |

---

## Adding a row

1. **The tool has to already exist and be reachable.** This is a catalogue, not
   a wish list.
2. **One line, and no behaviour.** If the row needs a flag to be useful, the
   flag belongs in upstream's documentation and the row belongs in
   [`containers.md`](containers.md) or nowhere.
3. **Say where it lives**, as a link a reader can open.
4. **A row for a tool nothing in this repository uses is a row somebody
   maintains for nothing.** Delete it instead.
