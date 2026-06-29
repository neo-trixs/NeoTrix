mod advanced_guard;
mod screeners;
mod types;

pub use advanced_guard::{
    default_guard, AdvancedPromptGuard, EvasionDetector, EvasionTechnique, JailbreakReport,
    OutlierScorer, SemanticJailbreakDetector,
};
pub use screeners::{ActionScreener, OutputScreener, PromptGuard};
pub use types::{glob_match, ActionRule, RiskLevel};

use std::sync::LazyLock;

pub fn default_output_screener() -> &'static OutputScreener {
    static SCREENER: LazyLock<OutputScreener> = LazyLock::new(OutputScreener::new);
    &SCREENER
}

#[cfg(test)]
mod tests;
