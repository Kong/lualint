use std::io::Read;

use clap::Subcommand;
use log::{error, trace};
use lualint::{
    lint::{self, Linter},
    rules::{self, LintReport},
};

mod tests;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run {
        /// File to lint
        filename: String,
        #[clap(long)]
        rules: String,
    },
    Rules,
}

type RuleName = String;
type RuleConfig = serde_json::Value;
// enabled_rules = "[rule_name1:{},rule_name2:{key: value, key: value}]"
pub fn handle_run_command(filename: &str, enabled_rules: &str) {
    let mut linter = match build_config_linter(enabled_rules) {
        Some(value) => value,
        None => return,
    };
    lint_file(filename, &mut linter, false);
}

fn build_config_linter(enabled_rules: &str) -> Option<Linter> {
    let mut enabled_rules_vec_result = parse_rule_json_file(enabled_rules);
    if enabled_rules_vec_result.is_err() {
        enabled_rules_vec_result = parse_rules_json(enabled_rules);
    }

    let enabled_rules_vec = match enabled_rules_vec_result {
        Ok(enabled_rules) => enabled_rules,
        Err(e) => {
            error!("Failed to parse enabled rules: {}. Value for param `--rules` msut be a valid json map. For example: {}", e, 
            r#"{ "rule_name1": {}, "rule_name2": { "key": "value" } }"#);
            trace!("given value: {}", enabled_rules);
            return None;
        }
    };
    
    let mut linter_builder = lint::LinterBuilder::default();
    for (rule_name, rule_config) in enabled_rules_vec {
        match rule_name.as_str() {
            "eof_blank_line" => {
                linter_builder = linter_builder
                    .with_rule::<rules::eof_blank_line::EofBlankLine>(&rule_name, &rule_config);
            }
            "func_separation" => {
                linter_builder = linter_builder
                    .with_rule::<rules::func_separation::FuncSeparation>(&rule_name, &rule_config);
            }
            "max_column_width" => {
                linter_builder = linter_builder
                    .with_rule::<rules::max_column_width::MaxColumnWidth>(&rule_name, &rule_config);
            }
            // "no_trailing_semicolon" => {
            //     linter_builder = linter_builder
            //         .with_rule::<rules::no_trailing_semicolon::NoTrailingSemicolon>(&rule_name, &rule_config);
            // },
            "one_line_before_else" => {
                linter_builder = linter_builder
                    .with_rule::<rules::one_line_before_else::OneLineBeforeElse>(
                        &rule_name,
                        &rule_config,
                    );
            }
            "table_ctor_comma" => {
                linter_builder = linter_builder
                    .with_rule::<rules::table_ctor_comma::TableCtorComma>(&rule_name, &rule_config);
            }
            _ => {
                error!("Unknown rule: {}", rule_name);
                return None;
            }
        }
    }
    let linter = linter_builder.build();
    Some(linter)
}

fn parse_rule_json_file(
    filename: &str,
) -> Result<Vec<(RuleName, RuleConfig)>, Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    parse_rules_json(&contents)
}

fn parse_rules_json(
    enabled_rules: &str,
) -> Result<Vec<(RuleName, RuleConfig)>, Box<dyn std::error::Error>> {

    let enabled_rules = &strip_jsonc_comments(enabled_rules, true);
    // parse as json

    let v = serde_json::from_str::<serde_json::Value>(enabled_rules)?;

    let mut enabled_rules: Vec<(RuleName, RuleConfig)> = vec![];

    for (rule_name, rule_config) in v.as_object().unwrap() {
        enabled_rules.push((rule_name.to_string(), rule_config.to_owned()));
    }

    Ok(enabled_rules)
}

pub fn print_rules() {
    // "name" "description" "version" "config_example"
    println!(
        "{:22} {:10} {:56} {:18}",
        yansi::Paint::green("NAME").bold().underline(),
        yansi::Paint::green("VERSION").bold().underline(),
        yansi::Paint::green("DESCRIPTION").bold().underline(),
        yansi::Paint::green("CONFIG EXAMPLE").bold().underline()
    );

    rules::ALL_RULES.lock().iter().for_each(|x| {
        x.values().for_each(|info| {
            println!(
                "{:22} {:10} {:56} {:18}",
                info.name, info.version, info.description, info.config_example
            );
        });
    });
}

