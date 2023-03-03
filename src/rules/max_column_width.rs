use super::{LintReport, NodeWrapper, Registry, Rule, RuleContext, RuleInfo};

decl_rule!(max_column_width, "Maximum column width", "20230224", "max_col: 80");
pub struct MaxColumnWidth {
    pub reports: Vec<LintReport>,

    max_column_width: usize,

    _last_check_line: usize,
}

impl Rule for MaxColumnWidth {
    fn apply(rules: &mut Registry, config: &serde_json::Value) -> Self {
        let rule_name = "max_column_width";
        rules.listen_token(rule_name, Self::on_token);

        let max_column_width = config["max_col"].as_u64().unwrap_or(80) as usize;

        Self { reports: vec![], max_column_width, _last_check_line: 0 }
    }

    fn context(&self) -> &dyn RuleContext {
        self
    }
}

impl RuleContext for MaxColumnWidth {
    fn get_reports(&self) -> &Vec<LintReport> {
        &self.reports
    }
}

impl MaxColumnWidth {
    pub fn apply(rules: &mut Registry, max_column_width: usize) -> Self {
        let rule_name = "max_column_width";
        rules.listen_token(rule_name, Self::on_token);

        Self { reports: vec![], max_column_width, _last_check_line: 0 }
    }

    pub fn on_token(rctx: &mut dyn RuleContext, token_w: NodeWrapper) -> NodeWrapper {
        let ctx: &mut MaxColumnWidth = rctx.downcast_mut().unwrap();
        let token = rule_cast!(token_w, NodeWrapper::Token);

        let real_len = token.end_position().character();

        if real_len > ctx.max_column_width {
            // never report the same line twice
            if ctx._last_check_line == token.end_position().line() {
                return NodeWrapper::Token(token);
            }
            ctx.reports.push(LintReport {
                pos: token.end_position().into(),
                level: super::ReportLevel::Warning,
                msg: format!(
                    "Line is expected to be at most {} characters, but is {} characters",
                    ctx.max_column_width, real_len
                ),
            });
            ctx._last_check_line = token.end_position().line();
        }

        NodeWrapper::Token(token)
    }
}
