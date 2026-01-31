//! Variable evaluation system for MuttonText (Milestone 7).
//!
//! Parses and evaluates variable expressions in snippet text using the `#{name}`
//! and `#{name:param1:param2}` syntax. Supports date/time, clipboard, combo
//! references, cursor positioning, user input prompts, environment variables,
//! key simulation markers, and script stubs.

use std::collections::HashSet;
use std::env;
use std::fmt;

use chrono::{Duration, Local, NaiveDateTime};
use thiserror::Error;

// ─── Errors ──────────────────────────────────────────────────────────────────

/// Errors that can occur during variable parsing or evaluation.
#[derive(Debug, Error, PartialEq)]
pub enum VariableError {
    #[error("Unclosed variable expression starting at position {0}")]
    UnclosedVariable(usize),

    #[error("Empty variable name at position {0}")]
    EmptyName(usize),

    #[error("Recursive combo reference detected (depth {depth}): {keyword}")]
    RecursionDetected { keyword: String, depth: usize },

    #[error("Combo not found: {0}")]
    ComboNotFound(String),

    #[error("Invalid time shift: {0}")]
    InvalidTimeShift(String),

    #[error("Invalid delay value: {0}")]
    InvalidDelay(String),

    #[error("Invalid key count: {0}")]
    InvalidKeyCount(String),

    #[error("Access to environment variable '{0}' is not allowed")]
    EnvVarNotAllowed(String),

    #[error("Output size exceeds maximum allowed length of {max} characters (got {actual})")]
    OutputTooLarge { max: usize, actual: usize },

    #[error("Too many variables in snippet (max {max}, got {actual})")]
    TooManyVariables { max: usize, actual: usize },

    #[error("Script variables are not yet supported (security review pending)")]
    ScriptNotSupported,
}

// ─── Parsed token types ──────────────────────────────────────────────────────

/// A token produced by the variable parser.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Literal text (no variable).
    Literal(String),
    /// A variable reference with name and optional parameters.
    Variable { name: String, params: Vec<String> },
}

// ─── Key action types ────────────────────────────────────────────────────────

/// An action to simulate after text insertion.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    /// Press a single key the given number of times.
    KeyPress { key: String, count: u32 },
    /// Press a key combination (e.g. Ctrl+C).
    Shortcut { keys: String },
    /// Pause for the given number of milliseconds.
    Delay { ms: u64 },
}

impl fmt::Display for KeyAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyAction::KeyPress { key, count } => write!(f, "KeyPress({} x{})", key, count),
            KeyAction::Shortcut { keys } => write!(f, "Shortcut({})", keys),
            KeyAction::Delay { ms } => write!(f, "Delay({}ms)", ms),
        }
    }
}

// ─── Evaluation context & result ─────────────────────────────────────────────

/// Context provided to the evaluator for resolving variables.
pub struct EvalContext<'a> {
    /// Current clipboard text.
    pub clipboard_text: String,
    /// Lookup function: given a keyword, return the snippet text of that combo.
    pub combo_lookup: Box<dyn Fn(&str) -> Option<String> + 'a>,
    /// Current recursion depth (callers should start at 0).
    pub depth: usize,
    /// Set of keywords currently being expanded (for loop detection).
    pub expanding: HashSet<String>,
}

impl<'a> EvalContext<'a> {
    /// Create a new top-level evaluation context.
    pub fn new(
        clipboard_text: String,
        combo_lookup: impl Fn(&str) -> Option<String> + 'a,
    ) -> Self {
        Self {
            clipboard_text,
            combo_lookup: Box::new(combo_lookup),
            depth: 0,
            expanding: HashSet::new(),
        }
    }
}

/// Sentinel string embedded in expanded text to mark cursor position.
pub const CURSOR_MARKER: &str = "\x00CURSOR\x00";

/// Sentinel prefix for input prompts embedded in expanded text.
pub const INPUT_MARKER_PREFIX: &str = "\x00INPUT:";
pub const INPUT_MARKER_SUFFIX: &str = "\x00";

/// Result of evaluating a snippet's variables.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalResult {
    /// The fully expanded text (may contain CURSOR_MARKER / INPUT markers).
    pub text: String,
    /// Cursor position within `text` (byte offset), if `#{cursor}` was used.
    pub cursor_position: Option<usize>,
    /// Prompts for `#{input:prompt}` variables, in order of appearance.
    pub pending_inputs: Vec<String>,
    /// Key simulation actions, in order of appearance.
    pub key_actions: Vec<KeyAction>,
}

