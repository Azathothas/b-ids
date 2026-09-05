//! The version that is serving, not the one that is published.
//!
//! ⛔ **A version-history endpoint asked for the newest build on a channel
//! answers with the newest build it KNOWS**, which during a staged rollout is a
//! build almost nobody runs. Measured elsewhere and inherited: the highest known
//! build sat at a rollout fraction of `0.005`, the build at fraction 1 was two
//! majors behind it, and the automation-build index disagreed with that by one
//! patch component. `docs/inherited-claims.md` section 7.
//!
//! ⭐ **So: read the fraction, take the highest build at full rollout, and print
//! what the naive answer would have been beside it** so a reader can check the
//! choice rather than take it. Chasing a build one user in two hundred has
//! produces a correct fingerprint of a browser that does not exist.
//!
//! # ⚠ Two first-party sources, and a disagreement is a finding
//!
//! The releases endpoint and the automation-build index are the same vendor
//! answering the same question twice. ⛔ Neither is silently preferred: when
//! they disagree, something is mid-rollout and the report says so. That
//! disagreement is how the defect above was found in the first place.
//!
//! # ⛔ Every fetch is trapped on its own
//!
//! One dead endpoint degrades the run and leaves the other intact. A check that
//! reports nothing during somebody else's outage is a check people switch off.
//!
//! # Why this shells out to a fetcher
//!
//! ⚠ **Three routes were considered and the reason the other two lost is worth
//! keeping**, because the obvious one looks cheapest:
//!
//! - **an HTTP client crate**: brings its own TLS stack into a workspace that
//!   vendors one. `Cargo.toml` already names two builds of the same primitives
//!   as a cost to refuse, and version discovery is not worth paying it.
//! - **a client written on the vendored rustls**: needs a root store, so a
//!   dependency anyway, plus an HTTP/1.1 client this project would then own.
//!   ⛔ This project has enough parsers to keep correct.
//! - ⭐ **a fetcher the host already has**, one process per request. No new
//!   dependency, no second TLS stack, and trapping each fetch separately falls
//!   out of it. Its cost is that a host without one cannot run this, which is
//!   exit 2 and an honest answer rather than a wrong one.
//!
//! `docs/history/todo/driver.md`, `DRIVER-02`.

use std::process::Command;
use std::time::Duration;

use b_ids_schema::version_order;
use serde::Serialize;

/// How long one fetch may take.
///
/// ⛔ **A hard ceiling, because a fetcher with no limit is a hang.** A run that
/// never returns has no message and no exit code, and in continuous integration
/// it consumes the whole job's timeout and reports nothing.
const FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// The automation-build index, which is the same vendor answering the same
/// question a second way.
///
/// ⚠ It carries every channel in one document, so it takes no channel in its
/// path and the channel is selected out of the payload instead.
const FOR_TESTING: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json";

/// The releases endpoint for one channel, which carries a rollout fraction per
/// release.
///
/// ⚠ **`filter=endtime%3Dnone` is what restricts it to releases still being
/// served.** Without it the answer includes builds that have been superseded,
/// and the highest of those is a build nobody runs at all.
///
/// ⚠ **The platform is `win` and that is a condition of the answer**, not a
/// detail: a rollout fraction is per platform, and reading one platform's while
/// capturing on another would compare two different questions. `CORPUS-02` is
/// where more platforms arrive.
#[must_use]
pub fn releases_url(channel: &str) -> String {
    format!(
        "https://versionhistory.googleapis.com/v1/chrome/platforms/win/channels/{channel}/\
         versions/all/releases?filter=endtime%3Dnone&order_by=version%20desc"
    )
}

/// Where an answer came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// The channel's releases endpoint, with a fraction per release.
    Releases,
    /// The automation-build index.
    ChromeForTesting,
}

impl Source {
    /// The source's name, as a report prints it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Releases => "releases",
            Self::ChromeForTesting => "chrome-for-testing",
        }
    }

    /// The URL this source is read from for a channel.
    #[must_use]
    pub fn url(self, channel: &str) -> String {
        match self {
            Self::Releases => releases_url(channel),
            Self::ChromeForTesting => FOR_TESTING.to_owned(),
        }
    }
}

