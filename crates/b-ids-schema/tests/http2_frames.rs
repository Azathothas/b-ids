//! SCHEMA-03. The HTTP/2 half, as an ordered frame sequence.
//!
//! The acceptance: a profile that omits one settings key and a profile that
//! carries it at the protocol default serialise differently and compare
//! unequal.
//!
//! ⛔ Every test name starts with `http2_frames`, because
//! `cargo test -p b-ids-schema http2_frames` is the entry's acceptance command.

mod support;

use b_ids_schema::http2::{Frame, Http2Half, SettingEntry, StreamPriority};
use support::fixture_http2;

/// `SETTINGS_MAX_FRAME_SIZE`, and the protocol's own default for it.
///
/// ⚠ This is the setting the absence rule was written for: one browser sends no
/// entry for it where a general-purpose stack sends the default.
const MAX_FRAME_SIZE: u16 = 5;
const MAX_FRAME_SIZE_DEFAULT: u32 = 16_384;

fn with_setting(half: &Http2Half, entry: SettingEntry) -> Http2Half {
    let mut out = half.clone();
    for frame in &mut out.frames {
        if let Frame::Settings { entries } = frame {
            entries.push(entry);
        }
    }
    out
}

#[test]
fn http2_frames_omitting_a_setting_differs_from_sending_it_at_the_default() {
    // ⛔ The one substitution this model exists to make impossible. A settings
    // MAP filled in with defaults cannot express the difference, and the
    // difference is visible on the wire.
    let omitted = fixture_http2();
    let at_default = with_setting(
        &omitted,
        SettingEntry {
            id: MAX_FRAME_SIZE,
            value: MAX_FRAME_SIZE_DEFAULT,
        },
    );

    assert_ne!(omitted, at_default);
    assert_ne!(
        serde_json::to_string(&omitted).expect("serialises"),
        serde_json::to_string(&at_default).expect("serialises")
    );
    assert!(!omitted.sends_setting(MAX_FRAME_SIZE));
    assert!(at_default.sends_setting(MAX_FRAME_SIZE));
}

#[test]
fn http2_frames_keep_settings_order() {
    // Order is part of the fingerprint, so two profiles differing only in the
    // order of one settings list are two different profiles.
    let ordered = fixture_http2();
    let mut reversed = ordered.clone();
    for frame in &mut reversed.frames {
        if let Frame::Settings { entries } = frame {
            entries.reverse();
        }
    }
    assert_ne!(ordered, reversed);
    let ids: Vec<u16> = ordered
        .settings()
        .expect("a settings frame")
        .iter()
        .map(|e| e.id)
        .collect();
    assert_eq!(ids, vec![1, 2, 4, 6]);
}

#[test]
fn http2_frames_window_update_is_named_for_the_wire() {
    // ⚠ The increment is the window minus the protocol's own 65535 default. One
    // shipped database held the window in one entry and the increment in seven
    // others, and the seven emitted a value 65535 short.
    let half = fixture_http2();
    assert_eq!(half.window_size_increment(), Some(15_663_105));
    let json = serde_json::to_value(&half).expect("serialises");
    let text = json.to_string();
    assert!(
        text.contains("window_size_increment"),
        "the field has to be named for the wire: {text}"
    );
    // ⚠ The rule is that no field is named for the window INSTEAD OF the
    // increment. SCHEMA-09 asks for the human number BESIDE it, separately
    // named, with a check asserting the arithmetic between the two, so
    // `connection_window` existing is required rather than forbidden. What is
    // forbidden is a bare `window_size` that could hold either.
    assert!(
        !text.contains("\"window_size\":"),
        "a field named for neither unit is how one field came to hold both: {text}"
    );
    assert_eq!(half.connection_window, Some(15_728_640));
    assert!(
        half.check_units().is_empty(),
        "the two units must differ by the protocol default"
    );
}

#[test]
fn http2_frames_stream_weight_carries_the_wire_unit() {
    // ⚠ HTTP/2 encodes weight as weight minus one, so a tool passing 256 puts
    // 255 on the wire. They are one quantity in two units.
    let priority = StreamPriority {
        exclusive: true,
        stream_dependency: 0,
        weight_wire: 255,
    };
    assert_eq!(priority.weight_spec(), 256);
    let text = serde_json::to_string(&priority).expect("serialises");
    assert!(text.contains("weight_wire"), "{text}");
    assert!(
        !text.contains("weight_spec"),
        "storing both units is how a field ends up holding whichever its last writer believed in: {text}"
    );
}

#[test]
fn http2_frames_an_absent_priority_block_is_not_a_block_of_zeroes() {
    // ⛔ A rendered Akamai string cannot tell "no block sent" from "block not
    // read", which is why the field is the parsed five bytes and the string is
    // derived from it.
    let absent = Http2Half {
        stream_priority: None,
        ..fixture_http2()
    };
    let zeroed = Http2Half {
        stream_priority: Some(StreamPriority {
            exclusive: false,
            stream_dependency: 0,
            weight_wire: 0,
        }),
        ..fixture_http2()
    };
    assert_ne!(absent, zeroed);
    assert_ne!(
        serde_json::to_string(&absent).expect("serialises"),
        serde_json::to_string(&zeroed).expect("serialises")
    );
}

#[test]
fn http2_frames_akamai_text_is_derived_and_loses_what_the_model_keeps() {
    // ⭐ Derived on request so nothing has to store it, and the test states the
    // loss rather than leaving a reader to discover it: an absent block and a
    // block of zeroes render the same.
    let half = fixture_http2();
    assert_eq!(
        half.akamai_text(),
        "1:65536;2:0;4:6291456;6:262144|15663105|1:1:0:255|m,a,s,p"
    );

    let absent = Http2Half {
        stream_priority: None,
        ..fixture_http2()
    };
    let zeroed = Http2Half {
        stream_priority: Some(StreamPriority {
            exclusive: false,
            stream_dependency: 0,
            weight_wire: 0,
        }),
        ..fixture_http2()
    };
    assert_eq!(
        absent.akamai_text().split('|').nth(2),
        Some("0"),
        "an absent block renders as 0"
    );
    assert_ne!(
        absent, zeroed,
        "the model still distinguishes what the rendering cannot"
    );
}

#[test]
fn http2_frames_keep_a_frame_type_with_no_name() {
    // The same rule as an unknown TLS extension: a sequence that silently omits
    // a frame is a sequence nobody can compare.
    let mut half = fixture_http2();
    half.frames.push(Frame::Other {
        frame_type: 0xfa,
        payload_hex: "0011".to_owned(),
    });
    let text = serde_json::to_string(&half).expect("serialises");
    let back: Http2Half = serde_json::from_str(&text).expect("deserialises");
    assert_eq!(half, back);
    assert!(matches!(
        back.frames.last(),
        Some(Frame::Other {
            frame_type: 0xfa,
            ..
        })
    ));
}
