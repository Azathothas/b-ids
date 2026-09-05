//! `PUB-01` and `PUB-02`. One assembler, two surfaces.
//!
//! ⛔ Every test name starts with `publish`, because
//! `cargo test -p b-ids-corpus --test publish` is what runs this file alone.

use std::path::Path;

use b_ids_corpus::publish::{
    CHECKSUMS, INITIAL_RELEASE_TAG, MANIFEST, NotReleasable, build, moving_tags, parse_tag,
    plan_initial_release, plan_release, tag, would_rewrite,
};

/// The workspace root, canonicalised. ⚠ **This is where scratch output goes and
/// it is NOT where the corpus is read from.** The two were one function until
/// `PUB-11`, which is what coupled this suite to the working tree.
fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root")
}

/// The corpus this run should build from.
///
/// ⛔ **Resolved, never assumed.** `corpus/` and `raw/` are to leave the default
/// branch, and this suite reached them by walking up from its own manifest, so
/// it was one of the three legs that still failed with the corpus moved out of
/// the working tree. docs/history/todo/publish.md, `PUB-11`.
///
/// ⭐ The order is `b-ids/build.rs`'s, deliberately: an explicit root wins, then
/// the nearest ancestor that actually holds a corpus. A checker that exports
/// `B_IDS_CORPUS_ROOT` therefore gets the same answer here, in the crate's build
/// script, and in every check that asks `corpus-root.sh`.
/// ⛔ **ONE RESOLVER, and this was the sixth copy of it.** `PUB-13`'s door sweep
/// collapsed five private walks onto `b_ids_schema::root` and left this one,
/// which is the shape that sweep exists to catch: the list you wrote from
/// memory has never once been complete.
fn corpus_root() -> std::path::PathBuf {
    b_ids_schema::root::corpus_root_or_explain(Path::new(env!("CARGO_MANIFEST_DIR")))
}

/// Build the repository's own corpus into a throwaway directory.
///
/// ⚠ **The REAL corpus, not a fixture.** The assembler's job is to publish what
/// this project actually holds, and a fixture would prove it can publish
/// something else.
///
/// ⚠ **The source and the destination are resolved separately.** The corpus may
/// be a materialised copy of the data branch; the scratch output belongs under
/// the workspace's own ignored directory either way.
fn built_into(name: &str) -> (b_ids_corpus::Built, std::path::PathBuf) {
    let root = corpus_root();
    let out = workspace_root()
        .join(".tmp")
        .join("publish-suite")
        .join(name);
    let _ = std::fs::remove_dir_all(&out);
    let built = build(root.to_str().expect("a utf-8 root"), &out).expect("the corpus assembles");
    (built, out)
}

#[test]
fn publish_two_builds_over_one_corpus_are_byte_identical() {
    // ⛔ A BUILD THAT DIFFERS BETWEEN RUNS is one a consumer cannot tell a real
    // change from. Nothing in the assembler reads a clock, and this is what
    // says so.
    let (first, first_dir) = built_into("a");
    let (second, second_dir) = built_into("b");
    assert_eq!(first, second, "two builds over one corpus differ");

    for artefact in &first.artefacts {
        let left = std::fs::read(first_dir.join(&artefact.path)).expect("it was written");
        let right = std::fs::read(second_dir.join(&artefact.path)).expect("it was written");
        assert_eq!(left, right, "{} differs between runs", artefact.path);
    }
    for name in [MANIFEST, CHECKSUMS] {
        let left = std::fs::read(first_dir.join(name)).expect("it was written");
        let right = std::fs::read(second_dir.join(name)).expect("it was written");
        assert_eq!(left, right, "{name} differs between runs");
    }
}

#[test]
fn publish_every_artefact_has_a_checksum_and_the_checksum_is_of_the_file() {
    // ⛔ A CHECKSUM FILE NOBODY CHECKED is a checksum file that agrees with
    // itself. Each digest is recomputed from the bytes on disk.
    let (built, dir) = built_into("checksums");
    let sums = std::fs::read_to_string(dir.join(CHECKSUMS)).expect("the checksums file");
    assert!(!built.artefacts.is_empty(), "the build produced nothing");
    for artefact in &built.artefacts {
        let body = std::fs::read(dir.join(&artefact.path)).expect("it was written");
        let digest = b_ids_harness::hex(&b_ids_harness::sha256(&body));
        assert_eq!(digest, artefact.sha256, "{}", artefact.path);
        assert_eq!(body.len(), artefact.bytes, "{}", artefact.path);
        assert!(
            sums.contains(&format!("{digest}  {}\n", artefact.path)),
            "{} has no line in {CHECKSUMS}",
            artefact.path
        );
    }
    // ⚠ The manifest and the checksums are NOT in the list, because a file
    // cannot carry its own digest.
    assert!(
        !built
            .artefacts
            .iter()
            .any(|a| a.path == MANIFEST || a.path == CHECKSUMS),
        "a file cannot carry its own digest"
    );
}

