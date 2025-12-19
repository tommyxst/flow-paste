use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::patterns::PIIType;
use super::scanner::{scan_pii, PIIItem, PIIScanResult};

const PLACEHOLDER_PREFIX: &str = "FP";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MaskMapping {
    pub mappings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskResult {
    pub masked: String,
    pub mapping: MaskMapping,
    pub scan_result: PIIScanResult,
}

fn generate_placeholder(pii_type: PIIType, index: usize) -> String {
    format!(
        "{{{{{}_{}_{}}}}}",
        PLACEHOLDER_PREFIX,
        pii_type.placeholder_prefix(),
        index.to_string().chars().map(|c| {
            match c {
                '0' => '0',
                '1' => '1',
                '2' => '2',
                '3' => '3',
                '4' => '4',
                '5' => '5',
                '6' => '6',
                '7' => '7',
                '8' => '8',
                '9' => '9',
                _ => c,
            }
        }).collect::<String>()
    )
}

pub fn mask_pii(text: &str) -> MaskResult {
    let scan_result = scan_pii(text);

    if !scan_result.has_pii {
        return MaskResult {
            masked: text.to_string(),
            mapping: MaskMapping::default(),
            scan_result,
        };
    }

    let mut masked = text.to_string();
    let mut mappings: HashMap<String, String> = HashMap::new();
    let mut type_counters: HashMap<PIIType, usize> = HashMap::new();

    // Process items in reverse order to preserve positions
    let mut items: Vec<&PIIItem> = scan_result.items.iter().collect();
    items.sort_by(|a, b| b.start.cmp(&a.start));

    for item in items {
        let counter = type_counters.entry(item.pii_type).or_insert(0);
        *counter += 1;

        let placeholder = generate_placeholder(item.pii_type, *counter);
        mappings.insert(placeholder.clone(), item.value.clone());

        // Replace in string
        masked.replace_range(item.start..item.end, &placeholder);
    }

    MaskResult {
        masked,
        mapping: MaskMapping { mappings },
        scan_result,
    }
}

pub fn restore_pii(text: &str, mapping: &MaskMapping) -> String {
    let mut restored = text.to_string();

    for (placeholder, original) in &mapping.mappings {
        restored = restored.replace(placeholder, original);
    }

    restored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_and_restore() {
        let original = "联系人：张三，手机：13800138000";
        let result = mask_pii(original);

        assert!(result.masked.contains("{{FP_PHONE_"));
        assert!(!result.masked.contains("13800138000"));

        let restored = restore_pii(&result.masked, &result.mapping);
        assert_eq!(restored, original);
    }

    #[test]
    fn test_mask_multiple_types() {
        let original = "邮箱：test@example.com，手机：13800138000";
        let result = mask_pii(original);

        assert!(result.masked.contains("{{FP_EMAIL_"));
        assert!(result.masked.contains("{{FP_PHONE_"));
        assert_eq!(result.mapping.mappings.len(), 2);

        let restored = restore_pii(&result.masked, &result.mapping);
        assert_eq!(restored, original);
    }

    #[test]
    fn test_mask_no_pii() {
        let original = "这是普通文本";
        let result = mask_pii(original);

        assert_eq!(result.masked, original);
        assert!(result.mapping.mappings.is_empty());
    }

    #[test]
    fn test_mask_multiple_same_type() {
        let original = "手机1：13800138001，手机2：13900139002";
        let result = mask_pii(original);

        assert_eq!(result.mapping.mappings.len(), 2);
        assert!(result.masked.contains("{{FP_PHONE_1}}"));
        assert!(result.masked.contains("{{FP_PHONE_2}}"));

        let restored = restore_pii(&result.masked, &result.mapping);
        assert_eq!(restored, original);
    }

    #[test]
    fn test_restore_partial() {
        // AI might modify text around placeholders
        let masked = "用户手机是 {{FP_PHONE_1}}，请核实";
        let mut mapping = MaskMapping::default();
        mapping.mappings.insert("{{FP_PHONE_1}}".to_string(), "13800138000".to_string());

        let restored = restore_pii(masked, &mapping);
        assert_eq!(restored, "用户手机是 13800138000，请核实");
    }
}
