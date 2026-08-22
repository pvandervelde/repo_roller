//! Property-based tests for `tracing_otel::resolve_otel_settings`.

use super::*;
use proptest::prelude::*;

proptest! {
    /// Never panics for any combination of arbitrary raw env-var values.
    #[test]
    fn prop_resolve_otel_settings_never_panics(
        enabled in ".{0,100}",
        endpoint in ".{0,100}",
        service_name in ".{0,100}",
    ) {
        let _ = resolve_otel_settings(Some(&enabled), Some(&endpoint), Some(&service_name));
    }

    /// Only the case-insensitive literals "true" and "1" ever enable export;
    /// any other arbitrary string leaves it disabled (fail-safe-off).
    #[test]
    fn prop_resolve_otel_settings_only_true_or_one_enable_export(
        raw in "[a-zA-Z0-9_-]{0,20}"
    ) {
        let settings = resolve_otel_settings(Some(&raw), None, None);
        let should_be_enabled = raw.eq_ignore_ascii_case("true") || raw == "1";
        prop_assert_eq!(settings.enabled, should_be_enabled);
    }

    /// A non-empty endpoint override is always used verbatim.
    #[test]
    fn prop_resolve_otel_settings_nonempty_endpoint_is_used_verbatim(
        endpoint in "[a-zA-Z0-9:/.-]{1,50}"
    ) {
        let settings = resolve_otel_settings(None, Some(&endpoint), None);
        prop_assert_eq!(settings.otlp_endpoint, endpoint);
    }

    /// A non-empty service name override is always used verbatim.
    #[test]
    fn prop_resolve_otel_settings_nonempty_service_name_is_used_verbatim(
        service_name in "[a-zA-Z0-9_-]{1,50}"
    ) {
        let settings = resolve_otel_settings(None, None, Some(&service_name));
        prop_assert_eq!(settings.service_name, service_name);
    }
}
