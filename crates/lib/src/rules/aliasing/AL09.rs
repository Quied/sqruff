use crate::core::config::Value;
use crate::core::parser::segments::base::ErasedSegment;
use crate::core::rules::base::{Erased, ErasedRule, LintFix, LintResult, Rule};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{Crawler, SegmentSeekerCrawler};
use crate::utils::functional::context::FunctionalContext;

#[derive(Default, Clone, Debug)]
pub struct RuleAL09 {}

impl Rule for RuleAL09 {
    fn name(&self) -> &'static str {
        "aliasing.self_alias.column"
    }

    fn description(&self) -> &'static str {
        "Find self-aliased columns and fix them"
    }

    fn load_from_config(&self, _config: &ahash::AHashMap<String, Value>) -> ErasedRule {
        RuleAL09::default().erased()
    }

    fn eval(&self, rule_cx: RuleContext) -> Vec<LintResult> {
        let mut violations = Vec::new();

        let children = FunctionalContext::new(rule_cx).segment().children(None);

        for clause_element in
            children.select(Some(|sp| sp.is_type("select_clause_element")), None, None, None)
        {
            let clause_element_raw_segment = clause_element.get_raw_segments();

            let column = clause_element.child(&["column_reference"]);
            let alias_expression = clause_element.child(&["alias_expression"]);

            if let Some(column) = column {
                if let Some(alias_expression) = alias_expression {
                    if column.child(&["identifier", "naked_identifier"]).is_some()
                        || column.child(&["quoted_identifier"]).is_some()
                    {
                        let whitespace = clause_element.child(&["whitespace"]).unwrap();

                        let column_identifier =
                            if let Some(quoted_identifier) = column.child(&["quoted_identifier"]) {
                                quoted_identifier.clone()
                            } else {
                                column
                                    .children(&["identifier", "naked_identifier"])
                                    .last()
                                    .expect("No naked_identifier found")
                                    .clone()
                            };

                        let alias_identifier = alias_expression
                            .child(&["naked_identifier"])
                            .or_else(|| alias_expression.child(&["quoted_identifier"]))
                            .expect("identifier is none");

                        if column_identifier.get_raw_upper() == alias_identifier.get_raw_upper() {
                            let mut fixes: Vec<LintFix> = Vec::new();

                            fixes.push(LintFix::delete(whitespace));
                            fixes.push(LintFix::delete(alias_expression));

                            violations.push(LintResult::new(
                                Some(clause_element_raw_segment[0].clone()),
                                fixes,
                                None,
                                Some(format!("Column should not be self-aliased.")),
                                None,
                            ));
                        }
                    }
                }
            }
        }
        violations
    }

    fn crawl_behaviour(&self) -> Crawler {
        SegmentSeekerCrawler::new(["select_clause"].into()).into()
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;

    use crate::api::simple::{fix, lint};
    use crate::core::rules::base::{Erased, ErasedRule};
    use crate::rules::aliasing::AL09::RuleAL09;

    fn rules() -> Vec<ErasedRule> {
        vec![RuleAL09::default().erased()]
    }

    #[test]
    pub fn test_pass_no_alias() {
        let sql = r#"SELECT col_a, col_b FROM foo"#;
        let violations = lint(sql.into(), "ansi".into(), rules(), None, None).unwrap();

        assert_eq!(violations, []);
    }

    #[test]
    pub fn test_pass_no_self_alias() {
        let sql = "SELECT col_a, col_b AS new_col_b FROM foo";
        let violations = lint(sql.into(), "ansi".into(), rules(), None, None).unwrap();

        assert_eq!(violations, []);
    }

    #[test]
    pub fn test_pass_no_self_alias_function() {
        let sql = r#"SELECT max(sum) AS max_sum FROM foo"#;
        let violations = lint(sql.into(), "ansi".into(), rules(), None, None).unwrap();

        assert_eq!(violations, []);
    }

    #[test]
    pub fn test_fail_self_alias() {
        let fail_str = "SELECT col_a AS col_a, col_b AS new_col_b FROM foo";
        let fix_str = "SELECT col_a, col_b AS new_col_b FROM foo";

        let result = fix(fail_str.into(), rules());
        assert_eq!(fix_str, result);
    }

    #[test]
    pub fn test_fail_self_alias_upper() {
        let fail_str = "SELECT col_a AS COL_A col_b AS new_col_b FROM foo";
        let fix_str = "SELECT col_a, col_b AS new_col_b FROM foo";

        let result = fix(fail_str.into(), rules());
        assert_eq!(fix_str, result);
    }

    #[test]
    pub fn test_fail_self_alias_implicit() {
        let fail_str = r#"SELECT col_a col_a col_b AS new_col_b FROM foo"#;
        let fix_str = r#"SELECT col_a, col_b AS new_col_b FROM foo"#;

        let result = fix(fail_str.into(), rules());
        assert_eq!(fix_str, result);
    }

    #[test]
    pub fn test_fail_self_alias_and_table_aliased() {
        let fail_str = r#"SELECT a.col_a AS col_a, a.col_b AS new_col_b FROM foo"#;
        let fix_str = r#"SELECT a.col_a, a.col_b AS new_col_b FROM foo AS a"#;

        let result = fix(fail_str.into(), rules());
        assert_eq!(fix_str, result);
    }

    #[test]
    pub fn test_fail_self_alias_quoted() {
        let fail_str = r#"SELECT "col_a" AS "col_a", col_b AS new_col_b FROM foo"#;
        let fix_str = r#"SELECT "col_a", col_b AS new_col_b FROM foo"#;

        let result = fix(fail_str.into(), rules());
        assert_eq!(fix_str, result);
    }

    #[test]
    pub fn test_pass_self_alias_case_insensitive() {
        let sql = r#"SELECT "col_a" AS col_a, col_b AS new_col_b FROM foo"#;
        let violations = lint(sql.into(), "ansi".into(), rules(), None, None).unwrap();

        assert_eq!(violations, []);
    }

    #[test]
    pub fn test_pass_self_alias_case_sensitive() {
        let sql = r#"SELECT col_a AS "col_a", col_b AS new_col_b FROM foo"#;
        let violations = lint(sql.into(), "ansi".into(), rules(), None, None).unwrap();

        assert_eq!(violations, []);
    }
}
