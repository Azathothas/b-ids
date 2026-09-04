//! The oracle: a listener a browser is pointed at.
//!
//! ⭐ **A server, not a probe inside a client.** Asking a client what it sent
//! returns what it intended, which is a different thing and is the commonest
//! way a whole set of numbers turns out to describe nothing.
//!
//! ⛔ **It does not complete a TLS handshake by default**, and the default is the
//! answer to the narrower question: completing one can change what a client
//! offers, so a digest read through a terminated handshake is not the digest
//! that client ships. `--ca-out` opts into termination, which is the only way to
//! reach a browser's HTTP/2, and [`crate::tls`] is where it happens.
//!
//! ⭐ **Multi-protocol from the first commit, even with one protocol
//! implemented.** [`Protocol`] is the seam: a TLS listener, a cleartext
//! listener and later a QUIC listener are capture surfaces, and retrofitting
//! the third into a TLS-shaped harness is a rewrite rather than an addition.
//!
//! ⚠ **The surface is not the protocol the peer speaks.** A cleartext surface
//! is offered and the peer decides: a browser sends an HTTP/1.1 request and a
//! client with prior knowledge sends the HTTP/2 connection preface. The reader
//! is chosen by the bytes that arrived, never by a flag the operator passed.

use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::ServerConfig;

use b_ids_schema::http::ValuePolicy;
use serde::{Deserialize, Serialize};

use crate::bytes::hex;
use crate::h2::{self, Http2Capture};
use crate::hello::{HelloCapture, parse_record};
use crate::note::Note;

/// Which capture surface a listener presents.
///
/// ⚠ Three variants today and the seam is here for more. A fourth, QUIC, is
/// added as a variant and a match arm, not as a second listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    /// Read the `ClientHello` and close, never completing the handshake.
    TlsRaw,
    /// Cleartext: an HTTP/1.1 request, or an HTTP/2 connection preface and the
    /// frames behind it.
    ///
    /// ⭐ The capture that works when a client cannot be told to trust
    /// anything.
    Cleartext,
    /// Read the `ClientHello`, then complete the handshake and read whatever
    /// the peer sends over it.
    ///
    /// ⭐ The only surface that reaches a browser's HTTP/2, because no browser
    /// speaks cleartext HTTP/2. ⚠ It is opt-in for the reason the module
    /// comment gives, and what it negotiated is recorded as a condition of the
    /// capture rather than as a finding about the subject.
    TlsTerminated,
}

impl Protocol {
    /// The scheme a caller should point a browser at.
    #[must_use]
    pub fn scheme(self) -> &'static str {
        match self {
            Self::TlsRaw | Self::TlsTerminated => "https",
            Self::Cleartext => "http",
        }
    }
}

/// What one terminated handshake negotiated, and what came over it.
///
/// ⚠ **Every field here is a property of THIS SERVER**, not of the browser. A
/// capture that reported the negotiated suite as a fact about the subject would
/// be reporting the harness.
///
/// ⭐ **Measured on 2026-09-01, and the surface changed nothing it could be
/// compared on.** `HARNESS-10` drove Chrome `151.0.7922.76` at both surfaces
/// over three rounds: seventeen of nineteen TLS fields agree exactly, none
/// differ, and the two that cannot be compared carry a per-connection draw.
/// ⚠ One browser, one build, one host, one day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Termination {
    /// The protocol the peer selected over ALPN, where it selected one.
    ///
    /// ⭐ This one IS a choice the peer made, and it is a fingerprint signal.
    pub alpn: Option<String>,
    /// The protocol version that was negotiated.
    pub version: Option<String>,
    /// The cipher suite that was negotiated.
    pub cipher_suite: Option<String>,
    /// How many plaintext bytes arrived before the read ended.
    pub plaintext_bytes: usize,
    /// The plaintext, hex-encoded.
    ///
    /// ⛔ Kept for the same reason `Capture::raw_hex` is: it is what the parsed
    /// halves below were read from, and a capture is a moment that cannot be
    /// retaken. ⚠ The wire carried ciphertext, so this is not the wire: it is
    /// what the peer sent, recovered by holding the key.
    pub plaintext_hex: String,
}

