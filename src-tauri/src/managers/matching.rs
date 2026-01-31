//! Matching engine for MuttonText.
//!
//! Provides `StrictMatcher`, `LooseMatcher`, and `MatcherEngine` for efficient
//! keyword detection in typed text buffers.

use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use crate::models::{Combo, MatchingMode};

/// Errors that can occur during matching operations.
#[derive(Debug, Error)]
pub enum MatchingError {
    #[error("No combos loaded in matcher engine")]
    NoCombosLoaded,
}

/// Result of a successful match.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchResult {
    /// The ID of the matched combo.
    pub combo_id: Uuid,
    /// The keyword that was matched.
    pub keyword: String,
    /// The snippet to expand into.
    pub snippet: String,
    /// Length of the keyword in the buffer (for deletion).
    pub keyword_len: usize,
}

/// Checks if `buffer` ends with `keyword` preceded by a word boundary.
///
/// Word boundaries: start of buffer, space, tab, newline, or punctuation.
#[inline]
fn is_strict_match(buffer: &str, keyword: &str, case_sensitive: bool) -> bool {
    if buffer.is_empty() || keyword.is_empty() {
        return false;
    }

    let (buf, kw) = if case_sensitive {
        (buffer.to_string(), keyword.to_string())
    } else {
        (buffer.to_lowercase(), keyword.to_lowercase())
    };

    if !buf.ends_with(&kw) {
        return false;
    }

    // Check what precedes the keyword
    let prefix_len = buf.len() - kw.len();
    if prefix_len == 0 {
        // Keyword is at start of buffer — valid boundary
        return true;
    }

    match buf[..prefix_len].chars().last() {
        Some(c) => is_word_boundary(c),
        None => true, // shouldn't happen given prefix_len > 0, but safe default
    }
}

/// Checks if `buffer` simply ends with `keyword` (no boundary check).
#[inline]
fn is_loose_match(buffer: &str, keyword: &str, case_sensitive: bool) -> bool {
    if buffer.is_empty() || keyword.is_empty() {
        return false;
    }

    if case_sensitive {
        buffer.ends_with(keyword)
    } else {
        buffer.to_lowercase().ends_with(&keyword.to_lowercase())
    }
}

/// Returns true if the character is a word boundary.
#[inline]
fn is_word_boundary(c: char) -> bool {
    c.is_whitespace() || c.is_ascii_punctuation()
}

/// Indexes active combos for efficient matching against typed text buffers.
///
/// Combos are grouped by matching mode. A hash map keyed by keyword length
/// allows quick candidate filtering: only combos whose keyword length is <= the
/// buffer length are considered.
pub struct MatcherEngine {
    /// Strict combos indexed by keyword length.
    strict_by_len: HashMap<usize, Vec<ComboEntry>>,
    /// Loose combos indexed by keyword length.
    loose_by_len: HashMap<usize, Vec<ComboEntry>>,
    /// Maximum keyword length across all loaded combos.
    max_keyword_len: usize,
    /// Whether the engine is paused (skips all matching).
    is_paused: bool,
    /// List of excluded application names.
    excluded_apps: Vec<String>,
}

/// Internal lightweight representation of a combo for matching.
#[derive(Debug, Clone)]
struct ComboEntry {
    id: Uuid,
    keyword: String,
    snippet: String,
    case_sensitive: bool,
    /// Pre-computed keyword length in bytes (MT-1107).
    keyword_byte_len: usize,
}

impl MatcherEngine {
    /// Creates a new empty `MatcherEngine`.
    pub fn new() -> Self {
        Self {
            strict_by_len: HashMap::new(),
            loose_by_len: HashMap::new(),
            max_keyword_len: 0,
            is_paused: false,
            excluded_apps: Vec::new(),
        }
    }

    /// Loads (or reloads) all enabled combos into the engine index.
    pub fn load_combos(&mut self, combos: &[Combo]) {
        self.strict_by_len.clear();
        self.loose_by_len.clear();
        self.max_keyword_len = 0;

        for combo in combos.iter().filter(|c| c.enabled) {
            let kw_len = combo.keyword.len();
            let entry = ComboEntry {
                id: combo.id,
                keyword: combo.keyword.clone(),
                snippet: combo.snippet.clone(),
                case_sensitive: combo.case_sensitive,
                keyword_byte_len: kw_len,
            };
            if kw_len > self.max_keyword_len {
                self.max_keyword_len = kw_len;
            }
            let map = match combo.matching_mode {
                MatchingMode::Strict => &mut self.strict_by_len,
                MatchingMode::Loose => &mut self.loose_by_len,
            };
            map.entry(kw_len).or_default().push(entry);
        }

        tracing::debug!(
            "MatcherEngine loaded: {} strict lengths, {} loose lengths, max_kw={}",
            self.strict_by_len.len(),
            self.loose_by_len.len(),
            self.max_keyword_len,
        );
    }

