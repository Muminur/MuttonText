//! Export functionality for combos and groups to various formats.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::combo::Combo;
use crate::models::group::Group;

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
    MuttonTextJson,
    TextExpanderCsv,
    CheatsheetCsv,
}

/// Errors that can occur during export.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Serialization failed: {0}")]
    Serialization(String),
}

/// Internal structure for native JSON export.
#[derive(Debug, Serialize, Deserialize)]
struct MuttonTextFile {
    combos: Vec<Combo>,
    groups: Vec<Group>,
}

pub struct ExportManager;

impl ExportManager {
    /// Export to native MuttonText JSON format.
    pub fn export_muttontext_json(combos: &[Combo], groups: &[Group]) -> Result<String, ExportError> {
        let file = MuttonTextFile {
            combos: combos.to_vec(),
            groups: groups.to_vec(),
        };
        serde_json::to_string_pretty(&file).map_err(|e| ExportError::Serialization(e.to_string()))
    }

    /// Export to TextExpander CSV format.
    /// Columns: Abbreviation,Content,Label
    pub fn export_textexpander_csv(combos: &[Combo]) -> Result<String, ExportError> {
        let mut out = String::from("Abbreviation,Content,Label\n");
        for combo in combos {
            out.push_str(&csv_escape(&combo.keyword));
            out.push(',');
            out.push_str(&csv_escape(&combo.snippet));
            out.push(',');
            out.push_str(&csv_escape(&combo.name));
            out.push('\n');
        }
        Ok(out)
    }

    /// Export to cheatsheet CSV format.
    /// Columns: Group,Keyword,Name,Description
    pub fn export_cheatsheet_csv(combos: &[Combo], groups: &[Group]) -> Result<String, ExportError> {
        let mut out = String::from("Group,Keyword,Name,Description\n");
        for combo in combos {
            let group_name = groups
                .iter()
                .find(|g| g.id == combo.group_id)
                .map(|g| g.name.as_str())
                .unwrap_or("");
            out.push_str(&csv_escape(group_name));
            out.push(',');
            out.push_str(&csv_escape(&combo.keyword));
            out.push(',');
            out.push_str(&csv_escape(&combo.name));
            out.push(',');
            out.push_str(&csv_escape(&combo.description));
            out.push('\n');
        }
        Ok(out)
    }

    /// Export to the specified format.
    pub fn export_to_format(
        combos: &[Combo],
        groups: &[Group],
        format: ExportFormat,
    ) -> Result<String, ExportError> {
        match format {
            ExportFormat::MuttonTextJson => Self::export_muttontext_json(combos, groups),
            ExportFormat::TextExpanderCsv => Self::export_textexpander_csv(combos),
            ExportFormat::CheatsheetCsv => Self::export_cheatsheet_csv(combos, groups),
        }
    }
}

