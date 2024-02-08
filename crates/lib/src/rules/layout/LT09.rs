use std::collections::HashSet;

use itertools::{enumerate, Itertools};

use crate::core::parser::segments::base::{NewlineSegment, Segment, WhitespaceSegment};
use crate::core::rules::base::{EditType, LintFix, LintResult, Rule};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{Crawler, SegmentSeekerCrawler};
use crate::utils::functional::context::FunctionalContext;
use crate::utils::functional::segments::Segments;

struct SelectTargetsInfo {
    select_idx: Option<usize>,
    first_new_line_idx: Option<usize>,
    first_select_target_idx: Option<usize>,
    first_whitespace_idx: Option<usize>,
    comment_after_select_idx: Option<usize>,
    select_targets: Segments,
    from_segment: Option<Box<dyn Segment>>,
    pre_from_whitespace: Segments,
}

#[derive(Debug)]
pub struct RuleLT09 {
    wildcard_policy: &'static str,
}

impl Rule for RuleLT09 {
    fn crawl_behaviour(&self) -> Crawler {
        SegmentSeekerCrawler::new(HashSet::from(["select_clause".into()])).into()
    }

    fn eval(&self, context: RuleContext) -> Vec<LintResult> {
        let select_targets_info = Self::get_indexes(context.clone());
        let select_clause = FunctionalContext::new(context.clone());

        // let wildcards = select_clause
        //     .children(sp.is_type("select_clause_element"))
        //     .children(sp.is_type("wildcard_expression"));
        let has_wildcard = false;

        if select_targets_info.select_targets.len() == 1
            && (!has_wildcard || self.wildcard_policy == "single")
        {
            return self.eval_single_select_target_element(select_targets_info, context);
        } else if !select_targets_info.select_targets.is_empty() {
            return self.eval_multiple_select_target_elements(select_targets_info, context.segment);
        }

        unimplemented!()
    }
}

impl RuleLT09 {
    fn get_indexes(context: RuleContext) -> SelectTargetsInfo {
        let children = FunctionalContext::new(context.clone()).segment().children(None);

        let select_targets = children.select(
            Some(|segment| segment.is_type("select_clause_element")),
            None,
            None,
            None,
        );

        let first_select_target_idx = children.find(select_targets.get(0, None).unwrap().as_ref());

        let selects = children.select(
            Some(|segment| {
                segment.get_type() == "keyword"
                    && segment.get_raw().unwrap().to_lowercase() == "select"
            }),
            None,
            None,
            None,
        );

        let select_idx = (!selects.is_empty())
            .then(|| children.find(selects.get(0, None).unwrap().as_ref()).unwrap());

        let newlines = children.select(Some(|it| it.is_type("newline")), None, None, None);

        let first_new_line_idx = (!newlines.is_empty())
            .then(|| children.find(newlines.get(0, None).unwrap().as_ref()).unwrap());

        if !newlines.is_empty() {
            let comment_after_select = children.select(
                Some(|seg| seg.is_type("comment")),
                Some(|seg| seg.is_type("comment") | seg.is_type("whitespace") | seg.is_meta()),
                selects.get(0, None).as_deref(),
                newlines.get(0, None).as_deref(),
            );

            if !comment_after_select.is_empty() {
                unimplemented!()
            }
        }

        let mut first_whitespace_idx = None;
        if let Some(first_new_line_idx) = first_new_line_idx {
            let segments_after_first_line = children.select(
                Some(|seg| seg.is_type("whitespace")),
                None,
                children[first_new_line_idx].as_ref().into(),
                None,
            );

            first_whitespace_idx =
                children.find(segments_after_first_line.get(0, None).unwrap().as_ref());
        }

        let siblings_post = FunctionalContext::new(context).siblings_post();
        let from_segment = siblings_post
            .find_first(Some(|seg: &dyn Segment| seg.is_type("from_clause")))
            .find_first::<fn(&dyn Segment) -> bool>(None)
            .get(0, None);
        let pre_from_whitespace = siblings_post.select(
            Some(|seg| seg.is_type("whitespace")),
            None,
            None,
            from_segment.as_deref(),
        );

        SelectTargetsInfo {
            select_idx,
            first_new_line_idx,
            first_select_target_idx,
            first_whitespace_idx,
            comment_after_select_idx: None,
            select_targets,
            from_segment,
            pre_from_whitespace,
        }
    }

    fn eval_multiple_select_target_elements(
        &self,
        select_targets_info: SelectTargetsInfo,
        segment: Box<dyn Segment>,
    ) -> Vec<LintResult> {
        let fixes = Vec::new();

        for (i, select_target) in enumerate(select_targets_info.select_targets.iter()) {
            let base_segment = if i == 0 {
                segment.clone()
            } else {
                select_targets_info.select_targets[i - 1].clone()
            };

            if let Some((a, b)) =
                base_segment.get_position_marker().zip(select_target.get_position_marker())
                && a.working_line_no == b.working_line_no
            {}

            if let Some(from_segment) = &select_targets_info.from_segment {}
        }

        fixes
    }

