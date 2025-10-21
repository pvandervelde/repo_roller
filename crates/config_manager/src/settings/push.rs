//! Push restriction settings.

use crate::OverridableValue;
use serde::{Deserialize, Serialize};

/// Push restriction settings with override controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PushSettings {
    /// Maximum number of branches that can be pushed at once
    pub max_branches_per_push: Option<OverridableValue<i32>>,

    /// Maximum number of tags that can be pushed at once
    pub max_tags_per_push: Option<OverridableValue<i32>>,
}

#[cfg(test)]
#[path = "push_tests.rs"]
mod tests;
