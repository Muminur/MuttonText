//! System tray state and menu management.

use serde::{Deserialize, Serialize};

/// The current state of the system tray icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrayState {
    /// MuttonText is actively listening for keywords.
    Active,
    /// Expansion is temporarily paused by the user.
    Paused,
    /// The current foreground application is in the exclusion list.
    ExcludedApp,
}

impl Default for TrayState {
    fn default() -> Self {
        Self::Active
    }
}

/// A menu item to be rendered in the system tray context menu.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrayMenuItem {
    /// Unique identifier for the menu item.
    pub id: String,
    /// Display label (empty string for separators).
    pub label: String,
    /// Whether the item is clickable.
    pub enabled: bool,
    /// For toggle items, whether it is currently checked.
    pub checked: Option<bool>,
}

impl TrayMenuItem {
    fn action(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            enabled: true,
            checked: None,
        }
    }

    fn toggle(id: &str, label: &str, checked: bool) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            enabled: true,
            checked: Some(checked),
        }
    }

    fn separator() -> Self {
        Self {
            id: "separator".to_string(),
            label: String::new(),
            enabled: false,
            checked: None,
        }
    }
}

/// Manages system tray icon state and menu construction.
pub struct TrayManager {
    state: TrayState,
}

impl TrayManager {
    /// Creates a new `TrayManager` in the `Active` state.
    pub fn new() -> Self {
        Self {
            state: TrayState::Active,
        }
    }

    /// Returns the current tray state.
    pub fn state(&self) -> TrayState {
        self.state
    }

    /// Sets the tray state.
    pub fn set_state(&mut self, state: TrayState) {
        self.state = state;
    }

    /// Builds the list of menu items for the tray context menu.
    pub fn build_menu_items(&self) -> Vec<TrayMenuItem> {
        let is_active = self.state == TrayState::Active;
        vec![
            TrayMenuItem::action("show", "Show MuttonText"),
            TrayMenuItem::separator(),
            TrayMenuItem::toggle("enabled", "Enabled", is_active),
            TrayMenuItem::action("pause", "Pause"),
            TrayMenuItem::separator(),
            TrayMenuItem::action("preferences", "Preferences..."),
            TrayMenuItem::action("about", "About"),
            TrayMenuItem::separator(),
            TrayMenuItem::action("quit", "Quit"),
        ]
    }

    /// Returns a tooltip string describing the current state.
    pub fn tooltip_text(&self) -> String {
        match self.state {
            TrayState::Active => "MuttonText - Active".to_string(),
            TrayState::Paused => "MuttonText - Paused".to_string(),
            TrayState::ExcludedApp => "MuttonText - Disabled (excluded app)".to_string(),
        }
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_active() {
        let mgr = TrayManager::new();
        assert_eq!(mgr.state(), TrayState::Active);
    }

    #[test]
    fn test_set_state_paused() {
        let mut mgr = TrayManager::new();
        mgr.set_state(TrayState::Paused);
        assert_eq!(mgr.state(), TrayState::Paused);
    }

    #[test]
    fn test_set_state_excluded_app() {
        let mut mgr = TrayManager::new();
        mgr.set_state(TrayState::ExcludedApp);
        assert_eq!(mgr.state(), TrayState::ExcludedApp);
    }

    #[test]
    fn test_state_transitions_round_trip() {
        let mut mgr = TrayManager::new();
        mgr.set_state(TrayState::Paused);
        mgr.set_state(TrayState::Active);
        assert_eq!(mgr.state(), TrayState::Active);
    }

    #[test]
    fn test_menu_items_count() {
        let mgr = TrayManager::new();
        let items = mgr.build_menu_items();
        // show, sep, enabled, pause, sep, preferences, about, sep, quit = 9
        assert_eq!(items.len(), 9);
    }

    #[test]
    fn test_menu_items_first_is_show() {
        let mgr = TrayManager::new();
        let items = mgr.build_menu_items();
        assert_eq!(items[0].id, "show");
        assert_eq!(items[0].label, "Show MuttonText");
    }

    #[test]
    fn test_menu_enabled_toggle_checked_when_active() {
        let mgr = TrayManager::new();
        let items = mgr.build_menu_items();
        let enabled_item = items.iter().find(|i| i.id == "enabled").unwrap();
        assert_eq!(enabled_item.checked, Some(true));
    }

    #[test]
    fn test_menu_enabled_toggle_unchecked_when_paused() {
        let mut mgr = TrayManager::new();
        mgr.set_state(TrayState::Paused);
        let items = mgr.build_menu_items();
        let enabled_item = items.iter().find(|i| i.id == "enabled").unwrap();
        assert_eq!(enabled_item.checked, Some(false));
    }

    #[test]
    fn test_menu_last_is_quit() {
        let mgr = TrayManager::new();
        let items = mgr.build_menu_items();
        assert_eq!(items.last().unwrap().id, "quit");
    }

    #[test]
    fn test_tooltip_active() {
        let mgr = TrayManager::new();
        assert_eq!(mgr.tooltip_text(), "MuttonText - Active");
    }

    #[test]
    fn test_tooltip_paused() {
        let mut mgr = TrayManager::new();
        mgr.set_state(TrayState::Paused);
        assert_eq!(mgr.tooltip_text(), "MuttonText - Paused");
    }

    #[test]
    fn test_tooltip_excluded() {
        let mut mgr = TrayManager::new();
        mgr.set_state(TrayState::ExcludedApp);
        assert!(mgr.tooltip_text().contains("excluded"));
    }

    #[test]
    fn test_tray_state_serialization() {
        let json = serde_json::to_string(&TrayState::Active).unwrap();
        assert_eq!(json, "\"active\"");
        let json = serde_json::to_string(&TrayState::Paused).unwrap();
        assert_eq!(json, "\"paused\"");
    }

    #[test]
    fn test_tray_state_deserialization() {
        let state: TrayState = serde_json::from_str("\"active\"").unwrap();
        assert_eq!(state, TrayState::Active);
    }

    #[test]
    fn test_tray_menu_item_serialization() {
        let item = TrayMenuItem::action("test", "Test");
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"id\":\"test\""));
        assert!(json.contains("\"label\":\"Test\""));
    }
}
