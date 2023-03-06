use full_moon::ast::lua52::{Goto, Label};
use full_moon::ast::span::ContainedSpan;

use full_moon::{ast::*, tokenizer::TokenReference};

use crate::rules::NodeWrapper as NW;

use full_moon::ast::punctuated::Pair;

use full_moon::ast::punctuated::Punctuated;

use crate::rules::WalkTy;

use crate::rules::NodeKey;

use crate::lint::LualintContext;

use crate::lint::Linter;

pub fn lint_ast<'a>(ast: &'a Ast, linter: &'a mut Linter) -> (Ast, LualintContext<'a>) {
    let ast = ast.clone();
    let mut ctx = LualintContext { linter };
    let new_block = lint_block(&mut ctx, ast.nodes());
    let new_eof = lint_token_ref(&mut ctx, ast.eof());

    (ast.with_nodes(new_block).with_eof(new_eof), ctx)
}

macro_rules! must_match {
    ($rule: expr, $rule_type: path) => {
        match $rule {
            $rule_type(rule) => rule,
            _ => unreachable!(),
        }
    };
}

pub fn lint_token_ref(ctx: &mut LualintContext, token_ref: &TokenReference) -> TokenReference {
    let tok = ctx
        .linter
        .rule_registry
        .notify_enter(NodeKey::TokenRef, NW::TokenRef(token_ref.to_owned()));
    must_match!(tok, NW::TokenRef)
}

pub fn lint_block(ctx: &mut LualintContext, block: &Block) -> Block {
    let mut blk_w = NW::Block(block.to_owned());

    ctx.linter.rule_registry.trigger_walker(NodeKey::Block, WalkTy::Enter, blk_w);

    let mut stmt_iterator = block.stmts_with_semicolon().peekable();
    let mut formatted_statements: Vec<(Stmt, Option<TokenReference>)> = Vec::new();

    for (stmt, semi) in stmt_iterator.by_ref() {
        let stmt = lint_stmt(ctx, stmt);
        let semi = semi.clone();
        formatted_statements.push((stmt, semi))
    }
    drop(stmt_iterator);

    let linted_last_stmt: Option<(LastStmt, Option<TokenReference>)> =
        match block.last_stmt_with_semicolon() {
            Some((last_stmt, semi)) => {
                let last_stmt = lint_last_stmt(ctx, last_stmt);
                let semi = semi.clone();
                Some((last_stmt, semi))
            }
            None => None,
        };

    let block = Block::new().with_stmts(formatted_statements).with_last_stmt(linted_last_stmt);

    blk_w = NW::Block(block);
    blk_w = ctx.linter.rule_registry.trigger_walker(NodeKey::Block, WalkTy::Leave, blk_w);

    must_match!(blk_w, NW::Block)
}

pub fn lint_last_stmt(ctx: &mut LualintContext, last_stmt: &LastStmt) -> LastStmt {
    let last_stmt = match last_stmt {
        LastStmt::Return(return_node) => lint_return(ctx, return_node),
        LastStmt::Break(break_stmt) => lint_break(ctx, break_stmt),
        _ => unreachable!("unimplemented last statement type: {:?}", last_stmt),
    };
    let last_stmt_w = NW::LastStmt(last_stmt);

    // match last_stmt_w {
    // NW::LastStmt(last_stmt) => last_stmt,
    // _ => unreachable!(),
    // }
    must_match!(last_stmt_w, NW::LastStmt)
}

pub fn lint_return(ctx: &mut LualintContext, return_node: &Return) -> LastStmt {
    let mut ret_node_w = NW::Return(return_node.to_owned());

    ctx.linter.rule_registry.trigger_walker(NodeKey::Return, WalkTy::Enter, ret_node_w);

    let return_node = return_node.to_owned();
    let return_token = return_node.token();

    let return_token = lint_token_ref(ctx, return_token);

    let returns = return_node
        .returns()
        .pairs()
        .map(|pair| pair.to_owned().map(|expression| lint_expr_block(ctx, &expression)))
        .collect();

    let return_node = return_node.with_returns(returns).with_token(return_token);

    ret_node_w = NW::Return(return_node);

    ret_node_w =
        ctx.linter.rule_registry.trigger_walker(NodeKey::Return, WalkTy::Leave, ret_node_w);

    // match ret_node_w {
    //     NW::Return(return_node) => LastStmt::Return(return_node),
    //     _ => unreachable!(),
    // }
    LastStmt::Return(must_match!(ret_node_w, NW::Return))
}

