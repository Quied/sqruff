use crate::core::rules::base::ErasedRule;

mod AM01;
mod AM02;

pub fn rules() -> Vec<ErasedRule> {
    use crate::core::rules::base::Erased as _;

    vec![AM01::RuleAM01.erased(), AM02::RuleAM02::default().erased()]
}
