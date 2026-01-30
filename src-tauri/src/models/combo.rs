use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::matching::MatchingMode;

/// Errors arising from combo validation.
#[derive(Debug, Error, PartialEq)]
pub enum ComboValidationError {
    #[error("Keyword must not be empty")]
    EmptyKeyword,
    #[error("Keyword must be at least 2 characters, got {0}")]
    KeywordTooShort(usize),
    #[error("Keyword must not contain spaces")]
    KeywordContainsSpaces,
    #[error("Snippet must not be empty")]
    EmptySnippet,
}

/// A combo maps a typed keyword to an expanded text snippet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Combo {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub keyword: String,
    pub snippet: String,
    pub group_id: Uuid,
    pub matching_mode: MatchingMode,
    pub case_sensitive: bool,
    pub enabled: bool,
    pub use_count: u64,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl Combo {
    /// Validates this combo's keyword and snippet fields.
    pub fn validate(&self) -> Result<(), ComboValidationError> {
        if self.keyword.is_empty() {
            return Err(ComboValidationError::EmptyKeyword);
        }
        if self.keyword.len() < 2 {
            return Err(ComboValidationError::KeywordTooShort(self.keyword.len()));
        }
        if self.keyword.contains(' ') {
            return Err(ComboValidationError::KeywordContainsSpaces);
        }
        if self.snippet.is_empty() {
            return Err(ComboValidationError::EmptySnippet);
        }
        Ok(())
    }
}

/// Builder for constructing `Combo` instances incrementally.
#[derive(Debug, Default)]
pub struct ComboBuilder {
    name: Option<String>,
    description: Option<String>,
    keyword: Option<String>,
    snippet: Option<String>,
    group_id: Option<Uuid>,
    matching_mode: Option<MatchingMode>,
    case_sensitive: Option<bool>,
    enabled: Option<bool>,
}

