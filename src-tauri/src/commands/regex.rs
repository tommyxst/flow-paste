use crate::config::ConfigManager;
use crate::regex::{self, Rule, RuleOrigin};
use crate::rules::{validate_rule, RuleCatalog, RuleValidationResult};
use tauri::State;

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

/// 列出所有规则（builtin + custom）
#[tauri::command]
pub fn list_all_rules(config_manager: State<ConfigManager>) -> Result<Vec<Rule>, String> {
    let config = config_manager.get_config().map_err(|e| e.to_string())?;
    Ok(RuleCatalog::all_rules(&config))
}

/// 创建或更新自定义规则
#[tauri::command]
pub fn upsert_rule(
    config_manager: State<ConfigManager>,
    mut rule: Rule,
) -> Result<RuleValidationResult, String> {
    // 校验规则
    let validation = validate_rule(&rule);
    if !validation.valid {
        return Ok(validation);
    }

    // 确保自定义规则的属性正确
    rule.is_builtin = false;
    if rule.origin == RuleOrigin::Builtin {
        rule.origin = RuleOrigin::User;
    }

    let mut config = config_manager.get_config().map_err(|e| e.to_string())?;

    // 查找是否已存在
    if let Some(pos) = config.custom_rules.iter().position(|r| r.id == rule.id) {
        config.custom_rules[pos] = rule;
    } else {
        config.custom_rules.push(rule);
    }

    config_manager.set_config(&config).map_err(|e| e.to_string())?;
    Ok(validation)
}

/// 删除自定义规则
#[tauri::command]
pub fn delete_rule(
    config_manager: State<ConfigManager>,
    rule_id: String,
) -> Result<bool, String> {
    let mut config = config_manager.get_config().map_err(|e| e.to_string())?;

    let original_len = config.custom_rules.len();
    config.custom_rules.retain(|r| r.id != rule_id);

    if config.custom_rules.len() < original_len {
        // 同时从 pinned_rule_ids 中移除
        config.pinned_rule_ids.retain(|id| id != &rule_id);
        config_manager.set_config(&config).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 重排序置顶规则
#[tauri::command]
pub fn reorder_pinned_rules(
    config_manager: State<ConfigManager>,
    rule_ids: Vec<String>,
) -> Result<(), String> {
    let mut config = config_manager.get_config().map_err(|e| e.to_string())?;
    config.pinned_rule_ids = rule_ids;
    config_manager.set_config(&config).map_err(|e| e.to_string())
}

/// 校验规则（不保存）
#[tauri::command]
pub fn validate_rule_cmd(rule: Rule) -> RuleValidationResult {
    validate_rule(&rule)
}
