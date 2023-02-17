use full_moon::{
    ast::{punctuated::Pair, Stmt},
    node::Node,
    tokenizer::{Token, TokenType},
};

use super::{LintReport, NodeKey, NodeWrapper, Registry, RuleContext, WalkTy};
pub struct NoGotoStatement {
    
    pub reports: Vec<LintReport>,
}

impl RuleContext for NoGotoStatement {
    fn get_reports(&self) -> &Vec<LintReport> {
        &self.reports
    }
}

impl NoGotoStatement {
    pub fn apply(rules: &mut Registry) -> Self {
        let rule_name = "no_goto_statement";
        rules.listen_enter(
            rule_name,
            NodeKey::TableConstructor,
            Box::new(Self::enter_table_ctor_block),
        );

        Self {  reports: vec![] }
    }

    pub fn enter_table_ctor_block(rctx: &mut dyn RuleContext, node: NodeWrapper) -> NodeWrapper {
        println!("enter_table_ctor_block");
        let node = rule_cast!(node, NodeWrapper::TableConstructor);
        let ctx: &mut NoGotoStatement = rctx.downcast_mut().unwrap();

        if let Some(last_field) = node.fields().last() {
            // If there is a last field and it is not on the same line as the closing brace
            // then we need to check if there is a comma after it
            let close_brace_line = node.braces().tokens().1.start_position().unwrap().line();
            match last_field {
                Pair::End(f) => {
                    if f.tokens().last().unwrap().end_position().unwrap().line() != close_brace_line
                    {
                        ctx.reports.push(LintReport {
                            pos: f.tokens().last().unwrap().end_position().unwrap().clone(),
                            level: super::ReportLevel::Warning,
                            msg: format!(
                                "Table constructor should have a comma after the last field",
                            ),
                        });
                    }
                }
                _ => {}
            }
        }

        NodeWrapper::TableConstructor(node.to_owned())
    }
}
