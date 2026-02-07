//! Expansion pipeline for MuttonText.
//!
//! Connects the matching engine to the substitution engine, orchestrating the
//! full flow: buffer analysis -> match detection -> keyword deletion -> snippet
//! insertion -> usage tracking.

use chrono::Utc;
use uuid::Uuid;
use thiserror::Error;

use crate::models::{Combo, Preferences};
use crate::managers::clipboard_manager::{ClipboardManager, ClipboardProvider};
use crate::managers::matching::{MatchResult, MatcherEngine};
use crate::managers::substitution::{SubstitutionEngine, SubstitutionError};

/// Errors arising from the expansion pipeline.
#[derive(Debug, Error)]
pub enum ExpansionError {
    #[error("Matching error: {0}")]
    Matching(String),
    #[error("Substitution error: {0}")]
    Substitution(#[from] SubstitutionError),
}

/// Result of a successful expansion.
#[derive(Debug, Clone)]
pub struct ExpansionResult {
    /// ID of the expanded combo.
    pub combo_id: Uuid,
    /// The keyword that was matched and removed.
    pub keyword: String,
    /// The snippet that was inserted.
    pub snippet: String,
}

/// The expansion pipeline connects buffer matching to text substitution.
///
/// It holds a `MatcherEngine` for detecting keywords and a `SubstitutionEngine`
/// for performing the actual text replacement.
pub struct ExpansionPipeline {
    matcher: MatcherEngine,
    substitution: SubstitutionEngine,
    /// Whether sound feedback is enabled (stub for future implementation).
    play_sound: bool,
}

impl ExpansionPipeline {
    /// Creates a new expansion pipeline.
    pub fn new(matcher: MatcherEngine, substitution: SubstitutionEngine) -> Self {
        Self {
            matcher,
            substitution,
            play_sound: false,
        }
    }

    /// Creates a pipeline with default engines.
    pub fn with_defaults() -> Self {
        Self {
            matcher: MatcherEngine::new(),
            substitution: SubstitutionEngine::with_defaults(),
            play_sound: false,
        }
    }

    /// Returns a reference to the matcher engine.
    pub fn matcher(&self) -> &MatcherEngine {
        &self.matcher
    }

    /// Returns a mutable reference to the matcher engine.
    pub fn matcher_mut(&mut self) -> &mut MatcherEngine {
        &mut self.matcher
    }

    /// Returns a reference to the substitution engine.
    pub fn substitution(&self) -> &SubstitutionEngine {
        &self.substitution
    }

    /// Returns a mutable reference to the substitution engine.
    pub fn substitution_mut(&mut self) -> &mut SubstitutionEngine {
        &mut self.substitution
    }

    /// Loads combos into the matcher engine.
    pub fn load_combos(&mut self, combos: &[Combo]) {
        self.matcher.load_combos(combos);
    }

    /// Applies preferences to the pipeline.
    pub fn apply_preferences(&mut self, prefs: &Preferences) {
        self.matcher.set_excluded_apps(prefs.excluded_apps.clone());
        self.play_sound = prefs.play_sound;

        if !prefs.enabled {
            self.matcher.pause();
        } else {
            self.matcher.resume();
        }
    }

    /// Sets whether sound feedback is enabled.
    pub fn set_play_sound(&mut self, play: bool) {
        self.play_sound = play;
    }

    /// Checks the buffer for a matching combo.
    ///
    /// This is the pure matching step without performing substitution.
    /// Returns `Some(MatchResult)` if a combo keyword is detected at the end
    /// of the buffer.
    pub fn process_buffer(
        &self,
        buffer: &str,
        current_app: Option<&str>,
    ) -> Option<MatchResult> {
        self.matcher.find_match(buffer, current_app)
    }

    /// Performs the full expansion: match detection, keyword deletion, and
    /// snippet insertion via clipboard.
    ///
    /// Returns `Some(ExpansionResult)` if a match was found and expansion succeeded.
    pub fn expand_via_clipboard<P: ClipboardProvider>(
        &self,
        buffer: &str,
        current_app: Option<&str>,
        clipboard_mgr: &mut ClipboardManager<P>,
    ) -> Result<Option<ExpansionResult>, ExpansionError> {
        let match_result = match self.matcher.find_match(buffer, current_app) {
            Some(m) => m,
            None => return Ok(None),
        };

        tracing::info!(
            "Expanding combo: keyword='{}', snippet_len={}",
            match_result.keyword,
            match_result.snippet.len()
        );

        self.substitution.substitute_via_clipboard(
            match_result.keyword_len,
            &match_result.snippet,
            clipboard_mgr,
        )?;

        if self.play_sound {
            play_expansion_sound();
        }

        Ok(Some(ExpansionResult {
            combo_id: match_result.combo_id,
            keyword: match_result.keyword,
            snippet: match_result.snippet,
        }))
    }

