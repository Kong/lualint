use clap::Subcommand;
use log::{error, info, trace};
use lualint::{
    lint::{self, linter_builder, Linter, LinterBuilder},
    rules::{self, LintReport},
};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
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
pub(crate) fn handle_run_command(filename: &str, enabled_rules: &str) {
    let enabled_rules_vec: Vec<(RuleName, RuleConfig)> = match parse_enabled_rules(enabled_rules) {
        Ok(enabled_rules) => enabled_rules,
        Err(e) => {
            error!("Failed to parse enabled rules: {}. Value for param `--rules` msut be a valid json map. For example: {}", e, 
            r#"{ "rule_name1": {}, "rule_name2": { "key": "value" } }"#);
            trace!("given value: {}", enabled_rules);
            return;
        }
    };
    let mut linter_builder = lint::LinterBuilder::new();
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
                return;
            }
        }
    }
    let mut linter = linter_builder.build();
    lint_file(filename, &mut linter, false);
}

fn parse_enabled_rules(
    enabled_rules: &str,
) -> Result<Vec<(RuleName, RuleConfig)>, Box<dyn std::error::Error>> {
    // parse as json

    let v = serde_json::from_str::<serde_json::Value>(enabled_rules)?;

    let mut enabled_rules: Vec<(RuleName, RuleConfig)> = vec![];

    for (rule_name, rule_config) in v.as_object().unwrap() {
        enabled_rules.push((rule_name.to_string(), rule_config.to_owned()));
    }

    Ok(enabled_rules)
}

pub(crate) fn print_rules() {
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

pub(crate) fn lint_file(filename: &str, linter: &mut Linter, write_back: bool) {
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
    let has_error = false;
    let exit_on_err = true; // TODO: make this configurable
    let processed = match std::fs::read_to_string(filename) {
        Ok(lua_src) => {
            let (out, has_error) = drive(filename, &lua_src, linter);
            if has_error && exit_on_err {
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

// TODO: filename shoud not be dependency
pub fn drive(filename: &str, lua_src: &str, linter: &mut Linter) -> (String, bool) {
    let lua_src = linter.rule_registry.trigger_preprocess(lua_src);
    let tokens = full_moon::tokenizer::tokens(lua_src.as_str()).unwrap();
    let tokens = lint::lint_tokens(&tokens, linter);
    let input_ast = full_moon::ast::Ast::from_tokens(tokens).unwrap();

    let (_formatted_ast, ctx) = lint::lint_visitor::lint_ast(&input_ast, linter);

    // println!("{}", full_moon::print(&formatted_ast));

    let mut report_str = String::new();
    ctx.linter.rule_registry.rule_ctx.iter().for_each(|(name, rule)| {
        let mut rule_report_str = String::new();
        rule.get_reports().iter().for_each(|report: &LintReport| {
            let mut report_tmp: LintReport = report.clone();
            report_tmp.pos.file = filename.to_string();
            rule_report_str.push_str(&format!("--  {:?}\n", report_tmp));
        });
        if rule_report_str.len() > 0 {
            report_str.push_str(&format!("[rule] {}:\n{}", name, rule_report_str));
        }
    });
    // TODO: move side effects out of this function
    if report_str.len() == 0 {
        println!("== lint report");
        println!("ok");
    } else {
        eprintln!("== lint report");
        eprintln!("{}", report_str);
    }

    return (lua_src.to_string(), report_str.len() > 0);
}
