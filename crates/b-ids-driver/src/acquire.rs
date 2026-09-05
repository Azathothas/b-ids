//! Where a build comes from, with more than one way to ask.
//!
//! ⭐ **Every download URL will one day 404, and by then the build is gone.** A
//! capture pipeline with one acquisition route stops working on that day, and
//! the corpus can only ever describe what somebody happened to install.
//! `docs/history/todo/driver.md`, `DRIVER-05`.
//!
//! ⛔ **This project never redistributes a browser binary.** It publishes
//! measurements, versions, digests and the URL a build was fetched from. The
//! artefact itself is the vendor's to serve.
//!
//! ⚠ **Resolving, acquiring and driving are three jobs.** [`crate::resolve`]
//! finds what is installed and deliberately does not fetch; this fetches and
//! deliberately does not launch. A resolver that downloaded a browser would
//! change the machine it was asked to describe.
//!
//! ⭐ **The fetcher is a parameter, which is what makes this testable at all.**
//! A route's failure is the interesting case and it cannot be arranged against
//! a live network, so [`acquire_with`] takes the fetch as a closure and the
//! tests hand it one that refuses.

use std::fmt;

use serde::Serialize;

use crate::resolve::Family;

/// The automation-build index, which is the one first-party route that serves
/// an exact build rather than only the current one.
const FOR_TESTING_INDEX: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

/// Where a build was obtained, in the order the routes are tried.
///
/// ⛔ **The order is the design.** A route that answers with the exact build
/// asked for beats one that answers with something close, and a copy already on
/// this machine beats a fetch that can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Route {
    /// A build already on this machine, found by [`crate::resolve`].
    ///
    /// ⚠ First because it cannot fail for a network reason, and last in value:
    /// it answers with whatever is installed rather than with what was asked
    /// for, so a caller that wants an exact build checks the version it gets.
    Installed,
    /// A copy this project already fetched, kept under its digest.
    Cache,
    /// The vendor's own release channel, which serves the CURRENT build.
    ///
    /// ⛔ **Not offered by [`plan`], and that is the difference between this
    /// route and the others.** The channel serves whatever is current and
    /// cannot be asked for a build, so a plan keyed by version has nothing to
    /// ask it. `scripts/common/provision-browser.sh --route vendor` is what
    /// uses it, and a profile taken through it records this name.
    Vendor,
    /// The vendor's automation-build index, which serves an exact build.
    ChromeForTesting,
    /// The Edge enterprise update index, which serves an exact build and
    /// publishes a digest for it.
    ///
    /// ⭐ **It states a SHA-256 and a byte count for every artefact**, which
    /// the automation index for Chrome does not, so an acquisition through this
    /// route can be checked against the publisher rather than only recorded.
    EdgeEnterprise,
    /// An APT archive publishing Chromium as a `.deb` for Ubuntu.
    ///
    /// ⛔ **NOT A VENDOR ROUTE, and the name says so.** Chromium is source
    /// rather than a product with channels: Google publishes continuous
    /// snapshots keyed by a revision that is not the version a profile records,
    /// and Ubuntu's own `chromium-browser` package is a shim for a snap that
    /// cannot be launched on a hosted runner. What remains is a distributor,
    /// and a distributor's build is a REPACKAGING rather than the upstream one.
    ///
    /// ⭐ **The archive states a SHA-256 and a byte count per artefact in its
    /// `Packages` index**, so an acquisition through it is checkable against
    /// the publisher, which is the property that makes it usable here at all.
    ///
    /// ⚠ **What the resulting profile does and does not isolate.** Measured
    /// 2026-09-04: this archive serves Chromium `152.0.7977.75` for `noble`,
    /// which is the exact build of branded Chrome the corpus already holds at
    /// `chrome/stable/linux64`. So the pair is same-build rather than
    /// same-major, and the confound is no longer the version. ⛔ It IS the
    /// packaging: a distributor may build against system libraries and disable
    /// features, so what differs between the two is branding OR packaging and
    /// this pair does not separate them. `docs/history/todo/corpus.md`, `CORPUS-02`.
    ChromiumUbuntuPpa,
}

impl Route {
    /// Every route a profile may record, in the order the vocabulary is
    /// written down.
    ///
    /// ⛔ **Kept identical to `b_ids_schema::ACQUISITION_ROUTES` and to the
    /// published schema's enum**, and a test asserts the first two agree.
    /// ⚠ It is NOT the order [`plan`] tries, and it never was a promise that
    /// it would be: [`Route::Vendor`] is not a plannable route at all.
    #[must_use]
    pub fn all() -> [Self; 6] {
        [
            Self::Installed,
            Self::Cache,
            Self::Vendor,
            Self::ChromeForTesting,
            Self::EdgeEnterprise,
            Self::ChromiumUbuntuPpa,
        ]
    }

