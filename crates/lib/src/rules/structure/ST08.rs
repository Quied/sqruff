use std::ascii::AsciiExt;
use std::mem::replace;

use crate::core::config::Value;
use crate::core::parser::segments::base::{ErasedSegment, WhitespaceSegment};
use crate::core::parser::segments::bracketed;
use crate::core::parser::segments::keyword::KeywordSegment;
use crate::core::rules::base::{Erased, ErasedRule, LintFix, LintResult, Rule};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{Crawler, SegmentSeekerCrawler};
use crate::helpers::IndexMap;
use crate::utils::analysis::query::Query;
use crate::utils::functional::context::FunctionalContext;
use crate::utils::functional::segments::Segments;
use crate::utils::reflow::sequence::ReflowSequence;

#[derive(Debug, Default, Clone)]
pub struct RuleST08 {
    filter_meta: i32,
}

impl RuleST08 {
    pub fn remove_unneeded_brackets(
        self,
        context: RuleContext,
        bracketed: Segments,
    ) -> (ErasedSegment, ReflowSequence) {
        let anchor = &bracketed.get(1, None).unwrap();
        let seq = ReflowSequence::from_around_target(
            anchor,
            context.parent_stack[0].clone(),
            "before",
            context.config.unwrap(),
        );

        let slice_ref = std::slice::from_ref(anchor);
        //let filtered = replace(&mut anchor, RuleST08::filter_meta(slice_ref, (anchor.segments());
        

        (anchor.clone(), seq)
    }

