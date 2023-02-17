use full_moon::{
    ast::{punctuated::Pair, Stmt},
    node::Node,
    tokenizer::{Position, Token, TokenType},
};
use serde_json::Value;

use super::{LintReport, NodeKey, NodeWrapper, Pos, Registry, Rule, RuleContext, RuleInfo, WalkTy};

decl_rule!(eof_blank_line, "Require a blank line at the end of the file", "20230224", "");

pub struct EofBlankLine {
    pub reports: Vec<LintReport>,
}

impl RuleContext for EofBlankLine {
    fn get_reports(&self) -> &Vec<LintReport> {
        &self.reports
    }
}


impl Rule for EofBlankLine {
    fn apply(rules: &mut Registry, config: &serde_json::Value) -> Self {
        rules.preprocess(RULE_NAME, Box::new(Self::preprocess));
        Self { reports: vec![] }
    }

    fn context(&self) -> &dyn RuleContext {
        self
    }
}

impl EofBlankLine {
    pub fn preprocess(rctx: &mut dyn RuleContext, node: NodeWrapper) -> NodeWrapper {
        let ctx: &mut EofBlankLine = rctx.downcast_mut().unwrap();
        let source = rule_cast!(node, NodeWrapper::Source);

        if !source.ends_with("\n") {
            ctx.reports.push(LintReport {
                pos: Pos::new(0, 0),
                level: super::ReportLevel::Warning,
                msg: format!("File is expected to end with a blank line, but does not",),
            });
        }

        NodeWrapper::Source(source)
    }
}
