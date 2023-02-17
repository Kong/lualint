use full_moon::{
    ast::{punctuated::Pair, Stmt},
    node::Node,
    tokenizer::{Token, TokenType},
};

use super::{LintReport, NodeKey, NodeWrapper, Registry, RuleContext, WalkTy, RuleInfo, Rule};

decl_rule!(one_line_before_else, "Require a blank line before else", "20230224", "");
pub struct OneLineBeforeElse {
    pub reports: Vec<LintReport>,
}

impl Rule for OneLineBeforeElse {
    fn apply(rules: &mut Registry, config: &serde_json::Value) -> Self {
        let rule_name = "one_line_before_else";
        rules.listen_enter(rule_name, NodeKey::If, Box::new(Self::enter_if));

        Self { reports: vec![] }
    }

    fn context(&self) -> &dyn RuleContext {
        self
    }
}

impl RuleContext for OneLineBeforeElse {
    fn get_reports(&self) -> &Vec<LintReport> {
        &self.reports
    }
}

impl OneLineBeforeElse {
    pub fn apply(rules: &mut Registry) -> Self {
        let rule_name = "one_line_before_else";
        rules.listen_enter(rule_name, NodeKey::If, Box::new(Self::enter_if));

        Self { reports: vec![] }
    }

    pub fn enter_if(rctx: &mut dyn RuleContext, node: NodeWrapper) -> NodeWrapper {
        let if_stmt = rule_cast!(node, NodeWrapper::If);
        let ctx: &mut OneLineBeforeElse = rctx.downcast_mut().unwrap();

        let mut prev_block_last_stmt_trail = None;

        if let Some(prev_block_last_stmt) = if_stmt.block().stmts().last() {
            prev_block_last_stmt_trail = Some(prev_block_last_stmt.tokens().last().unwrap());
        }

        if_stmt.else_if().map(|else_ifs| {
            else_ifs.iter().for_each(|else_if| {
                // if we got prev_block_last_stmt_trail and it is at -1 line of current else if token
                let else_if_token = else_if.else_if_token();
                let else_if_line = else_if_token.start_position().unwrap().line();
                if let Some(prev_block_last_stmt_trail) = prev_block_last_stmt_trail {
                    let prev_stmt_line = prev_block_last_stmt_trail.end_position().unwrap().line();
                    if prev_stmt_line + 1 != else_if_line {
                        return;
                    }
                    ctx.reports.push(LintReport {
                        pos: else_if_token.start_position().unwrap().clone().into(),
                        level: super::ReportLevel::Warning,
                        msg: format!("There should be a line before else if",),
                    });
                }

                // update prev_block_last_stmt_trail
                if let Some(prev_block_last_stmt) = else_if.block().stmts().last() {
                    prev_block_last_stmt_trail =
                        Some(prev_block_last_stmt.tokens().last().unwrap());
                }
            })
        });

        // check for "else"

        if if_stmt.else_block().is_some() {
            let else_token = if_stmt.else_token();
            let else_line = else_token.start_position().unwrap().line();
            if let Some(prev_block_last_stmt_trail) = prev_block_last_stmt_trail {
                let prev_stmt_line = prev_block_last_stmt_trail.end_position().unwrap().line();
                if prev_stmt_line + 1 != else_line {
                    return NodeWrapper::If(if_stmt);
                }
                ctx.reports.push(LintReport {
                    pos: else_token.start_position().unwrap().clone().into(),
                    level: super::ReportLevel::Warning,
                    msg: format!("There should be a line before else",),
                });
            }
        }

        NodeWrapper::If(if_stmt)
    }
}