// ─── Parser ──────────────────────────────────────────────────────────────────

const MAX_RECURSION_DEPTH: usize = 10;
const MAX_OUTPUT_SIZE: usize = 1_000_000;
const MAX_KEY_COUNT: u32 = 50;
const MAX_DELAY_MS: u64 = 10_000;
const MAX_VARIABLES_PER_SNIPPET: usize = 100;

/// Allowlist of safe environment variables that can be accessed
const ALLOWED_ENV_VARS: &[&str] = &[
    "HOME",
    "USER",
    "USERNAME",
    "LANG",
    "SHELL",
    "TERM",
    "PATH",
    "COMPUTERNAME",
    "HOSTNAME",
    "OS",
    "PROCESSOR_ARCHITECTURE",
    "TMP",
    "TEMP",
    "TMPDIR",
];

/// Parse a snippet string into a sequence of literal and variable tokens.
///
/// Supports `#{name}`, `#{name:p1}`, `#{name:p1:p2}`, escape `\}` inside
/// variables, and `\\` anywhere.
pub fn parse_tokens(input: &str) -> Result<Vec<Token>, VariableError> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut literal = String::new();

    while i < len {
        // Check for `#{`
        if i + 1 < len && chars[i] == '#' && chars[i + 1] == '{' {
            // Flush accumulated literal
            if !literal.is_empty() {
                tokens.push(Token::Literal(std::mem::take(&mut literal)));
            }
            let start = i;
            i += 2; // skip `#{`

            // Read variable content until unescaped `}`
            let mut var_content = String::new();
            let mut found_close = false;
            while i < len {
                if chars[i] == '\\' && i + 1 < len {
                    let next = chars[i + 1];
                    if next == '}' || next == '\\' {
                        var_content.push(next);
                        i += 2;
                        continue;
                    }
                }
                if chars[i] == '}' {
                    found_close = true;
                    i += 1;
                    break;
                }
                var_content.push(chars[i]);
                i += 1;
            }
            if !found_close {
                return Err(VariableError::UnclosedVariable(start));
            }
            // Split on `:` to get name and params
            let parts: Vec<&str> = var_content.splitn(usize::MAX, ':').collect();
            let name = parts[0].to_string();
            if name.is_empty() {
                return Err(VariableError::EmptyName(start));
            }
            let params: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            tokens.push(Token::Variable { name, params });
        } else if chars[i] == '\\' && i + 1 < len && chars[i + 1] == '\\' {
            literal.push('\\');
            i += 2;
        } else {
            literal.push(chars[i]);
            i += 1;
        }
    }

    if !literal.is_empty() {
        tokens.push(Token::Literal(literal));
    }

    Ok(tokens)
}

// ─── Evaluator ───────────────────────────────────────────────────────────────

/// The main variable evaluator.
pub struct VariableEvaluator;

impl VariableEvaluator {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate all variables in a snippet string.
    pub fn evaluate(
        &self,
        snippet: &str,
        ctx: &mut EvalContext<'_>,
    ) -> Result<EvalResult, VariableError> {
        let tokens = parse_tokens(snippet)?;

        // Count variables to prevent fan-out attacks
        let variable_count = tokens
            .iter()
            .filter(|t| matches!(t, Token::Variable { .. }))
            .count();

        if variable_count > MAX_VARIABLES_PER_SNIPPET {
            return Err(VariableError::TooManyVariables {
                max: MAX_VARIABLES_PER_SNIPPET,
                actual: variable_count,
            });
        }

        let mut text = String::new();
        let mut cursor_position: Option<usize> = None;
        let mut pending_inputs: Vec<String> = Vec::new();
        let mut key_actions: Vec<KeyAction> = Vec::new();

        for token in &tokens {
            match token {
                Token::Literal(s) => text.push_str(s),
                Token::Variable { name, params } => {
                    self.eval_variable(
                        name,
                        params,
                        ctx,
                        &mut text,
                        &mut cursor_position,
                        &mut pending_inputs,
                        &mut key_actions,
                    )?;
                }
            }
        }

        // Check output size limit
        if text.len() > MAX_OUTPUT_SIZE {
            return Err(VariableError::OutputTooLarge {
                max: MAX_OUTPUT_SIZE,
                actual: text.len(),
            });
        }

        // Resolve CURSOR_MARKER to byte position
        if let Some(pos) = text.find(CURSOR_MARKER) {
            cursor_position = Some(pos);
            text = text.replacen(CURSOR_MARKER, "", 1);
        }

        Ok(EvalResult {
            text,
            cursor_position,
            pending_inputs,
            key_actions,
        })
    }

