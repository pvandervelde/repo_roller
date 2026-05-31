// Property-based tests for HandlebarsTemplateEngine custom helpers.
//
// These verify idempotency, no-panic, and output-invariant properties across
// arbitrary string inputs — not just the hand-picked examples in the unit tests.

use super::HandlebarsTemplateEngine;
use crate::TemplateContext;
use proptest::prelude::*;
use serde_json::json;

proptest! {
    /// `upper_case` followed by `upper_case` must produce the same result as
    /// a single `upper_case` (idempotency).
    #[test]
    fn prop_upper_case_is_idempotent(s in "\\PC*") {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();
        let ctx = TemplateContext::new(json!({ "v": s }));

        let first = engine.render_template("{{upper_case v}}", &ctx);
        // Skip inputs the engine cannot process rather than failing the test —
        // proptest will warn (too many discards) if the engine rejects most inputs.
        prop_assume!(first.is_ok());
        let first = first.unwrap();
        let ctx2 = TemplateContext::new(json!({ "v": first.clone() }));
        let second = engine.render_template("{{upper_case v}}", &ctx2);
        prop_assume!(second.is_ok());
        let second = second.unwrap();
        prop_assert_eq!(
            first.clone(), second.clone(),
            "upper_case must be idempotent; got '{}' then '{}'", first, second
        );
    }

    /// `lower_case` followed by `lower_case` must be idempotent.
    #[test]
    fn prop_lower_case_is_idempotent(s in "\\PC*") {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();
        let ctx = TemplateContext::new(json!({ "v": s }));

        let first = engine.render_template("{{lower_case v}}", &ctx);
        // Skip inputs the engine cannot process; proptest warns if too many are discarded.
        prop_assume!(first.is_ok());
        let first = first.unwrap();
        let ctx2 = TemplateContext::new(json!({ "v": first.clone() }));
        let second = engine.render_template("{{lower_case v}}", &ctx2);
        prop_assume!(second.is_ok());
        let second = second.unwrap();
        prop_assert_eq!(
            first.clone(), second.clone(),
            "lower_case must be idempotent; got '{}' then '{}'", first, second
        );
    }

    /// `lower_case(upper_case(s))` == `lower_case(s)`.
    ///
    /// Converting to upper then back to lower must yield the same result as
    /// converting directly to lower (for ASCII-only strings).
    #[test]
    fn prop_lower_of_upper_equals_lower(s in "[a-zA-Z0-9 _-]{0,64}") {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();
        let ctx_s = TemplateContext::new(json!({ "v": s.clone() }));

        let lower_s = engine.render_template("{{lower_case v}}", &ctx_s);
        let lower_of_upper = engine.render_template("{{upper_case v}}", &ctx_s)
            .and_then(|u| {
                let ctx_u = TemplateContext::new(json!({ "v": u }));
                engine.render_template("{{lower_case v}}", &ctx_u)
            });

        // Skip inputs where either render fails; proptest warns if too many are discarded.
        prop_assume!(lower_s.is_ok());
        prop_assume!(lower_of_upper.is_ok());
        let a = lower_s.unwrap();
        let b = lower_of_upper.unwrap();
        prop_assert_eq!(
            a.clone(), b.clone(),
            "lower(upper(s)) must equal lower(s); s='{}' lower='{}' lower_of_upper='{}'",
            s, a, b
        );
    }

    /// `snake_case` must never produce a result containing a space.
    #[test]
    fn prop_snake_case_output_contains_no_spaces(s in "[a-zA-Z0-9 _-]{0,64}") {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();
        let ctx = TemplateContext::new(json!({ "v": s.clone() }));
        if let Ok(output) = engine.render_template("{{snake_case v}}", &ctx) {
            prop_assert!(
                !output.contains(' '),
                "snake_case output must contain no spaces; input='{}' output='{}'", s, output
            );
        }
    }

    /// `kebab_case` must never produce a result containing spaces or underscores.
    #[test]
    fn prop_kebab_case_output_contains_no_spaces_or_underscores(
        s in "[a-zA-Z0-9 _]{0,64}",
    ) {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();
        let ctx = TemplateContext::new(json!({ "v": s.clone() }));
        if let Ok(output) = engine.render_template("{{kebab_case v}}", &ctx) {
            prop_assert!(
                !output.contains(' '),
                "kebab_case output must not contain spaces; input='{}' output='{}'", s, output
            );
            prop_assert!(
                !output.contains('_'),
                "kebab_case output must not contain underscores; input='{}' output='{}'", s, output
            );
        }
    }

    /// Any template helper must not panic (return an error rather than unwind)
    /// for arbitrary printable input.
    #[test]
    fn prop_helpers_never_panic_on_arbitrary_input(s in "\\PC{0,256}") {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();
        let ctx = TemplateContext::new(json!({ "v": s }));
        // All these calls must either succeed or return Err — never panic.
        let _ = engine.render_template("{{upper_case v}}", &ctx);
        let _ = engine.render_template("{{lower_case v}}", &ctx);
        let _ = engine.render_template("{{snake_case v}}", &ctx);
        let _ = engine.render_template("{{kebab_case v}}", &ctx);
        let _ = engine.render_template("{{capitalize v}}", &ctx);
    }
}
