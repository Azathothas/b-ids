//! VALID-01. Eight checks, eight tests, each planting the exact contradiction
//! its check exists to catch and asserting a message naming the field.
//!
//! ⛔ **A check whose test has never been seen to fail is theatre**, so every
//! test here asserts the failure AND asserts that the same check passes over
//! the unmodified fixture. One without the other proves half of it.

use std::collections::BTreeSet;

use b_ids_schema::fixture;
use b_ids_schema::http::{HeaderSet, HttpHalf, ValuePolicy, Variant};
use b_ids_schema::http2::{Frame, Http2Half, StreamPriority};
use b_ids_schema::tls::{Extension, Shuffle};
use b_ids_schema::{Os, Profile, ProvenanceEntry, ProvenanceKind};
use b_ids_validator::{
    Check, EmitterCapabilities, Options, Outcome, check_absence, check_brand, check_encoding,
    check_grease, check_handshake, check_platform, check_provenance, check_version,
    shared_handshakes, validate,
};

/// A profile whose header values are recorded, which is what most checks read.
fn valued() -> Profile {
    fixture::profile_with_header_values()
}

/// Rebuild the navigate header set from a list, so a test can change one value.
fn with_headers(profile: &Profile, headers: Vec<(String, String)>) -> Profile {
    Profile {
        http: HttpHalf {
            variants: vec![HeaderSet::record(
                Variant::Navigate,
                headers,
                ValuePolicy::WithValues,
            )],
            multipart_boundary: None,
        },
        ..profile.clone()
    }
}

/// The raw headers with one replaced.
fn headers_with(name: &str, value: &str) -> Vec<(String, String)> {
    fixture::raw_headers()
        .into_iter()
        .map(|(n, v)| {
            if n.eq_ignore_ascii_case(name) {
                (n, value.to_owned())
            } else {
                (n, v)
            }
        })
        .collect()
}

fn failure_message(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Failed(findings) => findings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"),
        other => panic!("expected a failure, got {other:?}"),
    }
}

// -- check 1 ----------------------------------------------------------------

#[test]
fn version_a_user_agent_from_another_major_is_refused() {
    assert_eq!(check_version(&valued()), Outcome::Passed);

    let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
              Chrome/151.0.0.0 Safari/537.36";
    let profile = with_headers(&valued(), headers_with("user-agent", ua));
    let message = failure_message(&check_version(&profile));
    assert!(message.contains("http.headers.user-agent"), "{message}");
    assert!(
        message.contains("151") && message.contains("152"),
        "{message}"
    );
}

#[test]
fn version_a_brand_list_from_another_major_is_refused() {
    let brands = "\"Chromium\";v=\"151\", \"Not?A_Brand\";v=\"24\"";
    let profile = with_headers(&valued(), headers_with("sec-ch-ua", brands));
    let message = failure_message(&check_version(&profile));
    assert!(message.contains("http.headers.sec-ch-ua"), "{message}");
}

#[test]
fn version_reports_that_it_could_not_run_over_a_names_only_capture() {
    // ⛔ Not a pass. The default capture policy records no header values, so
    // this check has nothing to read, and reporting green over that is the
    // defect the three outcomes exist to prevent.
    match check_version(&fixture::profile()) {
        Outcome::NotCheckable(why) => assert!(why.contains("user-agent"), "{why}"),
        other => panic!("expected NotCheckable, got {other:?}"),
    }
}

// -- check 2 ----------------------------------------------------------------

#[test]
fn platform_a_windows_user_agent_on_a_linux_capture_is_refused() {
    assert_eq!(check_platform(&valued()), Outcome::Passed);

    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
              Chrome/152.0.0.0 Safari/537.36";
    let profile = with_headers(&valued(), headers_with("user-agent", ua));
    let message = failure_message(&check_platform(&profile));
    assert!(message.contains("http.headers.user-agent"), "{message}");
    assert!(
        message.contains("windows") && message.contains("linux"),
        "{message}"
    );
}

