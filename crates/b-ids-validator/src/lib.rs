//! Given a profile, answer one question: could a real browser have sent this?
//!
//! Pure logic over the model. No network, no browser, and no capture required
//! to write or to run it.
//!
//! # The three outcomes, and why there are three
//!
//! ⛔ **A check that cannot run is not a check that passed.** Most of these read
//! a header VALUE, and the default capture policy records header NAMES only, so
//! over an ordinary profile several of them have nothing to read. Reporting
//! those as passes would be the "step that exits 0 having done nothing it was
//! asked to do" defect, in the component whose whole job is refusing things.
//!
//! So every check answers [`Outcome::Passed`], [`Outcome::Failed`] or
//! [`Outcome::NotCheckable`], and [`Report`] carries all three. ⛔ It does not
//! WARN: a validator that only warns is a validator whose output nobody reads.
//!
//! # What a caller has to supply
//!
//! Three of the checks are about a profile IN A CONTEXT rather than about a
//! profile alone: what the consuming client can decode, what the target stack
//! can emit, and whether this profile is about to be published. [`Options`]
//! carries those, and each defaults to "not stated", which makes the check
//! report [`Outcome::NotCheckable`] rather than assume.

use std::collections::{BTreeMap, BTreeSet};

use b_ids_schema::http::Variant;
use b_ids_schema::tls::{Shuffle, is_grease_value};
use b_ids_schema::{Os, Profile, ProvenanceKind};

pub mod diff;
mod headers;
pub mod import;
pub mod reachable;

pub use diff::{Change, Diff, Uncontrolled, diff, render as render_diff};
pub use headers::{BrandEntry, parse_brand_list};
pub use import::{Exhibit, read as import_references, render as render_report};
pub use reachable::{Reachable, Unreachable, unreachable_dimensions};

/// One of the eight checks.
///
/// ⛔ Eight, each a separate function with its own test, and each test plants
/// the exact contradiction its check exists to catch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Check {
    /// The major in the brand list, in the User-Agent, and in `browser.major`.
    Version,
    /// The platform hint, the User-Agent's operating-system token, and
    /// `platform.os`, and the mobile hint against the platform.
    Platform,
    /// A branded profile has a vendor entry in its brand list; an unbranded one
    /// does not.
    Brand,
    /// The handshake came from a build whose major matches the claimed one.
    ///
    /// ⭐ The check that catches the worst failure mode in the field: new
    /// User-Agent, old hello.
    Handshake,
    /// The shuffle is recorded, and GREASE is drawn the way the hello shows.
    Grease,
    /// Every content-encoding token advertised is one the consumer can decode.
    Encoding,
    /// A profile that omits a setting is emitted by a stack that can omit it.
    Absence,
    /// No `vendor` field in a published profile, and no unreasoned
    /// `substituted` or `unreproducible`.
    Provenance,
}

impl Check {
    /// Every check, in the order they are run and reported.
    #[must_use]
    pub fn all() -> [Self; 8] {
        [
            Self::Version,
            Self::Platform,
            Self::Brand,
            Self::Handshake,
            Self::Grease,
            Self::Encoding,
            Self::Absence,
            Self::Provenance,
        ]
    }

    /// The check's name, as a report prints it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::Platform => "platform",
            Self::Brand => "brand",
            Self::Handshake => "handshake",
            Self::Grease => "grease",
            Self::Encoding => "encoding",
            Self::Absence => "absence",
            Self::Provenance => "provenance",
        }
    }
}

impl core::fmt::Display for Check {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One contradiction, naming the field it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Which check found it.
    pub check: Check,
    /// The dotted field path, as a provenance key would spell it.
    pub field: String,
    /// What contradicts what.
    pub message: String,
}

impl core::fmt::Display for Finding {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}: {}", self.check, self.field, self.message)
    }
}

/// What one check concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing contradicts anything.
    Passed,
    /// One or more contradictions.
    Failed(Vec<Finding>),
    /// ⛔ The check had nothing to read, and this is not a pass.
    NotCheckable(String),
}

