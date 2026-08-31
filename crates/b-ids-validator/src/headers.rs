//! Reading the three header values the coherence checks compare.
//!
//! ⚠ **Deliberately small and deliberately not a parser for the whole grammar.**
//! Each function answers one question a check asks, returns `None` where it
//! cannot, and never guesses. A header parser that guessed would turn a
//! validator's refusal into a coin toss.

use b_ids_schema::Os;

/// One entry of a `sec-ch-ua` brand list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandEntry {
    /// The brand, with its quotes removed.
    pub brand: String,
    /// The version the entry claims, as written.
    pub version: String,
}

/// Read a `sec-ch-ua` brand list.
///
/// ```text
/// "Chromium";v="152", "Not?A_Brand";v="24", "Google Chrome";v="152"
/// ```
///
/// ⚠ Split on the comma that separates entries. A brand containing a comma
/// would break this, and none of the shipped ones does; a brand containing a
/// semicolon would too. Both are recorded here rather than defended against,
/// because a defence nobody can test is a defence nobody should trust.
#[must_use]
pub fn parse_brand_list(raw: &str) -> Vec<BrandEntry> {
    raw.split(',')
        .filter_map(|entry| {
            let (brand, rest) = entry.trim().split_once(';')?;
            let version = rest.trim().strip_prefix("v=")?;
            Some(BrandEntry {
                brand: brand.trim().trim_matches('"').to_owned(),
                version: version.trim().trim_matches('"').to_owned(),
            })
        })
        .collect()
}

/// The major version a Chromium-family User-Agent claims.
///
/// Reads the `Chrome/<major>.` or `Firefox/<major>.` token, whichever is
/// present, and returns `None` where neither is.
#[must_use]
pub fn user_agent_major(ua: &str) -> Option<u32> {
    for token in ["Chrome/", "Firefox/", "Version/"] {
        if let Some(rest) = ua.split(token).nth(1) {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(major) = digits.parse::<u32>() {
                return Some(major);
            }
        }
    }
    None
}

/// The operating system a User-Agent names.
///
/// ⚠ Order matters here: an Android User-Agent also contains `Linux`, and a
/// check that matched `Linux` first would call every Android capture a Linux
/// one.
#[must_use]
pub fn user_agent_os(ua: &str) -> Option<Os> {
    if ua.contains("Android") {
        return Some(Os::Android);
    }
    if ua.contains("iPhone") || ua.contains("iPad") {
        return Some(Os::Ios);
    }
    if ua.contains("Windows") {
        return Some(Os::Windows);
    }
    if ua.contains("Macintosh") || ua.contains("Mac OS X") {
        return Some(Os::Mac);
    }
    if ua.contains("Linux") || ua.contains("X11") {
        return Some(Os::Linux);
    }
    None
}

/// The operating system a `sec-ch-ua-platform` hint names.
#[must_use]
pub fn platform_hint_os(hint: &str) -> Option<Os> {
    match hint.trim().trim_matches('"') {
        "Windows" => Some(Os::Windows),
        "macOS" => Some(Os::Mac),
        "Linux" => Some(Os::Linux),
        "Android" => Some(Os::Android),
        "iOS" => Some(Os::Ios),
        _ => None,
    }
}

/// The vendor's own brand entry for a browser name.
///
/// ⚠ `Chromium` is not it. An unbranded Chrome for Testing build carries
/// `Chromium` and a fake brand, and a branded one carries `Google Chrome`
/// beside them. That difference is the whole of check 3.
#[must_use]
pub fn vendor_brand(browser_name: &str) -> &'static str {
    match browser_name.to_lowercase().as_str() {
        "chrome" => "Google Chrome",
        "edge" => "Microsoft Edge",
        "opera" => "Opera",
        "brave" => "Brave",
        _ => "",
    }
}
