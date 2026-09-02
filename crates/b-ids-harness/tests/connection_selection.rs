//! HARNESS-07. A browser opens sockets it abandons, and it resumes.
//!
//! The acceptance: a fixture of a thirteen-connection navigation selects
//! connection two, and the resumed connections are emitted as a separate
//! labelled set whose digest differs from the cold one.
//!
//! ⛔ Every test name starts with `connection_selection`, because
//! `cargo test -p b-ids-harness connection_selection` is the entry's acceptance
//! command.

mod support;

use b_ids_harness::select::{Kind, kind, select};
use b_ids_harness::{Capture, Config, Protocol};

use support::{feed, fixture_bytes, one_connection, thirteen_connection_navigation};

#[test]
fn connection_selection_keeps_the_first_connection_that_reached_http2() {
    // ⛔ Neither the first nor the last. The first carried no HTTP/2 at all, a
    // preconnect the browser abandoned, and the last resumed.
    let navigation = thirteen_connection_navigation();
    assert_eq!(navigation.len(), 13);

    let selection = select(&navigation);
    let cold = selection.cold.expect("a cold connection");
    assert_eq!(
        cold.connection, 2,
        "the selection did not take connection 2"
    );
    assert_eq!(selection.connections(), 13, "a connection was dropped");
}

#[test]
fn connection_selection_records_the_resumed_ones_as_their_own_set() {
    // ⛔ Separately, and never averaged. Two connections that differ are the
    // data, and a corpus that folded them together would publish a handshake
    // neither of them sent.
    let navigation = thirteen_connection_navigation();
    let selection = select(&navigation);

    assert_eq!(selection.no_http2.len(), 1);
    assert_eq!(selection.no_http2[0].connection, 1);
    assert_eq!(selection.resumed.len(), 11);
    assert!(selection.additional_cold.is_empty());
    assert_eq!(
        selection
            .resumed
            .iter()
            .map(|c| c.connection)
            .collect::<Vec<_>>(),
        (3..=13).collect::<Vec<u32>>()
    );
    assert!(selection.resumption_observable);
}

#[test]
fn connection_selection_the_resumed_handshake_differs_from_the_cold_one() {
    // ⚠ The acceptance says "digest". The digest implementations are VALID-04
    // and none exists yet, so the comparison is over the RAW HELLO, which is a
    // stronger discriminator rather than a weaker one: every digest strips
    // GREASE and most sort, so two hellos that differ can share a digest while
    // no two differing hellos share their bytes.
    let navigation = thirteen_connection_navigation();
    let selection = select(&navigation);
    let cold = selection.cold.expect("a cold connection");

    for resumed in &selection.resumed {
        assert_ne!(
            resumed.raw_hex, cold.raw_hex,
            "connection {} is indistinguishable from the cold handshake",
            resumed.connection
        );
    }

    // ⭐ And the difference is the one the reading names: a pre-shared key
    // where the cold handshake offered a session ticket.
    let cold_tls = cold.tls.as_ref().expect("the cold hello parsed");
    assert!(!b_ids_harness::select::offers_pre_shared_key(cold_tls));
    let resumed_tls = selection.resumed[0].tls.as_ref().expect("parsed");
    assert!(b_ids_harness::select::offers_pre_shared_key(resumed_tls));
}

#[test]
fn connection_selection_never_deduplicates_two_connections_that_agree() {
    // ⛔ Deduplicating by digest is the tempting tidy-up and it destroys the
    // measurement: the number of connections a navigation opens is itself data.
    let navigation = thirteen_connection_navigation();
    let selection = select(&navigation);
    let raw: Vec<&str> = selection
        .resumed
        .iter()
        .map(|c| c.raw_hex.as_str())
        .collect();
    assert_eq!(raw.len(), 11);
    assert!(
        raw.windows(2).all(|w| w[0] == w[1]),
        "the constructed resumed connections are byte-identical by design"
    );
}

#[test]
fn connection_selection_says_when_resumption_is_not_observable() {
    // ⛔ A capture with no ClientHello cannot be asked whether it resumed, and
    // calling it cold would be a claim the bytes do not support. An unavailable
    // field is absent with a reason.
    let captures = feed(
        Config {
            protocol: Protocol::Cleartext,
            ..one_connection()
        },
        vec![fixture_bytes("h2-connection.hex")],
    );
    let selection = select(&captures);
    assert!(selection.cold.is_some());
    assert!(
        !selection.resumption_observable,
        "a cleartext capture carries no hello, so resumption is not readable"
    );
}

#[test]
fn connection_selection_classifies_a_connection_that_sent_nothing_as_abandoned() {
    let empty = Capture {
        schema: b_ids_harness::CAPTURE_SCHEMA.to_owned(),
        connection: 1,
        at: "2026-09-01T00:00:00Z".to_owned(),
        peer: "REDACTED".to_owned(),
        protocol: Protocol::TlsRaw,
        bytes_read: 0,
        raw_hex: String::new(),
        tls: None,
        http2: None,
        termination: None,
        request_line: None,
        header_names: Vec::new(),
        header_values: Vec::new(),
        notes: Vec::new(),
    };
    assert_eq!(kind(&empty), Kind::NoHttp2);
    let selection = select(std::slice::from_ref(&empty));
    assert!(selection.cold.is_none());
    assert_eq!(selection.no_http2.len(), 1);
    // ⛔ AND NEITHER HALF IS AVAILABLE. A connection that sent nothing carries
    // no hello either, so this is not the HARNESS-15 case: there is nothing to
    // take a TLS half from.
    assert!(selection.tls_from.is_none());
    assert!(selection.http2_from.is_none());
}

