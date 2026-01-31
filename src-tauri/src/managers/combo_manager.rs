//! Business logic for managing combos and groups.
//!
//! `ComboManager` wraps a `ComboLibrary` and provides CRUD operations
//! for combos and groups, with persistence via `ComboStorage`.

use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

use crate::managers::combo_storage::ComboStorage;
use crate::managers::storage::StorageError;
use crate::models::combo::{Combo, ComboBuilder, ComboValidationError};
use crate::models::group::Group;
use crate::models::library::ComboLibrary;
use crate::models::matching::MatchingMode;

/// Errors that may occur during combo/group management operations.
#[derive(Debug, Error)]
pub enum ComboManagerError {
    #[error("Combo not found: {0}")]
    ComboNotFound(Uuid),
    #[error("Group not found: {0}")]
    GroupNotFound(Uuid),
    #[error("Validation error: {0}")]
    Validation(#[from] ComboValidationError),
    #[error("{0}")]
    ValidationMessage(String),
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

/// Manages the in-memory combo library and persists changes to disk.
pub struct ComboManager {
    library: ComboLibrary,
    storage: ComboStorage,
}

impl ComboManager {
    /// Creates a new `ComboManager` by loading the library from the given storage.
    pub fn new(storage: ComboStorage) -> Result<Self, ComboManagerError> {
        let library = storage.load()?;
        let mut mgr = Self { library, storage };
        mgr.ensure_default_group()?;
        Ok(mgr)
    }

    /// Creates a `ComboManager` with the given library and storage (useful for testing).
    pub fn with_library(library: ComboLibrary, storage: ComboStorage) -> Self {
        Self { library, storage }
    }

    // ── Combo operations ────────────────────────────────────────────

    /// Returns all combos.
    pub fn get_all_combos(&self) -> Vec<Combo> {
        self.library.combos.clone()
    }

    /// Returns a combo by ID, or `None` if not found.
    pub fn get_combo(&self, id: Uuid) -> Option<Combo> {
        self.library.combos.iter().find(|c| c.id == id).cloned()
    }

    /// Creates a new combo and persists the library.
    pub fn create_combo(
        &mut self,
        name: String,
        keyword: String,
        snippet: String,
        group_id: Uuid,
        matching_mode: MatchingMode,
        case_sensitive: bool,
    ) -> Result<Combo, ComboManagerError> {
        if !self.library.groups.iter().any(|g| g.id == group_id) {
            return Err(ComboManagerError::GroupNotFound(group_id));
        }

        let combo = ComboBuilder::new()
            .name(name)
            .keyword(keyword)
            .snippet(snippet)
            .group_id(group_id)
            .matching_mode(matching_mode)
            .case_sensitive(case_sensitive)
            .build()?;

        self.library.add_combo(combo.clone());
        self.persist()?;
        Ok(combo)
    }

    /// Updates an existing combo. Only provided fields are changed.
    pub fn update_combo(
        &mut self,
        id: Uuid,
        name: Option<String>,
        keyword: Option<String>,
        snippet: Option<String>,
        group_id: Option<Uuid>,
        matching_mode: Option<MatchingMode>,
        case_sensitive: Option<bool>,
        enabled: Option<bool>,
    ) -> Result<Combo, ComboManagerError> {
        // Check group exists before mutating
        if let Some(gid) = group_id {
            if !self.library.groups.iter().any(|g| g.id == gid) {
                return Err(ComboManagerError::GroupNotFound(gid));
            }
        }

        let combo = self
            .library
            .combos
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(ComboManagerError::ComboNotFound(id))?;

        if let Some(name) = name {
            combo.name = name;
        }
        if let Some(keyword) = keyword {
            combo.keyword = keyword;
        }
        if let Some(snippet) = snippet {
            combo.snippet = snippet;
        }
        if let Some(gid) = group_id {
            combo.group_id = gid;
        }
        if let Some(mode) = matching_mode {
            combo.matching_mode = mode;
        }
        if let Some(cs) = case_sensitive {
            combo.case_sensitive = cs;
        }
        if let Some(en) = enabled {
            combo.enabled = en;
        }
        combo.modified_at = Utc::now();

        combo.validate()?;

        let updated = combo.clone();
        self.persist()?;
        Ok(updated)
    }

    /// Deletes a combo by ID.
    pub fn delete_combo(&mut self, id: Uuid) -> Result<(), ComboManagerError> {
        if !self.library.remove_combo(id) {
            return Err(ComboManagerError::ComboNotFound(id));
        }
        self.persist()?;
        Ok(())
    }

    /// Duplicates a combo, giving the copy a new ID and appended name.
    pub fn duplicate_combo(&mut self, id: Uuid) -> Result<Combo, ComboManagerError> {
        let original = self
            .library
            .combos
            .iter()
            .find(|c| c.id == id)
            .ok_or(ComboManagerError::ComboNotFound(id))?
            .clone();

        let now = Utc::now();
        let mut duplicate = original;
        duplicate.id = Uuid::new_v4();
        duplicate.name = format!("{} (copy)", duplicate.name);
        duplicate.use_count = 0;
        duplicate.last_used = None;
        duplicate.created_at = now;
        duplicate.modified_at = now;

        self.library.add_combo(duplicate.clone());
        self.persist()?;
        Ok(duplicate)
    }

    /// Moves a combo to a different group.
    pub fn move_combo_to_group(
        &mut self,
        combo_id: Uuid,
        group_id: Uuid,
    ) -> Result<(), ComboManagerError> {
        if !self.library.groups.iter().any(|g| g.id == group_id) {
            return Err(ComboManagerError::GroupNotFound(group_id));
        }

        let combo = self
            .library
            .combos
            .iter_mut()
            .find(|c| c.id == combo_id)
            .ok_or(ComboManagerError::ComboNotFound(combo_id))?;

        combo.group_id = group_id;
        combo.modified_at = Utc::now();

        self.persist()?;
        Ok(())
    }

    /// Toggles a combo's enabled state and returns the new state.
    pub fn toggle_combo(&mut self, id: Uuid) -> Result<bool, ComboManagerError> {
        let combo = self
            .library
            .combos
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(ComboManagerError::ComboNotFound(id))?;

        combo.enabled = !combo.enabled;
        combo.modified_at = Utc::now();
        let new_state = combo.enabled;

        self.persist()?;
        Ok(new_state)
    }

    // ── Group operations ────────────────────────────────────────────

    /// Returns all groups.
    pub fn get_all_groups(&self) -> Vec<Group> {
        self.library.groups.clone()
    }

    /// Returns a group by ID.
    pub fn get_group(&self, id: Uuid) -> Option<Group> {
        self.library.groups.iter().find(|g| g.id == id).cloned()
    }

    /// Creates a new group.
    pub fn create_group(
        &mut self,
        name: String,
        description: String,
    ) -> Result<Group, ComboManagerError> {
        let group = Group::with_description(name, description);
        self.library.add_group(group.clone());
        self.persist()?;
        Ok(group)
    }

    /// Updates an existing group.
    pub fn update_group(
        &mut self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Group, ComboManagerError> {
        let group = self
            .library
            .groups
            .iter_mut()
            .find(|g| g.id == id)
            .ok_or(ComboManagerError::GroupNotFound(id))?;

        if let Some(name) = name {
            group.name = name;
        }
        if let Some(desc) = description {
            group.description = desc;
        }
        group.modified_at = Utc::now();

        let updated = group.clone();
        self.persist()?;
        Ok(updated)
    }

    /// Deletes a group and moves its combos to the default group.
    /// The default group itself cannot be deleted.
    pub fn delete_group(&mut self, id: Uuid) -> Result<(), ComboManagerError> {
        // Prevent deleting default group
        let group = self.library.groups.iter().find(|g| g.id == id)
            .ok_or(ComboManagerError::GroupNotFound(id))?;
        if group.name == "Default" {
            return Err(ComboManagerError::ValidationMessage(
                "Cannot delete the default group".to_string(),
            ));
        }

        // Ensure default group exists and get its ID
        let default_group = self.ensure_default_group()?;
        let default_id = default_group.id;

        // Move combos to default group
        for combo in self.library.combos.iter_mut() {
            if combo.group_id == id {
                combo.group_id = default_id;
            }
        }

        // Remove the group
        self.library.groups.retain(|g| g.id != id);
        self.persist()?;
        Ok(())
    }

    /// Toggles a group's enabled state. Also toggles all combos in the group.
    pub fn toggle_group(&mut self, id: Uuid) -> Result<bool, ComboManagerError> {
        let group = self
            .library
            .groups
            .iter_mut()
            .find(|g| g.id == id)
            .ok_or(ComboManagerError::GroupNotFound(id))?;

        group.enabled = !group.enabled;
        group.modified_at = Utc::now();
        let new_state = group.enabled;

        for combo in self.library.combos.iter_mut().filter(|c| c.group_id == id) {
            combo.enabled = new_state;
            combo.modified_at = Utc::now();
        }

        self.persist()?;
        Ok(new_state)
    }

    // ── Utility ────────────────────────────────────────────────────

    /// Check if a keyword is unique across all combos.
    /// Returns true if the keyword is unique (no duplicates found).
    /// `exclude_id` allows excluding a specific combo (for update operations).
    pub fn check_keyword_uniqueness(&self, keyword: &str, exclude_id: Option<Uuid>) -> bool {
        !self.library.combos.iter().any(|c| {
            c.keyword == keyword && exclude_id.map_or(true, |id| c.id != id)
        })
    }

    /// Ensures a "Default" group exists. Creates one if none exists.
    /// Returns the default group.
    pub fn ensure_default_group(&mut self) -> Result<Group, ComboManagerError> {
        if let Some(group) = self.library.groups.iter().find(|g| g.name == "Default") {
            return Ok(group.clone());
        }
        let group = Group::new("Default".to_string());
        self.library.add_group(group.clone());
        self.storage.save(&self.library)?;
        Ok(group)
    }

    // ── Internal ────────────────────────────────────────────────────

    /// Persists the current library state to disk.
    fn persist(&self) -> Result<(), ComboManagerError> {
        self.storage.save(&self.library)?;
        Ok(())
    }

    /// Shrinks internal collections to fit their contents, releasing unused
    /// allocated memory (MT-1110).
    pub fn compact(&mut self) {
        self.library.combos.shrink_to_fit();
        self.library.groups.shrink_to_fit();
        tracing::debug!(
            "ComboManager compacted: {} combos, {} groups",
            self.library.combos.len(),
            self.library.groups.len(),
        );
    }

    /// Returns an approximate estimate of memory usage in bytes (MT-1110).
    ///
    /// This is a rough estimate based on the sizes of stored strings and
    /// struct overhead. It does not account for allocator overhead or
    /// alignment padding.
    pub fn memory_usage_estimate(&self) -> usize {
        let combo_size: usize = self
            .library
            .combos
            .iter()
            .map(|c| {
                std::mem::size_of::<Combo>()
                    + c.keyword.capacity()
                    + c.snippet.capacity()
                    + c.name.capacity()
                    + c.description.capacity()
            })
            .sum();

        let group_size: usize = self
            .library
            .groups
            .iter()
            .map(|g| {
                std::mem::size_of::<crate::models::group::Group>()
                    + g.name.capacity()
                    + g.description.capacity()
            })
            .sum();

        let vec_overhead = std::mem::size_of::<Vec<Combo>>()
            + std::mem::size_of::<Vec<crate::models::group::Group>>();

        combo_size + group_size + vec_overhead
    }

    /// Provides mutable access to the library (for testing only).
    #[cfg(test)]
    pub fn library_mut_for_testing(&mut self) -> &mut ComboLibrary {
        &mut self.library
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_manager() -> ComboManager {
        let tmp = tempdir().expect("tempdir");
        let path = tmp.path().join("combos.json");
        let _ = tmp.keep();
        let storage = ComboStorage::new(path);
        let mut library = ComboLibrary::new("1.0");
        let group = Group::new("Default");
        library.add_group(group);
        ComboManager::with_library(library, storage)
    }

    fn default_group_id(mgr: &ComboManager) -> Uuid {
        mgr.get_all_groups()[0].id
    }

    #[test]
    fn test_create_combo() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        assert_eq!(combo.keyword, "sig");
        assert_eq!(mgr.get_all_combos().len(), 1);
    }

    #[test]
    fn test_get_combo() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        let found = mgr.get_combo(combo.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, combo.id);
    }

    #[test]
    fn test_get_combo_not_found() {
        let mgr = make_manager();
        assert!(mgr.get_combo(Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_update_combo() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        let updated = mgr
            .update_combo(
                combo.id,
                Some("Signature".into()),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        assert_eq!(updated.name, "Signature");
        assert_eq!(updated.keyword, "sig");
    }

    #[test]
    fn test_delete_combo() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        mgr.delete_combo(combo.id).unwrap();
        assert!(mgr.get_all_combos().is_empty());
    }

    #[test]
    fn test_delete_combo_not_found() {
        let mut mgr = make_manager();
        let result = mgr.delete_combo(Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_combo() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        let dup = mgr.duplicate_combo(combo.id).unwrap();
        assert_ne!(dup.id, combo.id);
        assert_eq!(dup.name, "Sig (copy)");
        assert_eq!(dup.keyword, "sig");
        assert_eq!(mgr.get_all_combos().len(), 2);
    }

    #[test]
    fn test_move_combo_to_group() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let g2 = mgr.create_group("Other".into(), "".into()).unwrap();
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        mgr.move_combo_to_group(combo.id, g2.id).unwrap();
        assert_eq!(mgr.get_combo(combo.id).unwrap().group_id, g2.id);
    }

    #[test]
    fn test_toggle_combo() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        assert!(combo.enabled);
        let state = mgr.toggle_combo(combo.id).unwrap();
        assert!(!state);
        let state = mgr.toggle_combo(combo.id).unwrap();
        assert!(state);
    }

    #[test]
    fn test_create_group() {
        let mut mgr = make_manager();
        let group = mgr.create_group("New".into(), "desc".into()).unwrap();
        assert_eq!(group.name, "New");
        assert_eq!(mgr.get_all_groups().len(), 2);
    }

    #[test]
    fn test_update_group() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let updated = mgr.update_group(gid, Some("Renamed".into()), None).unwrap();
        assert_eq!(updated.name, "Renamed");
    }

    #[test]
    fn test_delete_group_moves_combos_to_default() {
        let mut mgr = make_manager();
        let default_gid = default_group_id(&mgr);
        let other = mgr.create_group("Other".into(), "".into()).unwrap();
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                other.id,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        mgr.delete_group(other.id).unwrap();
        // Combo should be moved to default group, not deleted
        assert_eq!(mgr.get_all_combos().len(), 1);
        assert_eq!(mgr.get_combo(combo.id).unwrap().group_id, default_gid);
    }

    #[test]
    fn test_cannot_delete_default_group() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let result = mgr.delete_group(gid);
        assert!(result.is_err());
    }

    #[test]
    fn test_toggle_group() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        mgr.create_combo(
            "Sig".into(),
            "sig".into(),
            "Regards".into(),
            gid,
            MatchingMode::Strict,
            false,
        )
        .unwrap();
        let state = mgr.toggle_group(gid).unwrap();
        assert!(!state);
        assert!(!mgr.get_all_combos()[0].enabled);
    }

