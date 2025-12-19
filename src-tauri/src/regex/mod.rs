use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

const RULE_TIMEOUT_MS: u64 = 50;
const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub replacement: String,
    pub is_builtin: bool,
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
}

struct CompiledRule {
    rule: Rule,
    regex: Regex,
}

static BUILTIN_RULES: Lazy<Vec<CompiledRule>> = Lazy::new(|| {
    let rules = vec![
        Rule {
            id: "remove_empty_lines".to_string(),
            name: "Remove Empty Lines".to_string(),
            description: "Remove consecutive empty lines".to_string(),
            pattern: r"\n\s*\n+".to_string(),
            replacement: "\n".to_string(),
            is_builtin: true,
        },
        Rule {
            id: "trim_whitespace".to_string(),
            name: "Trim Whitespace".to_string(),
            description: "Remove leading/trailing whitespace from each line".to_string(),
            pattern: r"(?m)^[ \t]+|[ \t]+$".to_string(),
            replacement: "".to_string(),
            is_builtin: true,
        },
        Rule {
            id: "cjk_spacing".to_string(),
            name: "CJK Spacing".to_string(),
            description: "Add space between CJK and Western characters".to_string(),
            pattern: r"([\p{Han}\p{Hiragana}\p{Katakana}])([A-Za-z0-9])".to_string(),
            replacement: "$1 $2".to_string(),
            is_builtin: true,
        },
        Rule {
            id: "cjk_spacing_reverse".to_string(),
            name: "CJK Spacing Reverse".to_string(),
            description: "Add space between Western and CJK characters".to_string(),
            pattern: r"([A-Za-z0-9])([\p{Han}\p{Hiragana}\p{Katakana}])".to_string(),
            replacement: "$1 $2".to_string(),
            is_builtin: true,
        },
        Rule {
            id: "to_plain_text".to_string(),
            name: "To Plain Text".to_string(),
            description: "Remove markdown/HTML formatting".to_string(),
            pattern: r"(\*\*|__|~~|`|<[^>]+>|\[([^\]]+)\]\([^)]+\))".to_string(),
            replacement: "$2".to_string(),
            is_builtin: true,
        },
        Rule {
            id: "collapse_spaces".to_string(),
            name: "Collapse Spaces".to_string(),
            description: "Replace multiple spaces with single space".to_string(),
            pattern: r"[ \t]+".to_string(),
            replacement: " ".to_string(),
            is_builtin: true,
        },
    ];

    rules
        .into_iter()
        .filter_map(|rule| {
            match Regex::new(&rule.pattern) {
                Ok(regex) => Some(CompiledRule { rule, regex }),
                Err(e) => {
                    log::error!("Failed to compile builtin rule '{}': {}", rule.id, e);
                    None
                }
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
    let regex = Regex::new(&rule.pattern).map_err(|e| RegexError::InvalidPattern(e.to_string()))?;
    let compiled = CompiledRule {
        rule: rule.clone(),
        regex,
    };
    apply_compiled_rule(text, &compiled)
}

fn apply_compiled_rule(text: &str, compiled: &CompiledRule) -> Result<String, RegexError> {
    let start = Instant::now();
    let timeout = Duration::from_millis(RULE_TIMEOUT_MS);

    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for cap in compiled.regex.captures_iter(text) {
        if start.elapsed() > timeout {
            log::warn!("Rule '{}' timed out after {}ms", compiled.rule.id, RULE_TIMEOUT_MS);
            return Err(RegexError::Timeout);
        }

        let full_match = cap.get(0).unwrap();
        result.push_str(&text[last_end..full_match.start()]);

        // Use expand() for efficient replacement with capture groups
        cap.expand(&compiled.rule.replacement, &mut result);

        last_end = full_match.end();

        // Check output size limit
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
    fn test_cjk_spacing() {
        let text = "中文English混合";
        let result = apply_rule(text, "cjk_spacing").unwrap();
        let result = apply_rule(&result, "cjk_spacing_reverse").unwrap();
        assert_eq!(result, "中文 English 混合");
    }

    #[test]
    fn test_collapse_spaces() {
        let text = "hello    world";
        let result = apply_rule(text, "collapse_spaces").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_to_plain_text() {
        let text = "**bold** and [link](url)";
        let result = apply_rule(text, "to_plain_text").unwrap();
        assert_eq!(result, "bold and link");
    }

    #[test]
    fn test_rule_not_found() {
        let result = apply_rule("test", "nonexistent");
        assert!(matches!(result, Err(RegexError::RuleNotFound(_))));
    }

    #[test]
    fn test_get_builtin_rules() {
        let rules = get_builtin_rules();
        assert!(rules.len() >= 5);
        assert!(rules.iter().any(|r| r.id == "remove_empty_lines"));
        assert!(rules.iter().any(|r| r.id == "collapse_spaces"));
    }
}