pub fn lint_file(filename: &str, linter: &mut Linter, write_back: bool) {
    let is_file_existing = std::path::Path::new(filename).exists();
    if !is_file_existing {
        println!("File not found: {}", filename);
        return;
    }

    let is_lua_file = filename.ends_with(".lua");
    if !is_lua_file {
        println!("File is not a lua file: {}", filename);
        return;
    }
    let _exit_on_err = true; // TODO: make this configurable
    let processed = match std::fs::read_to_string(filename) {
        Ok(lua_src) => {
            let out = drive(&lua_src, linter);
            let ok = print_lint_report(filename, linter);
            if !ok && _exit_on_err {
                std::process::exit(1);
            }
            out
        }
        Err(e) => {
            println!("Error reading file: {}", e);
            return;
        }
    };

    if write_back {
        match std::fs::write(filename, processed) {
            Ok(_) => println!("File written back: {}", filename),
            Err(e) => println!("Error writing file: {}", e),
        }
    } else {
        // info!("Linted: {}", processed);
    }
}

fn format_report(filename: &str, report: &LintReport) -> String {
    let file_contents = std::fs::read_to_string(filename).unwrap();
    let lines: Vec<&str> = file_contents.lines().collect();
    if report.pos.line == 0 {
        return format!("{}: {}", filename, report.msg);
    }
    let line = (report.pos.line, lines[report.pos.line - 1]);
    let continued_line = if report.pos.line < lines.len() {
        Some((report.pos.line + 1, lines[report.pos.line]))
    } else {
        None
    };
    pub fn make_space(n: usize) -> String {
        let mut s = String::new();
        for _ in 0..n.to_string().len() {
            s.push(' ');
        }
        s
    }
    pub fn format_impl(
        filename: &str,
        report: &LintReport,
        line: (usize, &str),
        continued_line: Option<(usize, &str)>,
    ) -> String {
        let spacing = make_space(line.0);
        let lineno = report.pos.line;
        let colno = report.pos.col - 1;
        let path = filename;
        let message = &report.msg;
        let line = line.1;

        fn underline(colno: usize, line: &str) -> String {
            let mut underline = String::new();

            let start = colno;
            let end = colno;
            let offset = std::cmp::max(start, 1) - 1;
            let line_chars = line.chars();

            for c in line_chars.take(offset) {
                match c {
                    '\t' => underline.push('\t'),
                    _ => underline.push(' '),
                }
            }

            underline.push('^');
            if end - start > 1 {
                for _ in 2..(end - start) {
                    underline.push('-');
                }
                underline.push('^');
            }

            underline
        }

        if let Some((next_lineno, continued_line)) = continued_line {
            let has_line_gap = next_lineno - lineno > 1;
            if has_line_gap {
                format!(
                    "{s    }--> {p}:{ls}:{c}\n\
                     {s    } |\n\
                     {ls:w$} | {line}\n\
                     {s    } | ...\n\
                     {le:w$} | {continued_line}\n\
                     {s    } | {underline}\n\
                     {s    } |\n\
                     {s    } = {message}",
                    s = spacing,
                    w = spacing.len(),
                    p = path,
                    ls = lineno,
                    le = next_lineno,
                    c = colno,
                    line = line,
                    continued_line = continued_line,
                    underline = underline(colno, line),
                    message = message,
                )
            } else {
                format!(
                    "{s    }--> {p}:{ls}:{c}\n\
                     {s    } |\n\
                     {ls:w$} | {line}\n\
                     {le:w$} | {continued_line}\n\
                     {s    } | {underline}\n\
                     {s    } |\n\
                     {s    } = {message}",
                    s = spacing,
                    w = spacing.len(),
                    p = path,
                    ls = lineno,
                    le = next_lineno,
                    c = colno,
                    line = line,
                    continued_line = continued_line,
                    underline = underline(colno, line),
                    message = message,
                )
            }
        } else {
            format!(
                "{s}--> {p}:{l}:{c}\n\
                 {s} |\n\
                 {l} | {line}\n\
                 {s} | {underline}\n\
                 {s} |\n\
                 {s} = {message}",
                s = spacing,
                p = path,
                l = lineno,
                c = colno,
                line = line,
                underline = underline(colno, line),
                message = message,
            )
        }
    }

    format_impl(filename, report, line, continued_line)
}

