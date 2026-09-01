//! The vendored TLS library is COMPILED by this tree, not merely present in it.
//!
//! ⛔ **A vendored tree nothing depends on is a directory, not a dependency.** It
//! would pass every check in `scripts/common/check-vendor.sh`, which reads the
//! manifest against the tree and never asks the compiler anything, and the first
//! session to reach for it would find out whether it builds. So one test names
//! the crate and asks it a question only a linked build can answer.
//!
//! ⚠ **These assertions are about what the SERVER can offer**, which is a
//! condition of a capture rather than a property of a browser. `HARNESS-13`
//! records the negotiated version and suite on the capture for that reason, and
//! `HARNESS-10` is where measuring-changed-the-measurement is checked.
//!
//! `TODO/vendor.md`, `VENDOR-01`.

use rustls::ProtocolVersion;
use rustls::crypto::CryptoProvider;

/// The provider this tree builds rustls against.
///
/// ⛔ Named rather than defaulted. rustls has more than one and the choice is
/// recorded in the workspace manifest with its reason.
fn provider() -> CryptoProvider {
    rustls::crypto::ring::default_provider()
}

#[test]
fn the_vendored_crate_links_and_its_provider_offers_cipher_suites() {
    let provider = provider();
    assert!(
        !provider.cipher_suites.is_empty(),
        "the vendored provider offers no cipher suite, so nothing could complete a handshake"
    );
    assert!(
        !provider.kx_groups.is_empty(),
        "the vendored provider offers no key exchange group"
    );
}

#[test]
fn the_provider_offers_both_versions_a_browser_may_negotiate() {
    let provider = provider();
    let versions: Vec<ProtocolVersion> = provider
        .cipher_suites
        .iter()
        .map(|suite| suite.version().version)
        .collect();
    // ⚠ TLS 1.3 is what a current browser negotiates and 1.2 is what the
    // fallback path uses. A server offering only one of them changes what the
    // subject does, which is the one thing a capture surface must not do
    // silently.
    assert!(
        versions.contains(&ProtocolVersion::TLSv1_3),
        "no TLS 1.3 suite: {versions:?}"
    );
    assert!(
        versions.contains(&ProtocolVersion::TLSv1_2),
        "no TLS 1.2 suite, and the tls12 feature is named in the workspace manifest: {versions:?}"
    );
}

#[test]
fn the_provider_offers_the_group_a_browser_puts_its_key_share_in() {
    let provider = provider();
    let groups: Vec<_> = provider
        .kx_groups
        .iter()
        .map(|group| group.name())
        .collect();
    // ⚠ A server without X25519 answers a browser's first key share with a
    // HelloRetryRequest, which makes the browser send a SECOND ClientHello. The
    // capture would then be of a retry rather than of a first flight.
    assert!(
        groups
            .iter()
            .any(|name| format!("{name:?}").contains("X25519")),
        "no X25519 group: {groups:?}"
    );
}
