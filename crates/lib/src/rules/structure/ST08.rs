use itertools::{chain, Itertools};

use crate::core::config::Value;
use crate::core::parser::segments::base::{
    ErasedSegment, SymbolSegment, SymbolSegmentNewArgs, WhitespaceSegment, WhitespaceSegmentNewArgs,
};
use crate::core::parser::segments::keyword::KeywordSegment;
use crate::core::rules::base::{Erased, ErasedRule, LintFix, LintResult, Rule};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{Crawler, SegmentSeekerCrawler};
use crate::helpers::ToErasedSegment;
use crate::utils::functional::context::FunctionalContext;
use crate::utils::functional::segments::Segments;
use crate::utils::reflow::sequence::{Filter, ReflowSequence};

#[derive(Debug, Default, Clone)]
pub struct RuleST08 {}

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
        )
        .replace(anchor.clone(), &Self::filter_meta(anchor.segments(), false)); // ? 

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
        let mut seq = None;
        let mut anchor = None;
        let children = FunctionalContext::new(rule_cx.clone()).segment().children(None);

        if rule_cx.segment.is_type("select_clause") {
            let modifier =
                children.select(Some(|it| it.is_type("select_clause_modifier")), None, None, None);

            let selected_elements =
                children.select(Some(|it| it.is_type("select_clause_element")), None, None, None);
            let first_element = selected_elements.first();

            let expression = first_element.and_then(|element| {
                if let Some(child) = element.children(&["expression"]).first() {
                    Some(child.clone())
                } else {
                    Some(element.clone())
                }
            });

            let cloned_expression = expression.clone().unwrap();
            let bracketed_children = cloned_expression.children(&["bracketed"]);
            let bracketed = bracketed_children.first();

            if !modifier.is_empty() && bracketed.is_some() {
                if expression.unwrap().segments().len() == 1 {
                    // (anchor, seq) =
                    // self.remove_unneeded_brackets(rule_cx.clone(),
                    // bracketed.);
                }
            } else {
                anchor = Some(modifier[0].clone());
                seq = Some(ReflowSequence::from_around_target(
                    &modifier[0],
                    rule_cx.parent_stack[0].clone(),
                    "after",
                    rule_cx.config.clone().unwrap(),
                ));
            }
        } else if rule_cx.segment.is_type("function") {
            anchor = Some(rule_cx.parent_stack[rule_cx.parent_stack.len() - 1].clone());

            if anchor.clone().unwrap().is_type("expression")
                || anchor.clone().unwrap().segments().len() != 1
            {
                return Vec::new();
            }

            let selected_functions =
                children.select(Some(|it| it.is_type("function_name")), None, None, None);
            let function_name = selected_functions.first();

            let bracketed = children.first();

            if function_name.is_none()
                || function_name.unwrap().get_raw_upper() != Some(String::from("DISTINCT"))
                || bracketed.is_none()
            {
                let edits = vec![SymbolSegment::create(
                    "DISTINCT",
                    &<_>::default(),
                    SymbolSegmentNewArgs { r#type: "function_name_identifier" },
                )];
    
                let fixes = vec![LintFix::replace(anchor.clone().unwrap(), edits.to_vec(), None)];
    
                return vec![LintResult::new(anchor, fixes, None, None, None)];
            }

            let edits = vec![SymbolSegment::create(
                "DISTINCT",
                &<_>::default(),
                SymbolSegmentNewArgs { r#type: "function_name_identifier" },
            )];

            let fixes = vec![LintFix::replace(anchor.clone().unwrap(), edits.to_vec(), None)];

            return vec![LintResult::new(anchor, fixes, None, None, None)];
        }

        if let Some(seq) = seq {
            if let Some(anchor) = anchor {
                let fixes = seq.respace(false, Filter::All).fixes();

                if !fixes.is_empty() {
                    return vec![LintResult::new(Some(anchor), fixes, None, None, None)];
                }
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
