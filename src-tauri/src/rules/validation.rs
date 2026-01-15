use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::regex::{Rule, TransformationType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// 后端权威校验
pub fn validate_rule(rule: &Rule) -> RuleValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 基础校验
    if rule.id.trim().is_empty() {
        errors.push("id cannot be empty".to_string());
    }
    if rule.name.trim().is_empty() {
        errors.push("name cannot be empty".to_string());
    }

    // 类型特定校验
    match rule.transformation_type {
        TransformationType::RegexReplace => {
            if rule.pattern.trim().is_empty() {
                errors.push("pattern cannot be empty".to_string());
            } else if rule.pattern.len() > 500 {
                errors.push("pattern too long".to_string());
            } else if let Err(e) = Regex::new(&rule.pattern) {
                errors.push(format!("invalid regex: {}", e));
            }

            // replacement 语法检查
            if !rule.replacement.is_empty() {
                if rule.replacement.contains("\\U") ||
                   rule.replacement.contains("\\L") ||
                   rule.replacement.contains("\\E") {
                    errors.push("unsupported \\U/\\L/\\E syntax".to_string());
                }
            }
        }
        _ => {
            if !rule.pattern.trim().is_empty() {
                warnings.push("pattern ignored for this type".to_string());
            }
        }
    }

    RuleValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_rule(transformation_type: TransformationType) -> Rule {
        Rule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: String::new(),
            transformation_type,
            pattern: String::new(),
            replacement: String::new(),
            is_builtin: false,
            origin: crate::regex::RuleOrigin::User,
            category: None,
            enabled: true,
            order: None,
        }
    }

    fn assert_has_error(result: &RuleValidationResult, expected: &str) {
        assert!(
            result.errors.iter().any(|e| e == expected),
            "expected error {:?}, got {:?}",
            expected,
            result.errors
        );
    }

    fn assert_has_warning(result: &RuleValidationResult, expected: &str) {
        assert!(
            result.warnings.iter().any(|w| w == expected),
            "expected warning {:?}, got {:?}",
            expected,
            result.warnings
        );
    }

    #[test]
    fn test_validate_rule_regex_replace_valid() {
        let mut rule = base_rule(TransformationType::RegexReplace);
        rule.pattern = r"\d+".to_string();
        rule.replacement = String::new();

        let result = validate_rule(&rule);

        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_rule_empty_id() {
        let mut rule = base_rule(TransformationType::RegexReplace);
        rule.id = "   ".to_string();
        rule.pattern = "a".to_string();

        let result = validate_rule(&rule);

        assert!(!result.valid);
        assert_has_error(&result, "id cannot be empty");
    }

    #[test]
    fn test_validate_rule_empty_name() {
        let mut rule = base_rule(TransformationType::RegexReplace);
        rule.name = "\n\t".to_string();
        rule.pattern = "a".to_string();

        let result = validate_rule(&rule);

        assert!(!result.valid);
        assert_has_error(&result, "name cannot be empty");
    }

    #[test]
    fn test_validate_rule_regex_replace_empty_pattern() {
        let mut rule = base_rule(TransformationType::RegexReplace);
        rule.pattern = "   ".to_string();

        let result = validate_rule(&rule);

        assert!(!result.valid);
        assert_has_error(&result, "pattern cannot be empty");
    }

    #[test]
    fn test_validate_rule_regex_replace_pattern_too_long() {
        let mut rule = base_rule(TransformationType::RegexReplace);
        rule.pattern = "a".repeat(501);

        let result = validate_rule(&rule);

        assert!(!result.valid);
        assert_has_error(&result, "pattern too long");
    }

    #[test]
    fn test_validate_rule_regex_replace_pattern_length_500_ok() {
        let mut rule = base_rule(TransformationType::RegexReplace);
        rule.pattern = "a".repeat(500);

        let result = validate_rule(&rule);

        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_rule_regex_replace_invalid_regex() {
        let mut rule = base_rule(TransformationType::RegexReplace);
        rule.pattern = "(".to_string();

        let result = validate_rule(&rule);

        assert!(!result.valid);
        assert!(
            result.errors.iter().any(|e| e.starts_with("invalid regex:")),
            "expected invalid regex error, got {:?}",
            result.errors
        );
    }

    #[test]
    fn test_validate_rule_regex_replace_unsupported_replacement_syntax() {
        let mut rule = base_rule(TransformationType::RegexReplace);
        rule.pattern = "a".to_string();
        rule.replacement = r"\Ufoo".to_string();

        let result = validate_rule(&rule);

        assert!(!result.valid);
        assert_has_error(&result, "unsupported \\U/\\L/\\E syntax");
    }

    #[test]
    fn test_validate_rule_non_regex_type_pattern_nonempty_warns() {
        let mut rule = base_rule(TransformationType::JsonFormat);
        rule.pattern = "should_be_ignored".to_string();

        let result = validate_rule(&rule);

        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert_has_warning(&result, "pattern ignored for this type");
    }

    #[test]
    fn test_validate_rule_non_regex_type_empty_pattern_no_warning() {
        let mut rule = base_rule(TransformationType::SortLines);
        rule.pattern = "   ".to_string();

        let result = validate_rule(&rule);

        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }
}