/// What one accepted connection produced.
///
/// ⛔ Every connection produces one of these even when nothing parsed, because
/// a browser opens sockets it abandons and a run that silently dropped them
/// would under-report what a navigation does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capture {
    /// The schema this record is written against.
    pub schema: String,
    /// Which connection of the run this was, from 1.
    pub connection: u32,
    /// When the connection was accepted, ISO 8601 UTC.
    ///
    /// ⛔ **Recorded by the thing that took the capture.** A profile's
    /// `captured.at` is never optional, so something has to produce it, and a
    /// reader that stamped one later would be recording when it read the file
    /// rather than when the bytes arrived.
    ///
    /// ⚠ It changes on every run, so the golden comparison drops it. See
    /// `normalise` in this crate's command.
    #[serde(default)]
    pub at: String,
    /// The peer address.
    pub peer: String,
    /// Which surface accepted it.
    pub protocol: Protocol,
    /// How many bytes arrived before the read ended.
    pub bytes_read: usize,
    /// The bytes, hex-encoded.
    ///
    /// ⛔ Kept whatever else happened. It is the one artefact that survives
    /// every hashing scheme and every parser defect, and a capture is a moment
    /// that cannot be retaken.
    ///
    /// ⚠ **On a terminated surface this is the first record and nothing more.**
    /// The rest of the handshake and every application record after it are
    /// ciphertext, and what the peer sent inside them is in
    /// [`Termination::plaintext_hex`] instead.
    pub raw_hex: String,
    /// The TLS half, where a `ClientHello` was read.
    pub tls: Option<b_ids_schema::tls::TlsHalf>,
    /// The HTTP/2 half, where a connection preface and its frames were read.
    pub http2: Option<Http2Capture>,
    /// What the handshake negotiated, where one was completed.
    ///
    /// ⛔ `None` on every surface that does not terminate, rather than a set of
    /// empty strings. A field that cannot tell "not attempted" from "attempted
    /// and negotiated nothing" is a field a reader has to guess at.
    pub termination: Option<Termination>,
    /// The request line, where a cleartext HTTP/1.1 request was read.
    pub request_line: Option<String>,
    /// The header names in wire order, where a cleartext HTTP/1.1 request was
    /// read.
    pub header_names: Vec<String>,
    /// The header values, only where the caller asked for them.
    ///
    /// ⛔ Empty by default. `SCHEMA-04` makes the names-only shape the default
    /// because a switch that has to be turned off for safety is a switch that
    /// ships on.
    pub header_values: Vec<String>,
    /// What could not be read.
    pub notes: Vec<Note>,
}

impl Capture {
    /// Whether this connection reached HTTP/2 and opened a stream.
    ///
    /// ⭐ Derived rather than stored, so a capture cannot say one thing in a
    /// flag and another in its frames.
    #[must_use]
    pub fn reached_h2(&self) -> bool {
        self.http2
            .as_ref()
            .is_some_and(Http2Capture::opened_a_stream)
    }
}

/// How often a deadlined accept looks again.
///
/// ⚠ Short enough that a run ends promptly and long enough that waiting costs
/// no measurable processor time.
const ACCEPT_POLL: Duration = Duration::from_millis(10);

/// The schema identifier a capture carries.
///
/// ⚠ Version 4 adds the instant the connection was accepted, which is what
/// `CORPUS-01` needs to fill a profile's `captured.at`. Version 3 added what a
/// terminated handshake negotiated. Version 2 added the HTTP/2 half and renamed
/// the cleartext surface, which used to name HTTP/1.1 alone. A version is part
/// of the data rather than implied by the reader.
pub const CAPTURE_SCHEMA: &str = "harness-capture/4";