    /// Whether a build obtained through this route carries the vendor's own
    /// brand entry.
    ///
    /// ⭐ **`None` is an answer, and it is the honest one for two of the
    /// five.** A build already on the machine, or a copy this project cached,
    /// is whatever somebody installed: the route says where the bytes came from
    /// and not what they are. ⛔ Returning `true` for them would be the
    /// synthesised brand list `DRIVER-06` forbids, and a profile wrong in that
    /// field is one whose brand list cannot be trusted.
    ///
    /// ⚠ **The automation index serves UNBRANDED builds**, and they are the
    /// ones automation reaches for first, so this is not an edge case. The
    /// enterprise index serves the vendor's own product and is branded, which
    /// is why the two indexes answer differently.
    ///
    /// `docs/history/todo/driver.md`, `DRIVER-06`.
    #[must_use]
    pub fn branded(self) -> Option<bool> {
        match self {
            Self::Installed | Self::Cache => None,
            Self::Vendor | Self::EdgeEnterprise => Some(true),
            // ⚠ Chromium is unbranded by construction rather than by which
            // index served it: there is no vendor brand entry to carry.
            Self::ChromeForTesting | Self::ChromiumUbuntuPpa => Some(false),
        }
    }

    /// The route's name, as a report and a profile spell it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Cache => "cache",
            Self::Vendor => "vendor",
            Self::ChromeForTesting => "chrome-for-testing",
            Self::EdgeEnterprise => "edge-enterprise",
            Self::ChromiumUbuntuPpa => "chromium-ubuntu-ppa",
        }
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One route to try, with what it would be asked for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    /// Which route.
    pub route: Route,
    /// What it would be asked for.
    ///
    /// ⚠ `None` for [`Route::Installed`], which asks the machine rather than a
    /// URL, and that absence is the difference rather than an omission.
    pub url: Option<String>,
}

/// What one route answered, or why it did not.
///
/// ⛔ **A refusal is kept rather than collapsed into an absent artefact.** "The
/// index was unreachable" and "the index does not have that build" are
/// different facts about a run, and `CI-06` requires the run to report which
/// sources answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Refusal {
    /// Which route refused.
    pub route: Route,
    /// What it said.
    pub why: String,
}

/// What was obtained, and from where.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Acquired {
    /// The route that answered.
    pub route: Route,
    /// The URL it answered from, where there was one.
    pub url: Option<String>,
    /// The digest of what arrived, lowercase hex.
    ///
    /// ⭐ **This is what makes an acquisition reproducible after the artefact
    /// stops being served.** A profile records it, so a later reader can say
    /// whether two captures used the same bytes even when neither can be
    /// fetched again.
    pub sha256: String,
    /// How many bytes arrived.
    pub bytes: usize,
    /// Every route tried before this one, and what each said.
    pub refusals: Vec<Refusal>,
}

/// The routes to try for one build, in order.
///
/// ⚠ **`version` decides whether the exact-build route is offered at all.** The
/// automation index is keyed by build, so a plan with no version cannot use it
/// and says so by leaving it out rather than by offering a URL that 404s.
#[must_use]
pub fn plan(family: Family, version: Option<&str>) -> Vec<Candidate> {
    let mut candidates = vec![
        Candidate {
            route: Route::Installed,
            url: None,
        },
        Candidate {
            route: Route::Cache,
            url: None,
        },
    ];
    // ⛔ READ FROM THE ROUTE TABLE, never from a branch here. A family with no
    // first-party index gets no candidate offered, and saying so is better than
    // a URL that cannot answer. docs/history/todo/driver.md, DRIVER-10.
    if let (Some(index), Some(route), true) =
        (index_url(family), index_route(family), version.is_some())
    {
        candidates.push(Candidate {
            route,
            url: Some(index.to_owned()),
        });
    }
    candidates
}