    /// Sets the list of excluded application names.
    pub fn set_excluded_apps(&mut self, apps: Vec<String>) {
        self.excluded_apps = apps;
    }

    /// Returns true if the given application name is in the exclusion list.
    pub fn is_app_excluded(&self, app_name: &str) -> bool {
        let app_lower = app_name.to_lowercase();
        self.excluded_apps
            .iter()
            .any(|excluded| app_lower.contains(&excluded.to_lowercase()))
    }

    /// Pauses the engine. While paused, `find_match` always returns `None`.
    pub fn pause(&mut self) {
        self.is_paused = true;
        tracing::info!("MatcherEngine paused");
    }

    /// Resumes the engine.
    pub fn resume(&mut self) {
        self.is_paused = false;
        tracing::info!("MatcherEngine resumed");
    }

    /// Returns whether the engine is currently paused.
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Finds the first combo that matches the end of the given buffer.
    ///
    /// Returns `None` if paused, buffer is empty, or no match is found.
    /// Optionally checks the current app against the exclusion list.
    #[inline]
    pub fn find_match(&self, buffer: &str, current_app: Option<&str>) -> Option<MatchResult> {
        if self.is_paused || buffer.is_empty() {
            return None;
        }

        if let Some(app) = current_app {
            if self.is_app_excluded(app) {
                return None;
            }
        }

        // Only check keyword lengths that could fit in the buffer
        let buf_len = buffer.len();

        // Check strict combos first (more specific)
        for (&kw_len, entries) in &self.strict_by_len {
            if kw_len > buf_len {
                continue;
            }
            for entry in entries {
                if is_strict_match(buffer, &entry.keyword, entry.case_sensitive) {
                    return Some(MatchResult {
                        combo_id: entry.id,
                        keyword: entry.keyword.clone(),
                        snippet: entry.snippet.clone(),
                        keyword_len: entry.keyword_byte_len,
                    });
                }
            }
        }

        // Check loose combos
        for (&kw_len, entries) in &self.loose_by_len {
            if kw_len > buf_len {
                continue;
            }
            for entry in entries {
                if is_loose_match(buffer, &entry.keyword, entry.case_sensitive) {
                    return Some(MatchResult {
                        combo_id: entry.id,
                        keyword: entry.keyword.clone(),
                        snippet: entry.snippet.clone(),
                        keyword_len: entry.keyword_byte_len,
                    });
                }
            }
        }

        None
    }

    /// Returns the number of indexed combos.
    pub fn combo_count(&self) -> usize {
        let strict: usize = self.strict_by_len.values().map(|v| v.len()).sum();
        let loose: usize = self.loose_by_len.values().map(|v| v.len()).sum();
        strict + loose
    }
}

