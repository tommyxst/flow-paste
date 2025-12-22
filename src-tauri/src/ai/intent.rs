use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Json,
    Code,
    Table,
    List,
    Prose,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionChip {
    pub id: String,
    pub label: String,
    pub action_type: ActionType,
    pub payload: String,
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ActionType {
    LocalRule,
    #[serde(rename = "AIPrompt")]
    AIPrompt,
}

// Regex patterns for content detection
static JSON_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*[\{\[]").unwrap());
static CODE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(function|class|def|pub fn|const|let|var|import|#include|package)\s+\w+").unwrap()
});
static LIST_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(\s*[-*+•]\s+|\s*\d+[.)]\s+)").unwrap()
});
static URL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s]+").unwrap()
});

pub fn detect_intent(text: &str) -> Vec<ActionChip> {
    if text.is_empty() {
        return vec![];
    }

    let content_type = detect_content_type(text);
    generate_action_chips(content_type, text)
}

fn detect_content_type(text: &str) -> ContentType {
    let trimmed = text.trim();

    // JSON detection (highest priority for structured data)
    if JSON_PATTERN.is_match(trimmed) {
        // Validate it's likely valid JSON
        if trimmed.ends_with('}') || trimmed.ends_with(']') {
            return ContentType::Json;
        }
    }

    // Code detection
    if CODE_PATTERN.is_match(text) {
        return ContentType::Code;
    }

    // Additional code detection: check for indentation patterns
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() >= 2 {
        let indented_lines = lines.iter().filter(|l| l.starts_with("    ") || l.starts_with("\t")).count();
        if indented_lines >= lines.len() / 3 && indented_lines >= 2 {
            return ContentType::Code;
        }
    }

    // Table detection (CSV/TSV)
    if lines.len() >= 2 {
        let has_tabs = lines.iter().filter(|l| l.contains('\t')).count();
        let has_commas = lines.iter().filter(|l| l.matches(',').count() >= 2).count();

        if has_tabs >= lines.len() / 2 || has_commas >= lines.len() / 2 {
            return ContentType::Table;
        }
    }

    // List detection
    if LIST_PATTERN.is_match(text) {
        let list_lines = LIST_PATTERN.find_iter(text).count();
        if list_lines >= 2 {
            return ContentType::List;
        }
    }

    // Prose detection (multiple sentences)
    let sentences = text.matches(&['.', '!', '?'][..]).count();
    if sentences >= 2 && text.len() > 50 {
        return ContentType::Prose;
    }

    ContentType::Unknown
}

fn generate_action_chips(content_type: ContentType, text: &str) -> Vec<ActionChip> {
    let mut chips = Vec::new();
    let mut shortcut_idx = 1;

    match content_type {
        ContentType::Json => {
            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "格式化 JSON".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Format this JSON with proper indentation".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "压缩 JSON".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Minify this JSON to a single line".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "转换为 YAML".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Convert this JSON to YAML format".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
        }
        ContentType::Code => {
            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "添加注释".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Add clear comments to explain this code".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "重构优化".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Refactor this code for better readability and performance".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "解释代码".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Explain what this code does in simple terms".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
        }
        ContentType::Table => {
            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "转换为 Markdown 表格".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Convert this table to Markdown table format".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "提取第一列".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Extract only the first column values".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "排序数据".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Sort this table by the first column".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
        }
        ContentType::List => {
            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "排序列表".to_string(),
                action_type: ActionType::LocalRule,
                payload: "sort_list".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "去重".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Remove duplicate items from this list".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "转为逗号分隔".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Convert this list to comma-separated values".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
        }
        ContentType::Prose => {
            let has_urls = URL_PATTERN.is_match(text);
            let is_long = text.len() > 500;

            if is_long {
                chips.push(ActionChip {
                    id: Uuid::new_v4().to_string(),
                    label: "总结要点".to_string(),
                    action_type: ActionType::AIPrompt,
                    payload: "Summarize the key points of this text in bullet points".to_string(),
                    shortcut: Some(shortcut_idx.to_string()),
                });
                shortcut_idx += 1;
            }

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "修正语法".to_string(),
                action_type: ActionType::AIPrompt,
                payload: "Fix grammar and spelling errors".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            if has_urls {
                chips.push(ActionChip {
                    id: Uuid::new_v4().to_string(),
                    label: "提取链接".to_string(),
                    action_type: ActionType::LocalRule,
                    payload: "extract_urls".to_string(),
                    shortcut: Some(shortcut_idx.to_string()),
                });
            } else {
                chips.push(ActionChip {
                    id: Uuid::new_v4().to_string(),
                    label: "翻译成英文".to_string(),
                    action_type: ActionType::AIPrompt,
                    payload: "Translate this text to English".to_string(),
                    shortcut: Some(shortcut_idx.to_string()),
                });
            }
        }
        ContentType::Unknown => {
            // Generic actions for unknown content
            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "去空行".to_string(),
                action_type: ActionType::LocalRule,
                payload: "remove_empty_lines".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "去首尾空格".to_string(),
                action_type: ActionType::LocalRule,
                payload: "trim_whitespace".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
            shortcut_idx += 1;

            chips.push(ActionChip {
                id: Uuid::new_v4().to_string(),
                label: "合并空格".to_string(),
                action_type: ActionType::LocalRule,
                payload: "collapse_spaces".to_string(),
                shortcut: Some(shortcut_idx.to_string()),
            });
        }
    }

    // Limit to 3 chips
    chips.truncate(3);
    chips
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_json() {
        let json = r#"{"name": "test", "value": 123}"#;
        assert_eq!(detect_content_type(json), ContentType::Json);

        let json_array = r#"[1, 2, 3]"#;
        assert_eq!(detect_content_type(json_array), ContentType::Json);
    }

    #[test]
    fn test_detect_code() {
        let code = r#"function hello() {
    console.log("Hello");
    return true;
}"#;
        assert_eq!(detect_content_type(code), ContentType::Code);
    }

    #[test]
    fn test_detect_table() {
        let table = "name\tage\nAlice\t30\nBob\t25";
        assert_eq!(detect_content_type(table), ContentType::Table);
    }

    #[test]
    fn test_detect_list() {
        let list = "- Item 1\n- Item 2\n- Item 3";
        assert_eq!(detect_content_type(list), ContentType::List);
    }

    #[test]
    fn test_detect_prose() {
        let prose = "This is a long sentence. And here is another one. This should be detected as prose.";
        assert_eq!(detect_content_type(prose), ContentType::Prose);
    }

    #[test]
    fn test_generate_chips_json() {
        let chips = detect_intent(r#"{"test": 1}"#);
        assert_eq!(chips.len(), 3);
        assert!(chips[0].label.contains("JSON"));
    }

    #[test]
    fn test_generate_chips_code() {
        let chips = detect_intent("function test() {\n    return 1;\n}");
        assert_eq!(chips.len(), 3);
        assert!(chips.iter().any(|c| c.label.contains("注释") || c.label.contains("重构")));
    }

    #[test]
    fn test_performance() {
        let text = "This is a test text with multiple sentences. It should be processed quickly.";
        let start = std::time::Instant::now();
        let _chips = detect_intent(text);
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 10, "Intent detection took {}ms", elapsed.as_millis());
    }
}
