//! Import functionality for combos and groups from various formats.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::models::combo::{Combo, ComboBuilder};
use crate::models::group::Group;
use crate::models::matching::MatchingMode;

/// Supported import formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportFormat {
    BeeftextJson,
    BeeftextCsv,
    TextExpanderCsv,
    MuttonTextJson,
}

/// How to resolve keyword conflicts during import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictResolution {
    Skip,
    Overwrite,
    Rename,
}

/// Result of an import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub errors: Vec<String>,
    pub combos: Vec<Combo>,
    pub groups: Vec<Group>,
}

/// Preview of what an import would produce.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub format: ImportFormat,
    pub combo_count: usize,
    pub group_count: usize,
}

/// Errors that can occur during import.
#[derive(Debug, Error)]
pub enum ImportError {
    #[error("Unrecognized import format")]
    UnrecognizedFormat,
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    #[error("Invalid CSV: {0}")]
    InvalidCsv(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Beeftext JSON structures for deserialization.
#[derive(Debug, Deserialize)]
struct BeeftextFile {
    combos: Option<Vec<BeeftextCombo>>,
    groups: Option<Vec<BeeftextGroup>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BeeftextCombo {
    name: Option<String>,
    keyword: Option<String>,
    snippet: Option<String>,
    matching_mode: Option<String>,
    group: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BeeftextGroup {
    name: Option<String>,
}

/// Native MuttonText JSON structures.
#[derive(Debug, Serialize, Deserialize)]
struct MuttonTextFile {
    combos: Vec<Combo>,
    groups: Vec<Group>,
}

pub struct ImportManager;

impl ImportManager {
    /// Auto-detect the format of the given content string.
    pub fn detect_format(content: &str) -> Result<ImportFormat, ImportError> {
        let trimmed = content.trim();

        // Try JSON first
        if trimmed.starts_with('{') {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                // MuttonText native has combos with "id" fields
                if let Some(combos) = val.get("combos").and_then(|c| c.as_array()) {
                    if let Some(first) = combos.first() {
                        if first.get("id").is_some() && first.get("groupId").is_some() {
                            return Ok(ImportFormat::MuttonTextJson);
                        }
                    }
                }
                // Beeftext JSON has combos with "keyword" and optional "group" string
                if val.get("combos").is_some() || val.get("groups").is_some() {
                    return Ok(ImportFormat::BeeftextJson);
                }
            }
        }

        // Try CSV detection
        if trimmed.contains(',') || trimmed.contains('\n') {
            let first_line = trimmed.lines().next().unwrap_or("");
            let lower = first_line.to_lowercase();
            // TextExpander CSV: Abbreviation,Content,Label
            if lower.contains("abbreviation") && lower.contains("content") {
                return Ok(ImportFormat::TextExpanderCsv);
            }
            // Beeftext CSV: Name,Keyword,Snippet,MatchingMode,Group
            if lower.contains("keyword") && lower.contains("snippet") {
                return Ok(ImportFormat::BeeftextCsv);
            }
        }

        Err(ImportError::UnrecognizedFormat)
    }

    /// Import from Beeftext JSON format.
    pub fn import_beeftext_json(
        content: &str,
        conflict: ConflictResolution,
    ) -> Result<ImportResult, ImportError> {
        let file: BeeftextFile =
            serde_json::from_str(content).map_err(|e| ImportError::InvalidJson(e.to_string()))?;

        let mut groups: Vec<Group> = Vec::new();
        let mut combos: Vec<Combo> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut skipped = 0usize;

        // Build groups
        if let Some(bt_groups) = &file.groups {
            for bg in bt_groups {
                if let Some(name) = &bg.name {
                    if !name.is_empty() {
                        groups.push(Group::new(name.clone()));
                    }
                }
            }
        }

        // Build combos
        if let Some(bt_combos) = &file.combos {
            for bc in bt_combos {
                let keyword = match &bc.keyword {
                    Some(k) if !k.is_empty() => k.clone(),
                    _ => {
                        errors.push("Combo missing keyword, skipped".to_string());
                        skipped += 1;
                        continue;
                    }
                };

                let snippet = match &bc.snippet {
                    Some(s) if !s.is_empty() => s.clone(),
                    _ => {
                        errors.push(format!("Combo '{}' missing snippet, skipped", keyword));
                        skipped += 1;
                        continue;
                    }
                };

                let final_keyword = match conflict {
                    ConflictResolution::Rename => format!("{}-imported", keyword),
                    _ => keyword.clone(),
                };

                // Find or create group
                let group_name = bc.group.clone().unwrap_or_default();
                let group_id = if !group_name.is_empty() {
                    if let Some(g) = groups.iter().find(|g| g.name == group_name) {
                        g.id
                    } else {
                        let g = Group::new(group_name);
                        let id = g.id;
                        groups.push(g);
                        id
                    }
                } else {
                    Uuid::nil()
                };

                let mode = match bc.matching_mode.as_deref() {
                    Some("loose") => MatchingMode::Loose,
                    _ => MatchingMode::Strict,
                };

                let result = ComboBuilder::new()
                    .name(bc.name.clone().unwrap_or_default())
                    .keyword(final_keyword)
                    .snippet(snippet)
                    .group_id(group_id)
                    .matching_mode(mode)
                    .build();

                match result {
                    Ok(combo) => combos.push(combo),
                    Err(e) => {
                        errors.push(format!("Combo '{}': {}", keyword, e));
                        skipped += 1;
                    }
                }
            }
        }

        Ok(ImportResult {
            imported_count: combos.len(),
            skipped_count: skipped,
            errors,
            combos,
            groups,
        })
    }

    /// Import from Beeftext CSV format.
    /// Columns: Name, Keyword, Snippet, MatchingMode, Group
    pub fn import_beeftext_csv(
        content: &str,
        conflict: ConflictResolution,
    ) -> Result<ImportResult, ImportError> {
        let mut lines = content.lines();
        // Skip header
        let header = lines.next().ok_or(ImportError::InvalidCsv("Empty CSV".to_string()))?;
        let _ = header; // consume header

        let mut groups: Vec<Group> = Vec::new();
        let mut combos: Vec<Combo> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut skipped = 0usize;

        for (i, line) in lines.enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let fields = parse_csv_line(line);
            if fields.len() < 3 {
                errors.push(format!("Line {}: too few fields", i + 2));
                skipped += 1;
                continue;
            }

            let name = fields[0].clone();
            let keyword = fields[1].clone();
            let snippet = fields[2].clone();
            let mode_str = fields.get(3).cloned().unwrap_or_default();
            let group_name = fields.get(4).cloned().unwrap_or_default();

            if keyword.is_empty() || snippet.is_empty() {
                errors.push(format!("Line {}: empty keyword or snippet", i + 2));
                skipped += 1;
                continue;
            }

            let final_keyword = match conflict {
                ConflictResolution::Rename => format!("{}-imported", keyword),
                _ => keyword.clone(),
            };

            let group_id = if !group_name.is_empty() {
                if let Some(g) = groups.iter().find(|g| g.name == group_name) {
                    g.id
                } else {
                    let g = Group::new(group_name);
                    let id = g.id;
                    groups.push(g);
                    id
                }
            } else {
                Uuid::nil()
            };

            let mode = match mode_str.to_lowercase().as_str() {
                "loose" => MatchingMode::Loose,
                _ => MatchingMode::Strict,
            };

            match ComboBuilder::new()
                .name(name)
                .keyword(final_keyword)
                .snippet(snippet)
                .group_id(group_id)
                .matching_mode(mode)
                .build()
            {
                Ok(combo) => combos.push(combo),
                Err(e) => {
                    errors.push(format!("Line {}: {}", i + 2, e));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            imported_count: combos.len(),
            skipped_count: skipped,
            errors,
            combos,
            groups,
        })
    }

    /// Import from TextExpander CSV format.
    /// Columns: Abbreviation, Content, Label
    pub fn import_textexpander_csv(
        content: &str,
        conflict: ConflictResolution,
    ) -> Result<ImportResult, ImportError> {
        let mut lines = content.lines();
        let _header = lines.next().ok_or(ImportError::InvalidCsv("Empty CSV".to_string()))?;

        let mut combos: Vec<Combo> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut skipped = 0usize;

        for (i, line) in lines.enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let fields = parse_csv_line(line);
            if fields.len() < 2 {
                errors.push(format!("Line {}: too few fields", i + 2));
                skipped += 1;
                continue;
            }

            let abbreviation = fields[0].clone();
            let text_content = fields[1].clone();
            let label = fields.get(2).cloned().unwrap_or_default();

            if abbreviation.is_empty() || text_content.is_empty() {
                errors.push(format!("Line {}: empty abbreviation or content", i + 2));
                skipped += 1;
                continue;
            }

            let final_keyword = match conflict {
                ConflictResolution::Rename => format!("{}-imported", abbreviation),
                _ => abbreviation.clone(),
            };

            match ComboBuilder::new()
                .name(label)
                .keyword(final_keyword)
                .snippet(text_content)
                .build()
            {
                Ok(combo) => combos.push(combo),
                Err(e) => {
                    errors.push(format!("Line {}: {}", i + 2, e));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            imported_count: combos.len(),
            skipped_count: skipped,
            errors,
            combos,
            groups: Vec::new(),
        })
    }

    /// Import from native MuttonText JSON format.
    pub fn import_muttontext_json(content: &str) -> Result<ImportResult, ImportError> {
        let file: MuttonTextFile =
            serde_json::from_str(content).map_err(|e| ImportError::InvalidJson(e.to_string()))?;

        let count = file.combos.len();
        Ok(ImportResult {
            imported_count: count,
            skipped_count: 0,
            errors: Vec::new(),
            combos: file.combos,
            groups: file.groups,
        })
    }

    /// Preview an import without actually creating combos.
    pub fn preview_import(content: &str) -> Result<ImportPreview, ImportError> {
        let format = Self::detect_format(content)?;
        match format {
            ImportFormat::MuttonTextJson => {
                let file: MuttonTextFile = serde_json::from_str(content)
                    .map_err(|e| ImportError::InvalidJson(e.to_string()))?;
                Ok(ImportPreview {
                    format,
                    combo_count: file.combos.len(),
                    group_count: file.groups.len(),
                })
            }
            ImportFormat::BeeftextJson => {
                let file: BeeftextFile = serde_json::from_str(content)
                    .map_err(|e| ImportError::InvalidJson(e.to_string()))?;
                Ok(ImportPreview {
                    format,
                    combo_count: file.combos.as_ref().map(|c| c.len()).unwrap_or(0),
                    group_count: file.groups.as_ref().map(|g| g.len()).unwrap_or(0),
                })
            }
            ImportFormat::BeeftextCsv | ImportFormat::TextExpanderCsv => {
                let data_lines = content
                    .lines()
                    .skip(1)
                    .filter(|l| !l.trim().is_empty())
                    .count();
                Ok(ImportPreview {
                    format,
                    combo_count: data_lines,
                    group_count: 0,
                })
            }
        }
    }
}

/// Simple CSV line parser that handles quoted fields.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    // Escaped quote
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else {
            match ch {
                '"' => in_quotes = true,
                ',' => {
                    fields.push(current.trim().to_string());
                    current = String::new();
                }
                _ => current.push(ch),
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Format Detection ─────────────────────────────────────────

    #[test]
    fn test_detect_beeftext_json() {
        let content = r#"{"combos":[{"keyword":"sig","snippet":"hello"}],"groups":[{"name":"G1"}]}"#;
        let fmt = ImportManager::detect_format(content).unwrap();
        assert_eq!(fmt, ImportFormat::BeeftextJson);
    }

    #[test]
    fn test_detect_muttontext_json() {
        let group_id = Uuid::new_v4();
        let combo = ComboBuilder::new()
            .keyword("sig")
            .snippet("hello")
            .group_id(group_id)
            .build()
            .unwrap();
        let group = Group::new("G1");
        let file = MuttonTextFile {
            combos: vec![combo],
            groups: vec![group],
        };
        let content = serde_json::to_string(&file).unwrap();
        let fmt = ImportManager::detect_format(&content).unwrap();
        assert_eq!(fmt, ImportFormat::MuttonTextJson);
    }

    #[test]
    fn test_detect_beeftext_csv() {
        let content = "Name,Keyword,Snippet,MatchingMode,Group\nSig,sig,hello,strict,G1";
        let fmt = ImportManager::detect_format(content).unwrap();
        assert_eq!(fmt, ImportFormat::BeeftextCsv);
    }

    #[test]
    fn test_detect_textexpander_csv() {
        let content = "Abbreviation,Content,Label\nsig,hello,Signature";
        let fmt = ImportManager::detect_format(content).unwrap();
        assert_eq!(fmt, ImportFormat::TextExpanderCsv);
    }

    #[test]
    fn test_detect_unrecognized() {
        let result = ImportManager::detect_format("just random text");
        assert!(result.is_err());
    }

    // ── Beeftext JSON Import ─────────────────────────────────────

    #[test]
    fn test_import_beeftext_json() {
        let content = r#"{
            "combos": [
                {"name":"Sig","keyword":"sig","snippet":"Best regards","matchingMode":"strict","group":"Email"},
                {"name":"Addr","keyword":"addr","snippet":"123 Main St","matchingMode":"loose","group":"Email"}
            ],
            "groups": [{"name":"Email"}]
        }"#;
        let result = ImportManager::import_beeftext_json(content, ConflictResolution::Skip).unwrap();
        assert_eq!(result.imported_count, 2);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.combos[0].keyword, "sig");
        assert_eq!(result.combos[1].matching_mode, MatchingMode::Loose);
    }

    #[test]
    fn test_import_beeftext_json_missing_keyword() {
        let content = r#"{"combos":[{"name":"Bad","snippet":"text"}],"groups":[]}"#;
        let result = ImportManager::import_beeftext_json(content, ConflictResolution::Skip).unwrap();
        assert_eq!(result.imported_count, 0);
        assert_eq!(result.skipped_count, 1);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_import_beeftext_json_conflict_rename() {
        let content = r#"{"combos":[{"keyword":"sig","snippet":"hello"}],"groups":[]}"#;
        let result =
            ImportManager::import_beeftext_json(content, ConflictResolution::Rename).unwrap();
        assert_eq!(result.combos[0].keyword, "sig-imported");
    }

    #[test]
    fn test_import_beeftext_json_invalid() {
        let result = ImportManager::import_beeftext_json("not json", ConflictResolution::Skip);
        assert!(result.is_err());
    }

    // ── Beeftext CSV Import ──────────────────────────────────────

    #[test]
    fn test_import_beeftext_csv() {
        let content = "Name,Keyword,Snippet,MatchingMode,Group\nSig,sig,hello,strict,Email";
        let result = ImportManager::import_beeftext_csv(content, ConflictResolution::Skip).unwrap();
        assert_eq!(result.imported_count, 1);
        assert_eq!(result.combos[0].keyword, "sig");
        assert_eq!(result.groups.len(), 1);
    }

    #[test]
    fn test_import_beeftext_csv_empty() {
        let content = "Name,Keyword,Snippet,MatchingMode,Group\n";
        let result = ImportManager::import_beeftext_csv(content, ConflictResolution::Skip).unwrap();
        assert_eq!(result.imported_count, 0);
    }

    #[test]
    fn test_import_beeftext_csv_too_few_fields() {
        let content = "Name,Keyword,Snippet,MatchingMode,Group\nSig,sig";
        let result = ImportManager::import_beeftext_csv(content, ConflictResolution::Skip).unwrap();
        assert_eq!(result.imported_count, 0);
        assert_eq!(result.skipped_count, 1);
    }

    // ── TextExpander CSV Import ──────────────────────────────────

    #[test]
    fn test_import_textexpander_csv() {
        let content = "Abbreviation,Content,Label\nsig,Best regards,Signature";
        let result =
            ImportManager::import_textexpander_csv(content, ConflictResolution::Skip).unwrap();
        assert_eq!(result.imported_count, 1);
        assert_eq!(result.combos[0].keyword, "sig");
        assert_eq!(result.combos[0].name, "Signature");
    }

    #[test]
    fn test_import_textexpander_csv_rename() {
        let content = "Abbreviation,Content,Label\nsig,hello,Sig";
        let result =
            ImportManager::import_textexpander_csv(content, ConflictResolution::Rename).unwrap();
        assert_eq!(result.combos[0].keyword, "sig-imported");
    }

    // ── MuttonText JSON Import ───────────────────────────────────

    #[test]
    fn test_import_muttontext_json_roundtrip() {
        let group = Group::new("Test");
        let combo = ComboBuilder::new()
            .keyword("sig")
            .snippet("hello")
            .group_id(group.id)
            .build()
            .unwrap();
        let file = MuttonTextFile {
            combos: vec![combo.clone()],
            groups: vec![group.clone()],
        };
        let json = serde_json::to_string(&file).unwrap();
        let result = ImportManager::import_muttontext_json(&json).unwrap();
        assert_eq!(result.imported_count, 1);
        assert_eq!(result.combos[0].keyword, "sig");
        assert_eq!(result.groups[0].name, "Test");
    }

    // ── Preview ──────────────────────────────────────────────────

    #[test]
    fn test_preview_beeftext_json() {
        let content = r#"{"combos":[{"keyword":"sig","snippet":"hello"}],"groups":[{"name":"G"}]}"#;
        let preview = ImportManager::preview_import(content).unwrap();
        assert_eq!(preview.format, ImportFormat::BeeftextJson);
        assert_eq!(preview.combo_count, 1);
        assert_eq!(preview.group_count, 1);
    }

    #[test]
    fn test_preview_csv() {
        let content = "Abbreviation,Content,Label\nsig,hello,Sig\naddr,123 Main,Addr";
        let preview = ImportManager::preview_import(content).unwrap();
        assert_eq!(preview.format, ImportFormat::TextExpanderCsv);
        assert_eq!(preview.combo_count, 2);
    }

    // ── CSV Parser ───────────────────────────────────────────────

    #[test]
    fn test_parse_csv_simple() {
        let fields = parse_csv_line("a,b,c");
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_csv_quoted() {
        let fields = parse_csv_line(r#""hello, world",b,c"#);
        assert_eq!(fields[0], "hello, world");
    }

    #[test]
    fn test_parse_csv_escaped_quotes() {
        let fields = parse_csv_line(r#""say ""hello""",b"#);
        assert_eq!(fields[0], r#"say "hello""#);
    }

    // ── Error Display ────────────────────────────────────────────

    #[test]
    fn test_import_error_display() {
        let err = ImportError::UnrecognizedFormat;
        assert_eq!(err.to_string(), "Unrecognized import format");
    }

    // ── Serialization ────────────────────────────────────────────

    #[test]
    fn test_import_format_serialization() {
        let json = serde_json::to_string(&ImportFormat::BeeftextJson).unwrap();
        assert_eq!(json, r#""beeftextJson""#);
    }

    #[test]
    fn test_conflict_resolution_serialization() {
        let json = serde_json::to_string(&ConflictResolution::Rename).unwrap();
        assert_eq!(json, r#""rename""#);
    }
}
