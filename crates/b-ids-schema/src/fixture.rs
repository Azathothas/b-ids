//! A well-formed profile, so a test can break exactly one thing.
//!
//! ⛔ **Nothing here is a measurement.** The values are SHAPED like a capture
//! and are not one, and no field of it may be copied into the corpus. A value
//! this project did not measure lives in `docs/inherited-claims.md` with its
//! source, and a value it did measure lives in a profile.
//!
//! ⚠ **Behind the `fixtures` feature**, so it is not compiled into anything a
//! consumer links. It exists here rather than in one crate's test directory
//! because two crates test against it, and a fixture copied into a second place
//! is two fixtures that drift.

use crate::http::{HeaderSet, HttpHalf, ValuePolicy, Variant};
use crate::http2::{Frame, Http2Half, SettingEntry, StreamPriority};
use crate::tls::{Ech, Extension, Grease, KeyShare, Shuffle, TlsHalf};
use crate::{
    Browser, Captured, Channel, Connections, Digests, Os, Platform, PlatformToken, Profile,
    ProfileId, Provenance, ProvenanceEntry, ProvenanceKind, Raw, Resumption, SCHEMA_ID, Trust,
};

/// A profile that is well formed and internally coherent.
#[must_use]
pub fn profile() -> Profile {
    let browser = Browser {
        name: "Chrome".to_owned(),
        version: "152.0.7977.64".to_owned(),
        major: 152,
        channel: Channel::Stable,
        branded: false,
    };
    let platform = Platform {
        os: Os::Linux,
        arch: "x86_64".to_owned(),
        distribution: Some("debian-bookworm".to_owned()),
    };
    let id = ProfileId::derive(
        &browser.name,
        &browser.version,
        &PlatformToken::derive(platform.os, &platform.arch),
        browser.channel,
    );

    let mut provenance = Provenance::new();
    provenance.insert(
        "tls.cipher_suites",
        ProvenanceEntry {
            kind: ProvenanceKind::Wire,
            reason: None,
        },
    );
    provenance.insert(
        "http.headers.sec-ch-ua-platform",
        ProvenanceEntry {
            kind: ProvenanceKind::Substituted,
            reason: Some("platform-token".to_owned()),
        },
    );

    Profile {
        schema: SCHEMA_ID.to_owned(),
        id,
        browser,
        platform,
        captured: Captured {
            at: "2026-08-30T03:53:11Z".to_owned(),
            method: "container".to_owned(),
            harness: "b-ids-harness 0.0.0".to_owned(),
            operator: "ci".to_owned(),
            // ⚠ Not `not-applicable`, and it cannot be: this fixture carries
            // both a `ClientHello` and HTTP/2 frames, and `Profile::check`
            // refuses that combination under a trust configuration that says no
            // handshake completed.
            trust: Trust::SpkiPin,
            // ⚠ `Offered`, because that is the configuration every capture
            // before 2026-09-02 was taken under and this fixture stands in for
            // one of them. ⛔ Not `None`: a fixture whose condition was
            // unrecorded would exercise the read path for an OLD profile, and
            // this one stands in for what the harness writes today.
            resumption: Some(Resumption::Offered),
            switches: vec![
                "--user-data-dir=(throwaway)".to_owned(),
                "--headless=new".to_owned(),
            ],
            // ⚠ None, because nothing fetched anything. A fixture that claimed
            // an acquisition would be claiming a route and a digest nobody
            // produced, which is the one thing this file must never do.
            acquisition: None,
            // ⚠ One connection carried both halves, which is the ordinary case
            // and the one this fixture stands in for. ⛔ Not `None`: absent
            // means a profile written before the field existed, and a fixture
            // standing in for what the harness writes today has to carry what
            // it writes today.
            connections: Some(Connections { tls: 1, http2: 1 }),
        },
        tls: tls(),
        http2: http2(),
        http: http(),
        digests: Digests::default(),
        raw: Raw::default(),
        provenance,
        supersedes: None,
    }
}

/// The same profile with header values recorded, for the checks that read one.
///
/// ⚠ Most checks in `b-ids-validator` read a header VALUE, and the default
/// capture policy records none. This is what a capture taken deliberately with
/// values looks like.
#[must_use]
pub fn profile_with_header_values() -> Profile {
    Profile {
        http: HttpHalf {
            variants: vec![HeaderSet::record(
                Variant::Navigate,
                raw_headers(),
                ValuePolicy::WithValues,
            )],
            multipart_boundary: None,
        },
        ..profile()
    }
}