#[test]
fn publish_the_tree_carries_no_source_and_no_vendored_dependency() {
    // ⛔ A CONSUMER OF THE DATA NEVER HAS TO REASON ABOUT SOMEBODY ELSE'S
    // LICENCE, because none of it is in what they downloaded.
    let (built, _) = built_into("scope");
    for artefact in &built.artefacts {
        for forbidden in ["crates/", "vendor/", "references/", "scripts/", "target/"] {
            assert!(
                !artefact.path.starts_with(forbidden),
                "{} is source and must not be published",
                artefact.path
            );
        }
    }
    // ⭐ And the licence IS there, so the tree says what it is.
    assert!(built.artefacts.iter().any(|a| a.path == "LICENSE"));
    assert_eq!(built.license, b_ids_schema::LICENSE);
}

#[test]
fn publish_the_tree_carries_the_corpus_the_formats_and_the_routes() {
    let (built, _) = built_into("shape");
    let has = |prefix: &str| built.artefacts.iter().any(|a| a.path.starts_with(prefix));
    for prefix in [
        "corpus/v1/",
        "raw/v1/",
        "formats/",
        "routes/",
        "anchors/",
        // ⭐ The published JA4 vectors, so an implementation in any language has
        // something to check itself against. docs/history/todo/validator.md, VALID-04.
        "vectors/",
    ] {
        assert!(has(prefix), "nothing was published under {prefix}");
    }
    // ⚠ A profile and its raw sidecar are published together, always.
    let profiles = built
        .artefacts
        .iter()
        .filter(|a| a.path.starts_with("corpus/v1/") && a.path.ends_with(".json"))
        .count();
    let sidecars = built
        .artefacts
        .iter()
        .filter(|a| a.path.starts_with("raw/v1/"))
        .count();
    // ⚠ Two of the corpus files are the derived index and pointer, which have
    // no sidecar.
    assert_eq!(
        profiles - 2,
        sidecars,
        "a profile is published without its bytes"
    );
}

#[test]
fn publish_a_tag_that_already_exists_is_refused() {
    // ⛔ A PUBLISHED RELEASE IS IMMUTABLE. Never re-upload an asset: cut a new
    // release that supersedes it. Consumers pin releases, and a mutated asset
    // breaks them silently.
    let (built, _) = built_into("tags");
    let want = tag(&built.layout, "2026-09-03", 1);
    assert_eq!(want, "v1.2026.09.03.1");

    let fresh = plan_release(&built, "2026-09-03", 1, &["v1.2026.09.02.1".to_owned()])
        .expect("a tag nothing holds");
    assert_eq!(fresh, want);

    let refused = plan_release(&built, "2026-09-03", 1, std::slice::from_ref(&want))
        .expect_err("a tag that already exists");
    assert_eq!(refused, NotReleasable::TagExists { tag: want });

    // ⚠ And the counter is what allows a second release on one day, so the
    // refusal is not a wall.
    plan_release(&built, "2026-09-03", 2, &["v1.2026.09.03.1".to_owned()])
        .expect("a second release on one day");
}

#[test]
fn publish_a_date_that_is_not_one_is_refused() {
    // ⛔ A TAG BUILT FROM A MALFORMED DATE sorts wrongly forever, and the tag is
    // the thing consumers pin.
    let (built, _) = built_into("dates");
    for bad in ["2026-9-3", "20260903", "tomorrow", ""] {
        assert!(
            matches!(
                plan_release(&built, bad, 1, &[]),
                Err(NotReleasable::BadDate { .. })
            ),
            "{bad} was accepted as a date"
        );
    }
}

#[test]
fn publish_the_moving_tags_are_one_per_major_and_one_overall() {
    // ⭐ So a script can fetch without listing releases.
    let (built, _) = built_into("moving");
    let moving = moving_tags(&built);
    assert!(moving.contains(&built.layout), "{moving:?}");
    assert!(moving.contains(&"latest".to_owned()), "{moving:?}");
}

#[test]
fn publish_a_build_with_no_artefact_is_not_releasable() {
    // ⛔ AN EMPTY RELEASE IS THE "step that exits 0 having done nothing" row of
    // docs/conventions/forbidden-patterns.md, wearing a version number.
    let (mut built, _) = built_into("empty");
    built.artefacts.clear();
    assert_eq!(
        plan_release(&built, "2026-09-03", 1, &[]),
        Err(NotReleasable::Empty)
    );
}