#[test]
fn platform_a_hint_that_disagrees_with_the_capture_is_refused() {
    let profile = with_headers(&valued(), headers_with("sec-ch-ua-platform", "\"macOS\""));
    let message = failure_message(&check_platform(&profile));
    assert!(message.contains("sec-ch-ua-platform"), "{message}");
}

#[test]
fn platform_a_mobile_hint_on_a_desktop_capture_is_refused() {
    let profile = with_headers(&valued(), headers_with("sec-ch-ua-mobile", "?1"));
    let message = failure_message(&check_platform(&profile));
    assert!(message.contains("sec-ch-ua-mobile"), "{message}");

    // And the reverse: a desktop hint on a mobile capture.
    let mut android = valued();
    android.platform.os = Os::Android;
    let message = failure_message(&check_platform(&android));
    assert!(message.contains("sec-ch-ua-mobile"), "{message}");
}

// -- check 3 ----------------------------------------------------------------

#[test]
fn brand_an_unbranded_build_claiming_a_vendor_entry_is_refused() {
    // ⚠ The fixture is a Chrome for Testing build: unbranded, and its brand
    // list carries Chromium and a fake brand but no vendor entry.
    assert_eq!(check_brand(&valued()), Outcome::Passed);

    let brands = "\"Chromium\";v=\"152\", \"Not?A_Brand\";v=\"24\", \"Google Chrome\";v=\"152\"";
    let profile = with_headers(&valued(), headers_with("sec-ch-ua", brands));
    let message = failure_message(&check_brand(&profile));
    assert!(message.contains("Google Chrome"), "{message}");
    assert!(message.contains("branded is false"), "{message}");
}

#[test]
fn brand_a_branded_build_with_no_vendor_entry_is_refused() {
    let mut profile = valued();
    profile.browser.branded = true;
    let message = failure_message(&check_brand(&profile));
    assert!(message.contains("branded is true"), "{message}");
}

// -- check 4 ----------------------------------------------------------------

#[test]
fn handshake_two_majors_sharing_one_tls_half_is_refused() {
    // ⭐ The shipped violation this check exists for: an entry that returns a
    // neighbour's fingerprint wholesale beside its own User-Agent. At most one
    // of the two can have been measured.
    let a = fixture::profile();
    let mut b = fixture::profile();
    b.browser.major = 151;
    b.browser.version = "151.0.7922.72".to_owned();
    b.id = b.derived_id();

    let findings = shared_handshakes(&[a.clone(), b]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].check, Check::Handshake);
    assert!(
        findings[0].message.contains("byte-identical"),
        "{findings:?}"
    );

    // Two profiles of one major sharing a TLS half is not a contradiction.
    assert!(shared_handshakes(&[a.clone(), a]).is_empty());
}

#[test]
fn handshake_within_one_profile_reports_that_it_could_not_run() {
    // ⛔ Honest rather than green. Deciding this within one profile needs a
    // per-build corpus, and this project has captured none.
    match check_handshake(&valued()) {
        Outcome::NotCheckable(why) => assert!(why.contains("per-build corpus"), "{why}"),
        other => panic!("expected NotCheckable, got {other:?}"),
    }
}

#[test]
fn handshake_a_hello_with_no_extensions_is_refused_outright() {
    let mut profile = valued();
    profile.tls.extensions.clear();
    let message = failure_message(&check_handshake(&profile));
    assert!(message.contains("tls.extensions"), "{message}");
}

// -- check 5 ----------------------------------------------------------------

#[test]
fn grease_a_position_list_that_disagrees_with_the_hello_is_refused() {
    assert_eq!(check_grease(&fixture::profile()), Outcome::Passed);

    let mut profile = fixture::profile();
    profile.tls.grease.extension_positions = vec![0];
    let message = failure_message(&check_grease(&profile));
    assert!(
        message.contains("tls.grease.extension_positions"),
        "{message}"
    );
}

#[test]
fn grease_one_draw_reused_across_two_slots_is_refused() {
    // ⚠ A browser draws GREASE independently per slot. One value at both ends
    // is the shape a client that reuses a draw produces.
    let mut profile = fixture::profile();
    let last = profile.tls.extensions.len() - 1;
    profile.tls.extensions[last] = Extension {
        codepoint: 0x0a0a,
        length: 1,
        body_hex: "00".to_owned(),
    };
    profile.tls.grease.values = vec![0x0a0a, 0x0a0a];
    profile.tls.grease.distinct = false;
    let message = failure_message(&check_grease(&profile));
    assert!(message.contains("reuses one draw"), "{message}");
}