/// One release the endpoint listed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Release {
    /// The build.
    pub version: String,
    /// The share of users it is being served to, where the endpoint said.
    ///
    /// ⚠ `None` where the field is absent, which is a different fact from zero:
    /// a release with no stated fraction is one the endpoint did not say about,
    /// and treating it as unserved would drop a build that may be the answer.
    pub fraction: Option<f64>,
}

/// What one source answered, or why it did not.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Answer {
    /// Which source.
    pub source: Source,
    /// The build it named, where it answered.
    pub version: Option<String>,
    /// Why it did not answer, where it did not.
    ///
    /// ⛔ Kept rather than collapsed into an absent version. "The endpoint was
    /// down" and "the endpoint said nothing is at full rollout" are different
    /// facts about a run.
    pub error: Option<String>,
}

/// The choice, and what the naive answer would have been.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Chosen {
    /// The build this picks.
    pub version: String,
    /// Its rollout fraction.
    pub fraction: Option<f64>,
    /// The highest build the endpoint listed at all.
    ///
    /// ⭐ **Printed beside the answer so a reader can check the choice rather
    /// than take it.** Where these two differ, a rollout is in progress and the
    /// difference is the whole point of this module.
    pub highest_known: String,
    /// The highest build's rollout fraction.
    pub highest_fraction: Option<f64>,
}

/// What a version discovery run found.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    /// Every source, in the order they were asked.
    pub answers: Vec<Answer>,
    /// The build to capture, where any source answered.
    pub chosen: Option<Chosen>,
    /// Whether two sources answered and named different builds.
    ///
    /// ⚠ **A finding rather than an error**, and it is how the defect this
    /// module exists for was found. Two first-party sources disagreeing by one
    /// patch component means something is mid-rollout.
    pub disagreement: bool,
}

impl Report {
    /// Whether any source answered at all.
    ///
    /// ⛔ A run where nothing answered verified nothing, which is a different
    /// fact from a run that found no disagreement.
    #[must_use]
    pub fn answered(&self) -> bool {
        self.answers.iter().any(|a| a.version.is_some())
    }
}

/// Fetch one URL with whatever fetcher this host has.
///
/// ⚠ **`curl` first and `wget` second**, and the one that answered is not
/// recorded because it is a property of the host rather than of the answer.
/// `TOOL-04` is the open entry about a fetcher elsewhere in this tree stopping
/// when one of its two routes is down.
///
/// # Errors
///
/// A sentence naming what failed: no fetcher, a non-zero exit, or empty output.
pub fn fetch(url: &str) -> Result<String, String> {
    let seconds = FETCH_TIMEOUT.as_secs().to_string();
    let attempts: [(&str, Vec<String>); 2] = [
        (
            "curl",
            vec![
                "-fsSL".to_owned(),
                "--max-time".to_owned(),
                seconds.clone(),
                url.to_owned(),
            ],
        ),
        (
            "wget",
            vec![
                "-qO-".to_owned(),
                format!("--timeout={seconds}"),
                url.to_owned(),
            ],
        ),
    ];
    let mut refusals = Vec::new();
    for (program, args) in attempts {
        match Command::new(program).args(&args).output() {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
            }
            Ok(output) => refusals.push(format!(
                "{program} exited {}",
                output.status.code().unwrap_or(-1)
            )),
            Err(err) => refusals.push(format!("{program}: {err}")),
        }
    }
    Err(refusals.join("; "))
}

/// Read the releases endpoint's payload into a list.
///
/// ⚠ **Permissive about what it does not recognise and exact about what it
/// does.** A release with no version is skipped; a release with no fraction is
/// kept with `None`, because absent and zero are different facts.
///
/// # Errors
///
/// A sentence where the payload is not JSON or carries no releases at all.
pub fn parse_releases(payload: &str) -> Result<Vec<Release>, String> {
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|e| format!("the payload is not JSON: {e}"))?;
    let releases = value
        .get("releases")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "the payload carries no releases array".to_owned())?;
    let out: Vec<Release> = releases
        .iter()
        .filter_map(|r| {
            let version = r.get("version")?.as_str()?.to_owned();
            Some(Release {
                version,
                fraction: r.get("fraction").and_then(serde_json::Value::as_f64),
            })
        })
        .collect();
    if out.is_empty() {
        return Err("the payload carried no release with a version".to_owned());
    }
    Ok(out)
}

