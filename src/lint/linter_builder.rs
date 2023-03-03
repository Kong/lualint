use crate::rules::{Registry, Rule, RuleContext};

use super::Linter;

#[derive(Default)]
pub struct LinterBuilder {
    rule_registry: Registry,
}

impl LinterBuilder {

    pub fn with_rule<T>(mut self, rule_name: &str, rule_config: &serde_json::Value) -> Self
    where
        T: RuleContext + Rule,
    {
        let rule = T::apply(&mut self.rule_registry, rule_config);
        self.rule_registry.bind_ctx(rule_name, Box::new(rule));
        self
    }

    pub fn build(self) -> Linter {
        Linter { rule_registry: self.rule_registry }
    }
}