pub fn lint_expr_block(ctx: &mut LualintContext, expression: &Expression) -> Expression {
    lint_expr(ctx, expression)
}

pub fn lint_expr(ctx: &mut LualintContext, expression: &Expression) -> Expression {
    match expression {
        Expression::BinaryOperator { lhs, binop, rhs } => Expression::BinaryOperator {
            lhs: Box::new(lint_expr_block(ctx, lhs)),
            binop: binop.to_owned(),
            rhs: Box::new(lint_expr_block(ctx, rhs)),
        },
        Expression::Parentheses { contained, expression } => Expression::Parentheses {
            contained: contained.to_owned(),
            expression: Box::new(lint_expr_block(ctx, expression)),
        },
        Expression::UnaryOperator { unop, expression } => Expression::UnaryOperator {
            unop: unop.to_owned(),
            expression: Box::new(lint_expr_block(ctx, expression)),
        },
        Expression::Value { value } => Expression::Value {
            value: Box::new(match &**value {
                Value::Function((function_token, body)) => {
                    let block = lint_block(ctx, body.block());
                    Value::Function((function_token.to_owned(), body.to_owned().with_block(block)))
                }
                Value::FunctionCall(func_call) => {
                    Value::FunctionCall(lint_func_call_block(ctx, func_call))
                }
                Value::TableConstructor(table_constructor) => {
                    Value::TableConstructor(lint_table_ctor(ctx, table_constructor))
                }
                Value::ParenthesesExpression(expression) => {
                    Value::ParenthesesExpression(lint_expr_block(ctx, expression))
                }
                // TODO: var?
                value => value.to_owned(),
            }),
        },
        _ => unreachable!("unimplemented expression type: {:?}", expression),
    }
}

pub fn lint_break(ctx: &mut LualintContext, break_stmt: &TokenReference) -> LastStmt {
    let break_stmt = lint_token_ref(ctx, break_stmt);
    LastStmt::Break(break_stmt)
}

pub fn lint_stmt(ctx: &mut LualintContext, stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::Assignment(assignment_stmt) => lint_assignment(ctx, assignment_stmt),
        Stmt::Do(do_stmt) => lint_do(ctx, do_stmt),
        Stmt::FunctionCall(func_call_stmt) => lint_func_call(ctx, func_call_stmt),
        Stmt::FunctionDeclaration(func_decl_stmt) => lint_func_decl(ctx, func_decl_stmt),
        Stmt::GenericFor(generic_for_stmt) => lint_generic_for(ctx, generic_for_stmt),
        Stmt::If(if_stmt) => lint_if(ctx, if_stmt),
        Stmt::LocalAssignment(local_assign_stmt) => lint_local_assign(ctx, local_assign_stmt),
        Stmt::LocalFunction(local_func_stmt) => lint_local_func(ctx, local_func_stmt),
        Stmt::NumericFor(numeric_for_stmt) => lint_numeric_for(ctx, numeric_for_stmt),
        Stmt::Repeat(repeat_stmt) => lint_repeat(ctx, repeat_stmt),
        Stmt::While(while_stmt) => lint_while(ctx, while_stmt),
        Stmt::Goto(goto_stmt) => lint_goto(ctx, goto_stmt),
        Stmt::Label(label_stmt) => lint_label(ctx, label_stmt),
        _ => unreachable!("unimplemented statement type: {:?}", stmt),
    }
}

pub fn lint_func_call_block(ctx: &mut LualintContext, func_call: &FunctionCall) -> FunctionCall {
    let prefix = match func_call.prefix() {
        Prefix::Expression(expression) => Prefix::Expression(lint_expr_block(ctx, expression)),
        Prefix::Name(name) => Prefix::Name(name.to_owned()),
        other => panic!("unknown node {:?}", other),
    };

    let suffixes = func_call
        .suffixes()
        .map(|suffix| match suffix {
            Suffix::Call(call) => Suffix::Call(match call {
                Call::AnonymousCall(function_args) => {
                    Call::AnonymousCall(lint_func_args_block(ctx, function_args))
                }
                Call::MethodCall(method_call) => {
                    let args = lint_func_args_block(ctx, method_call.args());
                    Call::MethodCall(method_call.to_owned().with_args(args))
                }
                other => panic!("unknown node {:?}", other),
            }),
            Suffix::Index(index) => Suffix::Index(match index {
                Index::Brackets { brackets, expression } => Index::Brackets {
                    brackets: brackets.to_owned(),
                    expression: lint_expr_block(ctx, expression),
                },
                _ => index.to_owned(),
            }),
            other => panic!("unknown node {:?}", other),
        })
        .collect();

    func_call.to_owned().with_prefix(prefix).with_suffixes(suffixes)
}