    fn eval_variable(
        &self,
        name: &str,
        params: &[String],
        ctx: &mut EvalContext<'_>,
        text: &mut String,
        _cursor_pos: &mut Option<usize>,
        pending_inputs: &mut Vec<String>,
        key_actions: &mut Vec<KeyAction>,
    ) -> Result<(), VariableError> {
        match name {
            // ── Clipboard ────────────────────────────────────────────
            "clipboard" => {
                text.push_str(&ctx.clipboard_text);
            }

            // ── Date/Time ────────────────────────────────────────────
            "date" => {
                let now = Local::now();
                text.push_str(&now.format("%Y-%m-%d").to_string());
            }
            "time" => {
                let now = Local::now();
                text.push_str(&now.format("%H:%M:%S").to_string());
            }
            "dateTime" => {
                if params.is_empty() {
                    let now = Local::now();
                    text.push_str(&now.format("%Y-%m-%d %H:%M:%S").to_string());
                } else if params.len() == 1 {
                    // #{dateTime:format}
                    let now = Local::now();
                    text.push_str(&now.format(&params[0]).to_string());
                } else if params.len() >= 2 {
                    // #{dateTime:shift:format}
                    let shifted = apply_time_shift(&params[0])?;
                    let fmt = if params.len() > 1 { &params[1] } else { "%Y-%m-%d %H:%M:%S" };
                    text.push_str(&shifted.format(fmt).to_string());
                }
            }

            // ── Combo references ─────────────────────────────────────
            "combo" | "lower" | "upper" => {
                if params.is_empty() {
                    return Err(VariableError::ComboNotFound(String::new()));
                }
                let keyword = &params[0];
                if ctx.depth >= MAX_RECURSION_DEPTH {
                    return Err(VariableError::RecursionDetected {
                        keyword: keyword.clone(),
                        depth: ctx.depth,
                    });
                }
                if ctx.expanding.contains(keyword.as_str()) {
                    return Err(VariableError::RecursionDetected {
                        keyword: keyword.clone(),
                        depth: ctx.depth,
                    });
                }
                let snippet_text = (ctx.combo_lookup)(keyword)
                    .ok_or_else(|| VariableError::ComboNotFound(keyword.clone()))?;

                // Recursively evaluate the referenced combo's snippet
                ctx.expanding.insert(keyword.clone());
                ctx.depth += 1;
                let sub_result = self.evaluate(&snippet_text, ctx)?;
                ctx.depth -= 1;
                ctx.expanding.remove(keyword.as_str());

                let expanded = match name {
                    "lower" => sub_result.text.to_lowercase(),
                    "upper" => sub_result.text.to_uppercase(),
                    _ => sub_result.text,
                };
                text.push_str(&expanded);
                pending_inputs.extend(sub_result.pending_inputs);
                key_actions.extend(sub_result.key_actions);
            }

            // ── Cursor ───────────────────────────────────────────────
            "cursor" => {
                text.push_str(CURSOR_MARKER);
            }

            // ── Input ────────────────────────────────────────────────
            "input" => {
                let prompt = if params.is_empty() {
                    "Enter value".to_string()
                } else {
                    params[0].clone()
                };
                pending_inputs.push(prompt.clone());
                // Insert a marker the UI layer will replace with user input
                text.push_str(INPUT_MARKER_PREFIX);
                text.push_str(&prompt);
                text.push_str(INPUT_MARKER_SUFFIX);
            }

            // ── Environment variable ─────────────────────────────────
            "envVar" => {
                if let Some(var_name) = params.first() {
                    // Check if the variable is in the allowlist
                    if !ALLOWED_ENV_VARS.contains(&var_name.as_str()) {
                        return Err(VariableError::EnvVarNotAllowed(var_name.clone()));
                    }
                    let val = env::var(var_name).unwrap_or_default();
                    text.push_str(&val);
                }
            }

            // ── Key simulation ───────────────────────────────────────
            "key" => {
                if let Some(key_name) = params.first() {
                    let count: u32 = if params.len() > 1 {
                        params[1]
                            .parse()
                            .map_err(|_| VariableError::InvalidKeyCount(params[1].clone()))?
                    } else {
                        1
                    };
                    // Cap key count to prevent resource exhaustion
                    if count > MAX_KEY_COUNT {
                        return Err(VariableError::InvalidKeyCount(format!(
                            "Count {} exceeds maximum of {}",
                            count, MAX_KEY_COUNT
                        )));
                    }
                    key_actions.push(KeyAction::KeyPress {
                        key: key_name.clone(),
                        count,
                    });
                }
            }
            "shortcut" => {
                if let Some(keys) = params.first() {
                    key_actions.push(KeyAction::Shortcut {
                        keys: keys.clone(),
                    });
                }
            }
            "delay" => {
                if let Some(ms_str) = params.first() {
                    let mut ms: u64 = ms_str
                        .parse()
                        .map_err(|_| VariableError::InvalidDelay(ms_str.clone()))?;
                    // Cap delay to maximum allowed
                    if ms > MAX_DELAY_MS {
                        tracing::warn!(
                            "Delay {} ms exceeds maximum {}, capping to {}",
                            ms,
                            MAX_DELAY_MS,
                            MAX_DELAY_MS
                        );
                        ms = MAX_DELAY_MS;
                    }
                    key_actions.push(KeyAction::Delay { ms });
                }
            }

            // ── Script stubs (MT-727–730) ────────────────────────────
            "script" | "shellScript" | "appleScript" | "powershell" => {
                // SECURITY: Script execution is not yet implemented.
                return Err(VariableError::ScriptNotSupported);
            }

            // ── Unknown variable → pass through as literal ───────────
            _ => {
                text.push_str("#{");
                text.push_str(name);
                for p in params {
                    text.push(':');
                    text.push_str(p);
                }
                text.push('}');
            }
        }

        Ok(())
    }
}

