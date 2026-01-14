use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use thiserror::Error;

const RULE_TIMEOUT_MS: u64 = 50;
const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransformationType {
    RegexReplace,
    JsonFormat,
    JsonMinify,
    SortLines,
    DedupeLines,
    ToUppercase,
    ToLowercase,
}

impl Default for TransformationType {
    fn default() -> Self {
        Self::RegexReplace
    }
}

/// 规则来源
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuleOrigin {
    Builtin,
    #[default]
    User,
    Ai,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub transformation_type: TransformationType,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub replacement: String,
    pub is_builtin: bool,
    #[serde(default)]
    pub origin: RuleOrigin,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Error)]
pub enum RegexError {
    #[error("invalid regex pattern: {0}")]
    InvalidPattern(String),
    #[error("rule not found: {0}")]
    RuleNotFound(String),
    #[error("rule execution timeout")]
    Timeout,
    #[error("output exceeds size limit")]
    OutputTooLarge,
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
}

struct CompiledRule {
    rule: Rule,
    regex: Option<Regex>,
}

static BUILTIN_RULES: Lazy<Vec<CompiledRule>> = Lazy::new(|| {
    let rules = vec![
        Rule {
            id: "remove_spaces".to_string(),
            name: "去除空格".to_string(),
            description: "Remove all spaces and tabs".to_string(),
            transformation_type: TransformationType::RegexReplace,
            pattern: r"[ \t]+".to_string(),
            replacement: String::new(),
            is_builtin: true,
            origin: RuleOrigin::Builtin,
            category: Some("cleanup".to_string()),
            enabled: true,
            order: Some(1),
        },
        Rule {
            id: "remove_newlines".to_string(),
            name: "去除换行".to_string(),
            description: "Remove all newlines".to_string(),
            transformation_type: TransformationType::RegexReplace,
            pattern: r"[\r\n]+".to_string(),
            replacement: String::new(),
            is_builtin: true,
            origin: RuleOrigin::Builtin,
            category: Some("cleanup".to_string()),
            enabled: true,
            order: Some(2),
        },
        Rule {
            id: "remove_empty_lines".to_string(),
            name: "去空行".to_string(),
            description: "Remove consecutive empty lines".to_string(),
            transformation_type: TransformationType::RegexReplace,
            pattern: r"\n\s*\n+".to_string(),
            replacement: "\n".to_string(),
            is_builtin: true,
            origin: RuleOrigin::Builtin,
            category: Some("cleanup".to_string()),
            enabled: true,
            order: Some(3),
        },
        Rule {
            id: "to_plain_text".to_string(),
            name: "转纯文本".to_string(),
            description: "Remove markdown/HTML formatting".to_string(),
            transformation_type: TransformationType::RegexReplace,
            pattern: r"(\*\*|__|~~|`|<[^>]+>|\[([^\]]+)\]\([^)]+\))".to_string(),
            replacement: "$2".to_string(),
            is_builtin: true,
            origin: RuleOrigin::Builtin,
            category: Some("format".to_string()),
            enabled: true,
            order: Some(4),
        },
        Rule {
            id: "format_json".to_string(),
            name: "格式化 JSON".to_string(),
            description: "Format JSON with indentation".to_string(),
            transformation_type: TransformationType::JsonFormat,
            pattern: String::new(),
            replacement: String::new(),
            is_builtin: true,
            origin: RuleOrigin::Builtin,
            category: Some("json".to_string()),
            enabled: true,
            order: Some(5),
        },
        Rule {
            id: "sort_lines".to_string(),
            name: "行排序".to_string(),
            description: "Sort lines alphabetically".to_string(),
            transformation_type: TransformationType::SortLines,
            pattern: String::new(),
            replacement: String::new(),
            is_builtin: true,
            origin: RuleOrigin::Builtin,
            category: Some("lines".to_string()),
            enabled: true,
            order: Some(6),
        },
        Rule {
            id: "dedupe_lines".to_string(),
            name: "行去重".to_string(),
            description: "Remove duplicate lines".to_string(),
            transformation_type: TransformationType::DedupeLines,
            pattern: String::new(),
            replacement: String::new(),
            is_builtin: true,
            origin: RuleOrigin::Builtin,
            category: Some("lines".to_string()),
            enabled: true,
            order: Some(7),
        },
    ];

    rules
        .into_iter()
        .filter_map(|rule| {
            match rule.transformation_type {
                TransformationType::RegexReplace => {
                    match Regex::new(&rule.pattern) {
                        Ok(regex) => Some(CompiledRule { rule, regex: Some(regex) }),
                        Err(e) => {
                            log::error!("Failed to compile builtin rule '{}': {}", rule.id, e);
                            None
                        }
                    }
                }
                _ => Some(CompiledRule { rule, regex: None }),
            }
        })
        .collect()
});

static RULE_INDEX: Lazy<HashMap<String, usize>> = Lazy::new(|| {
    BUILTIN_RULES
        .iter()
        .enumerate()
        .map(|(i, r)| (r.rule.id.clone(), i))
        .collect()
});

pub fn get_builtin_rules() -> Vec<Rule> {
    BUILTIN_RULES.iter().map(|r| r.rule.clone()).collect()
}

pub fn apply_rule(text: &str, rule_id: &str) -> Result<String, RegexError> {
    let idx = RULE_INDEX
        .get(rule_id)
        .ok_or_else(|| RegexError::RuleNotFound(rule_id.to_string()))?;
    let compiled = &BUILTIN_RULES[*idx];
    apply_compiled_rule(text, compiled)
}

