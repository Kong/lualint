use full_moon::{
    ast::{punctuated::Pair, Stmt},
    node::Node,
    tokenizer::{Position, Token, TokenType},
};

use crate::lint::trim_lua_comments;

use super::{LintReport, NodeKey, NodeWrapper, Pos, Registry, RuleContext, WalkTy};
pub struct NoTrailingSemicolon {
    
    pub reports: Vec<LintReport>,
}

impl RuleContext for NoTrailingSemicolon {
    fn get_reports(&self) -> &Vec<LintReport> {
        &self.reports
    }
}

impl NoTrailingSemicolon {
    pub fn apply(rules: &mut Registry) -> Self {
    let rule_name = "no_trailing_semicolon";
        rules.preprocess(rule_name, Box::new(Self::preprocess));

        Self {  reports: vec![] }
    }

    pub fn preprocess(rctx: &mut dyn RuleContext, node: NodeWrapper) -> NodeWrapper {
        let ctx: &mut NoTrailingSemicolon = rctx.downcast_mut().unwrap();
        let source = rule_cast!(node, NodeWrapper::Source);
        let source = trim_lua_comments(source.as_str());
        let mut lineno = 0;
        source.lines().for_each(|line| {
            lineno += 1;
            if line.ends_with(";") {
                let col = line.len() - line.trim_end().len() + 1;
                ctx.reports.push(LintReport {
                    pos: Pos::new(lineno, col),
                    level: super::ReportLevel::Warning,
                    msg: format!("Line ends with trailing semicolon",),
                });
            }
        });

        NodeWrapper::Source(source)
    }
}
