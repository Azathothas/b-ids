//! Terminating the handshake, and the authority that lets a browser complete it.
//!
//! ⛔ **The raw `ClientHello` is never read back out of the TLS library.** The
//! listener reads the first record off the socket and
//! [`crate::hello::parse_record`] stays the oracle for it; those same bytes are
//! then replayed into the terminator, which continues from there. A hello
//! reported by the implementation that consumed it is a hello filtered through
//! somebody else's parser, and the bytes are this project's whole contribution.
//!
//! ⚠ **What is negotiated here is a property of THIS SERVER, not of the
//! browser.** The version, the cipher suite and the selected protocol are
//! recorded on the capture as conditions of the measurement rather than as
//! findings about the subject. [`crate::modes`] is what compares this surface
//! against the raw one, and on 2026-09-01 it found no TLS field that both
//! surfaces can see and that they disagree on.
//!
//! ⛔ **Nothing here tells a client to skip verification.** The authority is
//! minted so a client can complete a VERIFIED handshake, which is the whole
//! reason `--ca-out` exists: a browser launched with certificate errors ignored
//! is a browser in a different configuration from the one being measured.
//!
//! `docs/history/todo/harness.md`, `HARNESS-13`.

use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::{ServerConfig, ServerConnection};

use b_ids_schema::Resumption;

/// The protocols offered over ALPN, in the order the server prefers them.
///
/// ⚠ **Both, and `h2` first.** Offering only `h2` would refuse a browser that
/// does not want it, and the refusal would be recorded as a failed capture
/// rather than as what the browser chose. The choice itself is a fingerprint
/// signal and it is recorded.
const ALPN: [&[u8]; 2] = [b"h2", b"http/1.1"];

/// A certificate authority minted for one run, and the leaf under it.
///
/// ⛔ **Per run, never reused.** Reusing one would put a long-lived private key
/// in the tree, and `docs/security/secrets.md` refuses that whatever it is for.
#[derive(Debug)]
pub struct Authority {
    /// The authority certificate, PEM encoded. This is what `--ca-out` writes
    /// and what a client is told to trust.
    pub ca_pem: String,
    /// SHA-256 of the authority's subject public key info.
    ///
    /// ⚠ Computed at mint time, because the key pair is here and reading it
    /// back out of the certificate would mean parsing X.509 to recover what
    /// was just put into it.
    spki_sha256: Vec<u8>,
    chain: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
}

/// What one terminated connection negotiated, and what it carried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terminated {
    /// The protocol the peer selected over ALPN, where it selected one.
    pub alpn: Option<String>,
    /// The protocol version that was negotiated.
    pub version: Option<String>,
    /// The cipher suite that was negotiated.
    pub cipher_suite: Option<String>,
    /// The plaintext the peer sent once the handshake completed.
    pub plaintext: Vec<u8>,
}

/// Mint an authority and a leaf naming `bind`.
///
/// ⚠ **The leaf names a literal address, not a hostname**, which is why
/// [`crate::listener::parse_bind`] refuses a hostname and says so in its own
/// message. The two halves of that rule live in different files and agree.
///
/// # Errors
///
/// A string naming which step of the minting failed.
pub fn mint(bind: IpAddr) -> Result<Authority, String> {
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new())
        .map_err(|e| format!("authority parameters: {e}"))?;
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::CrlSign,
        rcgen::KeyUsagePurpose::DigitalSignature,
    ];
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "b-ids capture authority");
    let ca_key = rcgen::KeyPair::generate().map_err(|e| format!("authority key: {e}"))?;
    let ca_cert = ca_params
        .self_signed(&ca_key)
        .map_err(|e| format!("authority certificate: {e}"))?;

    let mut leaf_params = rcgen::CertificateParams::new(vec![bind.to_string()])
        .map_err(|e| format!("leaf parameters for {bind}: {e}"))?;
    leaf_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, bind.to_string());
    leaf_params.use_authority_key_identifier_extension = true;
    let leaf_key = rcgen::KeyPair::generate().map_err(|e| format!("leaf key: {e}"))?;
    let issuer = rcgen::Issuer::from_params(&ca_params, &ca_key);
    let leaf_cert = leaf_params
        .signed_by(&leaf_key, &issuer)
        .map_err(|e| format!("leaf certificate: {e}"))?;

    Ok(Authority {
        ca_pem: ca_cert.pem(),
        spki_sha256: crate::bytes::sha256(&rcgen::PublicKeyData::subject_public_key_info(&ca_key)),
        chain: vec![leaf_cert.der().clone(), ca_cert.der().clone()],
        key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
    })
}

