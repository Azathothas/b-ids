//! Build the `ClientHello` fixture the listener test feeds over a loopback
//! socket, and print it as one hex line.
//!
//! ```text
//! cargo run -p b-ids-harness --example make-fixture > crates/b-ids-harness/fixtures/client-hello.hex
//! ```
//!
//! ⛔ **THE BYTES ARE CONSTRUCTED, NOT CAPTURED.** They are shaped like a
//! Chromium hello so the parser is exercised over the shapes a real one has:
//! GREASE at both ends, a codepoint with no name, a trailing GREASE carrying
//! one zero byte, and an extension order that is not sorted. ⛔ No value here
//! is a measurement and none of it may enter the corpus. A measured value lives
//! in a profile; a value somebody else measured lives in
//! `docs/inherited-claims.md` with its source.
//!
//! ⭐ **The fixture is generated rather than pasted** so that a reader can
//! re-derive it, and so a change to it is a change to this file rather than to
//! an opaque blob nobody can check.

fn u16be(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

/// A length-prefixed block: run `body`, then write its length in `width` bytes
/// in front of it.
fn framed(out: &mut Vec<u8>, width: usize, body: impl FnOnce(&mut Vec<u8>)) {
    let start = out.len();
    for _ in 0..width {
        out.push(0);
    }
    body(out);
    let len = out.len() - start - width;
    let bytes = (len as u64).to_be_bytes();
    let tail = &bytes[8 - width..];
    out[start..start + width].copy_from_slice(tail);
}

fn extension(out: &mut Vec<u8>, codepoint: u16, body: impl FnOnce(&mut Vec<u8>)) {
    u16be(out, codepoint);
    framed(out, 2, body);
}

fn main() {
    let mut hello = Vec::new();

    // legacy_version, then a 32-byte random. The random is zeroes because it is
    // not part of any fingerprint and a fixture with entropy in it would be a
    // fixture that changes.
    u16be(&mut hello, 0x0303);
    hello.extend_from_slice(&[0_u8; 32]);

    // A 32-byte session id, which is what a browser sends in TLS 1.3.
    hello.push(32);
    hello.extend_from_slice(&[0x11_u8; 32]);

    framed(&mut hello, 2, |out| {
        for suite in [
            0x0a0a_u16, 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8,
            0xc013, 0xc014, 0x009c, 0x009d, 0x002f, 0x0035,
        ] {
            u16be(out, suite);
        }
    });

    // One compression method: null.
    hello.push(1);
    hello.push(0);

    framed(&mut hello, 2, |out| {
        // ⭐ Leading GREASE, empty body.
        extension(out, 0x0a0a, |_| {});
        // server_name: asdf.com
        extension(out, 0x0000, |b| {
            framed(b, 2, |list| {
                list.push(0);
                framed(list, 2, |name| name.extend_from_slice(b"asdf.com"));
            });
        });
        // extended_master_secret, which has no body at all.
        extension(out, 0x0017, |_| {});
        // supported_versions: a ONE-byte list length, unlike most extensions.
        extension(out, 0x002b, |b| {
            framed(b, 1, |list| {
                u16be(list, 0x0a0a);
                u16be(list, 0x0304);
                u16be(list, 0x0303);
            });
        });
        // supported_groups
        extension(out, 0x000a, |b| {
            framed(b, 2, |list| {
                for group in [0x0a0a_u16, 0x11ec, 0x001d, 0x0017, 0x0018] {
                    u16be(list, group);
                }
            });
        });
        // signature_algorithms
        extension(out, 0x000d, |b| {
            framed(b, 2, |list| {
                for alg in [
                    0x0403_u16, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601,
                ] {
                    u16be(list, alg);
                }
            });
        });
        // application_layer_protocol_negotiation
        extension(out, 0x0010, |b| {
            framed(b, 2, |list| {
                for name in ["h2", "http/1.1"] {
                    framed(list, 1, |entry| entry.extend_from_slice(name.as_bytes()));
                }
            });
        });
        // key_share: one GREASE entry of one byte, then x25519 with 32 bytes.
        extension(out, 0x0033, |b| {
            framed(b, 2, |list| {
                u16be(list, 0x0a0a);
                framed(list, 2, |entry| entry.push(0));
                u16be(list, 0x001d);
                framed(list, 2, |entry| entry.extend_from_slice(&[0x22_u8; 32]));
            });
        });
        // ⭐ A codepoint this parser has no name for, kept with its bytes. This
        // is the shape that stopped a version bump in another repository.
        extension(out, 0xca34, |b| {
            b.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef])
        });
        // encrypted_client_hello, outer form: mode then kem id.
        extension(out, 0xfe0d, |b| {
            b.push(0);
            u16be(b, 0x0020);
        });
        // padding
        extension(out, 0x0015, |b| b.extend_from_slice(&[0_u8; 40]));
        // ⚠ Trailing GREASE, carrying one zero byte. A model that assumed an
        // empty body could not record this.
        extension(out, 0x5a5a, |b| b.push(0));
    });

    let mut handshake = vec![0x01_u8];
    framed(&mut handshake, 3, |out| out.extend_from_slice(&hello));

    let mut record = vec![0x16_u8, 0x03, 0x01];
    framed(&mut record, 2, |out| out.extend_from_slice(&handshake));

    println!("{}", b_ids_harness::hex(&record));
}
