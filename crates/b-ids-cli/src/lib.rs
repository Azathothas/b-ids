//! The smallest client that proves a profile is usable.
//!
//! ⛔ **This is a proof, not a product, and the constraint is the point.** It
//! selects a profile, puts that profile's `ClientHello` on a socket, and
//! stops. There is no cookie jar, no redirect policy, no retry logic, no proxy
//! support and no output formatting, and there must never be: every one of
//! those is a reason to add a flag, and a client with forty flags is a second
//! product this project has not agreed to maintain.
//!
//! ⭐ **It writes the hello itself rather than asking a TLS library to.** That
//! is not a shortcut, it is the finding `EMIT-02` produced: the ordered list of
//! codepoint-and-body pairs is what an emitter needs, this project's model
//! holds one, and no Rust TLS library examined will take it. A client built on
//! one of those stacks could not send this hello at all, which is the hole
//! `EMIT-01`'s matrix records.
//!
//! ⚠ **It completes no handshake, and that is the honest boundary.** Answering
//! a `ServerHello` needs a TLS state machine, and the only one in this tree is
//! the vendored terminator on the SERVER side. What this proves is that the
//! bytes a profile describes can be put on a wire and read back as the same
//! profile; what it does not prove is that a request can be completed with
//! them.
//!
//! `docs/history/todo/library.md`, `LIB-02`.

use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use b_ids_schema::tls::TlsHalf;

/// What one run put on the wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sent {
    /// The profile it claimed.
    pub profile: String,
    /// How many bytes the hello came to.
    pub bytes: usize,
    /// Where it was sent.
    pub peer: String,
}

/// Why a run could not send anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotSent {
    /// No profile in the embedded corpus has that identifier.
    ///
    /// ⛔ **Never a substitute.** A profile this project has not captured does
    /// not exist, and the same rule holds here as in the published routes.
    NoSuchProfile {
        /// What was asked for.
        wanted: String,
    },
    /// The profile cannot be put on a wire byte for byte.
    Unreproducible {
        /// Every reason, not the first.
        why: Vec<String>,
    },
    /// The socket said so.
    Io {
        /// What it said.
        why: String,
    },
}

impl core::fmt::Display for NotSent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchProfile { wanted } => write!(
                f,
                "no profile in the embedded corpus is {wanted}, and this client never substitutes \
                 a neighbouring one"
            ),
            Self::Unreproducible { why } => write!(f, "{}", why.join("; ")),
            Self::Io { why } => write!(f, "{why}"),
        }
    }
}

/// Thirty-two bytes for the hello's random.
///
/// ⚠ **Not cryptographic, and it does not need to be.** The random is the one
/// part of a `ClientHello` that carries no fingerprint, which is why the model
/// does not record it and why this client completes no handshake that would
/// depend on it. ⛔ It is not a constant either: a fixed random would make two
/// runs of this client indistinguishable in a field a real client varies.
#[must_use]
pub fn random() -> [u8; 32] {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut out = [0_u8; 32];
    let mut state = nanos as u64 ^ 0x9e37_79b9_7f4a_7c15;
    for chunk in out.chunks_mut(8) {
        // A trivial mixer, stated as trivial. See the note above.
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let bytes = state.rotate_left(31).to_be_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);
    }
    out
}

/// Put one profile's `ClientHello` on a socket.
///
/// # Errors
///
/// [`NotSent`], naming the profile, the reasons it cannot be emitted, or what
/// the socket said.
pub fn send(id: &str, peer: SocketAddr) -> Result<Sent, NotSent> {
    let profile = b_ids::profiles()
        .iter()
        .find(|p| p.id.to_string() == id)
        .ok_or_else(|| NotSent::NoSuchProfile {
            wanted: id.to_owned(),
        })?;
    let hello = b_ids_emit::client_hello(&profile.tls, &random()).map_err(|why| {
        NotSent::Unreproducible {
            why: why.iter().map(ToString::to_string).collect(),
        }
    })?;
    // ⚠ A timeout on both halves. A client that blocks forever on a listener
    // that went away is a client nobody can put in a test.
    let mut stream = TcpStream::connect_timeout(&peer, Duration::from_secs(10))
        .map_err(|e| NotSent::Io { why: e.to_string() })?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| NotSent::Io { why: e.to_string() })?;
    stream
        .write_all(&hello)
        .map_err(|e| NotSent::Io { why: e.to_string() })?;
    stream
        .flush()
        .map_err(|e| NotSent::Io { why: e.to_string() })?;
    Ok(Sent {
        profile: id.to_owned(),
        bytes: hello.len(),
        peer: peer.to_string(),
    })
}

/// Every field of the TLS half in which what was sent differs from what was
/// claimed.
///
/// ⛔ **Field by field, never a digest comparison.** Two profiles can share a
/// digest and differ in a field the digest sorts away, so a pass reported on a
/// digest alone is a pass reported on less than it claims.
///
/// ⚠ **The random is not compared and cannot be**, because the model does not
/// record it. Every other byte of the hello is.
#[must_use]
pub fn differences(claimed: &TlsHalf, sent: &TlsHalf) -> Vec<String> {
    let mut out = Vec::new();
    let mut note = |field: &str, left: String, right: String| {
        if left != right {
            out.push(format!("{field}: claimed {left}, sent {right}"));
        }
    };
    note(
        "tls.record_version",
        format!("{:#06x}", claimed.record_version),
        format!("{:#06x}", sent.record_version),
    );
    note(
        "tls.legacy_version",
        format!("{:#06x}", claimed.legacy_version),
        format!("{:#06x}", sent.legacy_version),
    );
    note(
        "tls.session_id_hex",
        claimed.session_id_hex.clone(),
        sent.session_id_hex.clone(),
    );
    note(
        "tls.cipher_suites",
        format!("{:?}", claimed.cipher_suites),
        format!("{:?}", sent.cipher_suites),
    );
    note(
        "tls.compression_methods",
        format!("{:?}", claimed.compression_methods),
        format!("{:?}", sent.compression_methods),
    );
    note(
        "tls.extensions.order",
        format!(
            "{:?}",
            claimed
                .extensions
                .iter()
                .map(|e| e.codepoint)
                .collect::<Vec<_>>()
        ),
        format!(
            "{:?}",
            sent.extensions
                .iter()
                .map(|e| e.codepoint)
                .collect::<Vec<_>>()
        ),
    );
    note(
        "tls.extensions.bodies",
        format!(
            "{:?}",
            claimed
                .extensions
                .iter()
                .map(|e| e.body_hex.clone())
                .collect::<Vec<_>>()
        ),
        format!(
            "{:?}",
            sent.extensions
                .iter()
                .map(|e| e.body_hex.clone())
                .collect::<Vec<_>>()
        ),
    );
    note(
        "tls.key_exchange_groups",
        format!("{:?}", claimed.key_exchange_groups),
        format!("{:?}", sent.key_exchange_groups),
    );
    note(
        "tls.signature_algorithms",
        format!("{:?}", claimed.signature_algorithms),
        format!("{:?}", sent.signature_algorithms),
    );
    note(
        "tls.alpn",
        format!("{:?}", claimed.alpn),
        format!("{:?}", sent.alpn),
    );
    out
}
