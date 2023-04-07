use std::{
    collections::HashMap,
    io::{self, Read},
};

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
        /// Ignore ranges in the file, optional
        #[clap(long)]
        ignore: Option<String>,
    },
    Rules,
}

pub struct IgnoreFileRanges {
    pub file: String,
    pub ranges: Vec<(usize, usize)>,
}

pub struct IgnoreRanges {
    pub file_ranges: Vec<IgnoreFileRanges>,
    _cache: HashMap<String, Vec<(usize, usize)>>,
}

impl IgnoreRanges {
    pub fn new() -> Self {
        Self { file_ranges: Vec::new(), _cache: HashMap::new() }
    }

    pub fn from_csv(csv: &str) -> Self {
        let mut ign_ranges = Self::new();
        for line in csv.lines() {
            // skip empty lines
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split(',');
            // warn on invalid lines
            if parts.clone().count() != 3 {
                error!("Invalid ignore line: {}", line);
                continue;
            }
            let file = parts.next().unwrap();
            let start = parts.next().unwrap().parse::<usize>().unwrap();
            let end = parts.next().unwrap().parse::<usize>().unwrap();
            ign_ranges.add(file, start, end);
        }
        ign_ranges.cache();
        ign_ranges
    }

    pub fn add(&mut self, file: &str, start: usize, end: usize) {
        let mut found_file = false;
        for file_range in &mut self.file_ranges {
            if file_range.file == file {
                file_range.ranges.push((start, end));
                found_file = true;
                break;
            }
        }
        if !found_file {
            self.file_ranges
                .push(IgnoreFileRanges { file: file.to_string(), ranges: vec![(start, end)] });
        }
    }

    fn cache(&mut self) {
        for file_range in &self.file_ranges {
            self._cache.insert(file_range.file.clone(), file_range.ranges.clone());
        }
    }

    pub fn is_ignored(&self, filename: &str, line: usize) -> bool {
        if let Some(ranges) = self._cache.get(filename) {
            for (start, end) in ranges {
                if line >= *start && line <= *end {
                    return true;
                }
            }
        }
        false
    }
}

type RuleName = String;
type RuleConfig = serde_json::Value;
// enabled_rules = "[rule_name1:{},rule_name2:{key: value, key: value}]"
pub fn handle_run_command(filename: &str, enabled_rules: &str, ignore_file_opt: Option<String>) {
    let mut linter = match build_config_linter(enabled_rules) {
        Some(value) => value,
        None => return,
    };
    if let Some(ignore_file) = ignore_file_opt {
        let mut ignore_file_content = String::new();
        let mut ignore_file = std::fs::File::open(ignore_file).unwrap();
        ignore_file.read_to_string(&mut ignore_file_content).unwrap();
        let ignore = IgnoreRanges::from_csv(&ignore_file_content);
        lint_file(filename, &mut linter, true, Some(&ignore), &mut io::stdout());
    } else {
        lint_file(filename, &mut linter, true, None, &mut io::stdout());
    }
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

pub fn lint_file(
    filename: &str,
    linter: &mut Linter,
    write_back: bool,
    ignore: Option<&IgnoreRanges>,
    writer: &mut dyn io::Write,
) {
    let is_file_existing = std::path::Path::new(filename).exists();
    if !is_file_existing {
        // println!("File not found: {filename}");
        writeln!(writer, "File not found: {}", filename).unwrap();
        return;
    }

    let is_lua_file = filename.ends_with(".lua");
    if !is_lua_file {
        // println!("File is not a lua file: {filename}");
        writeln!(writer, "File is not a lua file: {}", filename).unwrap();
        return;
    }
    let _exit_on_err = true; // TODO: make this configurable
    let processed = match std::fs::read_to_string(filename) {
        Ok(lua_src) => {
            let out = drive(&lua_src, linter);
            let ok = print_lint_report(filename, Some(&lua_src), linter, ignore, writer);
            if !ok && _exit_on_err {
                std::process::exit(1);
            }
            out
        }
        Err(e) => {
            // println!("Error reading file: {e}");
            writeln!(writer, "Error reading file: {}", e).unwrap();
            return;
        }
    };

    if write_back {
        match std::fs::write(filename, processed) {
            Ok(_) => writeln!(writer, "Wrote file: {}", filename).unwrap(),
            Err(e) => writeln!(writer, "Error writing file: {}", e).unwrap(),
        }
    } else {
        // info!("Linted: {}", processed);
    }
}

fn format_report(filename: &str, file_content_opt: Option<&str>, report: &LintReport) -> String {
    let file_contents;
    if file_content_opt.is_none() {
        file_contents = std::fs::read_to_string(filename).unwrap();
    } else {
        file_contents = file_content_opt.unwrap().to_string();
    }
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

fn print_lint_report(
    filename: &str,
    file_content: Option<&str>,
    linter: &mut Linter,
    ignore: Option<&IgnoreRanges>,
    writer: &mut dyn io::Write,
) -> bool {
    let mut report_str = String::new();
    linter.rule_registry.rule_ctx.iter().for_each(|(name, rule)| {
        let mut rule_report_str = String::new();
        rule.get_reports().iter().for_each(|report: &LintReport| {
            let mut report_tmp: LintReport = report.clone();
            report_tmp.pos.file = filename.to_string();
            if let Some(ignore) = ignore {
                if ignore.is_ignored(filename, report.pos.line) {
                    return ();
                }
            }
            rule_report_str.push_str(&format!("{}\n", format_report(filename, file_content, &report_tmp)));
        });
        if !rule_report_str.is_empty() {
            report_str.push_str(&format!("[rule] {name}:\n{rule_report_str}"));
        }
    });
    if report_str.is_empty() {
        writeln!(writer, "== lint report").unwrap();
        writeln!(writer, "ok").unwrap();
        true
    } else {
        writeln!(writer, "== lint report").unwrap();
        writeln!(writer, "{report_str}", report_str = report_str).unwrap();
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
                block_comment_depth = block_comment_depth.saturating_sub(1);
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
                } else if preserve_locations {
                    json_output.push(' ');
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