/// Try each route in order and report which answered.
///
/// ⭐ **Both the fetcher and the digest are injected**, and the second is a
/// boundary rather than a convenience: [`b_ids_harness`] is a DEV dependency of
/// this crate on purpose, because a driver that imported the harness would be
/// one component with two jobs. So the bytes are hashed by whoever asked for
/// them.
///
/// ⭐ **Injecting the fetch is what makes this testable at all.** The case that
/// matters is the first route down and the second answering, and a function
/// that reached the network itself could only be tested on a day the network
/// agreed.
///
/// # Errors
///
/// Every refusal, in order, when no route produced anything. ⛔ Not the last
/// one alone: a caller shown only the final failure cannot tell an outage from
/// a build that does not exist.
pub fn acquire_with<F, D>(
    candidates: &[Candidate],
    mut fetch: F,
    digest: D,
) -> Result<Acquired, Vec<Refusal>>
where
    F: FnMut(&Candidate) -> Result<Vec<u8>, String>,
    D: Fn(&[u8]) -> String,
{
    let mut refusals = Vec::new();
    for candidate in candidates {
        match fetch(candidate) {
            Ok(bytes) if bytes.is_empty() => refusals.push(Refusal {
                route: candidate.route,
                why: "answered with no bytes at all".to_owned(),
            }),
            Ok(bytes) => {
                return Ok(Acquired {
                    route: candidate.route,
                    url: candidate.url.clone(),
                    sha256: digest(&bytes),
                    bytes: bytes.len(),
                    refusals,
                });
            }
            Err(why) => refusals.push(Refusal {
                route: candidate.route,
                why,
            }),
        }
    }
    Err(refusals)
}

/// A platform the automation-build index publishes for.
///
/// ⛔ **These are the index's own spellings, read from it rather than chosen.**
/// The corpus spells the third one `macos-arm64` and the index spells it
/// `mac-arm64`, so a caller crossing the two translates deliberately instead of
/// discovering the difference from a 404.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    /// 64-bit Linux.
    Linux64,
    /// 64-bit Windows.
    Win64,
    /// 32-bit Windows.
    Win32,
    /// Apple silicon.
    MacArm64,
    /// 64-bit Intel macOS.
    MacX64,
}

impl Platform {
    /// Every platform the index is known to publish for.
    #[must_use]
    pub fn all() -> [Self; 5] {
        [
            Self::Linux64,
            Self::Win64,
            Self::Win32,
            Self::MacArm64,
            Self::MacX64,
        ]
    }

    /// The index's own spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linux64 => "linux64",
            Self::Win64 => "win64",
            Self::Win32 => "win32",
            Self::MacArm64 => "mac-arm64",
            Self::MacX64 => "mac-x64",
        }
    }

    /// Read a platform from the name a caller wrote.
    ///
    /// ⛔ **An unknown name is `None` rather than a default.** A caller asking
    /// for a platform this index has no branch for is asking for something that
    /// cannot be produced, and answering with the host's own would provision one
    /// machine with another machine's build.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|p| p.as_str() == name)
    }

    /// Every platform's name, for a message that has to say what is available.
    #[must_use]
    pub fn names() -> String {
        Self::all()
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The platform this binary is running on, where the index publishes one.
    ///
    /// ⚠ `None` on a target the index does not serve, which is an answer rather
    /// than an error: the caller then has to name a platform, and being told so
    /// is better than being given the nearest one.
    #[must_use]
    pub fn host() -> Option<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Some(Self::Linux64),
            ("windows", "x86_64") => Some(Self::Win64),
            ("windows", "x86") => Some(Self::Win32),
            ("macos", "aarch64") => Some(Self::MacArm64),
            ("macos", "x86_64") => Some(Self::MacX64),
            _ => None,
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why the index could not name a download.
///
/// ⛔ **Three facts, kept apart.** "The index did not parse", "the index does
/// not publish that build" and "that build has no archive for this platform"
/// send a caller to three different places, and a single string saying the
/// download was not found sends them to none of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexRefusal {
    /// The bytes were not the index this reader knows.
    Unparsable(String),
    /// The index parsed and does not carry that build.
    ///
    /// ⚠ **The common case, and it is not an error in this tree.** The
    /// automation index publishes a SUBSET of builds: measured 2026-09-02, it
    /// carried 67 builds of Chrome `151` and neither `151.0.7922.173` nor
    /// `151.0.7922.174`, which are the two the hosted runner images served.
    NoSuchBuild {
        /// What was asked for.
        version: String,
        /// How many builds the index does carry.
        known: usize,
        /// The nearest builds it carries, in the index's order.
        nearest: Vec<String>,
    },
    /// This project has no index reader for that family.
    ///
    /// ⛔ **A refusal rather than a fallback.** `index_url` answers `None` for
    /// Chromium and for Firefox, so nothing should reach this reader with one,
    /// and if something does, guessing a reader would parse one vendor's index
    /// with another vendor's meanings for every field. docs/history/todo/corpus.md,
    /// CORPUS-02.
    NoIndexForFamily(Family),
    /// The build is published and not for this platform.
    NoDownloadForPlatform {
        /// What was asked for.
        version: String,
        /// The platform asked for.
        platform: Platform,
        /// The platforms that build does have.
        had: Vec<String>,
    },
}