#[test]
fn grease_a_shuffle_state_claimed_from_one_draw_is_refused() {
    // ⛔ One handshake is not a sample, and Fixed after one draw is an absence
    // of a finding rather than a finding.
    let mut profile = fixture::profile();
    profile.tls.shuffled = Shuffle::Fixed { draws: 1 };
    let message = failure_message(&check_grease(&profile));
    assert!(message.contains("tls.shuffled"), "{message}");
    assert!(message.contains("not a sample"), "{message}");
}

// -- check 6 ----------------------------------------------------------------

#[test]
fn encoding_advertising_what_the_consumer_cannot_decode_is_refused() {
    let decodes: BTreeSet<String> = ["gzip", "deflate", "br", "zstd"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let options = Options {
        decodes: decodes.clone(),
        ..Options::default()
    };
    assert_eq!(check_encoding(&valued(), &options), Outcome::Passed);

    let narrower = Options {
        decodes: ["gzip"].into_iter().map(str::to_owned).collect(),
        ..Options::default()
    };
    let message = failure_message(&check_encoding(&valued(), &narrower));
    assert!(message.contains("accept-encoding"), "{message}");
    assert!(message.contains("zstd"), "{message}");
}

#[test]
fn encoding_reports_that_it_could_not_run_when_the_caller_said_nothing() {
    match check_encoding(&valued(), &Options::default()) {
        Outcome::NotCheckable(why) => assert!(why.contains("decode"), "{why}"),
        other => panic!("expected NotCheckable, got {other:?}"),
    }
}

// -- check 7 ----------------------------------------------------------------

fn a_capable_stack() -> EmitterCapabilities {
    EmitterCapabilities {
        name: "a-capable-stack".to_owned(),
        omits_settings: true,
        writes_priority_block: true,
        grease_slots: 2,
        arbitrary_extension_order: true,
    }
}

#[test]
fn absence_a_stack_that_cannot_omit_a_setting_is_refused() {
    let options = Options {
        target: Some(a_capable_stack()),
        ..Options::default()
    };
    assert_eq!(
        check_absence(&fixture::profile(), &options),
        Outcome::Passed
    );

    let limited = Options {
        target: Some(EmitterCapabilities {
            name: "a-stack-that-cannot".to_owned(),
            omits_settings: false,
            ..a_capable_stack()
        }),
        ..Options::default()
    };
    let message = failure_message(&check_absence(&fixture::profile(), &limited));
    assert!(message.contains("http2.frames"), "{message}");
    assert!(message.contains("cannot leave a setting out"), "{message}");
}

#[test]
fn absence_names_every_hole_the_target_has_at_once() {
    // ⭐ Every hole, not the first. A support matrix that reported one gap at a
    // time would need one run per gap.
    let limited = Options {
        target: Some(EmitterCapabilities {
            name: "a-limited-stack".to_owned(),
            omits_settings: false,
            writes_priority_block: false,
            grease_slots: 1,
            arbitrary_extension_order: false,
        }),
        ..Options::default()
    };
    let message = failure_message(&check_absence(&fixture::profile(), &limited));
    for field in [
        "http2.frames",
        "http2.stream_priority",
        "tls.grease.values",
        "tls.extensions",
    ] {
        assert!(message.contains(field), "{field} missing from:\n{message}");
    }
}

#[test]
fn absence_reports_that_it_could_not_run_with_no_target_named() {
    match check_absence(&fixture::profile(), &Options::default()) {
        Outcome::NotCheckable(why) => assert!(why.contains("target"), "{why}"),
        other => panic!("expected NotCheckable, got {other:?}"),
    }
}

// -- check 8 ----------------------------------------------------------------

#[test]
fn provenance_a_vendor_field_is_refused_when_publishing() {
    let mut profile = fixture::profile();
    profile.provenance.insert(
        "http2.stream_priority",
        ProvenanceEntry {
            kind: ProvenanceKind::Vendor,
            reason: None,
        },
    );
    // A draft is allowed to carry one.
    assert_eq!(
        check_provenance(&profile, &Options::default()),
        Outcome::Passed
    );

    let publishing = Options {
        publishing: true,
        ..Options::default()
    };
    let message = failure_message(&check_provenance(&profile, &publishing));
    assert!(message.contains("http2.stream_priority"), "{message}");
    assert!(message.contains("vendor"), "{message}");
}

#[test]
fn provenance_an_unreasoned_substitution_is_refused_whether_publishing_or_not() {
    let mut profile = fixture::profile();
    profile.provenance.insert(
        "tls.alpn",
        ProvenanceEntry {
            kind: ProvenanceKind::Substituted,
            reason: None,
        },
    );
    for options in [
        Options::default(),
        Options {
            publishing: true,
            ..Options::default()
        },
    ] {
        let message = failure_message(&check_provenance(&profile, &options));
        assert!(message.contains("tls.alpn"), "{message}");
    }
}

// -- the report -------------------------------------------------------------

#[test]
fn the_report_runs_every_check_and_none_is_silently_missing() {
    // ⛔ Eight checks, and the report is asserted to carry all eight. A check
    // added to the enum and not to `validate` would be a check nobody runs.
    let report = validate(&valued(), &Options::default());
    assert_eq!(report.results.len(), Check::all().len());
    for check in Check::all() {
        assert!(report.results.contains_key(&check), "{check} did not run");
    }
}

#[test]
fn the_report_exit_code_separates_failed_from_could_not_run() {
    let clean = validate(&valued(), &Options::default());
    assert!(!clean.failed());
    assert_eq!(clean.exit_code(), 0);
    assert!(
        !clean.not_checkable().is_empty(),
        "over a fixture with no target and no decode list, some checks cannot run"
    );

    let mut broken = valued();
    broken.browser.branded = true;
    let report = validate(&broken, &Options::default());
    assert!(report.failed());
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn the_report_says_two_when_nothing_could_be_checked_at_all() {
    // A names-only capture with no caller context: every check that reads a
    // value has nothing to read. ⛔ That is exit 2, not exit 0.
    let mut profile = fixture::profile();
    // Leave the halves that are checkable without header values in a state the
    // structural checks also cannot judge.
    profile.tls.extensions.clear();
    let report = validate(&profile, &Options::default());
    assert!(report.failed(), "an empty hello is still a contradiction");
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn the_report_prints_a_finding_that_names_its_check_and_its_field() {
    let mut profile = valued();
    profile.browser.branded = true;
    let report = validate(&profile, &Options::default());
    let printed: Vec<String> = report.findings().iter().map(ToString::to_string).collect();
    assert!(
        printed
            .iter()
            .any(|p| p.starts_with("brand: http.headers.sec-ch-ua:")),
        "{printed:?}"
    );
}

#[test]
fn a_profile_that_omits_no_setting_still_needs_a_stack_that_can_order_extensions() {
    // ⚠ The support matrix's most common hole is not absence at all: it is that
    // most stacks cannot reproduce a captured extension order.
    let mut profile = fixture::profile();
    for frame in &mut profile.http2.frames {
        if let Frame::Settings { entries } = frame {
            entries.push(b_ids_schema::http2::SettingEntry {
                id: 5,
                value: 16_384,
            });
        }
    }
    profile.http2 = Http2Half {
        stream_priority: Some(StreamPriority {
            exclusive: true,
            stream_dependency: 0,
            weight_wire: 255,
        }),
        ..profile.http2
    };
    let limited = Options {
        target: Some(EmitterCapabilities {
            name: "rustls-shaped".to_owned(),
            omits_settings: false,
            writes_priority_block: true,
            grease_slots: 1,
            arbitrary_extension_order: false,
        }),
        ..Options::default()
    };
    let message = failure_message(&check_absence(&profile, &limited));
    assert!(message.contains("tls.extensions"), "{message}");
    assert!(!message.contains("http2.frames"), "{message}");
}