#[test]
fn publish_a_push_that_would_rewrite_the_data_branch_is_refused() {
    // ⛔ APPEND-ONLY, NEVER FORCE-PUSHED. A consumer pinning a commit on the
    // data branch keeps working forever, and that property is free right up
    // until somebody rewrites the branch.
    assert!(
        !would_rewrite(None, None),
        "the first push creates the branch and rewrites nothing"
    );
    assert!(
        !would_rewrite(Some("abc"), Some("abc")),
        "a commit built on the branch head is an append"
    );
    assert!(
        would_rewrite(Some("abc"), Some("def")),
        "a commit built on something the branch has moved past is a rewrite"
    );
    assert!(
        would_rewrite(Some("abc"), None),
        "an orphan commit pushed over an existing branch discards every commit on it"
    );
}

#[test]
fn publish_the_tree_names_no_path_outside_itself() {
    // ⛔ THE ROUTE MANIFEST RECORDS WHICH PROFILE EACH ROUTE WAS READ OUT OF,
    // and it took that path from the caller's `--root`. A build under an
    // absolute root therefore published the BUILDER'S filesystem, and produced
    // different bytes from a build of the same corpus under a relative one:
    // 668384 against 666710 on one machine, one minute apart. That is the
    // reproducibility PUB-01 asserts, and check-release could not see it
    // because it builds twice under ONE root. Found by running the assembler
    // both ways. docs/history/todo/publish.md, PUB-10.
    let (built, dir) = built_into("paths");
    // ⚠ THE CORPUS ROOT, NOT THE WORKSPACE ROOT. This asserts that the tree
    // does not name the path it was BUILT FROM, and `built_into` builds from
    // the resolved corpus. With B_IDS_CORPUS_ROOT pointing outside this tree
    // the workspace root would be a path the build never saw, so the assertion
    // would pass while testing nothing.
    let root = corpus_root()
        .canonicalize()
        .expect("the corpus root resolves")
        .to_string_lossy()
        .replace('\\', "/");
    // ⚠ Windows canonicalises to a `\?\` prefixed path; the trimmed form is a
    // substring of the untrimmed one, so searching for it catches both.
    let root = root.trim_start_matches("//?/").to_owned();
    assert!(!root.is_empty(), "the root is not a path");

    let mut checked = 0_usize;
    for name in built
        .artefacts
        .iter()
        .map(|a| a.path.clone())
        .chain([MANIFEST.to_owned(), CHECKSUMS.to_owned()])
    {
        let body = std::fs::read(dir.join(&name)).expect("it was written");
        let text = String::from_utf8_lossy(&body).replace('\\', "/");
        assert!(
            !text.contains(&root),
            "{name} names the builder's own filesystem"
        );
        checked += 1;
    }
    assert!(checked > 2, "the build produced almost nothing to check");
}

#[test]
fn publish_a_tag_this_rule_did_not_produce_is_refused() {
    // ⛔ A RELEASE JOB IS HANDED A TAG SOMEBODY PUSHED rather than asked to
    // invent one, so the tag is parsed and then rebuilt from its own parts. A
    // tag that does not survive that round trip is one this project's rule
    // never produced, and publishing under it would put two names on one
    // release. docs/history/todo/publish.md, PUB-10.
    let (built, _) = built_into("parse");
    assert_eq!(
        parse_tag("v1.2026.09.03.1"),
        Some(("v1".to_owned(), "2026-09-03".to_owned(), 1))
    );
    for bad in [
        "v1.2026.09.03",
        "v1.2026.09.03.1.2",
        "v1.2026.09.03.x",
        "latest",
        "",
    ] {
        assert!(parse_tag(bad).is_none(), "{bad} parsed as a release tag");
    }

    // ⚠ A ZERO-PADDED COUNTER PARSES and rebuilds to a different string, which
    // is why the round trip is the check rather than the parse.
    let (layout, date, counter) = parse_tag("v1.2026.09.03.01").expect("it parses");
    assert_eq!(tag(&layout, &date, counter), "v1.2026.09.03.1");

    // ⚠ AND A MALFORMED DATE IS plan_release's REFUSAL, not the parser's: the
    // parser says where the parts are and the rule says whether they are good.
    let (layout, date, counter) = parse_tag("v1.2026.9.3.1").expect("it parses");
    assert_eq!(layout, built.layout);
    assert!(
        matches!(
            plan_release(&built, &date, counter, &[]),
            Err(NotReleasable::BadDate { .. })
        ),
        "a malformed date was accepted"
    );
}

#[test]
fn publish_the_initial_release_is_explicit_and_one_time() {
    let (built, _) = built_into("initial-release");
    assert_eq!(
        plan_initial_release(&built, &[]).expect("the first release is free"),
        INITIAL_RELEASE_TAG
    );
    assert_eq!(
        plan_initial_release(&built, &[INITIAL_RELEASE_TAG.to_owned()]),
        Err(NotReleasable::TagExists {
            tag: INITIAL_RELEASE_TAG.to_owned()
        })
    );

    let mut empty = built;
    empty.artefacts.clear();
    assert_eq!(plan_initial_release(&empty, &[]), Err(NotReleasable::Empty));
}