impl fmt::Display for IndexRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unparsable(why) => write!(f, "the index did not parse: {why}"),
            Self::NoIndexForFamily(family) => write!(
                f,
                "there is no first-party index this project reads for {family}, so no build can \
                 be asked for by version. The routes that remain are an install already on the \
                 machine and a copy this project cached."
            ),
            Self::NoSuchBuild {
                version,
                known,
                nearest,
            } => write!(
                f,
                "the index does not publish {version}. It carries {known} build(s) for this \
                 platform, and it publishes a subset rather than every build the vendor ships. \
                 Nearest: {}",
                if nearest.is_empty() {
                    "none".to_owned()
                } else {
                    nearest.join(", ")
                }
            ),
            Self::NoDownloadForPlatform {
                version,
                platform,
                had,
            } => write!(
                f,
                "the index publishes {version} and not for {platform}. It has {}",
                had.join(", ")
            ),
        }
    }
}

/// The archive URL the automation index names for one build on one platform.
///
/// ⛔ **The index is read, never constructed.** The URL is predictable enough
/// to spell out, and a spelled-out URL is a second copy of a value the vendor
/// owns: the day the layout moves, a constructed URL 404s and a read one does
/// not. `DRIVER-05`.
///
/// ⚠ **It selects by name at every level.** The index is a list rather than a
/// map, so the build is found by its `version` field and the archive by its
/// `platform` field, never by position.
///
/// # Errors
///
/// [`IndexRefusal`], which distinguishes bytes that did not parse from a build
/// the index does not carry from a build with no archive for this platform.
pub fn download_url(
    index_json: &str,
    version: &str,
    platform: Platform,
) -> Result<String, IndexRefusal> {
    let parsed: serde_json::Value = serde_json::from_str(index_json)
        .map_err(|err| IndexRefusal::Unparsable(err.to_string()))?;
    let versions = parsed
        .get("versions")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            IndexRefusal::Unparsable("no `versions` array at the top level".to_owned())
        })?;

    let Some(entry) = versions
        .iter()
        .find(|e| e.get("version").and_then(serde_json::Value::as_str) == Some(version))
    else {
        // ⚠ The near misses, because "not published" is nearly always answered
        // by picking a build that is. A caller shown only a refusal has to
        // fetch five megabytes again to find out what it could have asked for.
        let line = version.rsplit_once('.').map_or(version, |(head, _)| head);
        let nearest: Vec<String> = versions
            .iter()
            .filter_map(|e| e.get("version").and_then(serde_json::Value::as_str))
            .filter(|v| v.starts_with(line))
            .map(str::to_owned)
            .collect();
        return Err(IndexRefusal::NoSuchBuild {
            version: version.to_owned(),
            known: versions.len(),
            nearest,
        });
    };

    let downloads = entry
        .get("downloads")
        .and_then(|d| d.get("chrome"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            IndexRefusal::Unparsable(format!("{version} carries no `downloads.chrome` array"))
        })?;

    for download in downloads {
        if download.get("platform").and_then(serde_json::Value::as_str) == Some(platform.as_str()) {
            let url = download
                .get("url")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    IndexRefusal::Unparsable(format!("{version} on {platform} carries no `url`"))
                })?;
            return Ok(url.to_owned());
        }
    }

    Err(IndexRefusal::NoDownloadForPlatform {
        version: version.to_owned(),
        platform,
        had: downloads
            .iter()
            .filter_map(|d| d.get("platform").and_then(serde_json::Value::as_str))
            .map(str::to_owned)
            .collect(),
    })
}

/// The enterprise update index, which is the first-party route for Edge.
///
/// ⭐ **It carries the vendor's own digest for every artefact**, which the
/// automation index for Chrome does not. That makes an Edge acquisition
/// verifiable against the publisher rather than only recorded: a caller can
/// compare what arrived with what the vendor said it published.
const EDGE_ENTERPRISE_INDEX: &str =
    "https://edgeupdates.microsoft.com/api/products?view=enterprise";

