use full_moon::{
    ast::Stmt,
    tokenizer::{Token, TokenType},
};

use super::{LintReport, NodeKey, NodeWrapper, Registry, RuleContext, RuleInfo, WalkTy, Rule};

decl_rule!(
    func_separation,
    "Require a blank line between function declarations",
    "20230224",
    "min_line: 2"
);
pub struct FuncSeparation {
    pub min_empty_line: usize,

    pub reports: Vec<LintReport>,
}

impl RuleContext for FuncSeparation {
    fn get_reports(&self) -> &Vec<LintReport> {
        &self.reports
    }
}

impl Rule for FuncSeparation {
    fn apply(rules: &mut Registry, config: &serde_json::Value) -> Self {
        let min_linebreak = config
            .get("min_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as usize;

        Self::apply(rules, min_linebreak)
    }

    fn context(&self) -> &dyn RuleContext {
        self
    }
}

impl FuncSeparation {
    pub fn apply(rules: &mut Registry, min_linebreak: usize) -> Self {
        let rule_name = "func_separation";
        rules.listen_enter(rule_name, NodeKey::Block, Box::new(Self::enter_block));

        Self { min_empty_line: min_linebreak, reports: vec![] }
    }

    pub fn enter_block(rctx: &mut dyn RuleContext, node: NodeWrapper) -> NodeWrapper {
        let block = rule_cast!(node, NodeWrapper::Block);
        let ctx: &mut FuncSeparation = rctx.downcast_mut().unwrap();

        let mut prev_stmt: Option<Stmt> = None;
        // Check two adjacent function declarations has at least one empty line between them
        block.stmts_with_semicolon().peekable().for_each(|(stmt, _)| {
            if let (Some(Stmt::FunctionDeclaration(x)), Stmt::FunctionDeclaration(y)) =
                (prev_stmt.as_ref(), stmt)
            {
                let n_linebreak = Self::check(x.body().end_token().trailing_trivia())
                                + Self::check(y.function_token().leading_trivia());
                let empty_line = n_linebreak - 1;
                if empty_line < ctx.min_empty_line {
                    ctx.reports.push(LintReport { pos: x.body().end_token().end_position().clone().into(), level: super::ReportLevel::Warning,
                        msg: format!(
                            "Function declaration '{}' should be separated from the previous function declaration by at least {} empty lines",
                            y.name().to_string(),
                            ctx.min_empty_line
                        ) });
                }
            }
            prev_stmt = Some(stmt.clone());
        });
        NodeWrapper::Block(block.to_owned())
    }

    pub fn check<'a>(iter: impl Iterator<Item = &'a Token>) -> usize {
        let mut n_linebreak = 0;
        iter.for_each(|token| {
            let tt = token.token_type().clone();
            if let TokenType::Whitespace { characters } = tt {
                for c in characters.chars() {
                    if c == '\n' {
                        n_linebreak += 1;
                    }
                }
            }
        });
        return n_linebreak;
    }
}