/// Read the automation-build index's payload for a channel.
///
/// # Errors
///
/// A sentence where the payload is not JSON or names no build for the channel.
pub fn parse_for_testing(payload: &str, channel: &str) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|e| format!("the payload is not JSON: {e}"))?;
    // ⚠ The index capitalises its channel keys. Matched case-insensitively
    // rather than by rewriting the caller's channel, because the caller's
    // spelling is this project's vocabulary and the index's is the index's.
    let channels = value
        .get("channels")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "the payload carries no channels object".to_owned())?;
    channels
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(channel))
        .and_then(|(_, entry)| entry.get("version")?.as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("the index names no build for {channel}"))
}

/// Choose the build that is actually serving.
///
/// ⭐ **The highest build at full rollout**, and only where there is none does
/// the highest fraction win. ⛔ Never the highest build: that is the answer the
/// naive query gives and it is the defect this whole module is about.
///
/// ⚠ A release with no stated fraction cannot be shown to be at full rollout,
/// so it does not win the first pass. It can still win the fallback, because a
/// list where nothing states a fraction has to produce something.
#[must_use]
pub fn choose(releases: &[Release]) -> Option<Chosen> {
    let highest = releases
        .iter()
        .max_by(|a, b| version_order(&a.version).cmp(&version_order(&b.version)))?;

    let full: Option<&Release> = releases
        .iter()
        .filter(|r| r.fraction.is_some_and(|f| f >= 1.0))
        .max_by(|a, b| version_order(&a.version).cmp(&version_order(&b.version)));

    let chosen = full.or_else(|| {
        releases.iter().max_by(|a, b| {
            a.fraction
                .unwrap_or(0.0)
                .partial_cmp(&b.fraction.unwrap_or(0.0))
                .unwrap_or(core::cmp::Ordering::Equal)
        })
    })?;

    Some(Chosen {
        version: chosen.version.clone(),
        fraction: chosen.fraction,
        highest_known: highest.version.clone(),
        highest_fraction: highest.fraction,
    })
}

/// Assemble a report from what each source answered.
///
/// ⛔ **Pure, so the whole decision is testable without a network.** Only
/// [`discover`] fetches.
#[must_use]
pub fn report(
    releases: Result<Vec<Release>, String>,
    for_testing: Result<String, String>,
) -> Report {
    let chosen = releases.as_ref().ok().and_then(|r| choose(r));
    let answers = vec![
        Answer {
            source: Source::Releases,
            version: chosen.as_ref().map(|c| c.version.clone()),
            error: releases.err(),
        },
        Answer {
            source: Source::ChromeForTesting,
            version: for_testing.as_ref().ok().cloned(),
            error: for_testing.err(),
        },
    ];
    // ⛔ Only where BOTH answered. One source silent is a degraded run, not a
    // disagreement, and reporting it as one would make an outage look like a
    // rollout.
    let named: Vec<&String> = answers.iter().filter_map(|a| a.version.as_ref()).collect();
    let disagreement = named.len() > 1 && named.iter().any(|v| *v != named[0]);
    Report {
        answers,
        chosen,
        disagreement,
    }
}

/// Ask every source and assemble the report.
///
/// ⚠ **The one function here that touches the network**, and every fetch inside
/// it is trapped on its own.
#[must_use]
pub fn discover(channel: &str) -> Report {
    let releases =
        fetch(&Source::Releases.url(channel)).and_then(|payload| parse_releases(&payload));
    let for_testing = fetch(&Source::ChromeForTesting.url(channel))
        .and_then(|payload| parse_for_testing(&payload, channel));
    report(releases, for_testing)
}