/// The APT archive this project reads for Chromium, and the one it measured.
///
/// ⛔ **Three routes were considered and two were measured shut.** Google's
/// `chromium-browser-snapshots` bucket answers `200` and serves a real
/// unpatched Chromium, measured 2026-09-04 at revision `1692381`, but it is
/// keyed by a trunk revision rather than by a version a profile records and a
/// continuous trunk build belongs to no channel anybody runs. Ubuntu's own
/// `chromium-browser` package on `noble` is a shim for a snap, and
/// `capture.yml` run `33854002345` measured what that costs: the resolver finds
/// `/usr/bin/chromium`, reads `151.0.7922.0` from it, and the launch aborts on
/// signal 6 inside `content::ZygoteHostImpl::Init`.
///
/// ⭐ **This one answers by VERSION and publishes a digest.** Measured
/// 2026-09-04: `Packages.gz` for `noble` carries `chromium` at
/// `152.0.7977.75-1xtradeb1.2404.1`, with a `SHA256` and a `Size` beside it.
///
/// ⚠ **It is served gzipped and there is no uncompressed `Packages`.**
/// Measured the same day: `Packages` is `404`, `Packages.gz` is `200`. A caller
/// fetching this decompresses before handing the bytes to [`deb_download`].
const CHROMIUM_UBUNTU_PPA_INDEX: &str = "https://ppa.launchpadcontent.net/xtradeb/apps/ubuntu/dists/noble/main/binary-amd64/Packages.gz";

/// Where the archive's `Filename` fields are relative to.
///
/// ⛔ **Derived from the index URL rather than written twice.** A `Packages`
/// index states `Filename: pool/main/c/chromium/...`, relative to the archive
/// root, and the root is the index URL with `dists/...` cut off it. Two
/// literals here would be two things to move on the day the archive does.
fn archive_base(index_url: &str) -> &str {
    match index_url.find("/dists/") {
        Some(at) => &index_url[..at],
        None => index_url,
    }
}

/// What one index answered about one build on one platform.
///
/// ⛔ **The digest is the publisher's claim, not a measurement of what
/// arrived.** A caller hashes the bytes it received and compares; recording the
/// published value as though it were the digest of the download would be a
/// number nobody checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Download {
    /// Where the artefact is served.
    pub url: String,
    /// The build the index names for it.
    pub version: String,
    /// The digest the publisher states, lowercase hex, where it states one.
    ///
    /// ⚠ `None` for an index that publishes no digest, which is the automation
    /// index for Chrome. That absence is a fact about the publisher.
    pub published_sha256: Option<String>,
    /// The size the publisher states, where it states one.
    pub published_bytes: Option<u64>,
    /// Other artefacts from the same index this build cannot be installed
    /// without, in the order the index lists them.
    ///
    /// ⛔ **READ FROM THE INDEX, NEVER A LIST WRITTEN HERE.** An APT archive
    /// splits one build across several binary packages, and the one carrying
    /// the browser `Depends` on a sibling at the same exact version: measured
    /// 2026-09-04, `chromium` depends on `chromium-common (= 152.0.7977.75-...)`
    /// and recommends `chromium-sandbox`, which is the SUID helper without
    /// which the browser launches and opens nothing.
    ///
    /// ⚠ **Empty for an index that serves one archive per build**, which both
    /// the Chrome and the Edge indexes do. An empty vector is the honest answer
    /// there rather than a shape those readers have to pretend to fill.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub companions: Vec<String>,
}

/// Which index serves a family, and the URL it is served at.
///
/// ⭐ **This is the route table, and it is data rather than a case statement in
/// a caller.** Adding a family is a row here and a reader beside it.
/// `docs/history/todo/driver.md`, `DRIVER-10`.
///
/// ⚠ `None` for a family this project has found no first-party index for.
#[must_use]
pub fn index_url(family: Family) -> Option<&'static str> {
    match family {
        Family::Chrome => Some(FOR_TESTING_INDEX),
        Family::Edge => Some(EDGE_ENTERPRISE_INDEX),
        Family::Chromium => Some(CHROMIUM_UBUNTU_PPA_INDEX),
        // ⛔ NONE IS THE MEASURED ANSWER, not a gap left for later.
        //
        // Firefox has a first-party archive, and this project has not read it.
        // ⚠ Writing a URL here from memory is exactly what docs/history/todo/RULES.md rule 1
        // refuses: the value would be inherited rather than measured, and a
        // fetcher pointed at a URL nobody checked fails at provisioning time
        // with an error nobody can attribute.
        //
        // ⭐ `plan` offers no candidate for a family that answers None, so it
        // is reachable through Route::Installed and Route::Cache and nothing
        // else. docs/history/todo/corpus.md, CORPUS-02.
        Family::Firefox => None,
    }
}

