use serde::{Deserialize, Serialize};

use super::matching::MatchingMode;

/// How substituted text is pasted into the active application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PasteMethod {
    /// Copy to clipboard and simulate Ctrl+V / Cmd+V.
    Clipboard,
    /// Simulate individual keystrokes to type the snippet.
    SimulateKeystrokes,
}

impl Default for PasteMethod {
    fn default() -> Self {
        Self::Clipboard
    }
}

/// Application color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    /// Follow operating system preference.
    System,
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Self::System
    }
}

/// User-facing application preferences persisted as JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub enabled: bool,
    pub play_sound: bool,
    pub show_system_tray: bool,
    pub start_at_login: bool,
    pub start_minimized: bool,
    pub default_matching_mode: MatchingMode,
    pub default_case_sensitive: bool,
    pub combo_trigger_shortcut: String,
    pub picker_shortcut: String,
    pub paste_method: PasteMethod,
    pub theme: Theme,
    pub backup_enabled: bool,
    pub backup_interval_hours: u32,
    pub max_backups: u32,
    pub auto_check_updates: bool,
    pub excluded_apps: Vec<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            enabled: true,
            play_sound: false,
            show_system_tray: true,
            start_at_login: false,
            start_minimized: false,
            default_matching_mode: MatchingMode::default(),
            default_case_sensitive: false,
            combo_trigger_shortcut: String::new(),
            picker_shortcut: "Ctrl+Shift+Space".to_string(),
            paste_method: PasteMethod::default(),
            theme: Theme::default(),
            backup_enabled: true,
            backup_interval_hours: 24,
            max_backups: 10,
            auto_check_updates: true,
            excluded_apps: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PasteMethod tests ───────────────────────────────────────────

    #[test]
    fn test_paste_method_default_is_clipboard() {
        assert_eq!(PasteMethod::default(), PasteMethod::Clipboard);
    }

    #[test]
    fn test_paste_method_serialization_roundtrip() {
        for method in &[PasteMethod::Clipboard, PasteMethod::SimulateKeystrokes] {
            let json = serde_json::to_string(method).expect("serialize");
            let deserialized: PasteMethod = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*method, deserialized);
        }
    }

    // ── Theme tests ─────────────────────────────────────────────────

    #[test]
    fn test_theme_default_is_system() {
        assert_eq!(Theme::default(), Theme::System);
    }

    #[test]
    fn test_theme_serialization_roundtrip() {
        for theme in &[Theme::System, Theme::Light, Theme::Dark] {
            let json = serde_json::to_string(theme).expect("serialize");
            let deserialized: Theme = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*theme, deserialized);
        }
    }

    // ── Preferences Default tests ───────────────────────────────────

    #[test]
    fn test_preferences_default_enabled() {
        let prefs = Preferences::default();
        assert!(prefs.enabled);
    }

    #[test]
    fn test_preferences_default_play_sound_off() {
        let prefs = Preferences::default();
        assert!(!prefs.play_sound);
    }

    #[test]
    fn test_preferences_default_show_tray() {
        let prefs = Preferences::default();
        assert!(prefs.show_system_tray);
    }

    #[test]
    fn test_preferences_default_no_start_at_login() {
        let prefs = Preferences::default();
        assert!(!prefs.start_at_login);
    }

    #[test]
    fn test_preferences_default_not_minimized() {
        let prefs = Preferences::default();
        assert!(!prefs.start_minimized);
    }

    #[test]
    fn test_preferences_default_strict_matching() {
        let prefs = Preferences::default();
        assert_eq!(prefs.default_matching_mode, MatchingMode::Strict);
    }

    #[test]
    fn test_preferences_default_case_insensitive() {
        let prefs = Preferences::default();
        assert!(!prefs.default_case_sensitive);
    }

    #[test]
    fn test_preferences_default_picker_shortcut() {
        let prefs = Preferences::default();
        assert_eq!(prefs.picker_shortcut, "Ctrl+Shift+Space");
    }

    #[test]
    fn test_preferences_default_paste_clipboard() {
        let prefs = Preferences::default();
        assert_eq!(prefs.paste_method, PasteMethod::Clipboard);
    }

    #[test]
    fn test_preferences_default_theme_system() {
        let prefs = Preferences::default();
        assert_eq!(prefs.theme, Theme::System);
    }

    #[test]
    fn test_preferences_default_backup_enabled() {
        let prefs = Preferences::default();
        assert!(prefs.backup_enabled);
        assert_eq!(prefs.backup_interval_hours, 24);
        assert_eq!(prefs.max_backups, 10);
    }

    #[test]
    fn test_preferences_default_auto_updates() {
        let prefs = Preferences::default();
        assert!(prefs.auto_check_updates);
    }

    #[test]
    fn test_preferences_default_no_excluded_apps() {
        let prefs = Preferences::default();
        assert!(prefs.excluded_apps.is_empty());
    }

    // ── Preferences serialization ───────────────────────────────────

    #[test]
    fn test_preferences_serialization_roundtrip() {
        let prefs = Preferences::default();
        let json = serde_json::to_string(&prefs).expect("serialize");
        let deserialized: Preferences = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(prefs, deserialized);
    }

    #[test]
    fn test_preferences_json_uses_camel_case() {
        let prefs = Preferences::default();
        let json = serde_json::to_string(&prefs).expect("serialize");
        assert!(json.contains("playSound"));
        assert!(json.contains("showSystemTray"));
        assert!(json.contains("startAtLogin"));
        assert!(json.contains("defaultMatchingMode"));
        assert!(json.contains("pasteMethod"));
        assert!(json.contains("backupIntervalHours"));
        assert!(json.contains("excludedApps"));
        // Must NOT contain snake_case
        assert!(!json.contains("play_sound"));
        assert!(!json.contains("start_at_login"));
    }

    #[test]
    fn test_preferences_with_excluded_apps() {
        let mut prefs = Preferences::default();
        prefs.excluded_apps = vec!["1password".to_string(), "keepass".to_string()];
        let json = serde_json::to_string(&prefs).expect("serialize");
        let deserialized: Preferences = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.excluded_apps.len(), 2);
        assert_eq!(deserialized.excluded_apps[0], "1password");
    }

    #[test]
    fn test_preferences_clone() {
        let prefs = Preferences::default();
        let cloned = prefs.clone();
        assert_eq!(prefs, cloned);
    }
}