    pub fn filter_meta(segments: &[ErasedSegment], keep_meta: bool) -> Vec<ErasedSegment> {
        let mut buff = Vec::new();
        for elem in segments {
            if elem.is_meta() == keep_meta {
                buff.push(elem.clone());
            }
        }
        buff
    }
    
}
impl Rule for RuleST08 {
    fn name(&self) -> &'static str {
        "structure.distinc"
    }

    fn load_from_config(&self, _config: &ahash::AHashMap<String, Value>) -> ErasedRule {
        RuleST08::default().erased()
    }

    fn crawl_behaviour(&self) -> Crawler {
        SegmentSeekerCrawler::new(["select_statement"].into()).into()
    }

    fn description(&self) -> &'static str {
        "Looking for DISTINCT before a bracket"
    }

    fn eval(&self, rule_cx: RuleContext) -> Vec<LintResult> {
        let seq = None;
        let anchor = None;
        let children = FunctionalContext::new(rule_cx).segment().children(None);

        if rule_cx.segment.is_type("select_clause") {
            let modifier =
                children.select(Some(|it| it.is_type("select_clause_modifier")), None, None, None);
            let first_element = children
                .select(Some(|it| it.is_type("select_clause_element")), None, None, None)
                .first();

            let expression = first_element.and_then(|element| {
                if let Some(child) = element.children(&["expression"]).first() {
                    Some(child)
                } else {
                    Some(element)
                }
            });

            let bracketed = expression.unwrap().children(&["bracketed"]).first();

            if !modifier.is_empty() && bracketed.is_some() {
                if expression.unwrap().segments().len() == 1 {
                    // (anchor, seq) = self.remove_unneeded_brackets(rule_cx,
                    // bracketed)
                }
            } else {
                anchor = Some(modifier[0]);
                seq = Some(ReflowSequence::from_around_target(
                    &modifier[0],
                    rule_cx.parent_stack[0],
                    "after",
                    rule_cx.config.unwrap(),
                ));
            }
        } else if rule_cx.segment.is_type("function") {
            anchor = Some(rule_cx.parent_stack[rule_cx.parent_stack.len() - 1]);

            if anchor.unwrap().is_type("expression") || anchor.unwrap().segments().len() != 1 {
                return Vec::new();
            }

            let function_name =
                children.select(Some(|it| it.is_type("function_name")), None, None, None).first();
            let bracketed = children.first();

            if function_name.is_none()
                || function_name.unwrap().get_raw_upper() != Some(String::from("DISTINCT"))
                || bracketed.is_none()
            {
                return Vec::new();
            }

            return vec![LintResult::new(
                anchor.clone(),
                vec![LintFix::replace(
                    anchor.clone().unwrap(),
                    vec![
                        KeywordSegment::new("DISTINCT".to_owned()),
                        WhitespaceSegment::default(),
                    ],
                )],
                None,
                None,
                None,
            )];
            
            // return Some(LintResult {
            //     anchor: anchor.clone(),
            //     fixes: vec![LintFix::replace(
            //         anchor.clone(),
            //         vec![
            //             KeywordSegment::new("DISTINCT"),
            //             WhitespaceSegment::default(),
            //         ]
            //         .into_iter()
            //         .chain(self.filter_meta(bracketed.unwrap().segments)[1..bracketed.
            // unwrap().segments.len() - 1].to_vec().into_iter())
            //         .collect(),
            //     )],
            // });

            if seq.is_some() && anchor.is_some() {
                // let fixes = seq
            }
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {


    use pretty_assertions::assert_eq;

    use crate::api::simple::fix;
    use crate::core::rules::base::{Erased, ErasedRule};
    use crate::rules::structure::ST08::RuleST08;

    fn rules() -> Vec<ErasedRule> {
        vec![RuleST08::default().erased()]
    }

    fn test_fail_distinct_with_parenthesis_1() {
        let fail_str = "SELECT DISTINCT(a)";
        let fix_str = "SELECT DISTINCT a";

        let fixed = fix(fail_str.into(), rules());
        assert_eq!(fix_str, fixed);
    }
    fn test_fail_distinct_with_parenthesis_2() {
        let fail_str = "SELECT DISTINCT(a + b) * c";
        let fix_str = "SELECT DISTINCT (a + b) * c";

        let fixed = fix(fail_str.into(), rules());
        assert_eq!(fix_str, fixed);
    }
    fn test_fail_distinct_with_parenthesis_3() {
        let fail_str = "SELECT DISTINCT (a)";
        let fix_str = "SELECT DISTINCT a";

        let fixed = fix(fail_str.into(), rules());
        assert_eq!(fix_str, fixed);
    }
    fn test_fail_distinct_with_parenthesis_4() {
        let pass_str = "SELECT DISTINCT(a)";

    }
    fn test_fail_distinct_with_parenthesis_5() {
        let fail_str = r#"SELECT DISTINCT (field_1)
                                FROM my_table"#;

        let fix_str = "SELECT DISTINCT field_1
                             FROM my_table";

        let fixed = fix(fail_str.into(), rules());
        assert_eq!(fix_str, fixed);
    }
    fn test_fail_distinct_with_parenthesis_6() {
        let fail_str = "SELECT DISTINCT(a), b";
        let fix_str = "SELECT DISTINCT a,b";

        let fixed = fix(fail_str.into(), rules());
        assert_eq!(fix_str, fixed);
    }
    fn test_fail_distinct_with_parenthesis_7() {
        let pass_str = r#"SELECT DISTINCT ON(bcolor) bcolor, fcolor
                                FROM distinct_demo"#;

    }

    fn test_pass_no_distinct() {
        let fail_str = "SELECT a,b";
    }

    fn test_fail_distinct_column_inside_count() {
        let fail_str = "SELECT COUNT(DISTINCT(unique_key))";
        let fix_str = "SELECT COUNT(DISTINCT unique_key)";

        let fixed = fix(fail_str.into(), rules());
        assert_eq!(fix_str, fixed);
    }
    
    fn test_fail_distinct_concat_inside_count() {
        let fail_str = "SELECT COUNT (DISTINCT(CONCAT(col1, '-', col2, '-', col3)))";
        let fix_str = "SELECT COUNT (DISTINCT CONCAT(col1, '-', col2, '-', col3))";

        let fixed = fix(fail_str.into(), rules());
        assert_eq!(fix_str, fixed);
    }
}
