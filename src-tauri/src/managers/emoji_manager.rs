//! Emoji shortcode expansion and lookup.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from emoji operations.
#[derive(Debug, Error)]
pub enum EmojiError {
    #[error("Failed to parse emoji database: {0}")]
    ParseError(#[from] serde_json::Error),
}

/// A single emoji entry with shortcode and aliases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmojiEntry {
    pub shortcode: String,
    pub emoji: String,
    pub aliases: Vec<String>,
}

/// Manages emoji lookup and expansion.
pub struct EmojiManager {
    entries: Vec<EmojiEntry>,
    /// Maps shortcode/alias -> index in entries
    index: HashMap<String, usize>,
    enabled: bool,
}

impl EmojiManager {
    /// Creates an empty manager with built-in emoji set.
    pub fn new() -> Self {
        let mut mgr = Self {
            entries: Vec::new(),
            index: HashMap::new(),
            enabled: true,
        };
        mgr.load_builtin();
        mgr
    }

    /// Parses emoji entries from a JSON array string.
    pub fn load_from_json(json: &str) -> Result<Self, EmojiError> {
        let entries: Vec<EmojiEntry> = serde_json::from_str(json)?;
        let mut mgr = Self {
            entries: Vec::new(),
            index: HashMap::new(),
            enabled: true,
        };
        for entry in entries {
            mgr.add_entry(entry);
        }
        Ok(mgr)
    }

    /// Looks up an emoji by shortcode. Returns the emoji character(s).
    pub fn lookup(&self, shortcode: &str) -> Option<&str> {
        self.index
            .get(shortcode)
            .map(|&idx| self.entries[idx].emoji.as_str())
    }

    /// Expands `|shortcode|` patterns in text with their emoji equivalents.
    pub fn expand_emojis(&self, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }

        let mut result = String::with_capacity(text.len());
        let mut chars = text.char_indices().peekable();

        while let Some((i, ch)) = chars.next() {
            if ch == '|' {
                // Look for closing |
                let rest = &text[i + 1..];
                if let Some(end) = rest.find('|') {
                    let shortcode = &rest[..end];
                    if let Some(emoji) = self.lookup(shortcode) {
                        result.push_str(emoji);
                        // Skip past the closing |
                        for _ in 0..end + 1 {
                            chars.next();
                        }
                        continue;
                    }
                }
                result.push(ch);
            } else {
                result.push(ch);
            }
        }
        result
    }

    /// Searches entries by shortcode or alias prefix.
    pub fn search(&self, query: &str) -> Vec<&EmojiEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.shortcode.contains(&query_lower)
                    || e.aliases.iter().any(|a| a.contains(&query_lower))
            })
            .collect()
    }

    /// Returns whether emoji expansion is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables or disables emoji expansion.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn add_entry(&mut self, entry: EmojiEntry) {
        let idx = self.entries.len();
        self.index.insert(entry.shortcode.clone(), idx);
        for alias in &entry.aliases {
            self.index.insert(alias.clone(), idx);
        }
        self.entries.push(entry);
    }

    fn load_builtin(&mut self) {
        let builtins = vec![
            ("smile", "\u{1F604}", vec!["happy", "joy"]),
            ("heart", "\u{2764}\u{FE0F}", vec!["love"]),
            ("thumbsup", "\u{1F44D}", vec!["ok", "+1"]),
            ("thumbsdown", "\u{1F44E}", vec!["-1"]),
            ("fire", "\u{1F525}", vec!["hot", "lit"]),
            ("star", "\u{2B50}", vec![]),
            ("check", "\u{2705}", vec!["yes", "done"]),
            ("x", "\u{274C}", vec!["no", "cross"]),
            ("wave", "\u{1F44B}", vec!["hi", "hello"]),
            ("clap", "\u{1F44F}", vec!["applause"]),
            ("rocket", "\u{1F680}", vec!["launch"]),
            ("eyes", "\u{1F440}", vec!["look"]),
            ("thinking", "\u{1F914}", vec!["hmm"]),
            ("laugh", "\u{1F602}", vec!["lol", "rofl"]),
            ("cry", "\u{1F622}", vec!["sad", "tear"]),
            ("angry", "\u{1F620}", vec!["mad"]),
            ("sunglasses", "\u{1F60E}", vec!["cool"]),
            ("party", "\u{1F389}", vec!["celebrate", "tada"]),
            ("warning", "\u{26A0}\u{FE0F}", vec!["alert"]),
            ("bug", "\u{1F41B}", vec!["insect"]),
            ("sparkles", "\u{2728}", vec!["magic"]),
            ("pin", "\u{1F4CC}", vec!["pushpin"]),
            ("coffee", "\u{2615}", vec!["cafe"]),
            ("100", "\u{1F4AF}", vec!["hundred", "perfect"]),
        ];

        for (shortcode, emoji, aliases) in builtins {
            self.add_entry(EmojiEntry {
                shortcode: shortcode.to_string(),
                emoji: emoji.to_string(),
                aliases: aliases.into_iter().map(String::from).collect(),
            });
        }
    }
}