/// The TLS half, including a codepoint the parser has no name for.
#[must_use]
pub fn tls() -> TlsHalf {
    TlsHalf {
        record_version: 0x0301,
        legacy_version: 0x0303,
        session_id_len: 32,
        session_id_hex: "00".repeat(32),
        cipher_suites: vec![0x0a0a, 0x1301, 0x1302, 0x1303],
        compression_methods: vec![0],
        extensions: vec![
            Extension {
                codepoint: 0x0a0a,
                length: 0,
                body_hex: String::new(),
            },
            // server_name: list length 0x000b, host_name type 0x00, name length
            // 0x0008, then "asdf.com". 13 bytes, and the declared length says
            // 13.
            Extension {
                codepoint: 0x0000,
                length: 13,
                body_hex: "000b000008617364662e636f6d".to_owned(),
            },
            // ⭐ The whole reason for the ordered-list model: a codepoint the
            // parser has no name for, kept with its bytes.
            Extension {
                codepoint: 0xca34,
                length: 4,
                body_hex: "deadbeef".to_owned(),
            },
            // ⚠ The trailing GREASE, carrying one zero byte. A model that
            // assumed an empty body could not record this.
            Extension {
                codepoint: 0x5a5a,
                length: 1,
                body_hex: "00".to_owned(),
            },
        ],
        key_exchange_groups: vec![0x0a0a, 0x11ec, 0x001d],
        key_shares: vec![
            KeyShare {
                group: 0x0a0a,
                entry_len: 1,
            },
            KeyShare {
                group: 0x001d,
                entry_len: 32,
            },
        ],
        signature_algorithms: vec![0x0403, 0x0804, 0x0401],
        signature_algorithms_cert: None,
        alpn: vec!["h2".to_owned(), "http/1.1".to_owned()],
        ech: Some(Ech {
            mode: 0,
            kem_id: 0x0020,
        }),
        padding_len: Some(147),
        shuffled: Shuffle::Observed {
            draws: 8,
            distinct_orders: 7,
        },
        grease: Grease {
            extension_positions: vec![0, 3],
            values: vec![0x0a0a, 0x5a5a],
            distinct: true,
            bodies_hex: vec![String::new(), "00".to_owned()],
        },
    }
}

/// The HTTP/2 half.
#[must_use]
pub fn http2() -> Http2Half {
    Http2Half {
        frames: vec![
            Frame::Settings {
                entries: vec![
                    SettingEntry {
                        id: 1,
                        value: 65_536,
                    },
                    SettingEntry { id: 2, value: 0 },
                    SettingEntry {
                        id: 4,
                        value: 6_291_456,
                    },
                    SettingEntry {
                        id: 6,
                        value: 262_144,
                    },
                ],
            },
            Frame::WindowUpdate {
                window_size_increment: 15_663_105,
            },
            Frame::Headers {
                stream_id: 1,
                has_priority_block: true,
            },
        ],
        stream_priority: Some(StreamPriority {
            exclusive: true,
            stream_dependency: 0,
            weight_wire: 255,
        }),
        priority_frames: Vec::new(),
        connection_window: Some(15_728_640),
        pseudo_header_order: vec![
            ":method".to_owned(),
            ":authority".to_owned(),
            ":scheme".to_owned(),
            ":path".to_owned(),
        ],
    }
}

/// The HTTP half, recorded under the default policy.
#[must_use]
pub fn http() -> HttpHalf {
    HttpHalf {
        variants: vec![HeaderSet::record(
            Variant::Navigate,
            raw_headers(),
            ValuePolicy::NamesOnly,
        )],
        multipart_boundary: None,
    }
}

/// Headers as a wire read would hand them over, values included.
///
/// ⭐ This is what makes the privacy test able to fail: the input carries values
/// and a credential, so a recorder that forgot to drop either would be caught.
#[must_use]
pub fn raw_headers() -> Vec<(String, String)> {
    [
        (
            "sec-ch-ua",
            "\"Chromium\";v=\"152\", \"Not?A_Brand\";v=\"24\"",
        ),
        ("sec-ch-ua-mobile", "?0"),
        ("sec-ch-ua-platform", "\"Linux\""),
        ("upgrade-insecure-requests", "1"),
        (
            "user-agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/152.0.0.0 Safari/537.36",
        ),
        ("accept", "text/html"),
        ("cookie", "session=not-a-real-value"),
        ("authorization", "Bearer not-a-real-token"),
        ("accept-encoding", "gzip, deflate, br, zstd"),
        ("accept-language", "en-US,en;q=0.9"),
    ]
    .into_iter()
    .map(|(n, v)| (n.to_owned(), v.to_owned()))
    .collect()
}
