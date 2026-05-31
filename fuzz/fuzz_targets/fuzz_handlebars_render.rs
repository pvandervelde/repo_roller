#![no_main]

//! Fuzz target: HandlebarsTemplateEngine::render_template
//!
//! Feed arbitrary template strings and a fixed context into the Handlebars
//! engine.  The engine must never panic — returning Err is acceptable.

use libfuzzer_sys::fuzz_target;
use serde_json::json;
use template_engine::{HandlebarsTemplateEngine, TemplateContext};

fuzz_target!(|data: &[u8]| {
    // Ignore non-UTF-8 inputs; templates are UTF-8 strings.
    let Ok(template_str) = std::str::from_utf8(data) else {
        return;
    };

    let Ok(engine) = HandlebarsTemplateEngine::new() else {
        return;
    };

    // Fixed context with a variety of value types to exercise helper paths.
    let ctx = TemplateContext::new(json!({
        "name": "my-service",
        "version": "1.0.0",
        "author": "Test Author",
        "license": "Apache-2.0",
        "empty": "",
        "number": 42,
        "flag": true,
        "null_val": null
    }));

    // Must not panic. Returning Err is acceptable.
    let _ = engine.render_template(template_str, &ctx);
});