pub fn lint_func_args_block(
    ctx: &mut LualintContext,
    function_args: &FunctionArgs,
) -> FunctionArgs {
    match function_args {
        FunctionArgs::Parentheses { parentheses, arguments } => FunctionArgs::Parentheses {
            parentheses: parentheses.to_owned(),
            arguments: arguments
                .pairs()
                .map(|pair| pair.to_owned().map(|expression| lint_expr_block(ctx, &expression)))
                .collect(),
        },
        FunctionArgs::TableConstructor(table_constructor) => {
            FunctionArgs::TableConstructor(lint_table_ctor(ctx, table_constructor))
        }
        _ => function_args.to_owned(),
    }
}

pub fn lint_table_ctor(
    ctx: &mut LualintContext,
    table_constructor: &TableConstructor,
) -> TableConstructor {
    let fields = table_constructor
        .fields()
        .pairs()
        .map(|pair| pair.to_owned().map(|pair| lint_field(ctx, pair)))
        .collect();

    let node_w = NW::TableConstructor(table_constructor.to_owned());

    let node_w =
        ctx.linter.rule_registry.trigger_walker(NodeKey::TableConstructor, WalkTy::Enter, node_w);

    let x = must_match!(node_w, NW::TableConstructor);

    let node = x.with_fields(fields);

    let node_w = ctx.linter.rule_registry.trigger_walker(
        NodeKey::TableConstructor,
        WalkTy::Leave,
        NW::TableConstructor(node),
    );

    must_match!(node_w, NW::TableConstructor)
}

pub fn lint_field(ctx: &mut LualintContext, field: Field) -> Field {
    match field {
        Field::ExpressionKey { brackets, key, equal, value } => Field::ExpressionKey {
            brackets,
            key: lint_expr_block(ctx, &key),
            equal,
            value: lint_expr_block(ctx, &value),
        },
        Field::NameKey { key, equal, value } => {
            Field::NameKey { key, equal, value: lint_expr_block(ctx, &value) }
        }
        Field::NoKey(expression) => Field::NoKey(lint_expr_block(ctx, &expression)),
        other => panic!("unknown node {:?}", other),
    }
}

// todo: lint_assignment
pub fn lint_assignment(_ctx: &mut LualintContext, assignment_stmt: &Assignment) -> Stmt {
    Stmt::Assignment(assignment_stmt.to_owned())
}

// todo: lint_do
pub fn lint_do(_ctx: &mut LualintContext, do_stmt: &Do) -> Stmt {
    Stmt::Do(do_stmt.to_owned())
}

// todo: lint_func_call
pub fn lint_func_call(_ctx: &mut LualintContext, func_call_stmt: &FunctionCall) -> Stmt {
    Stmt::FunctionCall(func_call_stmt.to_owned())
}

pub fn lint_func_decl(ctx: &mut LualintContext, func_decl_stmt: &FunctionDeclaration) -> Stmt {
    let rt = NW::FunctionDeclaration(func_decl_stmt.to_owned());
    let rt = ctx.linter.rule_registry.notify_enter(NodeKey::FuncDecl, rt);

    let func_decl_stmt = must_match!(rt, NW::FunctionDeclaration);
    let func_token = lint_token_ref(ctx, func_decl_stmt.function_token());
    let name = lint_func_name(ctx, func_decl_stmt.name());
    let body = lint_func_body(ctx, func_decl_stmt.body());

    let func_decl_stmt =
        func_decl_stmt.to_owned().with_function_token(func_token).with_name(name).with_body(body);

    let rt = NW::FunctionDeclaration(func_decl_stmt);
    let rt = ctx.linter.rule_registry.notify_leave(NodeKey::FuncDecl, rt);

    Stmt::FunctionDeclaration(must_match!(rt, NW::FunctionDeclaration))
}

