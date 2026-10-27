//! Fork/mirror repository-creation capability toggles.
//!
//! Controls whether GitHub fork creation and the two mirror creation modes
//! (source+history, source-only) are available, independently of each other,
//! with the standard four-level override hierarchy
//! (Global → RepositoryType → Team → Template).
//!
//! See `docs/spec/interfaces/fork-mirror-operations.md` §2.

use crate::OverridableValue;
use serde::{Deserialize, Serialize};

/// Fork/mirror capability toggles with override controls.
///
/// Each field is independently toggleable. When a field is `None` at every
/// level of the configuration hierarchy, the corresponding capability is
/// **disabled** (fail-closed default) — see
/// `docs/spec/interfaces/fork-mirror-operations.md` §2 for the resolution
/// rule used by `repo_roller_core`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CreationCapabilitiesSettings {
    /// Enable GitHub fork creation.
    pub fork_enabled: Option<OverridableValue<bool>>,

    /// Enable source+history mirror creation.
    pub source_history_mirror_enabled: Option<OverridableValue<bool>>,

    /// Enable source-only mirror creation.
    pub source_only_mirror_enabled: Option<OverridableValue<bool>>,
}

#[cfg(test)]
#[path = "creation_capabilities_tests.rs"]
mod tests;