/// How a run is configured.
#[derive(Debug, Clone)]
pub struct Config {
    /// Which surface to present.
    pub protocol: Protocol,
    /// The address to bind.
    pub bind: IpAddr,
    /// The port, or 0 to let the operating system choose.
    pub port: u16,
    /// How many connections to accept before returning.
    ///
    /// ⚠ One handshake is not a sample, and one connection is not a
    /// navigation: driving a browser at a probe has produced thirteen.
    pub handshakes: u32,
    /// How long the whole run may spend waiting for connections.
    ///
    /// ⛔ `None` blocks until every requested connection arrives, which is what
    /// a run driving a browser wants. ⚠ A run whose subject never connects then
    /// never returns, and a hang has no message and no exit code: in continuous
    /// integration it consumes the job's whole timeout and reports nothing.
    /// `HARNESS-08` is why a deadline is available at all.
    pub run_timeout: Option<Duration>,
    /// Stop at the first connection that reached HTTP/2.
    ///
    /// ⚠ A browser opens sockets it abandons, and the first connection of a
    /// navigation has been measured carrying no HTTP/2 at all. This is how a
    /// run keeps the cold handshake rather than the first socket.
    pub until_h2: bool,
    /// Whether to record header values.
    pub header_values: bool,
    /// Whether to hand each caller its own capture back.
    ///
    /// ⭐ **`HARNESS-12`, the oracle mode.** Every hosted fingerprint service
    /// returns a subset and no raw hello, so somebody who wants to know what
    /// their own browser sends has to trust a page. This returns the full
    /// model, raw bytes included, to the caller that produced it.
    ///
    /// ⛔ **The mode is BUILT and it is not HOSTED.** The entry's own decision
    /// says so: a hosted endpoint receives traffic from people, which is the
    /// one thing this project's scope boundary says it does not do, and that
    /// question needs an answer written down and a person's approval before
    /// anything is stood up. `TODO/harness.md`, `HARNESS-12`.
    ///
    /// ⚠ **It answers over CLEARTEXT HTTP/1.1 and nothing else**, and the note
    /// on every other connection says why rather than leaving a caller waiting.
    /// Writing an HTTP/2 response needs an HPACK ENCODER, and this crate has a
    /// decoder: the encoder in this tree is the vendored `h2`, which
    /// `b-ids-emit` owns, and reaching it from here would invert the dependency
    /// the workspace has.
    pub serve: bool,
    /// How long to wait for bytes on an accepted connection.
    pub read_timeout: Duration,
    /// The server configuration a terminated surface serves.
    ///
    /// ⛔ Required by [`Protocol::TlsTerminated`] and refused by every other
    /// surface, both at [`Oracle::bind`]. A protocol that says it terminates
    /// beside a `None` here would be a mode that silently did nothing.
    pub terminator: Option<Arc<ServerConfig>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            protocol: Protocol::TlsRaw,
            bind: IpAddr::from([127, 0, 0, 1]),
            port: 0,
            // ⛔ EIGHT, and never one. Anything drawn per connection means a
            // single handshake tests a single draw, and a defect that fires on
            // three values in sixteen reaches a one-handshake check four times
            // in five and passes. `--once` is how a caller asks for one.
            handshakes: crate::sampling::DEFAULT_HANDSHAKES,
            run_timeout: None,
            until_h2: false,
            header_values: false,
            // ⛔ OFF BY DEFAULT. A harness that answered every caller by default
            // would be an oracle nobody chose to run. HARNESS-12.
            serve: false,
            read_timeout: Duration::from_secs(10),
            terminator: None,
        }
    }
}

/// Why a bind address was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindRefused {
    /// The address as the caller wrote it.
    pub given: String,
    /// Why it is refused.
    pub why: String,
}

impl core::fmt::Display for BindRefused {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "--bind {}: {}", self.given, self.why)
    }
}

/// Read a bind address, refusing a hostname and the unspecified address.
///
/// ⛔ **Both refusals are by name rather than by accident.** A hostname cannot
/// go in a leaf certificate the way a literal can, and the unspecified address
/// accepts the whole local network, which a fixture that records headers should
/// not do silently.
///
/// # Errors
///
/// [`BindRefused`] naming which of the two rules was broken.
pub fn parse_bind(given: &str) -> Result<IpAddr, BindRefused> {
    let Ok(addr) = given.parse::<IpAddr>() else {
        return Err(BindRefused {
            given: given.to_owned(),
            why: "is a hostname, and a leaf certificate needs a literal address".to_owned(),
        });
    };
    if addr.is_unspecified() {
        return Err(BindRefused {
            given: given.to_owned(),
            why: "is the unspecified address, which accepts the local network as well. \
                  Name the interface"
                .to_owned(),
        });
    }
    Ok(addr)
}

/// A bound listener, before any connection is accepted.
///
/// ⭐ The base URL is available as soon as this exists, which is what lets a
/// caller print it and point a browser at it before blocking on the accept.
#[derive(Debug)]
pub struct Oracle {
    listener: TcpListener,
    config: Config,
}