impl Authority {
    /// The base64 SHA-256 of this authority's subject public key info.
    ///
    /// ⭐ **This is what lets a client trust exactly this run without touching
    /// a trust store.** Installing a root certificate changes a machine's
    /// security configuration; a pin names one key, for one launch.
    ///
    /// ⚠ It is a condition of any capture taken through it, and it is not the
    /// same as a trusted root. ⛔ **That difference is still unmeasured**:
    /// `HARNESS-10` measured the capture SURFACE, which changes nothing, and
    /// answering this one needs a root installed into a machine's trust store.
    /// `DRIVER-04` is where the platform detail lives.
    #[must_use]
    pub fn spki_pin(&self) -> String {
        crate::bytes::base64(self.spki_sha256.as_ref())
    }

    /// Build the server configuration this authority serves.
    ///
    /// ⭐ **`resumption` is a condition of every capture taken through this
    /// configuration**, not a tuning knob. ⚠ Measured on hosted runners
    /// 2026-09-02: with tickets offered, Chrome on `ubuntu-latest` abandoned the
    /// only connections that were not resumed, so the navigation produced no
    /// cold handshake and nothing could be published from it.
    /// `docs/history/todo/corpus.md`, `CORPUS-02`.
    ///
    /// ⛔ **Refusing resumption removes the resumed connections from the
    /// sample; it does not change what a cold hello looks like.** A subject with
    /// no ticket for an origin sends the hello a fresh client sends.
    ///
    /// # Errors
    ///
    /// A string naming what rustls refused.
    pub fn server_config(&self, resumption: Resumption) -> Result<Arc<ServerConfig>, String> {
        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(self.chain.clone(), self.key.clone_key())
            .map_err(|e| format!("server configuration: {e}"))?;
        config.alpn_protocols = ALPN.iter().map(|p| p.to_vec()).collect();
        if resumption == Resumption::Refused {
            // ⛔ BOTH halves, because a subject that cannot resume under one
            // protocol version can under the other. TLS 1.3 resumes from a
            // ticket the server sends after the handshake; TLS 1.2 resumes from
            // a session the server stored. Setting one and not the other is a
            // switch that works on the version nobody measured.
            config.send_tls13_tickets = 0;
            config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
        }
        Ok(Arc::new(config))
    }
}

/// A socket whose first bytes have already been read and recorded.
///
/// ⭐ **This is the seam the whole entry turns on.** The listener reads the
/// first TLS record itself, so the terminator has to be handed those bytes
/// back before it reads any more. Reads come from the buffer until it is
/// empty and from the socket afterwards; writes always go to the socket.
struct Replayed<'a> {
    buffered: std::io::Cursor<Vec<u8>>,
    socket: &'a TcpStream,
}

impl Read for Replayed<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let taken = self.buffered.read(buf)?;
        if taken > 0 {
            return Ok(taken);
        }
        let mut socket = self.socket;
        socket.read(buf)
    }
}

impl Write for Replayed<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut socket = self.socket;
        socket.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut socket = self.socket;
        socket.flush()
    }
}

/// Complete the handshake and read what the peer sends over it.
///
/// `already_read` is the bytes the listener took off the socket and recorded,
/// which are replayed into the terminator rather than re-read.
///
/// # Errors
///
/// A string naming what the handshake or the read refused.
pub fn terminate(
    stream: &TcpStream,
    already_read: &[u8],
    config: &Arc<ServerConfig>,
) -> Result<Terminated, String> {
    let mut connection =
        ServerConnection::new(Arc::clone(config)).map_err(|e| format!("server connection: {e}"))?;
    let mut replayed = Replayed {
        buffered: std::io::Cursor::new(already_read.to_vec()),
        socket: stream,
    };

    connection
        .complete_io(&mut replayed)
        .map_err(|e| format!("handshake did not complete: {e}"))?;

    let alpn = connection
        .alpn_protocol()
        .map(|p| String::from_utf8_lossy(p).into_owned());
    let version = connection.protocol_version().map(|v| format!("{v:?}"));
    let cipher_suite = connection
        .negotiated_cipher_suite()
        .map(|s| format!("{:?}", s.suite()));

    // ⛔ The same completion rule the cleartext surface uses, over the decrypted
    // stream. One read path, chosen by the bytes rather than by the flag that
    // opened the connection.
    let mut tls = rustls::Stream::new(&mut connection, &mut replayed);
    let plaintext =
        crate::listener::read_first_message(&mut tls, crate::listener::Protocol::Cleartext)
            .map_err(|e| format!("read after the handshake: {e}"))?;

    Ok(Terminated {
        alpn,
        version,
        cipher_suite,
        plaintext,
    })
}
