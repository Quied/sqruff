use crate::core::config::Value;
use crate::core::parser::segments::base::{SymbolSegment, SymbolSegmentNewArgs};
use crate::core::rules::base::{ErasedRule, LintFix, LintResult, Rule};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{Crawler, SegmentSeekerCrawler};

#[derive(Default, Clone, Debug)]
pub struct RuleCV05 {}

impl Rule for RuleCV05 {
    fn name(&self) -> &'static str {
        "convention.is_null"
    }

    fn description(&self) -> &'static str {
        "Relational operators should not be used to check for NULL values."
    }

    fn crawl_behaviour(&self) -> Crawler {
        SegmentSeekerCrawler::new(["comparison_operator"].into()).into()
    }

    fn load_from_config(&self, _config: &ahash::AHashMap<String, Value>) -> ErasedRule {}

    fn eval(&self, rule_cx: RuleContext) -> Vec<LintResult> {
        if rule_cx.parent_stack.len() >= 2
            && rule_cx.parent_stack[rule_cx.parent_stack.len() - 2].is_type(&[
                "set_clause_list",
                "execute_script_statement",
                "options_segment",
            ])
        {
            return Vec::new();
        }

        if !rule_cx.parent_stack.is_empty()
            && rule_cx.parent_stack[rule_cx.parent_stack.len() - 1].is_type(&[
                "set_clause_list",
                "execute_script_statement",
                "assignment_operator",
            ])
        {
            return Vec::new();
        }

        if !rule_cx.parent_stack.is_empty()
            && rule_cx.parent_stack[rule_cx.parent_stack.len() - 1]
                .is_type("exclusion_constraint_element")
        {
            return Vec::new();
        }

        if rule_cx.segment.get_raw()

    }
}