/// ⭐ The report reads the cold count off the selection.
///
/// ⛔ **This is the guard for a defect that shipped.** `b-ids-corpus add`
/// printed `1 cold` as a literal inside its format string, so the line was true
/// only on the runs where it happened to be. A hardcoded metric is the
/// `docs/conventions/forbidden-patterns.md` row "a display that lies".
#[test]
fn connection_selection_reports_the_cold_count_it_measured() {
    let navigation = thirteen_connection_navigation();
    let selection = select(&navigation);
    assert_eq!(
        selection.cold_count(),
        1,
        "the fixture has one cold connection"
    );
    assert_eq!(
        selection.report(),
        "13 connection(s): 1 cold, 11 resumed, 0 further cold, 1 with no http2"
    );
}

/// ⭐ A navigation whose every HTTP/2 connection resumed has NO cold one.
///
/// ⚠ **Measured on a hosted runner, 2026-09-02.** `capture.yml` run
/// 33579619515's `linux64` lane abandoned both of its first two connections
/// after the handshake and every later one resumed, so nothing was publishable
/// from it. The report said `1 cold` on the line above the refusal saying there
/// was none.
#[test]
fn connection_selection_reports_no_cold_connection_when_every_one_resumed() {
    let mut navigation = thirteen_connection_navigation();
    // ⛔ The cold connection is abandoned, which is what the runner did. It is
    // not removed: a dropped connection would change the total and hide the
    // shape this test is about.
    navigation[1].http2 = None;

    let selection = select(&navigation);
    assert!(
        selection.cold.is_none(),
        "every connection that reached HTTP/2 resumed"
    );
    assert_eq!(selection.cold_count(), 0);
    assert_eq!(selection.connections(), 13, "a connection was dropped");
    assert_eq!(
        selection.report(),
        "13 connection(s): 0 cold, 11 resumed, 0 further cold, 2 with no http2"
    );
}

/// ⭐ HARNESS-15. The two halves are selected independently.
///
/// ⚠ **This is the runner's shape, reproduced.** `capture.yml` runs
/// 33579619515 and 33580371329: on `ubuntu-latest` every connection that
/// reached HTTP/2 had resumed, and the only cold hellos arrived on connections
/// the browser abandoned. The rule that required ONE connection to carry both
/// halves threw those hellos away and published nothing.
#[test]
fn connection_selection_takes_the_tls_half_from_a_connection_that_reached_no_http2() {
    let mut navigation = thirteen_connection_navigation();
    // ⛔ Connection 2 is the one that carried both halves. Taking its HTTP/2
    // away leaves connections 1 and 2 with cold hellos and no frames, and
    // connections 3 to 13 with frames and a resumed hello. No connection
    // carries both, which is the case that used to publish nothing.
    navigation[1].http2 = None;

    let selection = select(&navigation);

    let tls_from = selection.tls_from.expect("connection 1 sent a cold hello");
    assert_eq!(
        tls_from.connection, 1,
        "the TLS half comes from the first connection whose hello offers no pre-shared key, \
         whether or not it reached HTTP/2"
    );
    assert!(
        !tls_from.reached_h2(),
        "the point of this test is that the TLS half came from a connection with no HTTP/2"
    );

    let http2_from = selection.http2_from.expect("connection 3 reached HTTP/2");
    assert_eq!(
        http2_from.connection, 3,
        "the HTTP/2 half comes from the first connection that reached it, resumed or not"
    );

    // ⛔ AND THE PROFILE HAS TO SAY SO. Two halves from two sockets of one
    // navigation is a condition of the measurement rather than a detail.
    assert_eq!(selection.one_connection(), Some(false));
    assert_eq!(
        selection.halves(),
        "tls from connection 1, http2 from connection 3"
    );
}

/// ⚠ The ordinary case, and it must not have changed.
#[test]
fn connection_selection_takes_both_halves_from_one_connection_when_one_carries_both() {
    let navigation = thirteen_connection_navigation();
    let selection = select(&navigation);

    // ⚠ Connection 1 sent a cold hello and no frames; connection 2 sent both.
    // The TLS half is still connection 1's, because it is the FIRST cold hello
    // and the rule does not ask about HTTP/2.
    assert_eq!(selection.tls_from.expect("a cold hello").connection, 1);
    assert_eq!(selection.http2_from.expect("frames").connection, 2);
    assert_eq!(selection.one_connection(), Some(false));
}

/// ⛔ A navigation whose every hello resumed still publishes nothing.
///
/// ⚠ **The rule this entry relaxed is not the rule that a resumed hello is not
/// a cold one.** That one stands: a profile built from a resumed handshake
/// would record a hello no fresh client sends.
#[test]
fn connection_selection_publishes_nothing_when_every_hello_resumed() {
    let mut navigation = thirteen_connection_navigation();
    // ⛔ Every hello resumed, which the fixture does not do on its own: its
    // first two are cold. Truncating to the resumed ones is the shape.
    navigation.drain(0..2);
    assert_eq!(navigation.len(), 11);

    let selection = select(&navigation);
    assert!(
        selection.tls_from.is_none(),
        "every hello offered a pre-shared key, so there is no cold one to publish"
    );
    // ⚠ The HTTP/2 half IS available, and that is the point of selecting per
    // half: the absence is one-sided and the message says which side.
    assert!(selection.http2_from.is_some());
    assert_eq!(selection.one_connection(), None);
}