impl Oracle {
    /// Bind, without accepting anything yet.
    ///
    /// # Errors
    ///
    /// Whatever the operating system said about the bind.
    pub fn bind(config: Config) -> std::io::Result<Self> {
        // ⛔ Refused before the bind, so the failure names the configuration
        // rather than arriving as a connection that terminated nothing.
        if config.protocol == Protocol::TlsTerminated && config.terminator.is_none() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "the terminated surface needs a server configuration, and none was given",
            ));
        }
        let listener = TcpListener::bind(SocketAddr::new(config.bind, config.port))?;
        Ok(Self { listener, config })
    }

    /// The address actually bound, which is what resolves port 0.
    ///
    /// # Errors
    ///
    /// Whatever the operating system said.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// The base URL to point a browser at.
    ///
    /// # Errors
    ///
    /// Whatever the operating system said about the bound address.
    pub fn base_url(&self) -> std::io::Result<String> {
        let addr = self.local_addr()?;
        Ok(format!(
            "{}://{}:{}/",
            self.config.protocol.scheme(),
            addr.ip(),
            addr.port()
        ))
    }

    /// Accept connections and read each one, up to the configured count.
    ///
    /// ⚠ A connection that produces nothing still produces a [`Capture`], with
    /// its note. A browser opens sockets it abandons, and a run that dropped
    /// them would under-report what a navigation does.
    ///
    /// # Errors
    ///
    /// Whatever the operating system said about the accept.
    pub fn run(&self) -> std::io::Result<Vec<Capture>> {
        let deadline = self.config.run_timeout.map(|d| Instant::now() + d);
        // ⚠ Non-blocking ONLY where there is a deadline. A blocking accept is
        // what a run driving a browser wants, and polling for one would burn a
        // core to no purpose.
        self.listener.set_nonblocking(deadline.is_some())?;

        let mut out = Vec::new();
        for index in 1..=self.config.handshakes {
            let Some((stream, peer)) = self.accept_within(deadline)? else {
                break;
            };
            let capture = self.read_connection(&stream, peer, index);
            // ⛔ Read before the move, and read from the capture rather than
            // from a flag set beside it.
            let reached_h2 = capture.reached_h2();
            out.push(capture);
            if self.config.until_h2 && reached_h2 {
                break;
            }
        }
        Ok(out)
    }

    /// Accept one connection, giving up at `deadline`.
    ///
    /// ⛔ **A stream accepted from a non-blocking listener inherits that mode
    /// on some platforms**, and a non-blocking read returns `WouldBlock`
    /// immediately, which the reader would record as a peer that sent nothing.
    /// It is put back to blocking here rather than at the read, because there
    /// is one accept and several readers.
    fn accept_within(
        &self,
        deadline: Option<Instant>,
    ) -> std::io::Result<Option<(TcpStream, SocketAddr)>> {
        loop {
            match self.listener.accept() {
                Ok((stream, peer)) => {
                    if deadline.is_some() {
                        stream.set_nonblocking(false)?;
                    }
                    return Ok(Some((stream, peer)));
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    let Some(deadline) = deadline else {
                        return Err(err);
                    };
                    if Instant::now() >= deadline {
                        return Ok(None);
                    }
                    std::thread::sleep(ACCEPT_POLL);
                }
                Err(err) if err.kind() == ErrorKind::Interrupted => {}
                Err(err) => return Err(err),
            }
        }
    }

    fn read_connection(&self, stream: &TcpStream, peer: SocketAddr, index: u32) -> Capture {
        let mut capture = Capture {
            schema: CAPTURE_SCHEMA.to_owned(),
            connection: index,
            // ⛔ Stamped HERE, by the thing that took it, and never later by
            // whatever reads the capture back. A capture is a moment, and an
            // instant applied by a reader is the reader's clock rather than the
            // capture's.
            at: b_ids_schema::instant::now(),
            peer: peer.to_string(),
            protocol: self.config.protocol,
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

        let _ = stream.set_read_timeout(Some(self.config.read_timeout));
        let mut source = stream;
        let bytes = match read_first_message(&mut source, self.config.protocol) {
            Ok(bytes) => bytes,
            Err(err) => {
                capture
                    .notes
                    .push(Note::new("connection", format!("read failed: {err}")));
                return capture;
            }
        };

        capture.bytes_read = bytes.len();
        capture.raw_hex = hex(&bytes);
        if bytes.is_empty() {
            capture.notes.push(Note::new(
                "connection",
                "the peer opened a socket and sent nothing, then closed it",
            ));
            return capture;
        }

        match self.config.protocol {
            Protocol::TlsRaw => self.read_hello(&bytes, &mut capture),
            Protocol::Cleartext => {
                self.read_cleartext(&bytes, &mut capture);
                if self.config.serve {
                    self.answer(stream, &bytes, &mut capture);
                }
            }
            // ⛔ The hello is parsed from the bytes the listener read, BEFORE the
            // terminator sees them, and those same bytes are replayed into it.
            // A hello reported by the library that consumed it is a hello
            // filtered through somebody else's parser.
            Protocol::TlsTerminated => {
                self.read_hello(&bytes, &mut capture);
                self.terminate(stream, &bytes, &mut capture);
                if self.config.serve {
                    capture.notes.push(Note::new(
                        "serve",
                        "nothing was returned to the caller: this surface completes a TLS \
                         handshake and answering over it needs an HPACK encoder, which this \
                         crate does not have. --plain is the surface that answers. \
                         TODO/harness.md, HARNESS-12",
                    ));
                }
            }
        }
        capture
    }

    /// Hand the caller its own capture back.
    ///
    /// ⭐ **`HARNESS-12`. The full model, raw bytes included**, rather than a
    /// hash and a marketing page. ⛔ Nothing is written to disk and nothing is
    /// retained: the capture goes to the socket that produced it and to the
    /// run's own stdout, and the process keeps no copy a later run could read.
    ///
    /// ⚠ **HTTP/1.1 ONLY, and a connection that is not one is TOLD so** rather
    /// than left waiting. An HTTP/2 response needs an HPACK encoder and this
    /// crate has a decoder.
    ///
    /// ⚠ **A write that fails is a note rather than an error.** A caller that
    /// closed the socket after sending its request is ordinary, and a capture
    /// is still a capture.
    fn answer(&self, mut stream: &TcpStream, bytes: &[u8], capture: &mut Capture) {
        if h2::starts_like_preface(bytes) {
            capture.notes.push(Note::new(
                "serve",
                "nothing was returned to the caller: this connection is HTTP/2, and answering \
                 over it needs an HPACK encoder, which this crate does not have. \
                 TODO/harness.md, HARNESS-12",
            ));
            return;
        }
        // ⛔ SERIALISED, never formatted, like every other JSON this tree emits.
        let body = match serde_json::to_string_pretty(capture) {
            Ok(text) => text,
            Err(why) => {
                capture.notes.push(Note::new(
                    "serve",
                    format!("the capture did not serialise: {why}"),
                ));
                return;
            }
        };
        // ⚠ `Connection: close` because the caller gets one answer and the run
        // is counting connections. A kept-alive socket would make one browser
        // look like one connection for as long as it stayed open.
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
             Cache-Control: no-store\r\nConnection: close\r\n\r\n",
            body.len()
        );
        if let Err(why) = stream
            .write_all(head.as_bytes())
            .and_then(|()| stream.write_all(body.as_bytes()))
            .and_then(|()| stream.flush())
        {
            capture.notes.push(Note::new(
                "serve",
                format!("the answer did not send: {why}"),
            ));
            return;
        }
        capture.notes.push(Note::new(
            "serve",
            format!(
                "{} byte(s) of capture returned to the caller, and nothing retained",
                body.len()
            ),
        ));
    }

    fn read_hello(&self, bytes: &[u8], capture: &mut Capture) {
        match parse_record(bytes) {
            Ok(HelloCapture { tls, notes, .. }) => {
                capture.tls = Some(tls);
                capture.notes = notes;
            }
            Err(why) => capture.notes.push(Note::new("tls", why)),
        }
    }

    /// Complete the handshake and read what the peer sent over it.
    ///
    /// ⚠ A connection that never completes still produces a capture, with its
    /// note and its recorded hello. A browser opens sockets it abandons, and
    /// one of them abandoned mid-handshake is data rather than an error.
    fn terminate(&self, stream: &TcpStream, already_read: &[u8], capture: &mut Capture) {
        let Some(config) = self.config.terminator.as_ref() else {
            capture.notes.push(Note::new(
                "tls.terminate",
                "the terminated surface was selected with no server configuration",
            ));
            return;
        };
        let terminated = match crate::tls::terminate(stream, already_read, config) {
            Ok(terminated) => terminated,
            Err(why) => {
                capture.notes.push(Note::new("tls.terminate", why));
                return;
            }
        };
        capture.termination = Some(Termination {
            alpn: terminated.alpn.clone(),
            version: terminated.version.clone(),
            cipher_suite: terminated.cipher_suite.clone(),
            plaintext_bytes: terminated.plaintext.len(),
            plaintext_hex: hex(&terminated.plaintext),
        });
        if terminated.plaintext.is_empty() {
            capture.notes.push(Note::new(
                "tls.terminate",
                "the handshake completed and the peer sent nothing over it",
            ));
            return;
        }
        // ⛔ The SAME reader the cleartext surface uses, chosen by the bytes
        // rather than by the flag that opened the connection. One read path.
        self.read_cleartext(&terminated.plaintext, capture);
    }

    /// Read whichever cleartext protocol the peer actually spoke.
    ///
    /// ⛔ The bytes decide. A run does not get to declare what its peer will
    /// send, and a capture that recorded an HTTP/2 connection as an unparseable
    /// HTTP/1.1 request would be recording the harness rather than the client.
    fn read_cleartext(&self, bytes: &[u8], capture: &mut Capture) {
        if !h2::starts_like_preface(bytes) {
            self.read_http1(bytes, capture);
            return;
        }
        let mut notes = Vec::new();
        // ⛔ The same switch governs both protocols. A capture that recorded
        // values over HTTP/2 while withholding them over HTTP/1.1 would be one
        // rule enforced on one of two paths.
        let policy = if self.config.header_values {
            ValuePolicy::WithValues
        } else {
            ValuePolicy::NamesOnly
        };
        let parsed = h2::parse_connection(bytes, policy, &mut notes);
        capture.notes.append(&mut notes);
        match parsed {
            Ok(http2) => capture.http2 = Some(http2),
            Err(why) => capture.notes.push(Note::new("http2", why)),
        }
    }

    fn read_http1(&self, bytes: &[u8], capture: &mut Capture) {
        let text = String::from_utf8_lossy(bytes);
        let mut lines = text.split("\r\n");
        capture.request_line = lines.next().map(str::to_owned);
        for line in lines {
            if line.is_empty() {
                break;
            }
            let Some((name, value)) = line.split_once(':') else {
                capture.notes.push(Note::new(
                    "http.headers",
                    format!("a header line carries no colon: {line}"),
                ));
                continue;
            };
            let name = name.trim().to_owned();
            // ⛔ The credential filter runs here as well as in the model. One
            // gate per action, and this is a different door into the same one.
            //
            // ⭐ THE NAME IS KEPT AND THE VALUE IS NOT. `SCHEMA-14`: whether a
            // credential was sent, and where in the order, is a fingerprint signal
            // that carries no secret, and a name dropped here left the recorded
            // order closed over a gap nothing marked.
            //
            // ⚠ THE TWO LISTS ARE NOT PARALLEL AND NEVER WERE. `header_values`
            // is empty under the default policy while `header_names` is full, so
            // nothing may index one by the other's position. The published header
            // set is built from `HeaderSet::record`, which pairs a name with its
            // own value and marks a withheld one.
            if self.config.header_values && !b_ids_schema::http::is_never_recorded(&name) {
                capture.header_values.push(value.trim().to_owned());
            }
            capture.header_names.push(name);
        }
    }
}

