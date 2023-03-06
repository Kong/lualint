use super::{LintReport, NodeWrapper, Registry, Rule, RuleContext, RuleInfo};

decl_rule!(operator_spacing, "Operator spacing", "20230224", "");

pub struct OperatorSpacing {
    pub reports: Vec<LintReport>,
}

impl Rule for OperatorSpacing {
    fn apply(rules: &mut Registry, config: &serde_json::Value) -> Self {
        let rule_name = "operator_spacing";

        Self { reports: vec![] }
    }

    fn context(&self) -> &dyn RuleContext {
        self
    }
}

impl RuleContext for OperatorSpacing {
    fn get_reports(&self) -> &Vec<LintReport> {
        &self.reports
    }
}
