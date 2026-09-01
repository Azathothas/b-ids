//! The oracle: a listener a browser is pointed at.
//!
//! ⭐ **A server, not a probe inside a client.** Asking a client what it sent
//! returns what it intended, which is a different thing and is the commonest
//! way a whole set of numbers turns out to describe nothing.
//!
//! ⛔ **It does not complete a TLS handshake.** Completing one can change what a
//! client offers, so a digest read through a terminated handshake is not the
//! digest that client ships. Terminating one is what `--ca-out` needs and it is
//! not here.
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

use std::io::{ErrorKind, Read as _};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

use b_ids_schema::http::ValuePolicy;
use serde::{Deserialize, Serialize};

use crate::bytes::hex;
use crate::h2::{self, Http2Capture};
use crate::hello::{HelloCapture, parse_record};
use crate::note::Note;

/// Which capture surface a listener presents.
///
/// ⚠ Two variants today and the seam is here for more. A third is added as a
/// variant and a match arm, not as a second listener.
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
}

impl Protocol {
    /// The scheme a caller should point a browser at.
    #[must_use]
    pub fn scheme(self) -> &'static str {
        match self {
            Self::TlsRaw => "https",
            Self::Cleartext => "http",
        }
    }
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
    pub raw_hex: String,
    /// The TLS half, where a `ClientHello` was read.
    pub tls: Option<b_ids_schema::tls::TlsHalf>,
    /// The HTTP/2 half, where a connection preface and its frames were read.
    pub http2: Option<Http2Capture>,
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
/// ⚠ Version 2 adds the HTTP/2 half and renames the cleartext surface, which
/// used to name HTTP/1.1 alone. A version is part of the data rather than
/// implied by the reader.
pub const CAPTURE_SCHEMA: &str = "harness-capture/2";

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
    /// How long to wait for bytes on an accepted connection.
    pub read_timeout: Duration,
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
            read_timeout: Duration::from_secs(10),
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
            peer: peer.to_string(),
            protocol: self.config.protocol,
            bytes_read: 0,
            raw_hex: String::new(),
            tls: None,
            http2: None,
            request_line: None,
            header_names: Vec::new(),
            header_values: Vec::new(),
            notes: Vec::new(),
        };

        let _ = stream.set_read_timeout(Some(self.config.read_timeout));
        let bytes = match read_first_message(stream, self.config.protocol) {
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
            Protocol::TlsRaw => match parse_record(&bytes) {
                Ok(HelloCapture { tls, notes, .. }) => {
                    capture.tls = Some(tls);
                    capture.notes = notes;
                }
                Err(why) => capture.notes.push(Note::new("tls", why)),
            },
            Protocol::Cleartext => self.read_cleartext(&bytes, &mut capture),
        }
        capture
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
            if b_ids_schema::http::is_never_recorded(&name) {
                continue;
            }
            if self.config.header_values {
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
fn read_first_message(mut stream: &TcpStream, protocol: Protocol) -> std::io::Result<Vec<u8>> {
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
        Protocol::TlsRaw => {
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