fn print_lint_report(filename: &str, linter: &mut Linter) -> bool {
    let mut report_str = String::new();
    linter.rule_registry.rule_ctx.iter().for_each(|(name, rule)| {
        let mut rule_report_str = String::new();
        rule.get_reports().iter().for_each(|report: &LintReport| {
            let mut report_tmp: LintReport = report.clone();
            report_tmp.pos.file = filename.to_string();
            rule_report_str.push_str(&format!("{}\n", format_report(filename, &report_tmp)));
        });
        if !rule_report_str.is_empty() {
            report_str.push_str(&format!("[rule] {}:\n{}", name, rule_report_str));
        }
    });
    if report_str.is_empty() {
        println!("== lint report");
        println!("ok");
        true
    } else {
        eprintln!("== lint report");
        eprintln!("{}", report_str);
        false
    }
}

pub fn drive(lua_src: &str, linter: &mut Linter) -> String {
    let lua_src = linter.rule_registry.trigger_preprocess(lua_src);
    let tokens = full_moon::tokenizer::tokens(lua_src.as_str()).unwrap();
    let tokens = lint::lint_tokens(&tokens, linter);
    let input_ast = full_moon::ast::Ast::from_tokens(tokens).unwrap();

    let (_formatted_ast, _ctx) = lint::lint_visitor::lint_ast(&input_ast, linter);
    // should return stringified _formatted_ast
    //   but currently we don't provide a way to do that
    lua_src
}

/// Takes a string of jsonc content and returns a comment free version
/// which should parse fine as regular json.
/// Nested block comments are supported.
/// preserve_locations will replace most comments with spaces, so that JSON parsing
/// errors should point to the right location.
pub fn strip_jsonc_comments(jsonc_input: &str, preserve_locations: bool) -> String {
    let mut json_output = String::new();

    let mut block_comment_depth: u8 = 0;
    let mut is_in_string: bool = false; // Comments cannot be in strings

    for line in jsonc_input.split('\n') {
        let mut last_char: Option<char> = None;
        for cur_char in line.chars() {
            // Check whether we're in a string
            if block_comment_depth == 0 && last_char != Some('\\') && cur_char == '"' {
                is_in_string = !is_in_string;
            }

            // Check for line comment start
            if !is_in_string && last_char == Some('/') && cur_char == '/' {
                last_char = None;
                if preserve_locations {
                    json_output.push_str("  ");
                }
                break; // Stop outputting or parsing this line
            }
            // Check for block comment start
            if !is_in_string && last_char == Some('/') && cur_char == '*' {
                block_comment_depth += 1;
                last_char = None;
                if preserve_locations {
                    json_output.push_str("  ");
                }
            // Check for block comment end
            } else if !is_in_string && last_char == Some('*') && cur_char == '/' {
                if block_comment_depth > 0 {
                    block_comment_depth -= 1;
                }
                last_char = None;
                if preserve_locations {
                    json_output.push_str("  ");
                }
            // Output last char if not in any block comment
            } else {
                if block_comment_depth == 0 {
                    if let Some(last_char) = last_char {
                        json_output.push(last_char);
                    }
                } else {
                    if preserve_locations {
                        json_output.push_str(" ");
                    }
                }
                last_char = Some(cur_char);
            }
        }

        // Add last char and newline if not in any block comment
        if let Some(last_char) = last_char {
            if block_comment_depth == 0 {
                json_output.push(last_char);
            } else if preserve_locations {
                json_output.push(' ');
            }
        }

        // Remove trailing whitespace from line
        while json_output.ends_with(' ') {
            json_output.pop();
        }
        json_output.push('\n');
    }

    json_output
}