pub fn lint_func_name(ctx: &mut LualintContext, func_name: &FunctionName) -> FunctionName {
    let rt = NW::FunctionName(func_name.to_owned());
    let rt = ctx.linter.rule_registry.notify_enter(NodeKey::FuncName, rt);

    let func_name = must_match!(rt, NW::FunctionName);

    let names = lint_punctuated(ctx, func_name.names(), lint_token_ref);

    let mut method: Option<(TokenReference, TokenReference)> = None;

    if let Some(method_colon) = func_name.method_colon() {
        if let Some(token_reference) = func_name.method_name() {
            method =
                Some((lint_token_ref(ctx, method_colon), lint_token_ref(ctx, token_reference)));
        }
    };
    let func_name = func_name.with_names(names).with_method(method);

    let rt = NW::FunctionName(func_name);
    let rt = ctx.linter.rule_registry.notify_leave(NodeKey::FuncName, rt);

    must_match!(rt, NW::FunctionName)
}

pub fn lint_generic_for(ctx: &mut LualintContext, generic_for_stmt: &GenericFor) -> Stmt {
    // return Stmt::GenericFor(generic_for_stmt.to_owned());
    let mut rt = NW::GenericFor(generic_for_stmt.to_owned());
    rt = ctx.linter.rule_registry.notify_enter(NodeKey::GenericFor, rt);

    Stmt::GenericFor(must_match!(rt, NW::GenericFor))
}

pub fn lint_if(ctx: &mut LualintContext, if_stmt: &If) -> Stmt {
    let mut rt = NW::If(if_stmt.to_owned());
    rt = ctx.linter.rule_registry.notify_enter(NodeKey::If, rt);

    let if_stmt = must_match!(rt, NW::If);

    let if_token = lint_token_ref(ctx, if_stmt.if_token());

    let condition = lint_expr(ctx, if_stmt.condition());

    let then_token = lint_token_ref(ctx, if_stmt.then_token());

    let block = lint_block(ctx, if_stmt.block());

    let else_if = if_stmt.else_if().map(|else_ifs| {
        else_ifs
            .iter()
            .map(|else_if| {
                let else_if_token = lint_token_ref(ctx, else_if.else_if_token());
                let condition = lint_expr(ctx, else_if.condition());
                let then_token = lint_token_ref(ctx, else_if.then_token());
                let block = lint_block(ctx, else_if.block());
                else_if
                    .to_owned()
                    .with_else_if_token(else_if_token)
                    .with_condition(condition)
                    .with_then_token(then_token)
                    .with_block(block)
            })
            .collect()
    });

    let else_token = if_stmt.else_token().map(|else_token| lint_token_ref(ctx, else_token));

    let else_block = if_stmt.else_block().map(|else_block| lint_block(ctx, else_block));

    let end_token = lint_token_ref(ctx, if_stmt.end_token());

    let if_stmt = if_stmt
        .to_owned()
        .with_if_token(if_token)
        .with_condition(condition)
        .with_then_token(then_token)
        .with_block(block)
        .with_else_if(else_if)
        .with_else_token(else_token)
        .with_else(else_block)
        .with_end_token(end_token);

    let rt = NW::If(if_stmt);
    let rt = ctx.linter.rule_registry.notify_leave(NodeKey::If, rt);

    Stmt::If(must_match!(rt, NW::If))
}

// this can be a assignment or just decl
// assign: var a = b;
// decl: var a;
pub fn lint_local_assign(ctx: &mut LualintContext, las: &LocalAssignment) -> Stmt {
    // TODO: Lint the local assignment
    let name_list = las.names().to_owned();
    let expr_list = lint_punctuated(ctx, las.expressions(), lint_expr);
    let equal_token = las.equal_token().to_owned();
    let local_token = las.local_token().to_owned();
    let local_assignment = LocalAssignment::new(name_list);
    Stmt::LocalAssignment(
        local_assignment
            .with_local_token(local_token)
            .with_equal_token(equal_token.to_owned().cloned())
            .with_expressions(expr_list),
    )
}

