use crate::config::AppConfig;
use crate::regex::{get_builtin_rules, Rule, RuleOrigin};

/// RuleCatalog：统一规则元数据来源
pub struct RuleCatalog;

impl RuleCatalog {
    /// 聚合所有规则（builtin + custom）
    pub fn all_rules(config: &AppConfig) -> Vec<Rule> {
        let mut rules = Vec::new();

        // 内置规则
        let mut builtin = get_builtin_rules();
        for r in &mut builtin {
            r.origin = RuleOrigin::Builtin;
        }
        rules.extend(builtin);

        // 自定义规则
        let mut custom = config.custom_rules.clone();
        for r in &mut custom {
            r.is_builtin = false;
            if r.origin == RuleOrigin::Builtin {
                r.origin = RuleOrigin::User;
            }
        }
        rules.extend(custom);

        rules
    }

    /// 查找规则
    pub fn find_rule<'a>(rules: &'a [Rule], rule_id: &str) -> Option<&'a Rule> {
        rules.iter().find(|r| r.id == rule_id)
    }
}