pub fn apply_custom_rule(text: &str, rule: &Rule) -> Result<String, RegexError> {
    match rule.transformation_type {
        TransformationType::RegexReplace => {
            let regex = Regex::new(&rule.pattern)
                .map_err(|e| RegexError::InvalidPattern(e.to_string()))?;
            let compiled = CompiledRule { rule: rule.clone(), regex: Some(regex) };
            apply_compiled_rule(text, &compiled)
        }
        _ => {
            let compiled = CompiledRule { rule: rule.clone(), regex: None };
            apply_compiled_rule(text, &compiled)
        }
    }
}

fn apply_compiled_rule(text: &str, compiled: &CompiledRule) -> Result<String, RegexError> {
    match compiled.rule.transformation_type {
        TransformationType::JsonFormat => format_json(text),
        TransformationType::JsonMinify => minify_json(text),
        TransformationType::SortLines => Ok(sort_lines(text)),
        TransformationType::DedupeLines => Ok(dedupe_lines(text)),
        TransformationType::ToUppercase => Ok(text.to_uppercase()),
        TransformationType::ToLowercase => Ok(text.to_lowercase()),
        TransformationType::RegexReplace => apply_regex_rule(text, compiled),
    }
}

fn format_json(text: &str) -> Result<String, RegexError> {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(value) => serde_json::to_string_pretty(&value)
            .map_err(|e| RegexError::InvalidJson(e.to_string())),
        Err(_) => {
            let wrapped = serde_json::json!({ "text": text.trim() });
            serde_json::to_string_pretty(&wrapped)
                .map_err(|e| RegexError::InvalidJson(e.to_string()))
        }
    }
}

fn minify_json(text: &str) -> Result<String, RegexError> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| RegexError::InvalidJson(e.to_string()))?;
    serde_json::to_string(&value)
        .map_err(|e| RegexError::InvalidJson(e.to_string()))
}

fn sort_lines(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    lines.sort();
    lines.join("\n")
}

fn dedupe_lines(text: &str) -> String {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for line in text.lines() {
        if seen.insert(line) {
            result.push(line);
        }
    }
    result.join("\n")
}

fn apply_regex_rule(text: &str, compiled: &CompiledRule) -> Result<String, RegexError> {
    let regex = compiled.regex.as_ref()
        .ok_or_else(|| RegexError::InvalidPattern("No regex for regex rule".to_string()))?;

    let start = Instant::now();
    let timeout = Duration::from_millis(RULE_TIMEOUT_MS);

    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for cap in regex.captures_iter(text) {
        if start.elapsed() > timeout {
            log::warn!("Rule '{}' timed out after {}ms", compiled.rule.id, RULE_TIMEOUT_MS);
            return Err(RegexError::Timeout);
        }

        let full_match = cap.get(0).unwrap();
        result.push_str(&text[last_end..full_match.start()]);
        cap.expand(&compiled.rule.replacement, &mut result);
        last_end = full_match.end();

        if result.len() > MAX_OUTPUT_SIZE {
            log::warn!("Rule '{}' output exceeded size limit", compiled.rule.id);
            return Err(RegexError::OutputTooLarge);
        }
    }

    result.push_str(&text[last_end..]);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_empty_lines() {
        let text = "line1\n\n\nline2\n\nline3";
        let result = apply_rule(text, "remove_empty_lines").unwrap();
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn test_trim_whitespace() {
        let text = "  hello  \n  world  ";
        let result = apply_rule(text, "trim_whitespace").unwrap();
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn test_collapse_spaces() {
        let text = "hello    world";
        let result = apply_rule(text, "collapse_spaces").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_cjk_spacing() {
        let text = "中文English混合";
        let result = apply_rule(text, "cjk_spacing").unwrap();
        assert_eq!(result, "中文 English 混合");
    }

    #[test]
    fn test_to_plain_text() {
        let text = "**bold** and [link](url)";
        let result = apply_rule(text, "to_plain_text").unwrap();
        assert_eq!(result, "bold and link");
    }

    #[test]
    fn test_format_json() {
        let text = r#"{"name":"test","value":123}"#;
        let result = apply_rule(text, "format_json").unwrap();
        assert!(result.contains("\n"));
    }

    #[test]
    fn test_minify_json() {
        let text = "{\n  \"name\": \"test\"\n}";
        let result = apply_rule(text, "minify_json").unwrap();
        assert!(!result.contains("\n"));
    }

    #[test]
    fn test_sort_lines() {
        let text = "banana\napple\ncherry";
        let result = apply_rule(text, "sort_lines").unwrap();
        assert_eq!(result, "apple\nbanana\ncherry");
    }

    #[test]
    fn test_dedupe_lines() {
        let text = "apple\nbanana\napple\ncherry";
        let result = apply_rule(text, "dedupe_lines").unwrap();
        assert_eq!(result, "apple\nbanana\ncherry");
    }

    #[test]
    fn test_rule_not_found() {
        let result = apply_rule("test", "nonexistent");
        assert!(matches!(result, Err(RegexError::RuleNotFound(_))));
    }

    #[test]
    fn test_get_builtin_rules() {
        let rules = get_builtin_rules();
        assert!(rules.len() >= 9);
        assert!(rules.iter().any(|r| r.id == "format_json"));
        assert!(rules.iter().any(|r| r.id == "sort_lines"));
    }
}