fn print_token_ref(token_ref: &TokenReference) -> String {
    format!("{}:{}", token_ref.start_position().line(), token_ref.start_position().character())
}

pub fn lint_punctuated<T, F>(
    ctx: &mut LualintContext,
    old: &Punctuated<T>,
    value_linter: F,
) -> Punctuated<T>
where
    T: std::fmt::Display,
    F: Fn(&mut LualintContext, &T) -> T,
{
    let mut list: Punctuated<T> = Punctuated::new();

    for pair in old.pairs() {
        match pair {
            Pair::Punctuated(value, punctuation) => {
                let value = value_linter(ctx, value);
                let punctuation = lint_token_ref(ctx, punctuation);

                list.push(Pair::new(value, Some(punctuation)));
            }
            Pair::End(value) => {
                let value = value_linter(ctx, value);
                list.push(Pair::new(value, None));
            }
        }
    }

    list
}

pub fn lint_local_func(ctx: &mut LualintContext, local_func_stmt: &LocalFunction) -> Stmt {
    // Calculate trivia
    let local_token = lint_token_ref(ctx, local_func_stmt.local_token());
    let function_token = lint_token_ref(ctx, local_func_stmt.function_token());
    let formatted_name = lint_token_ref(ctx, local_func_stmt.name());

    let func_body = lint_func_body(ctx, local_func_stmt.body());

    let f = LocalFunction::new(formatted_name)
        .with_local_token(local_token)
        .with_function_token(function_token)
        .with_body(func_body);
    Stmt::LocalFunction(f)
}

/// Formats a FunctionBody node
pub fn lint_func_body(ctx: &mut LualintContext, func_body: &FunctionBody) -> FunctionBody {
    let parameters_parentheses = lint_contained_span(ctx, func_body.parameters_parentheses());
    let formatted_parameters = lint_parameters(ctx, func_body.parameters());
    let block = lint_block(ctx, func_body.block());
    let end_token = lint_token_ref(ctx, func_body.end_token());
    func_body
        .to_owned()
        .with_parameters_parentheses(parameters_parentheses)
        .with_parameters(formatted_parameters)
        .with_block(block)
        .with_end_token(end_token)
}

pub fn lint_contained_span(
    ctx: &mut LualintContext,
    span: &span::ContainedSpan,
) -> span::ContainedSpan {
    let (start_token, end_token) = span.tokens();

    ContainedSpan::new(lint_token_ref(ctx, start_token), lint_token_ref(ctx, end_token))
}

pub fn lint_parameters(
    ctx: &mut LualintContext,
    params: &Punctuated<Parameter>,
) -> Punctuated<Parameter> {
    params
        .pairs()
        .map(|pair| pair.to_owned().map(|mut parameter| lint_parameter(ctx, &mut parameter)))
        .collect()
}

pub fn lint_parameter(ctx: &mut LualintContext, parameter: &mut Parameter) -> Parameter {
    match parameter {
        Parameter::Name(name) => Parameter::Name(lint_token_ref(ctx, name)),
        Parameter::Ellipse(vararg) => Parameter::Ellipse(lint_token_ref(ctx, vararg)),
        _ => parameter.to_owned(),
    }
}

pub fn lint_numeric_for(ctx: &mut LualintContext, numeric_for: &NumericFor) -> Stmt {
    // return Stmt::NumericFor(numeric_for.to_owned());
    let mut rt = NW::NumericFor(numeric_for.to_owned());
    rt = ctx.linter.rule_registry.trigger_walker(NodeKey::NumericFor, WalkTy::Enter, rt);

    let numeric_for = must_match!(rt, NW::NumericFor);
    let for_token = lint_token_ref(ctx, numeric_for.for_token());
    let index_variable = lint_token_ref(ctx, numeric_for.index_variable());
    let equal_token = lint_token_ref(ctx, numeric_for.equal_token());
    let start = lint_expr(ctx, numeric_for.start());
    let start_end_comma = lint_token_ref(ctx, numeric_for.start_end_comma());
    let end = lint_expr(ctx, numeric_for.end());

    let end_step_comma = numeric_for.end_step_comma().map(|comma| lint_token_ref(ctx, comma));

    let step = numeric_for.step().map(|step| lint_expr(ctx, step));

    let do_token = lint_token_ref(ctx, numeric_for.do_token());
    let block = lint_block(ctx, numeric_for.block());
    let end_token = lint_token_ref(ctx, numeric_for.end_token());

    let numeric_for = numeric_for
        .to_owned()
        .with_for_token(for_token)
        .with_index_variable(index_variable)
        .with_equal_token(equal_token)
        .with_start(start)
        .with_start_end_comma(start_end_comma)
        .with_end(end)
        .with_end_step_comma(end_step_comma)
        .with_step(step)
        .with_do_token(do_token)
        .with_block(block)
        .with_end_token(end_token);

    rt = NW::NumericFor(numeric_for);

    rt = ctx.linter.rule_registry.trigger_walker(NodeKey::NumericFor, WalkTy::Leave, rt);

    let rt = must_match!(rt, NW::NumericFor);

    Stmt::NumericFor(rt)
}