/// The route a profile records for a build obtained from a family's index.
///
/// ⚠ **`None` where [`index_url`] is `None`**, and the two are asked together
/// so they cannot disagree. This returned a `Route` unconditionally while
/// `index_url` returned an `Option`, so a family with no index still had a
/// route name, which is a value describing an acquisition that cannot happen.
#[must_use]
pub fn index_route(family: Family) -> Option<Route> {
    match family {
        Family::Chrome => Some(Route::ChromeForTesting),
        Family::Edge => Some(Route::EdgeEnterprise),
        Family::Chromium => Some(Route::ChromiumUbuntuPpa),
        Family::Firefox => None,
    }
}

/// The artefact one family's index names for one build on one platform.
///
/// ⛔ **Two indexes, two readers, one entry point.** The shapes are not alike:
/// Chrome's is a flat list of builds each carrying a list of downloads, and
/// Edge's is a list of products each carrying a list of releases per platform
/// and architecture. A single reader that tried to cover both would be a parser
/// with two meanings for every field.
///
/// # Errors
///
/// [`IndexRefusal`], which distinguishes bytes that did not parse from a build
/// the index does not carry from a build with no artefact for this platform.
pub fn download(
    family: Family,
    index_json: &str,
    version: &str,
    platform: Platform,
) -> Result<Download, IndexRefusal> {
    match family {
        Family::Chrome => {
            let url = download_url(index_json, version, platform)?;
            Ok(Download {
                url,
                version: version.to_owned(),
                // ⚠ The automation index publishes no digest. Absent rather
                // than empty: a caller can tell "the publisher states none"
                // from "the publisher states nothing useful".
                published_sha256: None,
                published_bytes: None,
                companions: Vec::new(),
            })
        }
        Family::Edge => edge_download(index_json, version, platform),
        Family::Chromium => deb_download(
            index_json,
            version,
            platform,
            archive_base(CHROMIUM_UBUNTU_PPA_INDEX),
        ),
        Family::Firefox => Err(IndexRefusal::NoIndexForFamily(family)),
    }
}

