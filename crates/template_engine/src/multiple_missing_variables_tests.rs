//! Tests for multiple missing variables error reporting.

use crate::*;
use serde_json::json;

#[test]
fn test_extract_variables_from_template() {
    let engine = HandlebarsTemplateEngine::new().expect("Failed to create engine");

    let template = r#"
# {{project_name}}

Created by {{author_name}} ({{author_email}})
License: {{license}}
Version: {{version}}
"#;

    let mut variables = engine.extract_variables(template);
    variables.sort();

    assert_eq!(
        variables,
        vec![
            "author_email",
            "author_name",
            "license",
            "project_name",
            "version"
        ]
    );
}

#[test]
fn test_extract_variables_ignores_helpers() {
    let engine = HandlebarsTemplateEngine::new().expect("Failed to create engine");

    let template = r#"
{{#if debug_mode}}
Debug is enabled
{{/if}}

{{#each items}}
- {{this}}
{{/each}}

Project: {{project_name}}
"#;

    let mut variables = engine.extract_variables(template);
    variables.sort();

    // The regex captures debug_mode and items because they follow {{#if and {{#each
    // But it should NOT capture "if", "each", or "this" as variables
    // This is correct behavior - debug_mode and items ARE variables used in helpers
    assert_eq!(variables, vec!["debug_mode", "items", "project_name"]);

    // Verify that Handlebars keywords are not in the list
    assert!(!variables.contains(&"if".to_string()));
    assert!(!variables.contains(&"each".to_string()));
    assert!(!variables.contains(&"this".to_string()));
}

#[test]
fn test_multiple_missing_variables_error() {
    let engine = HandlebarsTemplateEngine::new().expect("Failed to create engine");

    let template = r#"
# {{project_name}}

Author: {{author_name}} <{{author_email}}>
License: {{license}}
Version: {{version}}
Description: {{description}}
"#;

    // Only provide one variable out of six required
    let context = TemplateContext::new(json!({
        "project_name": "test-project"
    }));

    let result = engine.render_template(template, &context);

    assert!(result.is_err());

    match result.unwrap_err() {
        HandlebarsError::MissingVariables { missing_variables } => {
            // Should list ALL missing variables, not just the first one
            assert_eq!(missing_variables.len(), 5);
            assert!(missing_variables.contains(&"author_name".to_string()));
            assert!(missing_variables.contains(&"author_email".to_string()));
            assert!(missing_variables.contains(&"license".to_string()));
            assert!(missing_variables.contains(&"version".to_string()));
            assert!(missing_variables.contains(&"description".to_string()));
        }
        other_error => panic!("Expected MissingVariables error, got: {:?}", other_error),
    }
}

#[test]
fn test_process_template_with_multiple_missing_variables() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    // Template files with multiple variable references
    let files = vec![
        (
            "README.md".to_string(),
            b"# {{project_name}}\nBy {{author_name}}".to_vec(),
        ),
        (
            "Cargo.toml".to_string(),
            b"[package]\nname = \"{{project_name}}\"\nauthors = [\"{{author_name}} <{{author_email}}>\"]\nlicense = \"{{license}}\"".to_vec(),
        ),
        (
            "config.yml".to_string(),
            b"version: {{version}}\nenvironment: {{environment}}".to_vec(),
        ),
    ];

    // Only provide one variable
    let request = TemplateProcessingRequest {
        variables: {
            let mut vars = std::collections::HashMap::new();
            vars.insert("project_name".to_string(), "test".to_string());
            vars
        },
        built_in_variables: std::collections::HashMap::new(),
        variable_configs: std::collections::HashMap::new(),
        templating_config: None,
    };

    let result = processor.process_template(&files, &request, std::path::Path::new("./output"));

    assert!(result.is_err());

    match result.unwrap_err() {
        Error::MissingVariables { variables, message } => {
            // Should list ALL missing variables from ALL files
            assert!(variables.len() >= 5); // at least: author_name, author_email, license, version, environment
            assert!(message.contains("author_name"));
            assert!(message.contains("author_email"));
            assert!(message.contains("license"));
            assert!(message.contains("version"));
            assert!(message.contains("environment"));

            // Check that the error message is helpful
            assert!(message.contains("variable(s) that were not provided"));
        }
        other_error => panic!("Expected MissingVariables error, got: {:?}", other_error),
    }
}

#[test]
fn test_no_error_when_all_variables_provided() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let files = vec![(
        "README.md".to_string(),
        b"# {{project_name}}\nBy {{author_name}}".to_vec(),
    )];

    let request = TemplateProcessingRequest {
        variables: {
            let mut vars = std::collections::HashMap::new();
            vars.insert("project_name".to_string(), "test".to_string());
            vars.insert("author_name".to_string(), "Alice".to_string());
            vars
        },
        built_in_variables: std::collections::HashMap::new(),
        variable_configs: std::collections::HashMap::new(),
        templating_config: None,
    };

    let result = processor.process_template(&files, &request, std::path::Path::new("./output"));

    assert!(result.is_ok());
    let processed = result.unwrap();
    assert_eq!(processed.files.len(), 1);

    let content = String::from_utf8_lossy(&processed.files[0].1);
    assert_eq!(content, "# test\nBy Alice");
}
