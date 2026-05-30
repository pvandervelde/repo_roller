#![no_main]

//! Fuzz target: RepositoryNamingValidator::validate
//!
//! Feed arbitrary UTF-8 names and a fixed rule set into the naming validator.
//! The validator must never panic — returning Err is acceptable, panicking is not.

use config_manager::RepositoryNamingRulesConfig;
use libfuzzer_sys::fuzz_target;
use repo_roller_core::RepositoryNamingValidator;

fuzz_target!(|data: &[u8]| {
    // Ignore non-UTF-8 inputs — the validator operates on &str.
    let Ok(name) = std::str::from_utf8(data) else {
        return;
    };

    let validator = RepositoryNamingValidator::new();

    // Rule set 1: length constraints
    let length_rules = [RepositoryNamingRulesConfig {
        min_length: Some(3),
        max_length: Some(64),
        ..Default::default()
    }];
    let _ = validator.validate(name, &length_rules);

    // Rule set 2: prefix + suffix
    let affix_rules = [RepositoryNamingRulesConfig {
        required_prefix: Some("svc-".to_string()),
        required_suffix: Some("-api".to_string()),
        ..Default::default()
    }];
    let _ = validator.validate(name, &affix_rules);

    // Rule set 3: reserved words
    let reserved_rules = [RepositoryNamingRulesConfig {
        reserved_words: vec!["test".to_string(), "temp".to_string(), "debug".to_string()],
        ..Default::default()
    }];
    let _ = validator.validate(name, &reserved_rules);

    // Rule set 4: combined — stress-test the full validation path
    let combined_rules = [RepositoryNamingRulesConfig {
        min_length: Some(5),
        max_length: Some(50),
        required_prefix: Some("svc-".to_string()),
        reserved_words: vec!["test".to_string()],
        ..Default::default()
    }];
    let _ = validator.validate(name, &combined_rules);

    // Rule set 5: empty rules — must always pass
    let _ = validator.validate(name, &[]);
});
