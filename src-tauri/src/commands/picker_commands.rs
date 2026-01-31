//! Tauri IPC commands for combo picker window operations.

use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::models::combo::Combo;

use super::error::CommandError;
use super::AppState;

/// Maximum number of search results returned.
const MAX_SEARCH_RESULTS: usize = 50;

/// Cached search results to avoid re-scoring on repeated queries (MT-1109).
pub struct SearchCache {
    /// The last query string.
    last_query: String,
    /// Cached results for the last query.
    last_results: Vec<ComboSearchResult>,
    /// Generation counter; incremented when combos change.
    generation: u64,
}

impl SearchCache {
    pub fn new() -> Self {
        Self {
            last_query: String::new(),
            last_results: Vec::new(),
            generation: 0,
        }
    }

    /// Invalidate the cache (call when combos are added/removed/modified).
    pub fn invalidate(&mut self) {
        self.generation += 1;
        self.last_query.clear();
        self.last_results.clear();
    }

    /// Check if the cache has a valid result for the given query and generation.
    pub fn get(&self, query: &str, generation: u64) -> Option<&[ComboSearchResult]> {
        if self.generation == generation && self.last_query == query && !query.is_empty() {
            Some(&self.last_results)
        } else {
            None
        }
    }

    /// Store results in the cache.
    pub fn set(&mut self, query: String, results: Vec<ComboSearchResult>, generation: u64) {
        self.last_query = query;
        self.last_results = results;
        self.generation = generation;
    }
}

/// Opens or shows the picker window.
#[tauri::command]
pub fn open_picker_window(app: AppHandle) -> Result<(), CommandError> {
    // Try to get existing picker window
    if let Some(window) = app.get_webview_window("picker") {
        // Window exists, show and focus it
        window
            .show()
            .map_err(|e| CommandError {
                code: "WINDOW_ERROR".to_string(),
                message: format!("Failed to show picker window: {}", e),
            })?;
        window
            .set_focus()
            .map_err(|e| CommandError {
                code: "WINDOW_ERROR".to_string(),
                message: format!("Failed to focus picker window: {}", e),
            })?;
    } else {
        // Create new picker window
        let _window = tauri::WebviewWindowBuilder::new(
            &app,
            "picker",
            tauri::WebviewUrl::App("/picker".into()),
        )
        .title("Combo Picker")
        .inner_size(600.0, 400.0)
        .resizable(true)
        .decorations(true)
        .always_on_top(true)
        .focused(true)
        .build()
        .map_err(|e| CommandError {
            code: "WINDOW_CREATE_ERROR".to_string(),
            message: format!("Failed to create picker window: {}", e),
        })?;
    }

    Ok(())
}

/// Closes or hides the picker window.
#[tauri::command]
pub fn close_picker_window(app: AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window("picker") {
        window
            .hide()
            .map_err(|e| CommandError {
                code: "WINDOW_ERROR".to_string(),
                message: format!("Failed to hide picker window: {}", e),
            })?;
    }
    // If window doesn't exist, that's fine - nothing to close
    Ok(())
}

/// Result for combo search including group name.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComboSearchResult {
    #[serde(flatten)]
    pub combo: Combo,
    pub group_name: String,
}

