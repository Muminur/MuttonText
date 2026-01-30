use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A named collection of combos. Groups allow users to organize
/// their text snippets by category, project, or context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl Group {
    /// Creates a new group with the given name and a generated UUID.
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            enabled: true,
            created_at: now,
            modified_at: now,
        }
    }

    /// Creates a new group with a description.
    pub fn with_description(name: impl Into<String>, description: impl Into<String>) -> Self {
        let mut group = Self::new(name);
        group.description = description.into();
        group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_new_has_uuid() {
        let group = Group::new("Test Group");
        assert!(!group.id.is_nil());
    }

    #[test]
    fn test_group_new_sets_name() {
        let group = Group::new("My Group");
        assert_eq!(group.name, "My Group");
    }

    #[test]
    fn test_group_new_enabled_by_default() {
        let group = Group::new("Test");
        assert!(group.enabled);
    }

    #[test]
    fn test_group_new_empty_description() {
        let group = Group::new("Test");
        assert!(group.description.is_empty());
    }

    #[test]
    fn test_group_with_description() {
        let group = Group::with_description("Dev", "Development snippets");
        assert_eq!(group.description, "Development snippets");
    }

    #[test]
    fn test_group_timestamps_set() {
        let before = Utc::now();
        let group = Group::new("Test");
        let after = Utc::now();
        assert!(group.created_at >= before && group.created_at <= after);
        assert_eq!(group.created_at, group.modified_at);
    }

    #[test]
    fn test_group_serialization_roundtrip() {
        let group = Group::new("Roundtrip");
        let json = serde_json::to_string(&group).expect("serialize");
        let deserialized: Group = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(group, deserialized);
    }

    #[test]
    fn test_group_json_uses_camel_case() {
        let group = Group::new("Test");
        let json = serde_json::to_string(&group).expect("serialize");
        assert!(json.contains("createdAt"));
        assert!(json.contains("modifiedAt"));
        assert!(!json.contains("created_at"));
    }

    #[test]
    fn test_group_unique_ids() {
        let g1 = Group::new("A");
        let g2 = Group::new("B");
        assert_ne!(g1.id, g2.id);
    }

    #[test]
    fn test_group_clone() {
        let group = Group::new("Clone");
        let cloned = group.clone();
        assert_eq!(group, cloned);
    }
}
