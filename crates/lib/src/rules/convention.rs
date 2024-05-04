use crate::core::rules::base::ErasedRule;

pub mod CV02;
pub mod CV04;
pub mod CV05;

pub fn rules() -> Vec<ErasedRule> {
    use crate::core::rules::base::Erased as _;

    vec![CV02::RuleCv02::default().erased()]
}
