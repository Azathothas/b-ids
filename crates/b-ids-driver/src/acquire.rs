//! Where a build comes from, with more than one way to ask.
//!
//! ⭐ **Every download URL will one day 404, and by then the build is gone.** A
//! capture pipeline with one acquisition route stops working on that day, and
//! the corpus can only ever describe what somebody happened to install.
//! `TODO/driver.md`, `DRIVER-05`.
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
    /// The vendor's automation-build index, which serves an exact build.
    ChromeForTesting,
}

impl Route {
    /// Every route, in the order [`plan`] tries them.
    #[must_use]
    pub fn all() -> [Self; 3] {
        [Self::Installed, Self::Cache, Self::ChromeForTesting]
    }

    /// The route's name, as a report and a profile spell it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Cache => "cache",
            Self::ChromeForTesting => "chrome-for-testing",
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
    // ⛔ Chrome only, and saying so is better than a URL that cannot answer.
    // Edge and the rest arrive with their own indexes in DRIVER-06.
    if family == Family::Chrome && version.is_some() {
        candidates.push(Candidate {
            route: Route::ChromeForTesting,
            url: Some(FOR_TESTING_INDEX.to_owned()),
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