/// What a caller knows that the profile does not carry.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Whether this profile is about to be published.
    ///
    /// ⛔ A `vendor` field is allowed in a draft and refused in a published
    /// profile, so the answer changes with the caller's intent rather than with
    /// the bytes.
    pub publishing: bool,
    /// The content encodings the consuming client can actually decode.
    ///
    /// ⚠ Empty means "not stated", which makes the encoding check report that
    /// it could not run. A client advertising an encoding it cannot decode
    /// hands compressed bytes to a parser.
    pub decodes: BTreeSet<String>,
    /// What the target stack can emit.
    pub target: Option<EmitterCapabilities>,
    /// Whether this browser family is known to shuffle its extension order.
    ///
    /// ⚠ `None` means "not stated", which makes the shuffle check report that
    /// it could not run. A profile cannot carry this: whether a FAMILY shuffles
    /// is a fact about a browser rather than about one connection, and a check
    /// that assumed it would report every non-shuffling browser as broken.
    /// `TODO/schema.md`, `SCHEMA-10`.
    pub expects_shuffle: Option<bool>,
}

/// What a stack this profile might be emitted through can and cannot do.
///
/// ⭐ The holes are the useful cells. A stack that cannot omit a setting cannot
/// carry a profile that omits one, and telling a client author that before they
/// ship is the whole point of the corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitterCapabilities {
    /// The stack's name, for the message.
    pub name: String,
    /// Whether it can leave a SETTINGS entry out entirely.
    pub omits_settings: bool,
    /// Whether it can write the PRIORITY block inside a HEADERS frame.
    pub writes_priority_block: bool,
    /// How many GREASE extension slots it can place at chosen codepoints.
    pub grease_slots: u8,
    /// Whether it can reproduce an arbitrary captured extension order.
    pub arbitrary_extension_order: bool,
}

/// What every check concluded about one profile.
#[derive(Debug, Clone)]
pub struct Report {
    /// One outcome per check, in [`Check::all`] order.
    pub results: BTreeMap<Check, Outcome>,
}

impl Report {
    /// Every finding, from every check that failed.
    #[must_use]
    pub fn findings(&self) -> Vec<&Finding> {
        self.results
            .values()
            .filter_map(|o| match o {
                Outcome::Failed(f) => Some(f),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Whether any check failed.
    #[must_use]
    pub fn failed(&self) -> bool {
        !self.findings().is_empty()
    }

    /// The checks that had nothing to read, with the reason each gave.
    #[must_use]
    pub fn not_checkable(&self) -> Vec<(Check, &str)> {
        self.results
            .iter()
            .filter_map(|(c, o)| match o {
                Outcome::NotCheckable(why) => Some((*c, why.as_str())),
                _ => None,
            })
            .collect()
    }

    /// The exit code a command should return: 0 clean, 1 a check failed, 2
    /// nothing could be checked at all.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        if self.failed() {
            return 1;
        }
        if self.not_checkable().len() == self.results.len() {
            return 2;
        }
        0
    }
}

/// Run every check over one profile.
#[must_use]
pub fn validate(profile: &Profile, options: &Options) -> Report {
    let mut results = BTreeMap::new();
    results.insert(Check::Version, check_version(profile));
    results.insert(Check::Platform, check_platform(profile));
    results.insert(Check::Brand, check_brand(profile));
    results.insert(Check::Handshake, check_handshake(profile));
    results.insert(Check::Grease, check_grease(profile, options));
    results.insert(Check::Encoding, check_encoding(profile, options));
    results.insert(Check::Absence, check_absence(profile, options));
    results.insert(Check::Provenance, check_provenance(profile, options));
    Report { results }
}

fn finding(check: Check, field: &str, message: String) -> Finding {
    Finding {
        check,
        field: field.to_owned(),
        message,
    }
}

fn verdict(findings: Vec<Finding>) -> Outcome {
    if findings.is_empty() {
        Outcome::Passed
    } else {
        Outcome::Failed(findings)
    }
}

fn navigate_header<'a>(profile: &'a Profile, name: &str) -> Option<&'a str> {
    profile
        .http
        .variant(Variant::Navigate)?
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))?
        .value
        .as_deref()
}

