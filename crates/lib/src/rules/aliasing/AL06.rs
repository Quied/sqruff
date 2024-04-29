use crate::core::config::Value;
use crate::core::parser::segments::base::ErasedSegment;
use crate::core::rules::base::{Erased, ErasedRule, LintResult, Rule};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{Crawler, SegmentSeekerCrawler};
use crate::utils::functional::context::FunctionalContext;
// use crate::utils::functional::context::FunctionalContext;
// use crate::utils::functional::segments::Segments;

#[derive(Debug, Clone, Default)]
pub struct RuleAL06 {
    min_alias_lenght: Option<u32>,
    max_alias_lenght: Option<u32>,
}

impl RuleAL06 {
    fn lint_aliases(&self, from_expression_elements: Vec<ErasedSegment>) -> Vec<LintResult> {
        let mut violation_buff = Vec::new();

        for from_expression_element in from_expression_elements {
            let table_ref = if let Some(table_expression) =
                from_expression_element.child(&["table_expression"])
            {
                table_expression.child(&["object_reference"])
            } else {
                None
            };

            if table_ref.is_none() {
                continue;
            }

            let alias_exp_ref = from_expression_element.child(&["alias_expression"]);
            if alias_exp_ref.is_none() {
                continue;
            }

            if self.min_alias_lenght.is_some() {
                if let Some(alias_identifier_ref) =
                    alias_exp_ref.clone().unwrap().child(&["identifier"])
                {
                    let alias_identifier = alias_identifier_ref.get_raw();
                    if String::len(&alias_identifier.unwrap())
                        > self.min_alias_lenght.unwrap() as usize
                    {
                        violation_buff.push(LintResult::new(
                            Some(alias_identifier_ref),
                            Vec::new(),
                            None,
                            format!(
                                "Aliases should be at least '{:?}' character(s) long",
                                self.min_alias_lenght
                            )
                            .into(),
                            None,
                        ))
                    }
                }
            }

            if self.max_alias_lenght.is_some() {
                if let Some(alias_identifier_ref) = alias_exp_ref.unwrap().child(&["identifier"]) {
                    let alias_identifier = alias_identifier_ref.get_raw();
                    if String::len(&alias_identifier.unwrap())
                        > self.max_alias_lenght.unwrap() as usize
                    {
                        violation_buff.push(LintResult::new(
                            Some(alias_identifier_ref),
                            Vec::new(),
                            None,
                            format!(
                                "Aliases should be no more than '{:?}' character(s) long.",
                                self.max_alias_lenght
                            )
                            .into(),
                            None,
                        ))
                    }
                }
            }
        }

        violation_buff
    }
}

impl Rule for RuleAL06 {
    fn name(&self) -> &'static str {
        "aliasing.lenght"
    }

    fn description(&self) -> &'static str {
        "Identify aliases in from clause and join conditions"
    }

    fn load_from_config(&self, _config: &ahash::AHashMap<String, Value>) -> ErasedRule {
        RuleAL06::default().erased()
    }

    fn eval(&self, rule_cx: RuleContext) -> Vec<LintResult> {
        // ==== < > ===
        // let children =
        // FunctionalContext::new(rule_cx.clone()).segment().children(None);

        let from_expression_elements =
            rule_cx.segment.recursive_crawl(&["from_expression_element"], true, None, true);

        self.lint_aliases(from_expression_elements)
    }

    fn crawl_behaviour(&self) -> Crawler {
        SegmentSeekerCrawler::new(["alias_expression"].into()).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::simple::{fix, lint};
    use crate::core::rules::base::{Erased, ErasedRule};
    use crate::rules::aliasing::AL06::RuleAL06;

    fn rules() -> Vec<ErasedRule> {
        vec![RuleAL06::default().erased()]
    }

    #[test]
    fn test_fail_alias_too_short() {
        let fail_str = r#"
        SELECT u.id, c.first_name, c.last_name,
            COUNT(o.user_id)
                FROM users AS u
                    JOIN customers AS c ON u.id = c.user_id
                    JOIN orders AS o ON u.id = o.user_id"#;

        let violations = lint(fail_str.into(), "ansi".into(), rules(), None, None).unwrap();

        // assert_eq!(violations.len(), 4);
    }

    #[test]
    fn test_pass_no_config() {
        let sql = r#"
        SELECT x.a, x_2.b
            FROM x 
            LEFT JOIN x AS x_2 ON x.foreign_key = x.foreign_key"#;

        let violations = lint(sql.into(), "ansi".into(), rules(), None, None).unwrap();
        assert_eq!(violations, []);
    }
}
