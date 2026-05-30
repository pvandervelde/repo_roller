#![no_main]

//! Fuzz target: TOML deserialization of RepositoryNamingRulesConfig
//!
//! Arbitrary bytes fed into the TOML parser for `RepositoryNamingRulesConfig`.
//! The parser must never panic — returning Err is acceptable.

use config_manager::RepositoryNamingRulesConfig;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Ignore non-UTF-8 inputs; TOML is text.
    let Ok(toml_str) = std::str::from_utf8(data) else {
        return;
    };

    // Must not panic.
    let _: Result<RepositoryNamingRulesConfig, _> = toml::from_str(toml_str);
});