    fn eval_single_select_target_element(
        &self,
        select_targets_info: SelectTargetsInfo,
        context: RuleContext,
    ) -> Vec<LintResult> {
        let select_clause = FunctionalContext::new(context.clone()).segment();
        let parent_stack = context.parent_stack;

        if !(select_targets_info.select_idx < select_targets_info.first_new_line_idx
            && select_targets_info.first_new_line_idx < select_targets_info.first_select_target_idx)
        {
            return Vec::new();
        }

        let select_children = select_clause.children(None);
        let modifier = select_children
            .find_first(Some(|seg: &dyn Segment| seg.is_type("select_clause_modifier")));

        let insert_buff = vec![
            WhitespaceSegment::new(" ", &<_>::default(), <_>::default()),
            select_children[select_targets_info.first_select_target_idx.unwrap()].clone(),
        ];

        let mut fixes = vec![LintFix::delete(
            select_children[select_targets_info.first_select_target_idx.unwrap()].clone(),
        )];

        let start_idx = if !modifier.is_empty() {
            unimplemented!()
        } else {
            select_targets_info.first_select_target_idx.unwrap()
        };

        if !parent_stack.is_empty() && parent_stack.last().unwrap().is_type("select_statement") {
            let select_stmt = parent_stack.last().unwrap();
            let select_clause_idx = select_stmt
                .get_segments()
                .iter()
                .position(|it| it.dyn_eq(select_clause.get(0, None).as_deref().unwrap()))
                .unwrap();
            let after_select_clause_idx = select_clause_idx + 1;

            let fixes_for_move_after_select_clause =
                |fixes: &mut Vec<LintFix>,
                 stop_seg: Box<dyn Segment>,
                 delete_segments: Option<Segments>,
                 add_newline: bool| {
                    let start_seg = if !modifier.is_empty() {
                        modifier[0].clone()
                    } else {
                        select_children[select_targets_info.first_new_line_idx.unwrap()].clone()
                    };
                    let move_after_select_clause = select_children.select(
                        None,
                        None,
                        start_seg.as_ref().into(),
                        stop_seg.as_ref().into(),
                    );
                    let mut local_fixes = Vec::new();
                    let mut all_deletes = fixes
                        .iter()
                        .filter(|fix| fix.edit_type == EditType::Delete)
                        .map(|fix| fix.anchor.clone())
                        .collect_vec();
                    for seg in delete_segments.unwrap_or_default() {
                        fixes.push(LintFix::delete(seg.clone()));
                        all_deletes.push(seg);
                    }

                    if !move_after_select_clause.is_empty() || add_newline {
                        local_fixes.push(LintFix::create_after(
                            select_clause[0].clone_box(),
                            if add_newline {
                                vec![NewlineSegment::new("\n", &<_>::default(), <_>::default())]
                            } else {
                                vec![]
                            }
                            .into_iter()
                            .chain(move_after_select_clause)
                            .collect_vec(),
                            None,
                        ));
                    }

                    local_fixes
                };

            if select_stmt.get_segments().len() > after_select_clause_idx {
                if select_stmt.get_segments()[after_select_clause_idx].is_type("newline") {
                    let to_delete = select_children.reversed().select(
                        None,
                        Some(|seg| seg.is_type("whitespace")),
                        select_children[start_idx].as_ref().into(),
                        None,
                    );

                    if !to_delete.is_empty() {
                        let delete_last_newline =
                            select_children[start_idx - to_delete.len() - 1].is_type("newline");

                        if delete_last_newline {
                            fixes.push(LintFix::delete(
                                select_stmt.get_segments()[after_select_clause_idx].clone(),
                            ));
                        }

                        let new_fixes = fixes_for_move_after_select_clause(
                            &mut fixes,
                            to_delete.last().unwrap().clone_box(),
                            to_delete.into(),
                            true,
                        );

                        fixes.extend(new_fixes);
                    }
                } else if select_stmt.get_segments()[after_select_clause_idx].is_type("whitespace")
                {
                    unimplemented!()
                } else if select_stmt.get_segments()[after_select_clause_idx].is_type("dedent") {
                    unimplemented!()
                } else {
                    unimplemented!()
                }
            }
        }

        if select_targets_info.comment_after_select_idx.is_none() {
            fixes.push(LintFix::replace(
                select_children[select_targets_info.first_new_line_idx.unwrap()].clone(),
                insert_buff,
                None,
            ));
        }

        vec![LintResult::new(
            select_clause.get(0, None).unwrap().clone().into(),
            fixes,
            None,
            None,
            None,
        )]
    }
}

impl Default for RuleLT09 {
    fn default() -> Self {
        Self { wildcard_policy: "single" }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::api::simple::{fix, lint};
    use crate::core::rules::base::{Erased, ErasedRule};
    use crate::rules::layout::LT09::RuleLT09;

    fn rules() -> Vec<ErasedRule> {
        vec![RuleLT09::default().erased()]
    }

    #[test]
    fn test_single_select_target_and_no_newline_between_select_and_select_target() {
        let violations =
            lint("select a from x".into(), "ansi".into(), rules(), None, None).unwrap();
        assert_eq!(violations, []);
    }

    #[test]
    fn test_single_wildcard_select_target_and_no_newline_between_select_and_select_target_2() {
        let violations =
            lint("select * from x".into(), "ansi".into(), rules(), None, None).unwrap();
        assert_eq!(violations, []);
    }

    #[test]
    fn test_single_select_target_and_newline_after_select_target_1() {
        let violations =
            lint("select *\nfrom x".into(), "ansi".into(), rules(), None, None).unwrap();
        assert_eq!(violations, []);
    }

    #[test]
    fn test_single_select_target_and_newline_before_select_target() {
        let fixed = fix(
            "
select
    a
from x"
                .into(),
            rules(),
        );

        assert_eq!(
            fixed,
            "
select a
from x"
        );
    }

    #[test]
    fn test_multiple_select_targets_on_newlines_and_newline_after_select() {
        let pass_str = "
select
    a,
    b,
    c
from x";

        let violations = lint(pass_str.into(), "ansi".into(), rules(), None, None).unwrap();
        assert_eq!(violations, []);
    }
}