#![no_main]

//! Fuzz target: TOML deserialization of LabelConfig
//!
//! Arbitrary bytes fed into the TOML parser for `HashMap<String, LabelConfig>`.
//! The parser must never panic — returning Err is acceptable.

use config_manager::LabelConfig;
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;

fuzz_target!(|data: &[u8]| {
    // Ignore non-UTF-8 inputs; TOML is text.
    let Ok(toml_str) = std::str::from_utf8(data) else {
        return;
    };

    // Must not panic.
    let _: Result<HashMap<String, LabelConfig>, _> = toml::from_str(toml_str);
});
