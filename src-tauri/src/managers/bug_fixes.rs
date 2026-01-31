//! MT-1101: Bug fixes audit for MuttonText.
//!
//! Documents and addresses known issues found during code audit:
//!
//! ## Fix 1: `matching.rs` line 57 - unwrap() on chars().last()
//! The `is_strict_match` function calls `.chars().last().unwrap()` on a prefix
//! slice that is guaranteed non-empty (prefix_len > 0 checked above), so this
//! is technically safe. However, we replace it with a safer pattern for defense
//! in depth.
//!
//! ## Fix 2: `variable_evaluator.rs` line 514 - unwrap() on chars().last()
//! The `apply_time_shift` function calls `.chars().last().unwrap()` on `rest`
//! which is guaranteed non-empty (checked above). Safe but replaced with
//! a match for consistency.
//!
//! ## Fix 3: `combo_storage.rs` line 151-152 - unwrap_or("") on file_stem
//! The `list_backups` function uses `unwrap_or("")` which is acceptable but
//! could hide issues. No change needed.
//!
//! ## Fix 4: `clipboard_manager.rs` line 96 - unwrap_or_default on read
//! The `preserve` method swallows read errors silently. This is by design
//! (preserve should not fail if clipboard is empty/inaccessible).
//!
//! ## Summary
//! Non-test unwrap() calls found:
//! - `matching.rs:57` - Fixed via safe_preceding_char helper
//! - `variable_evaluator.rs:514` - Fixed via match on last()
//! - `backup_manager.rs:151` - Acceptable (unwrap_or)
//! - `clipboard_manager.rs:96` - Acceptable by design (unwrap_or_default)

#[cfg(test)]
mod tests {
    #[test]
    fn test_bug_fixes_module_loads() {
        // This module is primarily documentation. Its fixes are applied
        // in the respective source files.
        assert!(true);
    }
}
