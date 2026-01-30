use serde::{Deserialize, Serialize};

/// Defines how keyword matching is performed against typed text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MatchingMode {
    /// Exact match after a word boundary (space, punctuation, start of line).
    Strict,
    /// Ends-with matching — triggers even mid-word.
    Loose,
}

impl Default for MatchingMode {
    fn default() -> Self {
        Self::Strict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_mode_default_is_strict() {
        assert_eq!(MatchingMode::default(), MatchingMode::Strict);
    }

    #[test]
    fn test_matching_mode_serialization_roundtrip() {
        let modes = [MatchingMode::Strict, MatchingMode::Loose];
        for mode in &modes {
            let json = serde_json::to_string(mode).expect("serialize");
            let deserialized: MatchingMode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*mode, deserialized);
        }
    }

    #[test]
    fn test_matching_mode_json_camel_case() {
        let json = serde_json::to_string(&MatchingMode::Strict).expect("serialize");
        assert_eq!(json, "\"strict\"");
        let json = serde_json::to_string(&MatchingMode::Loose).expect("serialize");
        assert_eq!(json, "\"loose\"");
    }

    #[test]
    fn test_matching_mode_clone() {
        let mode = MatchingMode::Loose;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_matching_mode_debug() {
        let debug_str = format!("{:?}", MatchingMode::Strict);
        assert!(debug_str.contains("Strict"));
    }
}
