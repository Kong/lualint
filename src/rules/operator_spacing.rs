use std::cell::RefCell;

use full_moon::{tokenizer::{Symbol, Token}, ast::Expression, ShortString};

use crate::trivial::{UpdateTrailingTrivia, FormatTriviaType};

use super::{LintReport, NodeWrapper, Registry, Rule, RuleContext, RuleInfo};

decl_rule!(operator_spacing, "Spaces around operator", "20230306", "");

pub struct OperatorSpacing {
    pub reports: Vec<LintReport>,

    prev_token: Option<Token>,
    prev_prev_token: Option<Token>,
}

impl Rule for OperatorSpacing {
    fn apply(rules: &mut Registry, _config: &serde_json::Value) -> Self {
        let rule_name = "operator_spacing";
        rules.listen_token(rule_name, Self::on_token);
        rules.listen_enter(rule_name, super::NodeKey::Expr, Self::on_expr);
        Self { reports: vec![], prev_token: None, prev_prev_token: None }
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

impl OperatorSpacing {
    fn on_token(rctx: &mut dyn RuleContext, token_w: NodeWrapper) -> NodeWrapper {
        let ctx: &mut OperatorSpacing = rctx.downcast_mut().unwrap();
        let token = rule_cast!(token_w, NodeWrapper::Token);

        if ctx.prev_token.is_some() && ctx.prev_prev_token.is_some() {
            let prev_token = ctx.prev_token.as_ref().unwrap();
            let prev_prev_token = ctx.prev_prev_token.as_ref().unwrap();

            // should be: prev_prev_token, prev_token, token
            //            ^ws              ^op         ^ws
            // or         prev_token, token
            //            ^unary_op
            if is_operator(prev_token) {
                if is_whitespace(prev_prev_token) && is_whitespace(&token) {
                    // ok
                } else if is_prefix_op(prev_token) {
                    // ok
                } else {
                    ctx.reports.push(LintReport {
                        pos: token.start_position().into(),
                        level: super::ReportLevel::Warning,
                        msg: format!(
                            "Should have spaces around operator '{:?}'",
                            prev_token.token_type()
                        ),
                    });
                }
            }
        }

        ctx.prev_prev_token = ctx.prev_token.clone();
        ctx.prev_token = Some(token.clone());

        NodeWrapper::Token(token)
    }

    pub fn on_expr(rctx: &mut dyn RuleContext, _node: NodeWrapper) -> NodeWrapper {
        let ctx: &mut OperatorSpacing = rctx.downcast_mut().unwrap();
        let expr = rule_cast!(_node, NodeWrapper::Expr);
        match &expr {
            Expression::BinaryOperator { lhs, binop, rhs } => {
                let out = RefCell::new(Token::new(full_moon::tokenizer::TokenType::Whitespace { characters: ShortString::new("text") }));
                lhs.update_trailing_trivia(FormatTriviaType::Get(out));
                println!("binop: {:?}", binop.to_string());
            },
            Expression::UnaryOperator { unop, expression } => {
                println!("unop: {:?}", unop.to_string());
            },
            _=> (),
        }
        NodeWrapper::Expr(expr)
    }
}

fn is_whitespace(token: &Token) -> bool {
    if token.token_kind() != full_moon::tokenizer::TokenKind::Whitespace {
        return false;
    }
    return true;
}

fn is_operator(token: &Token) -> bool {
    if token.token_kind() != full_moon::tokenizer::TokenKind::Symbol {
        return false;
    }

    let symbol;
    match token.token_type() {
        full_moon::tokenizer::TokenType::Symbol { symbol: s } => symbol = s,
        _ => return false,
    }

    match symbol {
        Symbol::PlusEqual
        | Symbol::MinusEqual
        | Symbol::StarEqual
        | Symbol::SlashEqual
        | Symbol::PercentEqual
        | Symbol::CaretEqual
        | Symbol::TwoDotsEqual
        | Symbol::TwoColons
        | Symbol::Caret
        | Symbol::Colon
        // | Symbol::Comma
        | Symbol::Ellipse
        | Symbol::TwoDots
        | Symbol::Dot
        | Symbol::TwoEqual
        | Symbol::Equal
        | Symbol::GreaterThanEqual
        | Symbol::GreaterThan
        | Symbol::Hash
        | Symbol::LessThanEqual
        | Symbol::LessThan
        | Symbol::Minus
        | Symbol::Percent
        | Symbol::Plus
        // | Symbol::Semicolon
        | Symbol::Slash
        | Symbol::Star
        | Symbol::TildeEqual => true,
        _ => false,
    }
}

fn is_prefix_op(token: &Token) -> bool {
    if token.token_kind() != full_moon::tokenizer::TokenKind::Symbol {
        return false;
    }

    let symbol;
    match token.token_type() {
        full_moon::tokenizer::TokenType::Symbol { symbol: s } => symbol = s,
        _ => return false,
    }

    match symbol {
        Symbol::Minus | Symbol::Not | Symbol::Hash => true,
        _ => false,
    }
}
