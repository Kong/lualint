
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