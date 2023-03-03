use full_moon::{ast::punctuated::Pair, node::Node};

use super::{LintReport, NodeKey, NodeWrapper, Registry, Rule, RuleContext, RuleInfo};

decl_rule!(table_ctor_comma, "Require comma after last field of table ctor", "20230224", "");

pub struct TableCtorComma {
    pub reports: Vec<LintReport>,
}

impl Rule for TableCtorComma {
    fn apply(rules: &mut Registry, _config: &serde_json::Value) -> Self {
        let rule_name = "table_ctor_comma";
        rules.listen_enter(rule_name, NodeKey::TableConstructor, Self::enter_table_ctor_block);

        Self { reports: vec![] }
    }

    fn context(&self) -> &dyn RuleContext {
        self
    }
}

impl RuleContext for TableCtorComma {
    fn get_reports(&self) -> &Vec<LintReport> {
        &self.reports
    }
}

impl TableCtorComma {
    pub fn apply(rules: &mut Registry) -> Self {
        let rule_name = "table_ctor_comma";
        rules.listen_enter(rule_name, NodeKey::TableConstructor, Self::enter_table_ctor_block);

        Self { reports: vec![] }
    }

    pub fn enter_table_ctor_block(rctx: &mut dyn RuleContext, node: NodeWrapper) -> NodeWrapper {
        // println!("enter_table_ctor_block");
        let node = rule_cast!(node, NodeWrapper::TableConstructor);
        let ctx: &mut TableCtorComma = rctx.downcast_mut().unwrap();

        if let Some(last_field) = node.fields().last() {
            // If there is a last field and it is not on the same line as the closing brace
            // then we need to check if there is a comma after it
            let close_brace_line = node.braces().tokens().1.start_position().unwrap().line();
            match last_field {
                Pair::End(f) => {
                    if f.tokens().last().unwrap().end_position().unwrap().line() != close_brace_line
                    {
                        ctx.reports.push(LintReport {
                            pos: f.tokens().last().unwrap().end_position().unwrap().into(),
                            level: super::ReportLevel::Warning,
                            msg: "Table constructor should have a comma after the last field"
                                .to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        NodeWrapper::TableConstructor(node)
    }
}
