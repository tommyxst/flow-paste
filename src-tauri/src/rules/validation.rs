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