impl Default for EmojiManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_builtins() {
        let mgr = EmojiManager::new();
        assert!(mgr.lookup("smile").is_some());
        assert!(mgr.lookup("heart").is_some());
        assert!(mgr.lookup("thumbsup").is_some());
    }

    #[test]
    fn test_lookup_returns_emoji() {
        let mgr = EmojiManager::new();
        assert_eq!(mgr.lookup("smile"), Some("\u{1F604}"));
    }

    #[test]
    fn test_lookup_alias() {
        let mgr = EmojiManager::new();
        assert_eq!(mgr.lookup("happy"), mgr.lookup("smile"));
    }

    #[test]
    fn test_lookup_missing() {
        let mgr = EmojiManager::new();
        assert!(mgr.lookup("nonexistent").is_none());
    }

    #[test]
    fn test_expand_emojis_simple() {
        let mgr = EmojiManager::new();
        let result = mgr.expand_emojis("Hello |smile| world");
        assert!(result.contains("\u{1F604}"));
        assert!(!result.contains("|smile|"));
    }

    #[test]
    fn test_expand_emojis_multiple() {
        let mgr = EmojiManager::new();
        let result = mgr.expand_emojis("|heart| and |fire|");
        assert!(result.contains("\u{2764}"));
        assert!(result.contains("\u{1F525}"));
    }

    #[test]
    fn test_expand_emojis_unknown_shortcode_unchanged() {
        let mgr = EmojiManager::new();
        let result = mgr.expand_emojis("Hello |unknown| world");
        assert_eq!(result, "Hello |unknown| world");
    }

    #[test]
    fn test_expand_emojis_no_patterns() {
        let mgr = EmojiManager::new();
        let text = "No emoji here";
        assert_eq!(mgr.expand_emojis(text), text);
    }

    #[test]
    fn test_expand_emojis_disabled() {
        let mut mgr = EmojiManager::new();
        mgr.set_enabled(false);
        let text = "Hello |smile| world";
        assert_eq!(mgr.expand_emojis(text), text);
    }

    #[test]
    fn test_search_by_shortcode() {
        let mgr = EmojiManager::new();
        let results = mgr.search("smi");
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.shortcode == "smile"));
    }

    #[test]
    fn test_search_by_alias() {
        let mgr = EmojiManager::new();
        let results = mgr.search("lol");
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.shortcode == "laugh"));
    }

    #[test]
    fn test_search_no_results() {
        let mgr = EmojiManager::new();
        let results = mgr.search("zzzzzzz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_enabled_default() {
        let mgr = EmojiManager::new();
        assert!(mgr.is_enabled());
    }

    #[test]
    fn test_set_enabled() {
        let mut mgr = EmojiManager::new();
        mgr.set_enabled(false);
        assert!(!mgr.is_enabled());
        mgr.set_enabled(true);
        assert!(mgr.is_enabled());
    }

    #[test]
    fn test_load_from_json() {
        let json = r#"[
            {"shortcode": "test", "emoji": "T", "aliases": ["t"]},
            {"shortcode": "foo", "emoji": "F", "aliases": []}
        ]"#;
        let mgr = EmojiManager::load_from_json(json).unwrap();
        assert_eq!(mgr.lookup("test"), Some("T"));
        assert_eq!(mgr.lookup("t"), Some("T"));
        assert_eq!(mgr.lookup("foo"), Some("F"));
    }

    #[test]
    fn test_load_from_json_invalid() {
        let result = EmojiManager::load_from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_emoji_entry_serialization() {
        let entry = EmojiEntry {
            shortcode: "test".to_string(),
            emoji: "T".to_string(),
            aliases: vec!["t".to_string()],
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: EmojiEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deser);
    }

    #[test]
    fn test_expand_with_alias() {
        let mgr = EmojiManager::new();
        let result = mgr.expand_emojis("Good job |+1|");
        assert!(result.contains("\u{1F44D}"));
    }

    #[test]
    fn test_builtin_count() {
        let mgr = EmojiManager::new();
        // We defined 24 builtins
        assert!(mgr.entries.len() >= 20);
    }
}