pub fn lint_repeat(ctx: &mut LualintContext, repeat_stmt: &Repeat) -> Stmt {
    // Stmt::Repeat(repeat_stmt.to_owned())
    let mut rt = NW::Repeat(repeat_stmt.to_owned());
    rt = ctx.linter.rule_registry.trigger_walker(NodeKey::Repeat, WalkTy::Enter, rt);

    let repeat_stmt = must_match!(rt, NW::Repeat);
    let repeat_token = lint_token_ref(ctx, repeat_stmt.repeat_token());
    let block = lint_block(ctx, repeat_stmt.block());
    let until_token = lint_token_ref(ctx, repeat_stmt.until_token());
    let until = lint_expr(ctx, repeat_stmt.until());

    let repeat_stmt = repeat_stmt
        .to_owned()
        .with_repeat_token(repeat_token)
        .with_block(block)
        .with_until_token(until_token)
        .with_until(until);

    rt = NW::Repeat(repeat_stmt);

    rt = ctx.linter.rule_registry.trigger_walker(NodeKey::Repeat, WalkTy::Leave, rt);

    let rt = must_match!(rt, NW::Repeat);

    Stmt::Repeat(rt)
}

pub fn lint_while(ctx: &mut LualintContext, while_stmt: &While) -> Stmt {
    // Stmt::While(while_stmt.to_owned())
    let mut rt = NW::While(while_stmt.to_owned());
    rt = ctx.linter.rule_registry.trigger_walker(NodeKey::While, WalkTy::Enter, rt);

    let while_stmt = must_match!(rt, NW::While);
    let while_token = lint_token_ref(ctx, while_stmt.while_token());
    let condition = lint_expr(ctx, while_stmt.condition());
    let do_token = lint_token_ref(ctx, while_stmt.do_token());
    let block = lint_block(ctx, while_stmt.block());
    let end_token = lint_token_ref(ctx, while_stmt.end_token());

    let while_stmt = while_stmt
        .to_owned()
        .with_while_token(while_token)
        .with_condition(condition)
        .with_do_token(do_token)
        .with_block(block)
        .with_end_token(end_token);

    rt = NW::While(while_stmt);

    rt = ctx.linter.rule_registry.trigger_walker(NodeKey::While, WalkTy::Leave, rt);

    let rt = must_match!(rt, NW::While);

    Stmt::While(rt)
}

pub fn lint_goto(ctx: &mut LualintContext, goto_stmt: &Goto) -> Stmt {
    // Stmt::Goto(goto_stmt.to_owned())
    let mut rt = NW::Goto(goto_stmt.to_owned());
    rt = ctx.linter.rule_registry.trigger_walker(NodeKey::Goto, WalkTy::Enter, rt);

    let goto_stmt = must_match!(rt, NW::Goto);
    let goto_token = lint_token_ref(ctx, goto_stmt.goto_token());
    let label_name = lint_token_ref(ctx, goto_stmt.label_name());

    let goto_stmt = goto_stmt.to_owned().with_goto_token(goto_token).with_label_name(label_name);

    rt = NW::Goto(goto_stmt);

    rt = ctx.linter.rule_registry.trigger_walker(NodeKey::Goto, WalkTy::Leave, rt);

    let rt = must_match!(rt, NW::Goto);

    Stmt::Goto(rt)
}

pub fn lint_label(_ctx: &mut LualintContext, label_stmt: &Label) -> Stmt {
    Stmt::Label(label_stmt.to_owned())
}
