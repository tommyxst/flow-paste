use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::patterns::{luhn_check, PIIType, PII_PATTERNS};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PIIItem {
    pub pii_type: PIIType,
    pub value: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PIIScanResult {
    pub has_pii: bool,
    pub items: Vec<PIIItem>,
}

pub fn scan_pii(text: &str) -> PIIScanResult {
    let mut items: Vec<PIIItem> = Vec::new();
    let mut covered_ranges: HashSet<(usize, usize)> = HashSet::new();

    // Sort patterns by priority (highest first)
    let mut patterns: Vec<_> = PII_PATTERNS.iter().collect();
    patterns.sort_by(|a, b| b.priority.cmp(&a.priority));

    for pattern in patterns {
        for mat in pattern.regex.find_iter(text) {
            let start = mat.start();
            let end = mat.end();
            let value = mat.as_str().to_string();

            // Skip if overlaps with existing match
            let overlaps = covered_ranges.iter().any(|&(s, e)| {
                start < e && end > s
            });

            if overlaps {
                continue;
            }

            // Additional validation for specific types
            let is_valid = match pattern.pii_type {
                PIIType::BankCard => luhn_check(&value),
                _ => true,
            };

            if is_valid {
                covered_ranges.insert((start, end));
                items.push(PIIItem {
                    pii_type: pattern.pii_type,
                    value,
                    start,
                    end,
                });
            }
        }
    }

    // Sort by position for consistent ordering
    items.sort_by_key(|item| item.start);

    PIIScanResult {
        has_pii: !items.is_empty(),
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_multiple_pii() {
        let text = "联系人：张三，手机：13800138000，邮箱：test@example.com";
        let result = scan_pii(text);

        assert!(result.has_pii);
        assert_eq!(result.items.len(), 2);

        let phone = result.items.iter().find(|i| i.pii_type == PIIType::Phone);
        assert!(phone.is_some());
        assert_eq!(phone.unwrap().value, "13800138000");

        let email = result.items.iter().find(|i| i.pii_type == PIIType::Email);
        assert!(email.is_some());
        assert_eq!(email.unwrap().value, "test@example.com");
    }

    #[test]
    fn test_scan_no_pii() {
        let text = "这是一段普通文本，没有敏感信息。";
        let result = scan_pii(text);

        assert!(!result.has_pii);
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_scan_idcard() {
        let text = "身份证号：110101199003074518";
        let result = scan_pii(text);

        assert!(result.has_pii);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].pii_type, PIIType::IDCard);
    }

    #[test]
    fn test_scan_apikey() {
        let text = "API密钥：sk-abcdefghijklmnopqrstuvwxyz123456";
        let result = scan_pii(text);

        assert!(result.has_pii);
        assert_eq!(result.items[0].pii_type, PIIType::APIKey);
    }

    #[test]
    fn test_no_overlap() {
        // ID card should win over phone due to higher priority
        let text = "110101199003074518";
        let result = scan_pii(text);

        // This looks like an ID card, not a phone
        assert!(result.has_pii);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].pii_type, PIIType::IDCard);
    }
}
