//! Tests for the Handlebars template engine functionality.
//!
//! This module contains comprehensive tests for all Handlebars engine capabilities,
//! including variable substitution, control structures, file path templating,
//! custom helpers, security validation, and error handling.

use crate::handlebars_engine::*;
use serde_json::json;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    // ================================
    // Template Context Tests
    // ================================

    #[test]
    fn test_template_context_new() {
        let variables = json!({
            "repo_name": "test-project",
            "author": {"name": "John Doe", "email": "john@example.com"}
        });

        let context = TemplateContext::new(variables.clone());
        assert_eq!(context.variables, variables);
        assert!(context.config.strict_variables);
        assert_eq!(context.config.max_template_size, 1_048_576);
    }

    #[test]
    fn test_template_context_from_map() {
        let mut vars = HashMap::new();
        vars.insert("repo_name".to_string(), "test-project".to_string());
        vars.insert("author".to_string(), "John Doe".to_string());

        let context = TemplateContext::from_map(vars.clone());

        // Verify conversion to JSON structure
        assert_eq!(
            context.variables["repo_name"].as_str().unwrap(),
            "test-project"
        );
        assert_eq!(context.variables["author"].as_str().unwrap(), "John Doe");
    }

    #[test]
    fn test_template_context_with_config() {
        let variables = json!({"name": "test"});
        let config = TemplateRenderConfig {
            strict_variables: false,
            max_render_time_ms: 5000,
            ..Default::default()
        };

        let context = TemplateContext::with_config(variables.clone(), config.clone());
        assert_eq!(context.variables, variables);
        assert!(!context.config.strict_variables);
        assert_eq!(context.config.max_render_time_ms, 5000);
    }

    // ================================
    // Handlebars Engine Creation Tests
    // ================================

    #[test]
    fn test_handlebars_engine_new() {
        let engine = HandlebarsTemplateEngine::new();
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        assert!(engine.config.strict_variables);
        assert_eq!(engine.config.max_template_size, 1_048_576);
        assert_eq!(engine.config.max_render_time_ms, 30_000);
        assert!(engine.config.enable_caching);
    }

    #[test]
    fn test_handlebars_engine_with_config() {
        let config = TemplateRenderConfig {
            strict_variables: false,
            max_template_size: 512_000,
            max_render_time_ms: 15_000,
            enable_caching: false,
        };

        let engine = HandlebarsTemplateEngine::with_config(config.clone());
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        assert!(!engine.config.strict_variables);
        assert_eq!(engine.config.max_template_size, 512_000);
        assert_eq!(engine.config.max_render_time_ms, 15_000);
        assert!(!engine.config.enable_caching);
    }

    #[test]
    fn test_register_custom_helpers() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        let result = engine.register_custom_helpers();
        assert!(result.is_ok());
    }

    // ================================
    // Basic Variable Substitution Tests
    // ================================

    #[test]
    fn test_render_template_simple_variables() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({
            "repo_name": "my-awesome-project",
            "author": "John Doe"
        }));

        let template = "# {{repo_name}}\n\nCreated by {{author}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered, "# my-awesome-project\n\nCreated by John Doe");
    }

    #[test]
    fn test_render_template_nested_objects() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({
            "author": {
                "name": "John Doe",
                "email": "john@example.com",
                "github": "johndoe"
            },
            "project": {
                "name": "awesome-lib",
                "version": "1.0.0"
            }
        }));

        let template = "Author: {{author.name}} ({{author.email}})\nProject: {{project.name}} v{{project.version}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(
            rendered,
            "Author: John Doe (john@example.com)\nProject: awesome-lib v1.0.0"
        );
    }

    #[test]
    fn test_render_template_array_access() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({
            "features": ["logging", "testing", "documentation"],
            "authors": [
                {"name": "Alice", "role": "Lead"},
                {"name": "Bob", "role": "Developer"}
            ]
        }));

        let template = "First feature: {{features.[0]}}\nLead author: {{authors.[0].name}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered, "First feature: logging\nLead author: Alice");
    }

    // ================================
    // Control Structure Tests
    // ================================

    #[test]
    fn test_render_template_if_conditional() {
        let engine = HandlebarsTemplateEngine::new().unwrap();

        // Test with truthy condition
        let context_true = TemplateContext::new(json!({
            "has_tests": true,
            "repo_name": "test-project"
        }));

        let template = "# {{repo_name}}\n{{#if has_tests}}✅ Tests included{{/if}}";
        let result = engine.render_template(template, &context_true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "# test-project\n✅ Tests included");

        // Test with falsy condition
        let context_false = TemplateContext::new(json!({
            "has_tests": false,
            "repo_name": "test-project"
        }));

        let result = engine.render_template(template, &context_false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "# test-project\n");
    }

    #[test]
    fn test_render_template_unless_conditional() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({
            "is_private": false,
            "repo_name": "public-project"
        }));

        let template = "# {{repo_name}}\n{{#unless is_private}}🌐 Public repository{{/unless}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "# public-project\n🌐 Public repository");
    }

    #[test]
    fn test_render_template_each_loop() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({
            "features": ["logging", "testing", "docs"],
            "contributors": [
                {"name": "Alice", "commits": 150},
                {"name": "Bob", "commits": 89}
            ]
        }));

        let template = "Features:\n{{#each features}}{{@index}}. {{this}}\n{{/each}}Contributors:\n{{#each contributors}}- {{name}} ({{commits}} commits)\n{{/each}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        let expected = "Features:\n0. logging\n1. testing\n2. docs\nContributors:\n- Alice (150 commits)\n- Bob (89 commits)\n";
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_template_with_context() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({
            "project": {
                "name": "awesome-lib",
                "description": "An awesome library",
                "maintainer": {
                    "name": "John Doe",
                    "email": "john@example.com"
                }
            }
        }));

        let template = "{{#with project}}# {{name}}\n{{description}}\n\nMaintainer: {{maintainer.name}}{{/with}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "# awesome-lib\nAn awesome library\n\nMaintainer: John Doe"
        );
    }

    // ================================
    // Custom Helper Tests
    // ================================

    #[test]
    fn test_snake_case_helper() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();

        let context = TemplateContext::new(json!({
            "project_name": "My Awesome Project"
        }));

        let template = "{{snake_case project_name}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my_awesome_project");
    }

    #[test]
    fn test_kebab_case_helper() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();

        let context = TemplateContext::new(json!({
            "project_name": "My Awesome Project"
        }));

        let template = "{{kebab_case project_name}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my-awesome-project");
    }

    #[test]
    fn test_case_helpers_with_different_inputs() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();

        let context = TemplateContext::new(json!({
            "text1": "hello world",
            "text2": "HELLO_WORLD",
            "text3": "hello-world",
            "text4": "HelloWorld"
        }));

        let template = "{{upper_case text1}} {{lower_case text2}} {{capitalize text3}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HELLO WORLD hello_world Hello-world");
    }

    #[test]
    fn test_default_helper() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();

        let context = TemplateContext::new(json!({
            "author": "John Doe"
            // missing "license" variable
        }));

        let template = "Author: {{author}}\nLicense: {{default license \"MIT\"}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Author: John Doe\nLicense: MIT");
    }

    #[test]
    fn test_timestamp_helper() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();

        let context = TemplateContext::new(json!({}));

        let template = "Generated at: {{timestamp}}";
        let result = engine.render_template(template, &context);

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.starts_with("Generated at: "));
        assert!(rendered.len() > 20); // Should contain a timestamp
    }

    // ================================
    // File Path Templating Tests
    // ================================

    #[test]
    fn test_template_file_path_simple() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({
            "repo_name": "my-project",
            "file_type": "md"
        }));

        let result = engine.template_file_path("{{repo_name}}.{{file_type}}", &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my-project.md");
    }

    #[test]
    fn test_template_file_path_with_directories() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({
            "language": "rust",
            "project_name": "awesome_lib",
            "environment": "production"
        }));

        let result = engine.template_file_path(
            "{{language}}/{{environment}}/{{project_name}}.toml",
            &context,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "rust/production/awesome_lib.toml");
    }

    #[test]
    fn test_template_file_path_with_helpers() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();

        let context = TemplateContext::new(json!({
            "project_name": "My Awesome Project"
        }));

        let result = engine.template_file_path("{{snake_case project_name}}/README.md", &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my_awesome_project/README.md");
    }

    // ================================
    // Security Validation Tests
    // ================================

    #[test]
    fn test_validate_file_path_valid_paths() {
        let engine = HandlebarsTemplateEngine::new().unwrap();

        // Valid relative paths
        assert!(engine.validate_file_path("README.md").is_ok());
        assert!(engine.validate_file_path("src/main.rs").is_ok());
        assert!(engine.validate_file_path("docs/api/index.html").is_ok());
        assert!(engine.validate_file_path("config/app.yaml").is_ok());
    }

    #[test]
    fn test_validate_file_path_directory_traversal() {
        let engine = HandlebarsTemplateEngine::new().unwrap();

        // Directory traversal attempts should fail
        let result = engine.validate_file_path("../../../etc/passwd");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HandlebarsError::InvalidPath { .. }
        ));

        let result = engine.validate_file_path("../config.txt");
        assert!(result.is_err());

        let result = engine.validate_file_path("subdir/../../../etc/hosts");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_path_absolute_paths() {
        let engine = HandlebarsTemplateEngine::new().unwrap();

        // Absolute paths should fail
        let result = engine.validate_file_path("/etc/passwd");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HandlebarsError::InvalidPath { .. }
        ));

        let result = engine.validate_file_path("/home/user/file.txt");
        assert!(result.is_err());

        // Windows absolute paths
        let result = engine.validate_file_path("C:\\Windows\\System32\\config");
        assert!(result.is_err());

        let result = engine.validate_file_path("\\\\server\\share\\file");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_path_invalid_characters() {
        let engine = HandlebarsTemplateEngine::new().unwrap();

        // Platform-specific invalid characters
        if cfg!(windows) {
            assert!(engine.validate_file_path("file<name>.txt").is_err());
            assert!(engine.validate_file_path("file>name.txt").is_err());
            assert!(engine.validate_file_path("file|name.txt").is_err());
            assert!(engine.validate_file_path("file?name.txt").is_err());
            assert!(engine.validate_file_path("file*name.txt").is_err());
        }

        // Null bytes should always be invalid
        assert!(engine.validate_file_path("file\0name.txt").is_err());
    }

    #[test]
    fn test_validate_file_path_empty_or_special() {
        let engine = HandlebarsTemplateEngine::new().unwrap();

        // Empty path should fail
        assert!(engine.validate_file_path("").is_err());

        // Special directory names should fail
        assert!(engine.validate_file_path(".").is_err());
        assert!(engine.validate_file_path("..").is_err());
    }

    // ================================
    // Error Handling Tests
    // ================================

    #[test]
    fn test_render_template_invalid_syntax() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({"name": "test"}));

        // Invalid template syntax
        let result = engine.render_template("{{#if unclosed", &context);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HandlebarsError::CompilationError { .. }
        ));

        let result = engine.render_template("{{invalid.}}", &context);
        assert!(result.is_err());

        let result = engine.render_template("{{#unknown_helper}}{{/unknown_helper}}", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_template_missing_variables_strict() {
        let engine = HandlebarsTemplateEngine::new().unwrap();
        let context = TemplateContext::new(json!({"name": "test"}));

        // Missing variable in strict mode should fail
        let result = engine.render_template("Hello {{name}} and {{missing_var}}!", &context);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HandlebarsError::VariableValidation { .. }
        ));
    }

    #[test]
    fn test_render_template_missing_variables_non_strict() {
        let config = TemplateRenderConfig {
            strict_variables: false,
            ..Default::default()
        };
        let engine = HandlebarsTemplateEngine::with_config(config).unwrap();
        let context = TemplateContext::new(json!({"name": "test"}));

        // Missing variable in non-strict mode should render as empty
        let result = engine.render_template("Hello {{name}} and {{missing_var}}!", &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello test and !");
    }

    #[test]
    fn test_render_template_large_template() {
        let config = TemplateRenderConfig {
            max_template_size: 100, // Very small limit
            ..Default::default()
        };
        let engine = HandlebarsTemplateEngine::with_config(config).unwrap();
        let context = TemplateContext::new(json!({}));

        // Template larger than limit should fail
        let large_template = "a".repeat(200);
        let result = engine.render_template(&large_template, &context);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HandlebarsError::ResourceLimit { .. }
        ));
    }

    // ================================
    // Integration Tests
    // ================================

    #[test]
    fn test_complex_template_with_all_features() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();

        let context = TemplateContext::new(json!({
            "project": {
                "name": "Awesome Library",
                "version": "1.0.0",
                "description": "A really awesome library"
            },
            "author": {
                "name": "John Doe",
                "email": "john@example.com"
            },
            "features": ["logging", "testing", "documentation"],
            "is_public": true,
            "license": "MIT"
        }));

        let template = r#"# {{snake_case project.name}}