impl Default for VariableEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Time shift helper ──────────────────────────────────────────────────────

/// Parse a shift string like `+1d`, `-2h`, `+30m` and apply it to `Local::now()`.
fn apply_time_shift(shift: &str) -> Result<NaiveDateTime, VariableError> {
    let now = Local::now().naive_local();
    if shift.is_empty() {
        return Ok(now);
    }

    let (sign, rest) = if shift.starts_with('+') {
        (1i64, &shift[1..])
    } else if shift.starts_with('-') {
        (-1i64, &shift[1..])
    } else {
        (1i64, shift)
    };

    if rest.is_empty() {
        return Err(VariableError::InvalidTimeShift(shift.to_string()));
    }

    let unit = rest.chars().last().unwrap();
    let num_str = &rest[..rest.len() - unit.len_utf8()];
    let num: i64 = num_str
        .parse()
        .map_err(|_| VariableError::InvalidTimeShift(shift.to_string()))?;

    let duration = match unit {
        's' => Duration::seconds(sign * num),
        'm' => Duration::minutes(sign * num),
        'h' => Duration::hours(sign * num),
        'd' => Duration::days(sign * num),
        'w' => Duration::weeks(sign * num),
        'M' => Duration::days(sign * num * 30), // approximate months
        'y' => Duration::days(sign * num * 365), // approximate years
        _ => return Err(VariableError::InvalidTimeShift(shift.to_string())),
    };

    Ok(now + duration)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── MT-703: Parser unit tests ────────────────────────────────────

    #[test]
    fn test_parse_literal_only() {
        let tokens = parse_tokens("hello world").unwrap();
        assert_eq!(tokens, vec![Token::Literal("hello world".into())]);
    }

    #[test]
    fn test_parse_single_variable_no_params() {
        let tokens = parse_tokens("#{clipboard}").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Variable {
                name: "clipboard".into(),
                params: vec![]
            }]
        );
    }

    #[test]
    fn test_parse_variable_with_one_param() {
        let tokens = parse_tokens("#{dateTime:%Y}").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Variable {
                name: "dateTime".into(),
                params: vec!["%Y".into()]
            }]
        );
    }

    #[test]
    fn test_parse_variable_with_two_params() {
        let tokens = parse_tokens("#{dateTime:+1d:%Y-%m-%d}").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Variable {
                name: "dateTime".into(),
                params: vec!["+1d".into(), "%Y-%m-%d".into()]
            }]
        );
    }

    #[test]
    fn test_parse_mixed_literal_and_variables() {
        let tokens = parse_tokens("Hello #{clipboard}, today is #{date}!").unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Literal("Hello ".into()));
        assert_eq!(
            tokens[1],
            Token::Variable {
                name: "clipboard".into(),
                params: vec![]
            }
        );
        assert_eq!(tokens[2], Token::Literal(", today is ".into()));
        assert_eq!(
            tokens[3],
            Token::Variable {
                name: "date".into(),
                params: vec![]
            }
        );
        assert_eq!(tokens[4], Token::Literal("!".into()));
    }

    #[test]
    fn test_parse_unclosed_variable() {
        let err = parse_tokens("#{unclosed").unwrap_err();
        assert!(matches!(err, VariableError::UnclosedVariable(0)));
    }

    #[test]
    fn test_parse_empty_variable_name() {
        let err = parse_tokens("#{}").unwrap_err();
        assert!(matches!(err, VariableError::EmptyName(0)));
    }

    // ── MT-702: Escape sequence tests ────────────────────────────────

    #[test]
    fn test_parse_escaped_closing_brace_in_variable() {
        // #{name\}} should parse the name as "name}"
        let tokens = parse_tokens("#{name\\}}").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Variable {
                name: "name}".into(),
                params: vec![]
            }]
        );
    }

    #[test]
    fn test_parse_escaped_backslash_in_literal() {
        let tokens = parse_tokens("a\\\\b").unwrap();
        assert_eq!(tokens, vec![Token::Literal("a\\b".into())]);
    }

    #[test]
    fn test_parse_escaped_backslash_inside_variable() {
        let tokens = parse_tokens("#{a\\\\b}").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Variable {
                name: "a\\b".into(),
                params: vec![]
            }]
        );
    }

    // ── MT-704: Clipboard variable ───────────────────────────────────

    #[test]
    fn test_clipboard_variable() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new("copied text".into(), |_| None);
        let result = evaluator.evaluate("Pasted: #{clipboard}", &mut ctx).unwrap();
        assert_eq!(result.text, "Pasted: copied text");
    }

    #[test]
    fn test_clipboard_variable_empty() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{clipboard}", &mut ctx).unwrap();
        assert_eq!(result.text, "");
    }

    // ── MT-705–710: Date/time variable tests ─────────────────────────

    #[test]
    fn test_date_variable_format() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{date}", &mut ctx).unwrap();
        // Should be YYYY-MM-DD format
        assert_eq!(result.text.len(), 10);
        assert_eq!(&result.text[4..5], "-");
        assert_eq!(&result.text[7..8], "-");
    }

    #[test]
    fn test_time_variable_format() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{time}", &mut ctx).unwrap();
        // Should be HH:MM:SS format
        assert_eq!(result.text.len(), 8);
        assert_eq!(&result.text[2..3], ":");
        assert_eq!(&result.text[5..6], ":");
    }

    #[test]
    fn test_datetime_variable_default() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{dateTime}", &mut ctx).unwrap();
        // Should be YYYY-MM-DD HH:MM:SS format (19 chars)
        assert_eq!(result.text.len(), 19);
    }

    #[test]
    fn test_datetime_custom_format() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{dateTime:%Y}", &mut ctx).unwrap();
        assert_eq!(result.text.len(), 4); // just the year
    }

    #[test]
    fn test_datetime_with_shift_and_format() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator
            .evaluate("#{dateTime:+0d:%Y-%m-%d}", &mut ctx)
            .unwrap();
        // +0d should equal today
        let today = Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(result.text, today);
    }

    #[test]
    fn test_datetime_invalid_shift() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let err = evaluator
            .evaluate("#{dateTime:+abc:%Y}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::InvalidTimeShift(_)));
    }

    #[test]
    fn test_time_shift_positive_days() {
        let now = Local::now().naive_local();
        let shifted = apply_time_shift("+1d").unwrap();
        let diff = shifted - now;
        // Should be approximately 1 day (within a few seconds)
        assert!((diff.num_seconds() - 86400).abs() < 5);
    }

    #[test]
    fn test_time_shift_negative_hours() {
        let now = Local::now().naive_local();
        let shifted = apply_time_shift("-2h").unwrap();
        let diff = now - shifted;
        assert!((diff.num_seconds() - 7200).abs() < 5);
    }

    #[test]
    fn test_time_shift_minutes() {
        let now = Local::now().naive_local();
        let shifted = apply_time_shift("+30m").unwrap();
        let diff = shifted - now;
        assert!((diff.num_seconds() - 1800).abs() < 5);
    }

    #[test]
    fn test_time_shift_weeks() {
        let now = Local::now().naive_local();
        let shifted = apply_time_shift("+1w").unwrap();
        let diff = shifted - now;
        assert!((diff.num_seconds() - 604800).abs() < 5);
    }

    #[test]
    fn test_time_shift_seconds() {
        let now = Local::now().naive_local();
        let shifted = apply_time_shift("+10s").unwrap();
        let diff = shifted - now;
        assert!((diff.num_seconds() - 10).abs() < 5);
    }

    #[test]
    fn test_time_shift_months_approx() {
        let now = Local::now().naive_local();
        let shifted = apply_time_shift("+1M").unwrap();
        let diff = shifted - now;
        assert!((diff.num_days() - 30).abs() <= 1);
    }

    #[test]
    fn test_time_shift_years_approx() {
        let now = Local::now().naive_local();
        let shifted = apply_time_shift("+1y").unwrap();
        let diff = shifted - now;
        assert!((diff.num_days() - 365).abs() <= 1);
    }

    #[test]
    fn test_time_shift_empty_returns_now() {
        let before = Local::now().naive_local();
        let shifted = apply_time_shift("").unwrap();
        let after = Local::now().naive_local();
        assert!(shifted >= before && shifted <= after);
    }

    #[test]
    fn test_time_shift_invalid_unit() {
        let err = apply_time_shift("+1x").unwrap_err();
        assert!(matches!(err, VariableError::InvalidTimeShift(_)));
    }

    // ── MT-711–715: Combo reference variable tests ───────────────────

    #[test]
    fn test_combo_reference() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |kw| match kw {
            "sig" => Some("Best regards".into()),
            _ => None,
        });
        let result = evaluator
            .evaluate("Sign off: #{combo:sig}", &mut ctx)
            .unwrap();
        assert_eq!(result.text, "Sign off: Best regards");
    }

    #[test]
    fn test_combo_lower() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |kw| match kw {
            "greeting" => Some("Hello World".into()),
            _ => None,
        });
        let result = evaluator.evaluate("#{lower:greeting}", &mut ctx).unwrap();
        assert_eq!(result.text, "hello world");
    }

    #[test]
    fn test_combo_upper() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |kw| match kw {
            "greeting" => Some("Hello World".into()),
            _ => None,
        });
        let result = evaluator.evaluate("#{upper:greeting}", &mut ctx).unwrap();
        assert_eq!(result.text, "HELLO WORLD");
    }

    #[test]
    fn test_combo_not_found() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let err = evaluator
            .evaluate("#{combo:nonexistent}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::ComboNotFound(_)));
    }

    #[test]
    fn test_combo_recursion_detection() {
        let evaluator = VariableEvaluator::new();
        // Combo "a" references combo "b" which references combo "a"
        let mut ctx = EvalContext::new(String::new(), |kw| match kw {
            "aa" => Some("#{combo:bb}".into()),
            "bb" => Some("#{combo:aa}".into()),
            _ => None,
        });
        let err = evaluator.evaluate("#{combo:aa}", &mut ctx).unwrap_err();
        assert!(matches!(err, VariableError::RecursionDetected { .. }));
    }

    #[test]
    fn test_combo_max_depth() {
        let evaluator = VariableEvaluator::new();
        // Chain: c0 -> c1 -> c2 -> ... -> c11 (exceeds depth 10)
        let mut ctx = EvalContext::new(String::new(), |kw| {
            if let Some(n) = kw.strip_prefix("cc") {
                let n: usize = n.parse().ok()?;
                if n < 15 {
                    Some(format!("#{{combo:cc{}}}", n + 1))
                } else {
                    Some("end".into())
                }
            } else {
                None
            }
        });
        let err = evaluator.evaluate("#{combo:cc0}", &mut ctx).unwrap_err();
        assert!(matches!(err, VariableError::RecursionDetected { .. }));
    }

    #[test]
    fn test_combo_nested_evaluation() {
        // Combo that itself contains a variable
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new("clipboard_val".into(), |kw| match kw {
            "sig" => Some("Regards, #{clipboard}".into()),
            _ => None,
        });
        let result = evaluator.evaluate("#{combo:sig}", &mut ctx).unwrap();
        assert_eq!(result.text, "Regards, clipboard_val");
    }

    // ── MT-716–719: Interactive variable tests ───────────────────────

    #[test]
    fn test_cursor_variable() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator
            .evaluate("Hello #{cursor}world", &mut ctx)
            .unwrap();
        assert_eq!(result.text, "Hello world");
        assert_eq!(result.cursor_position, Some(6));
    }

    #[test]
    fn test_cursor_at_end() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("Hello#{cursor}", &mut ctx).unwrap();
        assert_eq!(result.text, "Hello");
        assert_eq!(result.cursor_position, Some(5));
    }

    #[test]
    fn test_input_variable_with_prompt() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator
            .evaluate("Dear #{input:Name}, hello", &mut ctx)
            .unwrap();
        assert!(result.pending_inputs.contains(&"Name".to_string()));
        assert_eq!(result.pending_inputs.len(), 1);
    }

    #[test]
    fn test_input_variable_default_prompt() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{input}", &mut ctx).unwrap();
        assert_eq!(result.pending_inputs, vec!["Enter value".to_string()]);
    }

    #[test]
    fn test_multiple_inputs() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator
            .evaluate("#{input:First} #{input:Last}", &mut ctx)
            .unwrap();
        assert_eq!(result.pending_inputs.len(), 2);
        assert_eq!(result.pending_inputs[0], "First");
        assert_eq!(result.pending_inputs[1], "Last");
    }

    // ── MT-720–721: Environment variable tests ───────────────────────

    #[test]
    fn test_env_var_existing() {
        // PATH should exist on all platforms
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{envVar:PATH}", &mut ctx).unwrap();
        assert!(!result.text.is_empty());
    }

    #[test]
    fn test_env_var_nonexistent() {
        // Non-allowlisted var should be rejected
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator
            .evaluate("#{envVar:MUTTONTEXT_NONEXISTENT_VAR_XYZ}", &mut ctx);
        assert!(result.is_err());
    }

    // ── MT-722–726: Key simulation tests ─────────────────────────────

    #[test]
    fn test_key_single_press() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{key:Enter}", &mut ctx).unwrap();
        assert_eq!(result.key_actions.len(), 1);
        assert_eq!(
            result.key_actions[0],
            KeyAction::KeyPress {
                key: "Enter".into(),
                count: 1
            }
        );
    }

    #[test]
    fn test_key_repeated_press() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{key:Tab:3}", &mut ctx).unwrap();
        assert_eq!(
            result.key_actions[0],
            KeyAction::KeyPress {
                key: "Tab".into(),
                count: 3
            }
        );
    }

    #[test]
    fn test_key_invalid_count() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let err = evaluator
            .evaluate("#{key:Tab:abc}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::InvalidKeyCount(_)));
    }

    #[test]
    fn test_shortcut() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator
            .evaluate("#{shortcut:Ctrl+C}", &mut ctx)
            .unwrap();
        assert_eq!(
            result.key_actions[0],
            KeyAction::Shortcut {
                keys: "Ctrl+C".into()
            }
        );
    }

    #[test]
    fn test_delay() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator.evaluate("#{delay:500}", &mut ctx).unwrap();
        assert_eq!(result.key_actions[0], KeyAction::Delay { ms: 500 });
    }

    #[test]
    fn test_delay_invalid() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let err = evaluator
            .evaluate("#{delay:notanumber}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::InvalidDelay(_)));
    }

    #[test]
    fn test_multiple_key_actions_in_order() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator
            .evaluate("text#{key:Tab}#{delay:100}#{shortcut:Ctrl+V}", &mut ctx)
            .unwrap();
        assert_eq!(result.text, "text");
        assert_eq!(result.key_actions.len(), 3);
        assert!(matches!(result.key_actions[0], KeyAction::KeyPress { .. }));
        assert!(matches!(result.key_actions[1], KeyAction::Delay { .. }));
        assert!(matches!(result.key_actions[2], KeyAction::Shortcut { .. }));
    }

    // ── MT-727–730: Script variable stubs ────────────────────────────

    #[test]
    fn test_script_stub() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let err = evaluator
            .evaluate("#{script:echo hello}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::ScriptNotSupported));
    }

    #[test]
    fn test_shell_script_stub() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let err = evaluator
            .evaluate("#{shellScript:ls}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::ScriptNotSupported));
    }

    #[test]
    fn test_powershell_stub() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let err = evaluator
            .evaluate("#{powershell:Get-Date}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::ScriptNotSupported));
    }

    // ── Unknown variable passthrough ─────────────────────────────────

    #[test]
    fn test_unknown_variable_passthrough() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        let result = evaluator
            .evaluate("#{unknownVar:param}", &mut ctx)
            .unwrap();
        assert_eq!(result.text, "#{unknownVar:param}");
    }

    // ── Complex integration test ─────────────────────────────────────

    #[test]
    fn test_complex_snippet() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new("clipboard_data".into(), |kw| match kw {
            "sig" => Some("John Doe".into()),
            _ => None,
        });
        let result = evaluator
            .evaluate(
                "Date: #{date}\nFrom: #{combo:sig}\nClipboard: #{clipboard}\n#{cursor}",
                &mut ctx,
            )
            .unwrap();
        assert!(result.text.contains("John Doe"));
        assert!(result.text.contains("clipboard_data"));
        assert!(result.cursor_position.is_some());
    }

    // ── KeyAction Display ────────────────────────────────────────────

    #[test]
    fn test_key_action_display() {
        let kp = KeyAction::KeyPress {
            key: "Enter".into(),
            count: 2,
        };
        assert_eq!(format!("{}", kp), "KeyPress(Enter x2)");

        let sc = KeyAction::Shortcut {
            keys: "Ctrl+C".into(),
        };
        assert_eq!(format!("{}", sc), "Shortcut(Ctrl+C)");

        let d = KeyAction::Delay { ms: 100 };
        assert_eq!(format!("{}", d), "Delay(100ms)");
    }

    // ── VariableError Display ────────────────────────────────────────

    #[test]
    fn test_error_messages() {
        let e = VariableError::UnclosedVariable(5);
        assert!(e.to_string().contains("5"));

        let e = VariableError::RecursionDetected {
            keyword: "aa".into(),
            depth: 3,
        };
        assert!(e.to_string().contains("aa"));
        assert!(e.to_string().contains("3"));
    }

    #[test]
    fn test_default_evaluator() {
        let _e = VariableEvaluator::default();
    }

    // ── Security tests ───────────────────────────────────────────────────

    #[test]
    fn test_env_var_allowlist_allowed() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        // HOME should be allowed
        let result = evaluator.evaluate("#{envVar:HOME}", &mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_env_var_allowlist_blocked() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        // AWS_SECRET_ACCESS_KEY should be blocked
        let err = evaluator
            .evaluate("#{envVar:AWS_SECRET_ACCESS_KEY}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::EnvVarNotAllowed(_)));
    }

    #[test]
    fn test_output_size_limit() {
        let evaluator = VariableEvaluator::new();
        // Create a snippet that would produce output larger than MAX_OUTPUT_SIZE
        let large_text = "x".repeat(500_000);
        let mut ctx = EvalContext::new(large_text.clone(), |_| None);
        // Two clipboard variables will exceed 1MB
        let err = evaluator
            .evaluate("#{clipboard}#{clipboard}#{clipboard}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::OutputTooLarge { .. }));
    }

    #[test]
    fn test_too_many_variables() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        // Create a snippet with more than MAX_VARIABLES_PER_SNIPPET (100)
        let mut snippet = String::new();
        for i in 0..101 {
            snippet.push_str(&format!("#{{clipboard}}"));
        }
        let err = evaluator.evaluate(&snippet, &mut ctx).unwrap_err();
        assert!(matches!(err, VariableError::TooManyVariables { .. }));
    }

    #[test]
    fn test_key_count_limit() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        // Try to simulate 1000 key presses (exceeds MAX_KEY_COUNT of 50)
        let err = evaluator
            .evaluate("#{key:Tab:1000}", &mut ctx)
            .unwrap_err();
        assert!(matches!(err, VariableError::InvalidKeyCount(_)));
    }

    #[test]
    fn test_delay_cap() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        // Request a 60 second delay (exceeds MAX_DELAY_MS of 10000)
        let result = evaluator.evaluate("#{delay:60000}", &mut ctx).unwrap();
        // Should succeed but cap the delay to 10000
        assert_eq!(result.key_actions.len(), 1);
        if let KeyAction::Delay { ms } = result.key_actions[0] {
            assert_eq!(ms, 10_000);
        } else {
            panic!("Expected Delay action");
        }
    }

    #[test]
    fn test_delay_within_limit() {
        let evaluator = VariableEvaluator::new();
        let mut ctx = EvalContext::new(String::new(), |_| None);
        // Request a 5 second delay (within limit)
        let result = evaluator.evaluate("#{delay:5000}", &mut ctx).unwrap();
        assert_eq!(result.key_actions.len(), 1);
        if let KeyAction::Delay { ms } = result.key_actions[0] {
            assert_eq!(ms, 5000);
        } else {
            panic!("Expected Delay action");
        }
    }
}