    #[test]
    fn test_check_keyword_uniqueness() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let combo = mgr
            .create_combo(
                "Sig".into(),
                "sig".into(),
                "Regards".into(),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();

        // "sig" is taken
        assert!(!mgr.check_keyword_uniqueness("sig", None));
        // "sig" is unique if we exclude the combo that owns it
        assert!(mgr.check_keyword_uniqueness("sig", Some(combo.id)));
        // "other" is unique
        assert!(mgr.check_keyword_uniqueness("other", None));
    }

    #[test]
    fn test_ensure_default_group_idempotent() {
        let mut mgr = make_manager();
        let g1 = mgr.ensure_default_group().unwrap();
        let g2 = mgr.ensure_default_group().unwrap();
        assert_eq!(g1.id, g2.id);
        // Should still have exactly one "Default" group
        let default_count = mgr.get_all_groups().iter().filter(|g| g.name == "Default").count();
        assert_eq!(default_count, 1);
    }

    // ── MT-1110: Memory optimization tests ─────────────────────

    #[test]
    fn test_compact() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        for i in 0..10 {
            mgr.create_combo(
                format!("Combo {}", i),
                format!("kw{:02}", i),
                format!("Snippet {}", i),
                gid,
                MatchingMode::Strict,
                false,
            )
            .unwrap();
        }
        // Delete half
        let combos = mgr.get_all_combos();
        for combo in combos.iter().take(5) {
            mgr.delete_combo(combo.id).unwrap();
        }
        // Compact should not panic
        mgr.compact();
        assert_eq!(mgr.get_all_combos().len(), 5);
    }

    #[test]
    fn test_memory_usage_estimate() {
        let mgr = make_manager();
        let estimate = mgr.memory_usage_estimate();
        // Should be non-zero (at least the Default group exists)
        assert!(estimate > 0);
    }

    #[test]
    fn test_memory_usage_grows_with_combos() {
        let mut mgr = make_manager();
        let gid = default_group_id(&mgr);
        let before = mgr.memory_usage_estimate();
        mgr.create_combo(
            "Big".into(),
            "big".into(),
            "x".repeat(10000),
            gid,
            MatchingMode::Strict,
            false,
        )
        .unwrap();
        let after = mgr.memory_usage_estimate();
        assert!(after > before);
    }

    #[test]
    fn test_create_combo_invalid_group() {
        let mut mgr = make_manager();
        let result = mgr.create_combo(
            "Sig".into(),
            "sig".into(),
            "Regards".into(),
            Uuid::new_v4(),
            MatchingMode::Strict,
            false,
        );
        assert!(result.is_err());
    }
}
