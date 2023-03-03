use full_moon::tokenizer::Token;

use crate::rules::Registry;

#[derive(Default)]
pub struct Linter {
    pub rule_registry: Registry,
}

impl Linter {}

pub struct LualintContext<'a> {
    pub linter: &'a mut Linter,
}

pub mod lint_visitor;
pub mod linter_builder;

pub type LinterBuilder = linter_builder::LinterBuilder;

pub fn lint_tokens(tokens: &Vec<Token>, linter: &mut Linter) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::new();

    for token in tokens {
        let new_token = linter.rule_registry.notify_token(token.clone());
        new_tokens.push(new_token);
    }

    new_tokens
}

pub fn lint_src(src: &str, linter: &mut Linter) -> String {
    let source = linter.rule_registry.trigger_preprocess(src);
    source
}

// trim comments including inline comments and block comments
pub fn trim_lua_comments(input: &str) -> String {
    let tokens = full_moon::tokenizer::tokens(input);
    let tokens = tokens.unwrap();
    tokens
        .iter()
        .filter(|token| match token.token_type() {
            full_moon::tokenizer::TokenType::MultiLineComment { blocks: _, comment: _ } => false,
            full_moon::tokenizer::TokenType::SingleLineComment { comment: _ } => false,
            _ => true,
        })
        .map(|token| token.to_string())
        .collect::<Vec<String>>()
        .join("")
}

#[test]
fn test_trim_lua_comments_inline_comment() {
    let input = "print(\"Hello, world!\") -- This is a comment";
    let expected_output = "print(\"Hello, world!\") ";
    assert_eq!(trim_lua_comments(input), expected_output);
}

#[test]
fn test_trim_lua_comments_block_comment() {
    let input = r#"print("Hello, world!") --[[
This is a block comment
on multiple lines
]] print("Goodbye, world!")"#;
    let expected_output = "print(\"Hello, world!\")  print(\"Goodbye, world!\")";
    assert_eq!(trim_lua_comments(input), expected_output);
}

#[test]
fn test_trim_lua_comments_no_comment() {
    let input = "print(\"Hello, world!\")";
    let expected_output = "print(\"Hello, world!\")";
    assert_eq!(trim_lua_comments(input), expected_output);
}

#[test]
fn test_trim_lua_comments_empty_input() {
    let input = "";
    let expected_output = "";
    assert_eq!(trim_lua_comments(input), expected_output);
}

#[test]
fn test_trim_lua_comments_inline_comment_at_end_of_line() {
    let input = "print(\"Hello, world!\") -- This is a comment\n";
    let expected_output = "print(\"Hello, world!\") \n";
    assert_eq!(trim_lua_comments(input), expected_output);
}

#[test]
fn test_trim_lua_comments_block_comment_on_single_line() {
    let input = "print(\"Hello, world!\") --[[This is a block comment on a single line]] print(\"Goodbye, world!\")";
    let expected_output = "print(\"Hello, world!\")  print(\"Goodbye, world!\")";
    assert_eq!(trim_lua_comments(input), expected_output);
}

#[test]
fn test_trim_lua_comments_nested_block_comment() {
    let input = r#"
        print("Hello, world!") --[=[
        This is a block comment
        on multiple lines
        --[[
            This is a nested block comment
        ]]--
    ]=] print("Goodbye, world!")"#;
    let expected_output = "
        print(\"Hello, world!\")  print(\"Goodbye, world!\")";

    assert_eq!(trim_lua_comments(input), expected_output);
}
