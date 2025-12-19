use crate::regex::{self, Rule};

#[tauri::command]
pub fn get_builtin_rules() -> Vec<Rule> {
    regex::get_builtin_rules()
}

#[tauri::command]
pub fn apply_rule(text: String, rule_id: String) -> Result<String, String> {
    regex::apply_rule(&text, &rule_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_custom_rule(text: String, rule: Rule) -> Result<String, String> {
    regex::apply_custom_rule(&text, &rule).map_err(|e| e.to_string())
}