/// Why a header carried no value to check, in the words that fact deserves.
///
/// ⛔ **THREE DIFFERENT FACTS ARRIVE AS ONE `None`, and reporting them alike
/// tells a reader the capture was thin when the truth is about the browser.**
/// A browser that sends no `sec-ch-ua` at all is SIGNAL, and Firefox is one:
/// the header is a Chromium feature. A capture that recorded the header under
/// the names-only policy is a gap in the capture. A profile with no navigation
/// set recorded nothing about HTTP.
///
/// ⚠ Found by the door sweep on 2026-09-04, when `b_ids_driver::Family` learned
/// `firefox` and made the second case reachable. `TODO/corpus.md`, `CORPUS-02`.
fn why_no_value(profile: &Profile, name: &str) -> String {
    let Some(set) = profile.http.variant(Variant::Navigate) else {
        return "no navigation header set was recorded, so nothing about HTTP was".to_owned();
    };
    match set
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
    {
        // ⭐ The header was sent and its value was not kept. That is the
        // names-only policy, which is this project's default.
        Some(_) => format!("{name} was sent and no VALUE was recorded"),
        // ⛔ The header was not sent. That is a fact about the browser.
        None => format!("this browser sent no {name} header at all"),
    }
}

/// Check 1. The major in the brand list, in the User-Agent, and in
/// `browser.major` all agree.
///
/// ⭐ The check with a working implementation to read: the origin repository
/// asserts the same three things against one constant, in fourteen lines and
/// with no network, which is why a header block and a version constant cannot
/// drift apart in that tree at all.
#[must_use]
pub fn check_version(profile: &Profile) -> Outcome {
    let Some(ua) = navigate_header(profile, "user-agent") else {
        return Outcome::NotCheckable(
            "no user-agent VALUE was recorded; the default capture policy keeps names only"
                .to_owned(),
        );
    };
    let mut findings = Vec::new();
    let claimed = profile.browser.major;

    match headers::user_agent_major(ua) {
        Some(major) if major == claimed => {}
        Some(major) => findings.push(finding(
            Check::Version,
            "http.headers.user-agent",
            format!("carries major {major}, and browser.major is {claimed}"),
        )),
        None => findings.push(finding(
            Check::Version,
            "http.headers.user-agent",
            format!("no browser version token in {ua}"),
        )),
    }

    if let Some(raw) = navigate_header(profile, "sec-ch-ua") {
        let brands = parse_brand_list(raw);
        if brands.is_empty() {
            findings.push(finding(
                Check::Version,
                "http.headers.sec-ch-ua",
                format!("no brand entry could be read from {raw}"),
            ));
        } else if !brands.iter().any(|b| b.version == claimed.to_string()) {
            findings.push(finding(
                Check::Version,
                "http.headers.sec-ch-ua",
                format!(
                    "no brand claims major {claimed}; it claims {}",
                    brands
                        .iter()
                        .map(|b| format!("{}={}", b.brand, b.version))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
    }

    if !profile.browser.version.starts_with(&format!("{claimed}.")) {
        findings.push(finding(
            Check::Version,
            "browser.version",
            format!(
                "{} does not begin with the claimed major {claimed}",
                profile.browser.version
            ),
        ));
    }

    verdict(findings)
}

/// Check 2. The platform hint, the User-Agent's operating-system token and
/// `platform.os` agree, and the mobile hint agrees with the platform.
#[must_use]
pub fn check_platform(profile: &Profile) -> Outcome {
    let Some(ua) = navigate_header(profile, "user-agent") else {
        return Outcome::NotCheckable(why_no_value(profile, "user-agent"));
    };
    let mut findings = Vec::new();
    let os = profile.platform.os;

    if let Some(token) = headers::user_agent_os(ua) {
        if token != os {
            findings.push(finding(
                Check::Platform,
                "http.headers.user-agent",
                format!("names {token}, and platform.os is {os}"),
            ));
        }
    } else {
        findings.push(finding(
            Check::Platform,
            "http.headers.user-agent",
            format!("no operating-system token could be read from {ua}"),
        ));
    }

    if let Some(hint) = navigate_header(profile, "sec-ch-ua-platform") {
        let hint = hint.trim_matches('"');
        match headers::platform_hint_os(hint) {
            Some(hinted) if hinted == os => {}
            Some(hinted) => findings.push(finding(
                Check::Platform,
                "http.headers.sec-ch-ua-platform",
                format!("says {hinted}, and platform.os is {os}"),
            )),
            None => findings.push(finding(
                Check::Platform,
                "http.headers.sec-ch-ua-platform",
                format!("{hint} is not a platform this check knows"),
            )),
        }
    }

    if let Some(mobile) = navigate_header(profile, "sec-ch-ua-mobile") {
        let says_mobile = mobile.trim() == "?1";
        let is_mobile = matches!(os, Os::Android | Os::Ios);
        if says_mobile != is_mobile {
            findings.push(finding(
                Check::Platform,
                "http.headers.sec-ch-ua-mobile",
                format!(
                    "says {mobile}, and platform.os {os} is {}mobile",
                    if is_mobile { "" } else { "not " }
                ),
            ));
        }
    }

    verdict(findings)
}

/// Check 3. A branded profile has a vendor entry in its brand list; an
/// unbranded one does not, and says so in `browser.branded`.
///
/// ⚠ Chrome for Testing builds are unbranded, and they are the ones automation
/// reaches for first, so this is not an edge case.
#[must_use]
pub fn check_brand(profile: &Profile) -> Outcome {
    let Some(raw) = navigate_header(profile, "sec-ch-ua") else {
        return Outcome::NotCheckable(why_no_value(profile, "sec-ch-ua"));
    };
    let brands = parse_brand_list(raw);
    let vendor = headers::vendor_brand(&profile.browser.name);
    let has_vendor = brands.iter().any(|b| b.brand == vendor);

    if has_vendor == profile.browser.branded {
        return Outcome::Passed;
    }
    let message = if profile.browser.branded {
        format!("browser.branded is true and the brand list has no {vendor} entry")
    } else {
        format!("browser.branded is false and the brand list carries a {vendor} entry")
    };
    verdict(vec![finding(
        Check::Brand,
        "http.headers.sec-ch-ua",
        message,
    )])
}

/// Check 4. The handshake came from a build whose major matches the claimed
/// one.
///
/// ⛔ **Within one profile this cannot be decided, and saying so is the honest
/// answer.** Deciding it needs a per-build corpus of handshakes to compare
/// against, and this project has captured none. [`shared_handshakes`] is the
/// form of the check that CAN run today, across a set of profiles, and it is
/// what catches the shipped violations one reference database already has.
#[must_use]
pub fn check_handshake(profile: &Profile) -> Outcome {
    if profile.tls.extensions.is_empty() {
        return verdict(vec![finding(
            Check::Handshake,
            "tls.extensions",
            "no extensions at all, so the hello cannot have come from a browser".to_owned(),
        )]);
    }
    Outcome::NotCheckable(format!(
        "deciding whether this hello came from a {} build needs a per-build corpus to compare \
         against, and none exists yet. b-ids-validator::shared_handshakes is the form that runs \
         across a set of profiles today",
        profile.browser.major
    ))
}

/// The cross-profile form of check 4: two profiles claiming different majors
/// and carrying a byte-identical TLS half.
///
/// ⭐ **At most one of them can be right**, and this is what catches an entry
/// that returns a neighbour's fingerprint wholesale beside its own User-Agent.
#[must_use]
pub fn shared_handshakes(profiles: &[Profile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, a) in profiles.iter().enumerate() {
        for b in profiles.iter().skip(i + 1) {
            if a.browser.name == b.browser.name
                && a.browser.major != b.browser.major
                && a.tls == b.tls
            {
                findings.push(finding(
                    Check::Handshake,
                    "tls",
                    format!(
                        "{} and {} claim majors {} and {} and carry a byte-identical TLS half, \
                         so at most one of them was measured",
                        a.id, b.id, a.browser.major, b.browser.major
                    ),
                ));
            }
        }
    }
    findings
}

/// Check 5. The shuffle is recorded, and GREASE is drawn the way the hello
/// shows.
#[must_use]
pub fn check_grease(profile: &Profile, options: &Options) -> Outcome {
    let tls = &profile.tls;
    let mut findings = Vec::new();

    let positions: Vec<usize> = tls
        .extensions
        .iter()
        .enumerate()
        .filter(|(_, e)| e.is_grease())
        .map(|(i, _)| i)
        .collect();
    if positions != tls.grease.extension_positions {
        findings.push(finding(
            Check::Grease,
            "tls.grease.extension_positions",
            format!(
                "records {:?}, and the extension list has GREASE at {positions:?}",
                tls.grease.extension_positions
            ),
        ));
    }

    let values: Vec<u16> = tls
        .extensions
        .iter()
        .filter(|e| e.is_grease())
        .map(|e| e.codepoint)
        .collect();
    if values != tls.grease.values {
        findings.push(finding(
            Check::Grease,
            "tls.grease.values",
            format!(
                "records {:?}, and the extension list carries {values:?}",
                tls.grease.values
            ),
        ));
    }

    let bodies: Vec<&str> = tls
        .extensions
        .iter()
        .filter(|e| e.is_grease())
        .map(|e| e.body_hex.as_str())
        .collect();
    if bodies != tls.grease.bodies_hex {
        findings.push(finding(
            Check::Grease,
            "tls.grease.bodies_hex",
            format!(
                "records {:?}, and the extension list carries {bodies:?}",
                tls.grease.bodies_hex
            ),
        ));
    }

    let all_different = {
        let unique: BTreeSet<u16> = values.iter().copied().collect();
        unique.len() == values.len()
    };
    if all_different != tls.grease.distinct {
        findings.push(finding(
            Check::Grease,
            "tls.grease.distinct",
            format!(
                "records {}, and the drawn values {values:?} are {}",
                tls.grease.distinct,
                if all_different {
                    "all different"
                } else {
                    "not all different"
                }
            ),
        ));
    }

    // ⚠ A browser draws GREASE independently per slot, so two slots carrying
    // one value is the shape a client that reuses one draw produces.
    if values.len() >= 2 && !all_different {
        findings.push(finding(
            Check::Grease,
            "tls.grease.values",
            format!("{values:?} reuses one draw across slots, which a browser does not"),
        ));
    }

    if let Shuffle::Observed { draws, .. } | Shuffle::Fixed { draws } = tls.shuffled
        && draws < 2
    {
        findings.push(finding(
            Check::Grease,
            "tls.shuffled",
            format!("claims a shuffle state from {draws} draw(s), and one draw is not a sample"),
        ));
    }

    // ⛔ A BROWSER THE CALLER SAYS SHUFFLES, THAT DID NOT. The profile cannot
    // carry this: whether a FAMILY shuffles is a fact about a browser rather
    // than about one connection, so the caller states it and a check that
    // assumed it would report every non-shuffling browser as broken.
    // ⚠ `Unknown` and one draw are handled above; this is the case where the
    // sample WAS big enough and the order never moved. `TODO/schema.md`,
    // `SCHEMA-10`, and `docs/inherited-claims.md` section 2 is why: reproducing
    // a recorded order exactly is a reason to doubt the capture.
    if options.expects_shuffle == Some(true)
        && let Shuffle::Fixed { draws } = tls.shuffled
        && draws >= 2
    {
        findings.push(finding(
            Check::Grease,
            "tls.shuffled",
            format!(
                "{draws} draw(s) of a family the caller says shuffles produced one \
                 order, and a shuffling browser that never moved is a reason to \
                 doubt the capture"
            ),
        ));
    }

    // GREASE in the cipher list or the group list, with none in the extension
    // list, is a hello no browser sends.
    let elsewhere = tls.cipher_suites.iter().any(|c| is_grease_value(*c))
        || tls.key_exchange_groups.iter().any(|g| is_grease_value(*g));
    if elsewhere && values.is_empty() {
        findings.push(finding(
            Check::Grease,
            "tls.extensions",
            "GREASE appears in the cipher or group list and nowhere in the extensions".to_owned(),
        ));
    }

    verdict(findings)
}

/// Check 6. Every content-encoding token advertised is one the consuming client
/// can actually decode.
///
/// ⛔ A client advertising an encoding it cannot decode hands compressed bytes
/// to a parser.
#[must_use]
pub fn check_encoding(profile: &Profile, options: &Options) -> Outcome {
    if options.decodes.is_empty() {
        return Outcome::NotCheckable(
            "the caller did not say what the consuming client can decode".to_owned(),
        );
    }
    let Some(raw) = navigate_header(profile, "accept-encoding") else {
        return Outcome::NotCheckable(why_no_value(profile, "accept-encoding"));
    };
    let findings = raw
        .split(',')
        .map(|t| t.split(';').next().unwrap_or("").trim().to_lowercase())
        .filter(|t| !t.is_empty() && t != "identity" && !options.decodes.contains(t))
        .map(|t| {
            finding(
                Check::Encoding,
                "http.headers.accept-encoding",
                format!("advertises {t}, which the consuming client cannot decode"),
            )
        })
        .collect();
    verdict(findings)
}

/// Check 7. A profile that omits a setting is emitted by a stack that can omit
/// one.
///
/// ⚠ An absent setting is as load-bearing as a present one, and most stacks
/// cannot express absence at all.
#[must_use]
pub fn check_absence(profile: &Profile, options: &Options) -> Outcome {
    let Some(target) = options.target.as_ref() else {
        return Outcome::NotCheckable("the caller named no target stack".to_owned());
    };
    let mut findings = Vec::new();

    // SETTINGS_MAX_FRAME_SIZE is the entry this rule was written for.
    const MAX_FRAME_SIZE: u16 = 5;
    if !profile.http2.sends_setting(MAX_FRAME_SIZE) && !target.omits_settings {
        findings.push(finding(
            Check::Absence,
            "http2.frames",
            format!(
                "omits setting {MAX_FRAME_SIZE}, and {} cannot leave a setting out",
                target.name
            ),
        ));
    }

    if profile.http2.stream_priority.is_some() && !target.writes_priority_block {
        findings.push(finding(
            Check::Absence,
            "http2.stream_priority",
            format!(
                "carries a PRIORITY block, and {} cannot write one",
                target.name
            ),
        ));
    }

    let grease_slots = profile.tls.grease.values.len();
    if grease_slots > usize::from(target.grease_slots) {
        findings.push(finding(
            Check::Absence,
            "tls.grease.values",
            format!(
                "needs {grease_slots} GREASE slot(s), and {} has {}",
                target.name, target.grease_slots
            ),
        ));
    }

    if !target.arbitrary_extension_order {
        findings.push(finding(
            Check::Absence,
            "tls.extensions",
            format!(
                "records an exact extension order, and {} cannot reproduce an arbitrary one",
                target.name
            ),
        ));
    }

    verdict(findings)
}

/// Check 8. No `vendor` field in a published profile, and no unreasoned
/// `substituted` or `unreproducible`.
#[must_use]
pub fn check_provenance(profile: &Profile, options: &Options) -> Outcome {
    let mut findings: Vec<Finding> = profile
        .provenance
        .check()
        .into_iter()
        .map(|d| Finding {
            check: Check::Provenance,
            field: d.field().to_owned(),
            message: d.to_string(),
        })
        .collect();

    if options.publishing {
        for field in profile.provenance.vendor_fields() {
            findings.push(finding(
                Check::Provenance,
                field,
                format!(
                    "is {}, and a published profile carries none",
                    ProvenanceKind::Vendor
                ),
            ));
        }
    }

    verdict(findings)
}