/// Searches combos by query string, returning results sorted by relevance.
///
/// Search priority:
/// 1. Keyword exact match (case-insensitive)
/// 2. Name contains query
/// 3. Description contains query
/// 4. Snippet contains query
///
/// Returns maximum 50 results.
#[tauri::command]
pub fn search_combos(state: State<AppState>, query: String) -> Result<Vec<ComboSearchResult>, CommandError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;

    let combos = manager.get_all_combos();
    let groups = manager.get_all_groups();

    // Create a map of group IDs to group names
    let group_map: std::collections::HashMap<Uuid, String> = groups
        .into_iter()
        .map(|g| (g.id, g.name))
        .collect();

    let query_lower = query.to_lowercase();

    // Score and filter combos
    let mut scored_results: Vec<(i32, ComboSearchResult)> = combos
        .into_iter()
        .filter(|c| c.enabled) // Only search enabled combos
        .filter_map(|combo| {
            let keyword_lower = combo.keyword.to_lowercase();
            let name_lower = combo.name.to_lowercase();
            let description_lower = combo.description.to_lowercase();
            let snippet_lower = combo.snippet.to_lowercase();

            // Calculate relevance score (higher = more relevant)
            let score = if keyword_lower == query_lower {
                1000 // Exact keyword match
            } else if keyword_lower.contains(&query_lower) {
                900 // Keyword contains query
            } else if name_lower.starts_with(&query_lower) {
                800 // Name starts with query
            } else if name_lower.contains(&query_lower) {
                700 // Name contains query
            } else if description_lower.contains(&query_lower) {
                600 // Description contains query
            } else if snippet_lower.contains(&query_lower) {
                500 // Snippet contains query
            } else {
                return None; // No match
            };

            let group_name = group_map
                .get(&combo.group_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());

            Some((score, ComboSearchResult {
                combo,
                group_name,
            }))
        })
        .collect();

    // Sort by score (descending)
    scored_results.sort_by(|a, b| b.0.cmp(&a.0));

    // Take top 50 results
    let results: Vec<ComboSearchResult> = scored_results
        .into_iter()
        .take(50)
        .map(|(_, result)| result)
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combo_search_result_serialization() {
        // Test that ComboSearchResult can be serialized properly
        use crate::models::combo::ComboBuilder;
        use crate::models::matching::MatchingMode;
        use uuid::Uuid;

        let group_id = Uuid::new_v4();
        let combo = ComboBuilder::new()
            .name("Test".to_string())
            .keyword("test".to_string())
            .snippet("test snippet".to_string())
            .group_id(group_id)
            .matching_mode(MatchingMode::Strict)
            .build()
            .unwrap();

        let result = ComboSearchResult {
            combo,
            group_name: "Test Group".to_string(),
        };

        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("Test Group"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_picker_commands_module_compiles() {
        // Basic compilation test
        assert!(true);
    }

    // ── MT-1109: SearchCache tests ───────────────────────────────

    #[test]
    fn test_search_cache_new_is_empty() {
        let cache = SearchCache::new();
        assert!(cache.get("test", 0).is_none());
    }

    #[test]
    fn test_search_cache_set_and_get() {
        use crate::models::combo::ComboBuilder;
        use crate::models::matching::MatchingMode;

        let mut cache = SearchCache::new();
        let results = vec![ComboSearchResult {
            combo: ComboBuilder::new()
                .name("T".to_string())
                .keyword("tt".to_string())
                .snippet("s".to_string())
                .group_id(Uuid::new_v4())
                .matching_mode(MatchingMode::Strict)
                .build()
                .unwrap(),
            group_name: "G".to_string(),
        }];
        cache.set("hello".to_string(), results.clone(), 0);
        let cached = cache.get("hello", 0);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[test]
    fn test_search_cache_miss_on_different_query() {
        let mut cache = SearchCache::new();
        cache.set("hello".to_string(), vec![], 0);
        assert!(cache.get("world", 0).is_none());
    }

    #[test]
    fn test_search_cache_miss_on_different_generation() {
        let mut cache = SearchCache::new();
        cache.set("hello".to_string(), vec![], 0);
        assert!(cache.get("hello", 1).is_none());
    }

    #[test]
    fn test_search_cache_invalidate() {
        let mut cache = SearchCache::new();
        cache.set("hello".to_string(), vec![], 0);
        cache.invalidate();
        assert!(cache.get("hello", 0).is_none());
    }

    #[test]
    fn test_search_cache_empty_query_never_cached() {
        let mut cache = SearchCache::new();
        cache.set(String::new(), vec![], 0);
        assert!(cache.get("", 0).is_none());
    }

    #[test]
    fn test_max_search_results_constant() {
        assert_eq!(MAX_SEARCH_RESULTS, 50);
    }
}