impl ComboBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keyword = Some(keyword.into());
        self
    }

    pub fn snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    pub fn group_id(mut self, group_id: Uuid) -> Self {
        self.group_id = Some(group_id);
        self
    }

    pub fn matching_mode(mut self, mode: MatchingMode) -> Self {
        self.matching_mode = Some(mode);
        self
    }

    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = Some(case_sensitive);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Builds the `Combo`, returning a validation error if the keyword or snippet
    /// are invalid.
    pub fn build(self) -> Result<Combo, ComboValidationError> {
        let now = Utc::now();
        let combo = Combo {
            id: Uuid::new_v4(),
            name: self.name.unwrap_or_default(),
            description: self.description.unwrap_or_default(),
            keyword: self.keyword.unwrap_or_default(),
            snippet: self.snippet.unwrap_or_default(),
            group_id: self.group_id.unwrap_or_else(Uuid::new_v4),
            matching_mode: self.matching_mode.unwrap_or_default(),
            case_sensitive: self.case_sensitive.unwrap_or(false),
            enabled: self.enabled.unwrap_or(true),
            use_count: 0,
            last_used: None,
            created_at: now,
            modified_at: now,
        };
        combo.validate()?;
        Ok(combo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ComboBuilder tests ──────────────────────────────────────────

    #[test]
    fn test_builder_creates_valid_combo() {
        let group_id = Uuid::new_v4();
        let combo = ComboBuilder::new()
            .name("Signature")
            .keyword("sig")
            .snippet("Best regards,\nJohn")
            .group_id(group_id)
            .build()
            .expect("should build");

        assert_eq!(combo.name, "Signature");
        assert_eq!(combo.keyword, "sig");
        assert_eq!(combo.snippet, "Best regards,\nJohn");
        assert_eq!(combo.group_id, group_id);
        assert!(combo.enabled);
        assert!(!combo.case_sensitive);
        assert_eq!(combo.matching_mode, MatchingMode::Strict);
        assert_eq!(combo.use_count, 0);
        assert!(combo.last_used.is_none());
    }

    #[test]
    fn test_builder_generates_uuid() {
        let c1 = ComboBuilder::new()
            .keyword("aa")
            .snippet("text")
            .build()
            .unwrap();
        let c2 = ComboBuilder::new()
            .keyword("bb")
            .snippet("text")
            .build()
            .unwrap();
        assert_ne!(c1.id, c2.id);
        assert!(!c1.id.is_nil());
    }

    #[test]
    fn test_builder_with_all_fields() {
        let combo = ComboBuilder::new()
            .name("Full")
            .description("A full combo")
            .keyword("full")
            .snippet("full text")
            .matching_mode(MatchingMode::Loose)
            .case_sensitive(true)
            .enabled(false)
            .build()
            .unwrap();

        assert_eq!(combo.description, "A full combo");
        assert_eq!(combo.matching_mode, MatchingMode::Loose);
        assert!(combo.case_sensitive);
        assert!(!combo.enabled);
    }

    #[test]
    fn test_builder_fails_empty_keyword() {
        let result = ComboBuilder::new().snippet("text").build();
        assert_eq!(result, Err(ComboValidationError::EmptyKeyword));
    }

    #[test]
    fn test_builder_fails_short_keyword() {
        let result = ComboBuilder::new().keyword("x").snippet("text").build();
        assert_eq!(result, Err(ComboValidationError::KeywordTooShort(1)));
    }

    #[test]
    fn test_builder_fails_keyword_with_spaces() {
        let result = ComboBuilder::new()
            .keyword("my key")
            .snippet("text")
            .build();
        assert_eq!(result, Err(ComboValidationError::KeywordContainsSpaces));
    }

    #[test]
    fn test_builder_fails_empty_snippet() {
        let result = ComboBuilder::new().keyword("sig").build();
        assert_eq!(result, Err(ComboValidationError::EmptySnippet));
    }

    // ── Validation tests ────────────────────────────────────────────

    #[test]
    fn test_validate_valid_combo() {
        let combo = ComboBuilder::new()
            .keyword("sig")
            .snippet("Regards")
            .build()
            .unwrap();
        assert!(combo.validate().is_ok());
    }

    #[test]
    fn test_validate_two_char_keyword() {
        let combo = ComboBuilder::new()
            .keyword("ab")
            .snippet("text")
            .build();
        assert!(combo.is_ok());
    }

    // ── Serialization tests ─────────────────────────────────────────

    #[test]
    fn test_combo_serialization_roundtrip() {
        let combo = ComboBuilder::new()
            .name("Test")
            .keyword("test")
            .snippet("Hello")
            .build()
            .unwrap();
        let json = serde_json::to_string(&combo).expect("serialize");
        let deserialized: Combo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(combo, deserialized);
    }

    #[test]
    fn test_combo_json_uses_camel_case() {
        let combo = ComboBuilder::new()
            .keyword("ck")
            .snippet("s")
            // We need to bypass validation for this test — build a combo directly.
            .build();
        // build will fail because snippet "s" is valid and keyword "ck" is 2 chars, so it succeeds
        let combo = combo.unwrap();
        let json = serde_json::to_string(&combo).expect("serialize");
        assert!(json.contains("groupId"));
        assert!(json.contains("matchingMode"));
        assert!(json.contains("caseSensitive"));
        assert!(json.contains("useCount"));
        assert!(json.contains("lastUsed"));
        assert!(json.contains("createdAt"));
        assert!(json.contains("modifiedAt"));
        // Must NOT contain snake_case
        assert!(!json.contains("group_id"));
        assert!(!json.contains("matching_mode"));
    }

    #[test]
    fn test_combo_clone() {
        let combo = ComboBuilder::new()
            .keyword("cl")
            .snippet("clone")
            .build()
            .unwrap();
        let cloned = combo.clone();
        assert_eq!(combo, cloned);
    }

    #[test]
    fn test_combo_debug_format() {
        let combo = ComboBuilder::new()
            .keyword("db")
            .snippet("debug")
            .build()
            .unwrap();
        let debug = format!("{:?}", combo);
        assert!(debug.contains("Combo"));
    }

    #[test]
    fn test_combo_timestamps_are_set() {
        let before = Utc::now();
        let combo = ComboBuilder::new()
            .keyword("ts")
            .snippet("time")
            .build()
            .unwrap();
        let after = Utc::now();
        assert!(combo.created_at >= before && combo.created_at <= after);
        assert_eq!(combo.created_at, combo.modified_at);
    }
}