{{project.description}}

## Information

- **Version**: {{project.version}}
- **Author**: {{author.name}} ({{author.email}})
- **License**: {{default license "Apache-2.0"}}

{{#if is_public}}
## Public Repository

This is a public repository.
{{/if}}

## Features

{{#each features}}
{{@index}}. {{upper_case this}}
{{/each}}

## Installation

```bash
cargo add {{kebab_case project.name}}
```
"#;

        let result = engine.render_template(template, &context);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("# awesome_library"));
        assert!(rendered.contains("**Version**: 1.0.0"));
        assert!(rendered.contains("**Author**: John Doe"));
        assert!(rendered.contains("**License**: MIT"));
        assert!(rendered.contains("This is a public repository."));
        assert!(rendered.contains("0. LOGGING"));
        assert!(rendered.contains("1. TESTING"));
        assert!(rendered.contains("2. DOCUMENTATION"));
        assert!(rendered.contains("cargo add awesome-library"));
    }

    #[test]
    fn test_file_path_templating_integration() {
        let mut engine = HandlebarsTemplateEngine::new().unwrap();
        engine.register_custom_helpers().unwrap();

        let context = TemplateContext::new(json!({
            "project_name": "My Awesome Project",
            "language": "rust",
            "environment": "production"
        }));

        // Test various file path templates
        let templates_and_expected = vec![
            (
                "{{snake_case project_name}}/Cargo.toml",
                "my_awesome_project/Cargo.toml",
            ),
            (
                "src/{{kebab_case project_name}}.rs",
                "src/my-awesome-project.rs",
            ),
            (
                "{{language}}/{{environment}}/config.yaml",
                "rust/production/config.yaml",
            ),
            ("README.md", "README.md"), // No templating
        ];

        for (template, expected) in templates_and_expected {
            let result = engine.template_file_path(template, &context);
            assert!(result.is_ok(), "Failed to process template: {}", template);
            assert_eq!(result.unwrap(), expected);
        }
    }
}
