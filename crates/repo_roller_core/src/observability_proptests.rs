//! Property-based tests for `observability::business_context_attributes`.

use super::*;
use proptest::prelude::*;

proptest! {
    /// Never panics for any combination of arbitrary organization, optional
    /// template, and requestor strings.
    #[test]
    fn prop_business_context_attributes_never_panics(
        organization in ".{0,200}",
        has_template in any::<bool>(),
        template in ".{0,200}",
        requestor in ".{0,200}",
    ) {
        let template_opt = if has_template { Some(template.as_str()) } else { None };
        let _ = business_context_attributes(&organization, template_opt, &requestor);
    }

    /// Every attribute value produced is bounded by
    /// `github_client::span_attributes::MAX_SPAN_ATTRIBUTE_LEN` characters,
    /// regardless of how long the raw input was.
    #[test]
    fn prop_business_context_attributes_values_are_bounded(
        organization in ".{0,2000}",
        requestor in ".{0,2000}",
    ) {
        let attrs = business_context_attributes(&organization, None, &requestor);
        for (_, value) in &attrs {
            prop_assert!(
                value.chars().count()
                    <= github_client::span_attributes::MAX_SPAN_ATTRIBUTE_LEN
            );
        }
    }

    /// `template` key presence exactly tracks whether `template` was `Some`.
    #[test]
    fn prop_business_context_attributes_template_key_presence_tracks_option(
        organization in "[a-z-]{1,30}",
        has_template in any::<bool>(),
        template in "[a-z-]{1,30}",
        requestor in "[a-z-]{1,30}",
    ) {
        let template_opt = if has_template { Some(template.as_str()) } else { None };
        let attrs = business_context_attributes(&organization, template_opt, &requestor);
        let has_template_key = attrs.iter().any(|(k, _)| *k == ATTR_TEMPLATE);
        prop_assert_eq!(has_template_key, has_template);
    }
}
