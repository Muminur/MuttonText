use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::combo::Combo;
use super::group::Group;

/// The top-level container for all groups and combos.
/// Persisted as `combos.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComboLibrary {
    pub version: String,
    pub groups: Vec<Group>,
    pub combos: Vec<Combo>,
}

impl ComboLibrary {
    /// Creates a new empty library with the given schema version.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            groups: Vec::new(),
            combos: Vec::new(),
        }
    }

    /// Adds a group to the library.
    pub fn add_group(&mut self, group: Group) {
        self.groups.push(group);
    }

    /// Adds a combo to the library.
    pub fn add_combo(&mut self, combo: Combo) {
        self.combos.push(combo);
    }

    /// Removes a combo by its ID. Returns `true` if a combo was removed.
    pub fn remove_combo(&mut self, combo_id: Uuid) -> bool {
        let before = self.combos.len();
        self.combos.retain(|c| c.id != combo_id);
        self.combos.len() < before
    }

    /// Returns all combos belonging to the given group.
    pub fn get_combos_by_group(&self, group_id: Uuid) -> Vec<&Combo> {
        self.combos.iter().filter(|c| c.group_id == group_id).collect()
    }

    /// Finds the first combo whose keyword matches exactly (case-sensitive).
    pub fn find_combo_by_keyword(&self, keyword: &str) -> Option<&Combo> {
        self.combos.iter().find(|c| c.keyword == keyword)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::combo::ComboBuilder;

    fn make_combo(keyword: &str, snippet: &str, group_id: Uuid) -> Combo {
        ComboBuilder::new()
            .keyword(keyword)
            .snippet(snippet)
            .group_id(group_id)
            .build()
            .unwrap()
    }

    #[test]
    fn test_library_new() {
        let lib = ComboLibrary::new("1.0");
        assert_eq!(lib.version, "1.0");
        assert!(lib.groups.is_empty());
        assert!(lib.combos.is_empty());
    }

    #[test]
    fn test_add_group() {
        let mut lib = ComboLibrary::new("1.0");
        lib.add_group(Group::new("General"));
        assert_eq!(lib.groups.len(), 1);
        assert_eq!(lib.groups[0].name, "General");
    }

    #[test]
    fn test_add_combo() {
        let mut lib = ComboLibrary::new("1.0");
        let group = Group::new("G");
        let combo = make_combo("sig", "Regards", group.id);
        lib.add_group(group);
        lib.add_combo(combo);
        assert_eq!(lib.combos.len(), 1);
    }

    #[test]
    fn test_remove_combo_existing() {
        let mut lib = ComboLibrary::new("1.0");
        let group = Group::new("G");
        let combo = make_combo("rm", "remove", group.id);
        let combo_id = combo.id;
        lib.add_group(group);
        lib.add_combo(combo);
        assert!(lib.remove_combo(combo_id));
        assert!(lib.combos.is_empty());
    }

    #[test]
    fn test_remove_combo_nonexistent() {
        let mut lib = ComboLibrary::new("1.0");
        assert!(!lib.remove_combo(Uuid::new_v4()));
    }

    #[test]
    fn test_get_combos_by_group() {
        let mut lib = ComboLibrary::new("1.0");
        let g1 = Group::new("G1");
        let g2 = Group::new("G2");
        let g1_id = g1.id;
        let g2_id = g2.id;
        lib.add_group(g1);
        lib.add_group(g2);
        lib.add_combo(make_combo("aa", "text", g1_id));
        lib.add_combo(make_combo("bb", "text", g1_id));
        lib.add_combo(make_combo("cc", "text", g2_id));

        let g1_combos = lib.get_combos_by_group(g1_id);
        assert_eq!(g1_combos.len(), 2);
        let g2_combos = lib.get_combos_by_group(g2_id);
        assert_eq!(g2_combos.len(), 1);
    }

    #[test]
    fn test_get_combos_by_group_empty() {
        let lib = ComboLibrary::new("1.0");
        let result = lib.get_combos_by_group(Uuid::new_v4());
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_combo_by_keyword_found() {
        let mut lib = ComboLibrary::new("1.0");
        let group = Group::new("G");
        let gid = group.id;
        lib.add_group(group);
        lib.add_combo(make_combo("sig", "Signature", gid));
        lib.add_combo(make_combo("addr", "Address", gid));

        let found = lib.find_combo_by_keyword("sig");
        assert!(found.is_some());
        assert_eq!(found.unwrap().keyword, "sig");
    }

    #[test]
    fn test_find_combo_by_keyword_not_found() {
        let lib = ComboLibrary::new("1.0");
        assert!(lib.find_combo_by_keyword("nope").is_none());
    }

    #[test]
    fn test_library_serialization_roundtrip() {
        let mut lib = ComboLibrary::new("1.0");
        let group = Group::new("General");
        let gid = group.id;
        lib.add_group(group);
        lib.add_combo(make_combo("hi", "Hello!", gid));

        let json = serde_json::to_string(&lib).expect("serialize");
        let deserialized: ComboLibrary = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(lib, deserialized);
    }

    #[test]
    fn test_library_json_uses_camel_case() {
        let lib = ComboLibrary::new("1.0");
        let json = serde_json::to_string(&lib).expect("serialize");
        // Top-level fields are already single words, check nested once combos added
        assert!(json.contains("version"));
        assert!(json.contains("groups"));
        assert!(json.contains("combos"));
    }

    #[test]
    fn test_remove_combo_preserves_others() {
        let mut lib = ComboLibrary::new("1.0");
        let group = Group::new("G");
        let gid = group.id;
        lib.add_group(group);
        let c1 = make_combo("aa", "first", gid);
        let c2 = make_combo("bb", "second", gid);
        let c1_id = c1.id;
        lib.add_combo(c1);
        lib.add_combo(c2);
        lib.remove_combo(c1_id);
        assert_eq!(lib.combos.len(), 1);
        assert_eq!(lib.combos[0].keyword, "bb");
    }
}
