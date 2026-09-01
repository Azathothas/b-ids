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

    assert_eq!(selection.abandoned.len(), 1);
    assert_eq!(selection.abandoned[0].connection, 1);
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
    assert_eq!(kind(&empty), Kind::Abandoned);
    let selection = select(std::slice::from_ref(&empty));
    assert!(selection.cold.is_none());
    assert_eq!(selection.abandoned.len(), 1);
}