/// Read until the first message is complete, or the peer stops.
///
/// ⚠ **A TLS record can arrive split across reads**, so the length in its
/// header is what says when to stop rather than the first read returning. A
/// parser fed one read's worth of a two-read hello reports a truncation that
/// never happened.
pub(crate) fn read_first_message<R: Read + ?Sized>(
    stream: &mut R,
    protocol: Protocol,
) -> std::io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                if message_is_complete(&buffer, protocol) {
                    break;
                }
            }
            Err(err)
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut =>
            {
                break;
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => {}
            Err(err) => return Err(err),
        }
    }
    Ok(buffer)
}

fn message_is_complete(buffer: &[u8], protocol: Protocol) -> bool {
    match protocol {
        // ⚠ The TERMINATED surface uses the record rule here, not the cleartext
        // one: this read is the first TLS record and the handshake has not
        // happened yet. The read AFTER it passes Cleartext explicitly.
        Protocol::TlsRaw | Protocol::TlsTerminated => {
            if buffer.len() < 5 {
                return false;
            }
            let declared = usize::from(u16::from(buffer[3]) << 8 | u16::from(buffer[4]));
            buffer.len() >= 5 + declared
        }
        // ⚠ The preface is checked FIRST and the blank-line rule second. The
        // HTTP/2 preface carries a blank line at byte 16, so a reader that
        // asked the HTTP/1.1 question first would stop four bytes into an
        // HTTP/2 connection and record none of its frames.
        Protocol::Cleartext => {
            if h2::starts_like_preface(buffer) {
                h2::first_header_block_complete(buffer)
            } else {
                buffer.windows(4).any(|w| w == b"\r\n\r\n")
            }
        }
    }
}