    /// Performs the full expansion via keystroke simulation.
    pub fn expand_via_keystrokes(
        &self,
        buffer: &str,
        current_app: Option<&str>,
    ) -> Result<Option<ExpansionResult>, ExpansionError> {
        let match_result = match self.matcher.find_match(buffer, current_app) {
            Some(m) => m,
            None => return Ok(None),
        };

        tracing::info!(
            "Expanding combo via keystrokes: keyword='{}', snippet_len={}",
            match_result.keyword,
            match_result.snippet.len()
        );

        self.substitution.substitute_via_keystrokes(
            match_result.keyword_len,
            &match_result.snippet,
        )?;

        if self.play_sound {
            play_expansion_sound();
        }

        Ok(Some(ExpansionResult {
            combo_id: match_result.combo_id,
            keyword: match_result.keyword,
            snippet: match_result.snippet,
        }))
    }

    /// Performs the full expansion via xdotool type command.
    pub fn expand_via_xdotool(
        &self,
        buffer: &str,
        current_app: Option<&str>,
    ) -> Result<Option<ExpansionResult>, ExpansionError> {
        let match_result = match self.matcher.find_match(buffer, current_app) {
            Some(m) => m,
            None => return Ok(None),
        };

        tracing::info!(
            "Expanding combo via xdotool: keyword='{}', snippet_len={}",
            match_result.keyword,
            match_result.snippet.len()
        );

        self.substitution.substitute_via_xdotool(
            match_result.keyword_len,
            &match_result.snippet,
        )?;

        if self.play_sound {
            play_expansion_sound();
        }

        Ok(Some(ExpansionResult {
            combo_id: match_result.combo_id,
            keyword: match_result.keyword,
            snippet: match_result.snippet,
        }))
    }
}

/// Updates usage statistics for a combo after successful expansion.
///
/// Increments `use_count` and sets `last_used` to now. The caller is
/// responsible for persisting the updated combo.
pub fn update_usage_stats(combo: &mut Combo) {
    combo.use_count += 1;
    combo.last_used = Some(Utc::now());
    combo.modified_at = Utc::now();
    tracing::debug!(
        "Updated usage stats for combo '{}': use_count={}",
        combo.keyword,
        combo.use_count,
    );
}

/// Plays an expansion notification sound.
///
/// Generates a brief beep (880Hz sine wave, ~50ms duration) using the rodio crate.
/// The sound plays in a background thread and will not block the caller.
/// If the "sound" feature is disabled or audio playback fails, this silently does nothing.
fn play_expansion_sound() {
    #[cfg(feature = "sound")]
    {
        std::thread::spawn(|| {
            if let Err(e) = play_beep() {
                tracing::debug!("Failed to play expansion sound: {}", e);
            }
        });
    }

    #[cfg(not(feature = "sound"))]
    {
        tracing::debug!("Sound feature disabled, skipping expansion sound");
    }
}

#[cfg(feature = "sound")]
fn play_beep() -> Result<(), Box<dyn std::error::Error>> {
    use rodio::{OutputStream, Sink};
    use rodio::source::{SineWave, Source};
    use std::time::Duration;

    // Create output stream and sink
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    // Generate 880Hz sine wave (A5 note) for 50ms
    let source = SineWave::new(880.0)
        .take_duration(Duration::from_millis(50))
        .amplify(0.20); // 20% volume to avoid being too loud

    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::combo::ComboBuilder;
    use crate::models::matching::MatchingMode;

    fn make_combo(keyword: &str, snippet: &str) -> Combo {
        ComboBuilder::new()
            .keyword(keyword)
            .snippet(snippet)
            .matching_mode(MatchingMode::Strict)
            .build()
            .unwrap()
    }

    fn make_loose_combo(keyword: &str, snippet: &str) -> Combo {
        ComboBuilder::new()
            .keyword(keyword)
            .snippet(snippet)
            .matching_mode(MatchingMode::Loose)
            .build()
            .unwrap()
    }

    // ── process_buffer tests ──────────────────────────────────────

    #[test]
    fn test_process_buffer_match() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_combo("sig", "Best regards")]);

        let result = pipeline.process_buffer("hello sig", None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.keyword, "sig");
        assert_eq!(m.snippet, "Best regards");
    }

    #[test]
    fn test_process_buffer_no_match() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_combo("sig", "Best regards")]);

        let result = pipeline.process_buffer("hello world", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_process_buffer_empty() {
        let pipeline = ExpansionPipeline::with_defaults();
        assert!(pipeline.process_buffer("", None).is_none());
    }

    #[test]
    fn test_process_buffer_paused() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_combo("sig", "Best regards")]);
        pipeline.matcher_mut().pause();

        assert!(pipeline.process_buffer("hello sig", None).is_none());
    }

    #[test]
    fn test_process_buffer_excluded_app() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_combo("sig", "Best regards")]);
        pipeline.matcher_mut().set_excluded_apps(vec!["1password".into()]);

        assert!(pipeline.process_buffer("hello sig", Some("1Password")).is_none());
        assert!(pipeline.process_buffer("hello sig", Some("notepad")).is_some());
    }

    #[test]
    fn test_process_buffer_strict_no_mid_word() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_combo("sig", "Best regards")]);

        assert!(pipeline.process_buffer("testsig", None).is_none());
    }

    #[test]
    fn test_process_buffer_loose_mid_word() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_loose_combo("sig", "Best regards")]);

        assert!(pipeline.process_buffer("testsig", None).is_some());
    }

    #[test]
    fn test_process_buffer_disabled_combo() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        let mut combo = make_combo("sig", "Best regards");
        combo.enabled = false;
        pipeline.load_combos(&[combo]);

        assert!(pipeline.process_buffer("hello sig", None).is_none());
    }

    // ── apply_preferences tests ───────────────────────────────────

    #[test]
    fn test_apply_preferences_disabled() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_combo("sig", "Best regards")]);

        let mut prefs = Preferences::default();
        prefs.enabled = false;
        pipeline.apply_preferences(&prefs);

        assert!(pipeline.matcher().is_paused());
        assert!(pipeline.process_buffer("hello sig", None).is_none());
    }

    #[test]
    fn test_apply_preferences_enabled() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_combo("sig", "Best regards")]);

        let prefs = Preferences::default(); // enabled = true
        pipeline.apply_preferences(&prefs);

        assert!(!pipeline.matcher().is_paused());
        assert!(pipeline.process_buffer("hello sig", None).is_some());
    }

    #[test]
    fn test_apply_preferences_excluded_apps() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[make_combo("sig", "Best regards")]);

        let mut prefs = Preferences::default();
        prefs.excluded_apps = vec!["keepass".to_string()];
        pipeline.apply_preferences(&prefs);

        assert!(pipeline.process_buffer("hello sig", Some("KeePass")).is_none());
    }

    #[test]
    fn test_apply_preferences_sound() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        let mut prefs = Preferences::default();
        prefs.play_sound = true;
        pipeline.apply_preferences(&prefs);
        // Just verify it doesn't panic
    }

    // ── update_usage_stats tests ──────────────────────────────────

    #[test]
    fn test_update_usage_stats_increments_count() {
        let mut combo = make_combo("sig", "Best regards");
        assert_eq!(combo.use_count, 0);
        assert!(combo.last_used.is_none());

        update_usage_stats(&mut combo);

        assert_eq!(combo.use_count, 1);
        assert!(combo.last_used.is_some());
    }

    #[test]
    fn test_update_usage_stats_multiple() {
        let mut combo = make_combo("sig", "Best regards");

        update_usage_stats(&mut combo);
        update_usage_stats(&mut combo);
        update_usage_stats(&mut combo);

        assert_eq!(combo.use_count, 3);
    }

    #[test]
    fn test_update_usage_stats_updates_modified_at() {
        let mut combo = make_combo("sig", "Best regards");
        let original_modified = combo.modified_at;

        // Small sleep not needed; chrono is precise enough
        update_usage_stats(&mut combo);

        assert!(combo.modified_at >= original_modified);
    }

    // ── Pipeline construction tests ───────────────────────────────

    #[test]
    fn test_pipeline_with_defaults() {
        let pipeline = ExpansionPipeline::with_defaults();
        assert_eq!(pipeline.matcher().combo_count(), 0);
        assert!(!pipeline.matcher().is_paused());
    }

    #[test]
    fn test_pipeline_load_combos() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.load_combos(&[
            make_combo("sig", "Signature"),
            make_combo("addr", "Address"),
        ]);
        assert_eq!(pipeline.matcher().combo_count(), 2);
    }

    #[test]
    fn test_pipeline_set_play_sound() {
        let mut pipeline = ExpansionPipeline::with_defaults();
        pipeline.set_play_sound(true);
        // No panic = success (sound is a stub)
    }

    // ── E2E-style integration test (no actual key simulation) ─────

    #[test]
    fn test_full_expansion_flow_detection() {
        // This tests the detection part of the pipeline end-to-end.
        // Actual key simulation requires a display server so we test the
        // match + result structure.
        let mut pipeline = ExpansionPipeline::with_defaults();
        let combo = make_combo("sig", "Best regards,\nJohn");
        let combo_id = combo.id;
        pipeline.load_combos(&[combo]);

        // Simulate typing "hello sig"
        let buffers = ["h", "he", "hel", "hell", "hello", "hello ", "hello s", "hello si", "hello sig"];

        for &buf in &buffers[..buffers.len() - 1] {
            assert!(pipeline.process_buffer(buf, None).is_none(), "Should not match on '{}'", buf);
        }

        let result = pipeline.process_buffer("hello sig", None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.combo_id, combo_id);
        assert_eq!(m.keyword, "sig");
        assert_eq!(m.snippet, "Best regards,\nJohn");
        assert_eq!(m.keyword_len, 3);
    }

    #[test]
    fn test_expansion_error_display() {
        let err = ExpansionError::Matching("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_play_expansion_sound_does_not_panic() {
        // Verify that calling play_expansion_sound() doesn't panic
        // even if audio system is unavailable or feature is disabled
        super::play_expansion_sound();
        // Give thread a moment to spawn if sound feature is enabled
        std::thread::sleep(std::time::Duration::from_millis(10));
        // If we reach here without panic, test passes
    }
}
