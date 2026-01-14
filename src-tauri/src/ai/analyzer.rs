use crate::regex::Rule;
use super::intent::{ActionChip, ActionType, ContentType, detect_content_type};
use uuid::Uuid;

/// 数据驱动的意图分析器
pub struct IntentAnalyzer<'a> {
    rules: &'a [Rule],
    pinned_ids: &'a [String],
}

impl<'a> IntentAnalyzer<'a> {
    pub fn new(rules: &'a [Rule], pinned_ids: &'a [String]) -> Self {
        Self { rules, pinned_ids }
    }

    /// 分析文本并生成推荐的操作
    pub fn analyze(&self, text: &str) -> Vec<ActionChip> {
        if text.is_empty() {
            return vec![];
        }

        let content_type = detect_content_type(text);
        self.generate_chips(content_type)
    }

    fn generate_chips(&self, content_type: ContentType) -> Vec<ActionChip> {
        let mut chips = Vec::new();

        // 1. 首先添加置顶规则
        for (idx, pinned_id) in self.pinned_ids.iter().enumerate() {
            if let Some(rule) = self.rules.iter().find(|r| &r.id == pinned_id && r.enabled) {
                chips.push(self.rule_to_chip(rule, Some((idx + 1).to_string())));
            }
        }

        // 2. 根据内容类型添加匹配的规则
        let category = self.content_type_to_category(content_type);
        if let Some(cat) = category {
            for rule in self.rules.iter().filter(|r| {
                r.enabled
                    && r.category.as_deref() == Some(cat)
                    && !self.pinned_ids.contains(&r.id)
            }) {
                if chips.len() >= 5 {
                    break;
                }
                chips.push(self.rule_to_chip(rule, None));
            }
        }

        // 3. 如果chips不足，添加通用规则
        if chips.len() < 3 {
            // 先收集已有的 payload，避免借用冲突
            let existing_ids: Vec<String> = chips.iter().map(|c| c.payload.clone()).collect();
            for rule in self.rules.iter().filter(|r| {
                r.enabled
                    && r.category.as_deref() == Some("cleanup")
                    && !existing_ids.contains(&r.id)
            }) {
                if chips.len() >= 3 {
                    break;
                }
                chips.push(self.rule_to_chip(rule, None));
            }
        }

        // 重新分配快捷键
        for (idx, chip) in chips.iter_mut().enumerate() {
            if idx < 9 {
                chip.shortcut = Some((idx + 1).to_string());
            }
        }

        chips.truncate(5);
        chips
    }

    fn content_type_to_category(&self, content_type: ContentType) -> Option<&'static str> {
        match content_type {
            ContentType::Json => Some("json"),
            ContentType::List => Some("lines"),
            ContentType::Table => Some("lines"),
            ContentType::Code => Some("format"),
            ContentType::Prose => Some("cleanup"),
            ContentType::Unknown => None,
        }
    }

    fn rule_to_chip(&self, rule: &Rule, shortcut: Option<String>) -> ActionChip {
        ActionChip {
            id: Uuid::new_v4().to_string(),
            label: rule.name.clone(),
            action_type: ActionType::LocalRule,
            payload: rule.id.clone(),
            shortcut,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regex::{RuleOrigin, TransformationType};

    fn create_test_rules() -> Vec<Rule> {
        vec![
            Rule {
                id: "format_json".to_string(),
                name: "格式化 JSON".to_string(),
                description: "Format JSON".to_string(),
                transformation_type: TransformationType::JsonFormat,
                pattern: String::new(),
                replacement: String::new(),
                is_builtin: true,
                origin: RuleOrigin::Builtin,
                category: Some("json".to_string()),
                enabled: true,
                order: Some(1),
            },
            Rule {
                id: "sort_lines".to_string(),
                name: "行排序".to_string(),
                description: "Sort lines".to_string(),
                transformation_type: TransformationType::SortLines,
                pattern: String::new(),
                replacement: String::new(),
                is_builtin: true,
                origin: RuleOrigin::Builtin,
                category: Some("lines".to_string()),
                enabled: true,
                order: Some(2),
            },
            Rule {
                id: "remove_spaces".to_string(),
                name: "去除空格".to_string(),
                description: "Remove spaces".to_string(),
                transformation_type: TransformationType::RegexReplace,
                pattern: r"[ \t]+".to_string(),
                replacement: String::new(),
                is_builtin: true,
                origin: RuleOrigin::Builtin,
                category: Some("cleanup".to_string()),
                enabled: true,
                order: Some(3),
            },
        ]
    }

    #[test]
    fn test_analyzer_with_pinned() {
        let rules = create_test_rules();
        let pinned = vec!["sort_lines".to_string()];
        let analyzer = IntentAnalyzer::new(&rules, &pinned);

        let chips = analyzer.analyze(r#"{"test": 1}"#);
        assert!(!chips.is_empty());
        // 置顶规则应该在第一位
        assert_eq!(chips[0].payload, "sort_lines");
    }

    #[test]
    fn test_analyzer_json_content() {
        let rules = create_test_rules();
        let pinned: Vec<String> = vec![];
        let analyzer = IntentAnalyzer::new(&rules, &pinned);

        let chips = analyzer.analyze(r#"{"test": 1}"#);
        assert!(!chips.is_empty());
        // JSON 内容应该推荐 json 类别的规则
        assert!(chips.iter().any(|c| c.payload == "format_json"));
    }
}
