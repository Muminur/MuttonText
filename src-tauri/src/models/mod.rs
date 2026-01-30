//! Data models for MuttonText.
//!
//! This module defines the core domain types: combos, groups, preferences,
//! matching modes, and the top-level combo library container.

pub mod combo;
pub mod group;
pub mod library;
pub mod matching;
pub mod preferences;

// Re-export primary types for convenience.
pub use combo::{Combo, ComboBuilder, ComboValidationError};
pub use group::Group;
pub use library::ComboLibrary;
pub use matching::MatchingMode;
pub use preferences::{PasteMethod, Preferences, Theme};