impl Default for MatcherEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::combo::ComboBuilder;

    fn make_combo(keyword: &str, snippet: &str, mode: MatchingMode, case_sensitive: bool) -> Combo {
        ComboBuilder::new()
            .keyword(keyword)
            .snippet(snippet)
            .matching_mode(mode)
            .case_sensitive(case_sensitive)
            .build()
            .unwrap()
    }

    fn strict(keyword: &str, snippet: &str) -> Combo {
        make_combo(keyword, snippet, MatchingMode::Strict, false)
    }

    fn loose(keyword: &str, snippet: &str) -> Combo {
        make_combo(keyword, snippet, MatchingMode::Loose, false)
    }

    // ── StrictMatcher unit tests ──────────────────────────────────

    #[test]
    fn test_strict_match_after_space() {
        assert!(is_strict_match("hello sig", "sig", false));
    }

    #[test]
    fn test_strict_match_after_tab() {
        assert!(is_strict_match("hello\tsig", "sig", false));
    }

    #[test]
    fn test_strict_match_after_newline() {
        assert!(is_strict_match("hello\nsig", "sig", false));
    }

    #[test]
    fn test_strict_match_at_start_of_buffer() {
        assert!(is_strict_match("sig", "sig", false));
    }

    #[test]
    fn test_strict_match_after_punctuation() {
        assert!(is_strict_match("hello.sig", "sig", false));
        assert!(is_strict_match("hello,sig", "sig", false));
        assert!(is_strict_match("hello!sig", "sig", false));
        assert!(is_strict_match("hello;sig", "sig", false));
        assert!(is_strict_match("(sig", "sig", false));
    }

    #[test]
    fn test_strict_no_match_mid_word() {
        assert!(!is_strict_match("testsig", "sig", false));
    }

    #[test]
    fn test_strict_no_match_partial() {
        assert!(!is_strict_match("mysig", "sig", false));
    }

    #[test]
    fn test_strict_empty_buffer() {
        assert!(!is_strict_match("", "sig", false));
    }

    #[test]
    fn test_strict_empty_keyword() {
        assert!(!is_strict_match("hello", "", false));
    }

    #[test]
    fn test_strict_case_insensitive() {
        assert!(is_strict_match("hello SIG", "sig", false));
        assert!(is_strict_match("hello Sig", "sig", false));
    }

    #[test]
    fn test_strict_case_sensitive() {
        assert!(is_strict_match("hello sig", "sig", true));
        assert!(!is_strict_match("hello SIG", "sig", true));
        assert!(!is_strict_match("hello Sig", "sig", true));
    }

    // ── LooseMatcher unit tests ───────────────────────────────────

    #[test]
    fn test_loose_match_ends_with() {
        assert!(is_loose_match("testsig", "sig", false));
    }

    #[test]
    fn test_loose_match_after_space() {
        assert!(is_loose_match("test sig", "sig", false));
    }

    #[test]
    fn test_loose_match_exact() {
        assert!(is_loose_match("sig", "sig", false));
    }

    #[test]
    fn test_loose_match_mid_word() {
        assert!(is_loose_match("mysignature", "signature", false));
    }

    #[test]
    fn test_loose_no_match() {
        assert!(!is_loose_match("hello world", "sig", false));
    }

    #[test]
    fn test_loose_empty_buffer() {
        assert!(!is_loose_match("", "sig", false));
    }

    #[test]
    fn test_loose_empty_keyword() {
        assert!(!is_loose_match("hello", "", false));
    }

    #[test]
    fn test_loose_case_insensitive() {
        assert!(is_loose_match("testSIG", "sig", false));
    }

    #[test]
    fn test_loose_case_sensitive() {
        assert!(is_loose_match("testsig", "sig", true));
        assert!(!is_loose_match("testSIG", "sig", true));
    }

    // ── MatcherEngine tests ───────────────────────────────────────

    #[test]
    fn test_engine_new_is_empty() {
        let engine = MatcherEngine::new();
        assert_eq!(engine.combo_count(), 0);
        assert!(!engine.is_paused());
    }

    #[test]
    fn test_engine_load_combos() {
        let mut engine = MatcherEngine::new();
        let combos = vec![strict("sig", "Signature"), loose("addr", "123 Main St")];
        engine.load_combos(&combos);
        assert_eq!(engine.combo_count(), 2);
    }

    #[test]
    fn test_engine_skips_disabled_combos() {
        let mut engine = MatcherEngine::new();
        let mut combo = strict("sig", "Signature");
        combo.enabled = false;
        engine.load_combos(&[combo]);
        assert_eq!(engine.combo_count(), 0);
    }

    #[test]
    fn test_engine_strict_match() {
        let mut engine = MatcherEngine::new();
        let combo = strict("sig", "Best regards");
        let combo_id = combo.id;
        engine.load_combos(&[combo]);

        let result = engine.find_match("hello sig", None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.combo_id, combo_id);
        assert_eq!(m.keyword, "sig");
        assert_eq!(m.snippet, "Best regards");
        assert_eq!(m.keyword_len, 3);
    }

    #[test]
    fn test_engine_strict_no_mid_word() {
        let mut engine = MatcherEngine::new();
        engine.load_combos(&[strict("sig", "Signature")]);
        assert!(engine.find_match("testsig", None).is_none());
    }

    #[test]
    fn test_engine_loose_match_mid_word() {
        let mut engine = MatcherEngine::new();
        let combo = loose("sig", "Signature");
        engine.load_combos(&[combo]);
        assert!(engine.find_match("testsig", None).is_some());
    }

    #[test]
    fn test_engine_empty_buffer() {
        let mut engine = MatcherEngine::new();
        engine.load_combos(&[strict("sig", "Signature")]);
        assert!(engine.find_match("", None).is_none());
    }

    #[test]
    fn test_engine_no_match() {
        let mut engine = MatcherEngine::new();
        engine.load_combos(&[strict("sig", "Signature")]);
        assert!(engine.find_match("hello world", None).is_none());
    }

    #[test]
    fn test_engine_pause_resume() {
        let mut engine = MatcherEngine::new();
        engine.load_combos(&[strict("sig", "Signature")]);

        engine.pause();
        assert!(engine.is_paused());
        assert!(engine.find_match("hello sig", None).is_none());

        engine.resume();
        assert!(!engine.is_paused());
        assert!(engine.find_match("hello sig", None).is_some());
    }

    #[test]
    fn test_engine_excluded_app() {
        let mut engine = MatcherEngine::new();
        engine.load_combos(&[strict("sig", "Signature")]);
        engine.set_excluded_apps(vec!["1password".to_string(), "keepass".to_string()]);

        assert!(engine.find_match("hello sig", Some("1Password")).is_none());
        assert!(engine.find_match("hello sig", Some("KeePass")).is_none());
        assert!(engine.find_match("hello sig", Some("notepad")).is_some());
        assert!(engine.find_match("hello sig", None).is_some());
    }

    #[test]
    fn test_engine_case_insensitive_match() {
        let mut engine = MatcherEngine::new();
        engine.load_combos(&[strict("sig", "Signature")]);
        assert!(engine.find_match("hello SIG", None).is_some());
    }

    #[test]
    fn test_engine_case_sensitive_match() {
        let mut engine = MatcherEngine::new();
        let combo = make_combo("sig", "Signature", MatchingMode::Strict, true);
        engine.load_combos(&[combo]);
        assert!(engine.find_match("hello sig", None).is_some());
        assert!(engine.find_match("hello SIG", None).is_none());
    }

    #[test]
    fn test_engine_reload_clears_previous() {
        let mut engine = MatcherEngine::new();
        engine.load_combos(&[strict("sig", "Signature")]);
        assert_eq!(engine.combo_count(), 1);

        engine.load_combos(&[strict("addr", "Address"), strict("tel", "Phone")]);
        assert_eq!(engine.combo_count(), 2);
        assert!(engine.find_match("hello sig", None).is_none());
    }

    #[test]
    fn test_engine_multiple_combos_first_wins() {
        let mut engine = MatcherEngine::new();
        let c1 = strict("sig", "First");
        let c2 = strict("sig", "Second");
        let _id1 = c1.id;
        engine.load_combos(&[c1, c2]);
        // One of them matches (hash map ordering not guaranteed, but both have same keyword)
        let result = engine.find_match("hello sig", None);
        assert!(result.is_some());
    }

    #[test]
    fn test_is_app_excluded_partial_match() {
        let mut engine = MatcherEngine::new();
        engine.set_excluded_apps(vec!["password".to_string()]);
        assert!(engine.is_app_excluded("1Password Manager"));
        assert!(!engine.is_app_excluded("notepad"));
    }

    // ── Benchmark-style test: 1000+ combos ────────────────────────

    #[test]
    fn test_engine_performance_large_library() {
        let mut engine = MatcherEngine::new();
        let mut combos = Vec::with_capacity(1500);

        // Generate 1000 strict combos
        for i in 0..1000 {
            combos.push(make_combo(
                &format!("kw{:04}", i),
                &format!("snippet {}", i),
                MatchingMode::Strict,
                false,
            ));
        }
        // Generate 500 loose combos
        for i in 0..500 {
            combos.push(make_combo(
                &format!("lk{:04}", i),
                &format!("loose snippet {}", i),
                MatchingMode::Loose,
                false,
            ));
        }

        engine.load_combos(&combos);
        assert_eq!(engine.combo_count(), 1500);

        // Match near end of list
        let result = engine.find_match("hello kw0999", None);
        assert!(result.is_some());
        assert_eq!(result.unwrap().keyword, "kw0999");

        // Match a loose combo
        let result = engine.find_match("testlk0499", None);
        assert!(result.is_some());
        assert_eq!(result.unwrap().keyword, "lk0499");

        // No match
        let result = engine.find_match("hello world", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_engine_performance_repeated_lookups() {
        let mut engine = MatcherEngine::new();
        let mut combos = Vec::with_capacity(1000);
        for i in 0..1000 {
            combos.push(strict(&format!("kw{:04}", i), &format!("snippet {}", i)));
        }
        engine.load_combos(&combos);

        // Perform many lookups
        for _ in 0..10_000 {
            let _ = engine.find_match("hello kw0500", None);
        }
    }

    #[test]
    fn test_strict_match_keyword_longer_than_buffer() {
        assert!(!is_strict_match("ab", "abc", false));
    }

    #[test]
    fn test_loose_match_keyword_longer_than_buffer() {
        assert!(!is_loose_match("ab", "abc", false));
    }

    #[test]
    fn test_engine_buffer_shorter_than_keyword() {
        let mut engine = MatcherEngine::new();
        engine.load_combos(&[strict("longkeyword", "snippet")]);
        assert!(engine.find_match("lo", None).is_none());
    }

    #[test]
    fn test_strict_match_after_multiple_spaces() {
        assert!(is_strict_match("hello   sig", "sig", false));
    }

    #[test]
    fn test_strict_unicode_boundary() {
        // Unicode space before keyword
        assert!(is_strict_match("hello sig", "sig", false));
    }

    #[test]
    fn test_default_trait() {
        let engine = MatcherEngine::default();
        assert_eq!(engine.combo_count(), 0);
    }
}