/// The artefact an APT archive's `Packages` index names for one build.
///
/// ⛔ **A third shape, and a third reader, for the reason the other two have
/// their own.** A `Packages` index is neither JSON nor a list of products: it
/// is RFC-822-shaped stanzas separated by blank lines, one per binary package,
/// with continuation lines indented. A reader that tried to cover it and a JSON
/// index at once would have two meanings for every field.
///
/// ⚠ **THE VERSION IS MATCHED ON ITS UPSTREAM PART, and that is not laxness.**
/// A Debian version carries the packager's own revision after a hyphen:
/// `152.0.7977.75-1xtradeb1.2404.1`. The version a PROFILE records is what the
/// binary reports about itself, which is the upstream part alone, so a caller
/// asking for `152.0.7977.75` is asking the only question it can ask. ⛔ The
/// match is anchored: `152.0.7977.7` does not match `152.0.7977.75`, because
/// the character after the upstream part has to be a hyphen or the end.
///
/// ⚠ **One architecture, because the index is per architecture.** The URL
/// already names `binary-amd64`, so a request for another platform is refused
/// here rather than answered from the wrong index.
///
/// # Errors
///
/// [`IndexRefusal`], distinguishing bytes that are not a `Packages` index from
/// a build the archive does not carry from a platform this index cannot serve.
fn deb_download(
    index_text: &str,
    version: &str,
    platform: Platform,
    base: &str,
) -> Result<Download, IndexRefusal> {
    if platform != Platform::Linux64 {
        return Err(IndexRefusal::NoDownloadForPlatform {
            version: version.to_owned(),
            platform,
            had: vec!["linux64".to_owned()],
        });
    }

    // ⛔ THE STANZAS ARE SPLIT ON A BLANK LINE, which is the format's own
    // record separator, and a run with no `Package:` field at all is not one.
    let stanzas: Vec<&str> = index_text
        .split("\n\n")
        .filter(|s| s.contains("Package:"))
        .collect();
    if stanzas.is_empty() {
        return Err(IndexRefusal::Unparsable(
            "no Packages stanza with a Package field".to_owned(),
        ));
    }

    let field = |stanza: &str, name: &str| -> Option<String> {
        stanza.lines().find_map(|line| {
            let rest = line.strip_prefix(name)?.strip_prefix(':')?;
            Some(rest.trim().to_owned())
        })
    };

    // ⚠ The binary package is named `chromium`, matched exactly. An archive
    // carrying `chromium-common` and `chromium-l10n` beside it would otherwise
    // match on a prefix and provision a language pack.
    let ours: Vec<&&str> = stanzas
        .iter()
        .filter(|s| field(s, "Package").as_deref() == Some("chromium"))
        .collect();

    let upstream_is = |deb_version: &str| -> bool {
        deb_version
            .strip_prefix(version)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
    };

    let Some(stanza) = ours
        .iter()
        .find(|s| field(s, "Version").as_deref().is_some_and(upstream_is))
    else {
        let nearest: Vec<String> = ours
            .iter()
            .filter_map(|s| field(s, "Version"))
            .take(8)
            .collect();
        return Err(IndexRefusal::NoSuchBuild {
            version: version.to_owned(),
            known: ours.len(),
            nearest,
        });
    };

    let filename = field(stanza, "Filename").ok_or_else(|| {
        IndexRefusal::Unparsable(format!(
            "the chromium stanza for {version} carries no Filename"
        ))
    })?;

    // ⛔ THE SIBLINGS THE ARCHIVE ITSELF NAMES, at the archive's own exact
    // version, and no list written here. An APT archive splits one build across
    // several binary packages: `chromium` cannot be installed without
    // `chromium-common` at the same version, and without `chromium-sandbox` it
    // launches and opens nothing, which is the failure DRIVER-10 already paid
    // for once on the Edge lane.
    //
    // ⚠ SELECTED BY BEING NAMED, not by being present. The archive also
    // publishes `chromium-l10n`, `chromium-driver` and `chromium-shell` at the
    // same version, and a rule that took every sibling would download a hundred
    // megabytes of translations to run a browser. This reads the chosen
    // stanza's own Depends and Recommends and keeps the siblings that appear in
    // them.
    let deb_version = field(stanza, "Version").unwrap_or_default();
    // ⛔ JOINED WITH A COMMA, NOT A SPACE, AND DRIVING IT IS WHAT FOUND THAT.
    // A dependency field is comma-separated, so a space here glued the last
    // term of Depends to the first of Recommends: the run against the real
    // index returned `chromium-common` and NOT `chromium-sandbox`, because
    // `chromium-sandbox` was no longer the first token of its term. ⚠ The
    // sandbox helper is the difference between a browser that launches and one
    // that opens a socket, which DRIVER-10 already paid for once on the Edge
    // lane, so this would have been a lane that installed and captured nothing.
    let wanted = format!(
        "{}, {}",
        field(stanza, "Depends").unwrap_or_default(),
        field(stanza, "Recommends").unwrap_or_default()
    );
    let base = base.trim_end_matches('/');
    let companions: Vec<String> = stanzas
        .iter()
        .filter(|s| field(s, "Version").as_deref() == Some(deb_version.as_str()))
        .filter_map(|s| {
            let name = field(s, "Package")?;
            if name == "chromium" || !named_in(&wanted, &name) {
                return None;
            }
            field(s, "Filename").map(|f| format!("{base}/{f}"))
        })
        .collect();

    Ok(Download {
        url: format!("{base}/{filename}"),
        version: version.to_owned(),
        // ⛔ LOWERCASED AND LENGTH-CHECKED. A `Packages` index may carry MD5Sum
        // and SHA1 beside SHA256, and a digest recorded under the wrong
        // algorithm looks comparable and never matches.
        published_sha256: field(stanza, "SHA256")
            .map(|d| d.to_ascii_lowercase())
            .filter(|d| d.len() == 64 && d.chars().all(|c| c.is_ascii_hexdigit())),
        published_bytes: field(stanza, "Size").and_then(|s| s.parse().ok()),
        companions,
    })
}

/// Whether a dependency field names a package, as a whole token.
///
/// ⛔ **Whole-token, because a substring match is wrong here in both
/// directions.** `chromium-common` contains `chromium`, and `chromium-shell`
/// appears inside nothing but itself; a `contains` would pull in every sibling
/// whose name is a prefix of another. A Debian dependency field separates
/// alternatives with `|` and terms with `,`, and a term may carry a version
/// constraint in parentheses, so the token ends at any of those or at a space.
fn named_in(field: &str, package: &str) -> bool {
    field.split([',', '|']).any(|term| {
        term.split_whitespace()
            .next()
            .is_some_and(|name| name == package)
    })
}

