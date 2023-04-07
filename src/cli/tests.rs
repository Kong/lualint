

// test read rules from file
#[test]
fn test_read_rules_from_file() {
    use super::{build_config_linter};
    let enabled_rules = "scripts/all_rules.jsonc";
    let out = build_config_linter(enabled_rules);
    assert!(out.is_some());
    let linter = out.unwrap();
    assert!(!linter.rule_registry.get_all_ctx().is_empty());
}

// test ignore lines
#[test]
fn test_ignore_lines() {
    use super::{build_config_linter};
    use crate::cli::{SpecificRanges, drive, print_lint_report};

    let ignore_range = r"test_ignore_lines.txt,1,2
test_ignore_lines.txt,4,4";
    let filename = "test_ignore_lines.txt";
    let enabled_rules = r#"{"max_column_width": {}}"#;
    let lua_src= r#"line1 = 1
line2 = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
line3 = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
line4 = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
line5 = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
"#;
    let ignore = SpecificRanges::from_csv(ignore_range);
    let mut linter = match build_config_linter(enabled_rules) {
        Some(value) => value,
        None => return,
    };
    let mut stdout = Vec::new();
    let _ = drive(&lua_src, &mut linter);
    let _ = print_lint_report(filename,Some(&lua_src), &mut linter, Some(&ignore), None, &mut stdout);
    assert_eq!(String::from_utf8(stdout).unwrap(), r#"== lint report
[rule] max_column_width:
 --> test_ignore_lines.txt:3:86
  |
3 | line3 = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
4 | line4 = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
  |                                                                                      ^
  |
  = Line is expected to be at most 80 characters, but is more than 87 characters
 --> test_ignore_lines.txt:5:86
  |
5 | line5 = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
  |                                                                                      ^
  |
  = Line is expected to be at most 80 characters, but is more than 87 characters

"#);
}