/// Escape a field for CSV output. Quotes the field if it contains commas,
/// quotes, or newlines. Prevents CSV injection by prefixing dangerous characters.
fn csv_escape(field: &str) -> String {
    // Prevent CSV injection - prefix dangerous characters with single quote
    let needs_injection_protection = if let Some(first_char) = field.chars().next() {
        first_char == '=' || first_char == '+' || first_char == '-' ||
        first_char == '@' || first_char == '\t' || first_char == '\r'
    } else {
        false
    };

    let result = if field.contains(',') || field.contains('"') || field.contains('\n') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        field.to_string()
    };

    if needs_injection_protection {
        format!("'{}", result)
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::combo::ComboBuilder;
    use crate::models::group::Group;

    fn test_combo(keyword: &str, snippet: &str, name: &str, group_id: uuid::Uuid) -> Combo {
        ComboBuilder::new()
            .name(name)
            .keyword(keyword)
            .snippet(snippet)
            .group_id(group_id)
            .build()
            .unwrap()
    }

    // ── MuttonText JSON ──────────────────────────────────────────

    #[test]
    fn test_export_muttontext_json() {
        let group = Group::new("Email");
        let combo = test_combo("sig", "hello", "Sig", group.id);
        let json = ExportManager::export_muttontext_json(&[combo], &[group]).unwrap();
        assert!(json.contains("sig"));
        assert!(json.contains("Email"));
    }

    #[test]
    fn test_export_muttontext_json_empty() {
        let json = ExportManager::export_muttontext_json(&[], &[]).unwrap();
        assert!(json.contains("combos"));
        assert!(json.contains("groups"));
    }

    #[test]
    fn test_export_import_roundtrip() {
        use crate::managers::import_manager::ImportManager;

        let group = Group::new("Test");
        let combo = test_combo("sig", "hello", "Sig", group.id);
        let json = ExportManager::export_muttontext_json(&[combo.clone()], &[group.clone()]).unwrap();
        let result = ImportManager::import_muttontext_json(&json).unwrap();
        assert_eq!(result.combos.len(), 1);
        assert_eq!(result.combos[0].keyword, "sig");
        assert_eq!(result.groups[0].name, "Test");
    }

    // ── TextExpander CSV ─────────────────────────────────────────

    #[test]
    fn test_export_textexpander_csv() {
        let combo = test_combo("sig", "Best regards", "Signature", uuid::Uuid::nil());
        let csv = ExportManager::export_textexpander_csv(&[combo]).unwrap();
        assert!(csv.starts_with("Abbreviation,Content,Label\n"));
        assert!(csv.contains("sig,Best regards,Signature"));
    }

    #[test]
    fn test_export_textexpander_csv_empty() {
        let csv = ExportManager::export_textexpander_csv(&[]).unwrap();
        assert_eq!(csv, "Abbreviation,Content,Label\n");
    }

    // ── Cheatsheet CSV ───────────────────────────────────────────

    #[test]
    fn test_export_cheatsheet_csv() {
        let group = Group::new("Email");
        let combo = test_combo("sig", "hello", "Signature", group.id);
        let csv = ExportManager::export_cheatsheet_csv(&[combo], &[group]).unwrap();
        assert!(csv.starts_with("Group,Keyword,Name,Description\n"));
        assert!(csv.contains("Email,sig,Signature,"));
    }

    #[test]
    fn test_export_cheatsheet_csv_no_group() {
        let combo = test_combo("sig", "hello", "Sig", uuid::Uuid::nil());
        let csv = ExportManager::export_cheatsheet_csv(&[combo], &[]).unwrap();
        assert!(csv.contains(",sig,Sig,"));
    }

    // ── CSV Escaping ─────────────────────────────────────────────

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_comma() {
        assert_eq!(csv_escape("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn test_csv_escape_quotes() {
        assert_eq!(csv_escape(r#"say "hi""#), r#""say ""hi""""#);
    }

    #[test]
    fn test_csv_escape_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    // ── CSV Injection Prevention ─────────────────────────────────

    #[test]
    fn test_csv_escape_formula_injection_equals() {
        assert_eq!(csv_escape("=1+1"), "'=1+1");
    }

    #[test]
    fn test_csv_escape_formula_injection_plus() {
        assert_eq!(csv_escape("+1+1"), "'+1+1");
    }

    #[test]
    fn test_csv_escape_formula_injection_minus() {
        assert_eq!(csv_escape("-1-1"), "'-1-1");
    }

    #[test]
    fn test_csv_escape_formula_injection_at() {
        assert_eq!(csv_escape("@SUM(A1:A10)"), "'@SUM(A1:A10)");
    }

    #[test]
    fn test_csv_escape_formula_injection_tab() {
        assert_eq!(csv_escape("\tdata"), "'\tdata");
    }

    #[test]
    fn test_csv_escape_formula_injection_carriage_return() {
        assert_eq!(csv_escape("\rdata"), "'\rdata");
    }

    #[test]
    fn test_csv_escape_formula_with_comma() {
        // Should quote AND prefix with single quote
        assert_eq!(csv_escape("=1,2"), "'\"=1,2\"");
    }

    // ── export_to_format ─────────────────────────────────────────

    #[test]
    fn test_export_to_format_dispatches() {
        let group = Group::new("G");
        let combo = test_combo("sig", "hello", "Sig", group.id);
        let combos = [combo];
        let groups = [group];

        let json = ExportManager::export_to_format(&combos, &groups, ExportFormat::MuttonTextJson).unwrap();
        assert!(json.contains("combos"));

        let csv = ExportManager::export_to_format(&combos, &groups, ExportFormat::TextExpanderCsv).unwrap();
        assert!(csv.contains("Abbreviation"));

        let cheat = ExportManager::export_to_format(&combos, &groups, ExportFormat::CheatsheetCsv).unwrap();
        assert!(cheat.contains("Group,Keyword"));
    }

    // ── Error Display ────────────────────────────────────────────

    #[test]
    fn test_export_error_display() {
        let err = ExportError::Serialization("test".to_string());
        assert_eq!(err.to_string(), "Serialization failed: test");
    }

    // ── Format Serialization ─────────────────────────────────────

    #[test]
    fn test_export_format_serialization() {
        let json = serde_json::to_string(&ExportFormat::MuttonTextJson).unwrap();
        assert_eq!(json, r#""muttonTextJson""#);
    }
}