/// The artefact the Edge enterprise index names, and the digest it publishes.
///
/// ⚠ **The index spells platforms its own way**, `Linux` and `Windows` with an
/// `Architecture` beside them, and it serves a `deb`, an `rpm` and an `msi`
/// rather than one archive. This maps [`Platform`] onto that pair and picks the
/// artefact a provisioning run can install.
fn edge_download(
    index_json: &str,
    version: &str,
    platform: Platform,
) -> Result<Download, IndexRefusal> {
    let (want_platform, want_arch, want_artifact) = match platform {
        Platform::Linux64 => ("Linux", "x64", "deb"),
        Platform::Win64 => ("Windows", "x64", "msi"),
        // ⛔ Named rather than guessed. The index does publish other pairs, and
        // a reader that mapped one of these onto the nearest would provision a
        // machine with another machine's build.
        other => {
            return Err(IndexRefusal::NoDownloadForPlatform {
                version: version.to_owned(),
                platform: other,
                had: vec!["linux64".to_owned(), "win64".to_owned()],
            });
        }
    };

    let parsed: serde_json::Value = serde_json::from_str(index_json)
        .map_err(|err| IndexRefusal::Unparsable(err.to_string()))?;
    let products = parsed.as_array().ok_or_else(|| {
        IndexRefusal::Unparsable("no array of products at the top level".to_owned())
    })?;
    let stable = products
        .iter()
        .find(|p| p.get("Product").and_then(serde_json::Value::as_str) == Some("Stable"))
        .ok_or_else(|| IndexRefusal::Unparsable("no Stable product".to_owned()))?;
    let releases = stable
        .get("Releases")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| IndexRefusal::Unparsable("Stable carries no Releases array".to_owned()))?;

    // ⚠ Selected by name at every level: the platform, the architecture, the
    // build and the artefact kind are all matched on their own fields.
    let matching = |r: &&serde_json::Value| {
        r.get("Platform").and_then(serde_json::Value::as_str) == Some(want_platform)
            && r.get("Architecture").and_then(serde_json::Value::as_str) == Some(want_arch)
    };
    let Some(release) = releases
        .iter()
        .filter(matching)
        .find(|r| r.get("ProductVersion").and_then(serde_json::Value::as_str) == Some(version))
    else {
        let nearest: Vec<String> = releases
            .iter()
            .filter(matching)
            .filter_map(|r| r.get("ProductVersion").and_then(serde_json::Value::as_str))
            .take(8)
            .map(str::to_owned)
            .collect();
        return Err(IndexRefusal::NoSuchBuild {
            version: version.to_owned(),
            known: releases.iter().filter(matching).count(),
            nearest,
        });
    };

    let artifacts = release
        .get("Artifacts")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            IndexRefusal::Unparsable(format!("{version} on {platform} carries no Artifacts"))
        })?;
    let Some(artifact) = artifacts
        .iter()
        .find(|a| a.get("ArtifactName").and_then(serde_json::Value::as_str) == Some(want_artifact))
    else {
        return Err(IndexRefusal::NoDownloadForPlatform {
            version: version.to_owned(),
            platform,
            had: artifacts
                .iter()
                .filter_map(|a| a.get("ArtifactName").and_then(serde_json::Value::as_str))
                .map(str::to_owned)
                .collect(),
        });
    };

    let url = artifact
        .get("Location")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            IndexRefusal::Unparsable(format!("{version} on {platform} carries no Location"))
        })?;

    // ⛔ ONLY WHERE THE ALGORITHM IS THE ONE THIS PROJECT COMPARES WITH. A
    // digest recorded under the wrong algorithm is worse than none: it looks
    // comparable and never matches.
    let published_sha256 = artifact
        .get("HashAlgorithm")
        .and_then(serde_json::Value::as_str)
        .filter(|a| a.eq_ignore_ascii_case("SHA256"))
        .and_then(|_| artifact.get("Hash").and_then(serde_json::Value::as_str))
        .map(str::to_ascii_lowercase);

    Ok(Download {
        url: url.to_owned(),
        version: version.to_owned(),
        published_sha256,
        published_bytes: artifact
            .get("SizeInBytes")
            .and_then(serde_json::Value::as_u64),
        companions: Vec::new(),
    })
}
