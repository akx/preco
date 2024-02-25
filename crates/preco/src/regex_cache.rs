use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::Mutex;
use tracing::warn;

type RegexMap = BTreeMap<String, Result<Regex, regex::Error>>;

static REGEX_CACHE: Lazy<Mutex<RegexMap>> = Lazy::new(|| Mutex::new(RegexMap::default()));

/// Get a regex from the cache, or compile and cache it.
pub(crate) fn get_regex(pattern: &str) -> Result<Regex, regex::Error> {
    REGEX_CACHE
        .lock()
        .unwrap()
        .entry(pattern.to_string())
        .or_insert_with(|| Regex::new(pattern))
        .clone()
}

pub(crate) fn get_regex_with_warning(pattern: Option<&str>, warn_prefix: &str) -> Option<Regex> {
    pattern.and_then(|pattern| {
        get_regex(pattern)
            .map_err(|e| {
                warn!("{}: {}", warn_prefix, e);
                e
            })
            .ok()
    })
}
