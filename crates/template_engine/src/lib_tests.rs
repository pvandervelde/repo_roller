use super::*;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn test_built_in_variables_generation() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let params = BuiltInVariablesParams {
        repo_name: "test-repo",
        org_name: "test-org",
        template_name: "rust-library",
        template_repo: "templates/rust-library",
        user_login: "testuser",
        user_name: "Test User",
        default_branch: "main",
    };

    let variables = processor.generate_built_in_variables(&params);

    assert_eq!(variables.get("repo_name"), Some(&"test-repo".to_string()));
    assert_eq!(variables.get("org_name"), Some(&"test-org".to_string()));
    assert_eq!(
        variables.get("template_name"),
        Some(&"rust-library".to_string())
    );
    assert_eq!(
        variables.get("template_repo"),
        Some(&"templates/rust-library".to_string())
    );
    assert_eq!(variables.get("user_login"), Some(&"testuser".to_string()));
    assert_eq!(variables.get("user_name"), Some(&"Test User".to_string()));
    assert_eq!(variables.get("default_branch"), Some(&"main".to_string()));

    // Check that timestamp variables are present
    assert!(variables.contains_key("timestamp"));
    assert!(variables.contains_key("timestamp_unix"));
}

#[test]
fn test_is_text_file() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    // Text content
    assert!(processor.is_text_file(b"Hello, world!"));
    assert!(processor.is_text_file(b"# README\n\nThis is a test."));
    assert!(processor.is_text_file(b"fn main() { println!(\"Hello\"); }"));

    // Binary content (contains null bytes)
    assert!(!processor.is_text_file(b"Hello\0world"));
    assert!(!processor.is_text_file(b"\x89PNG\r\n\x1a\n")); // PNG header

    // Invalid UTF-8
    assert!(!processor.is_text_file(&[0xFF, 0xFE, 0xFD]));
}

#[test]
fn test_process_template_basic() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let files = vec![
        (
            "README.md".to_string(),
            b"# {{project_name}}\n\nBy {{author}}".to_vec(),
        ),
        (
            "src/main.rs".to_string(),
            b"// {{project_name}} by {{author}}\nfn main() {}".to_vec(),
        ),
    ];

    let mut variables = HashMap::new();
    variables.insert("project_name".to_string(), "Test Project".to_string());
    variables.insert("author".to_string(), "John Doe".to_string());

    let request = TemplateProcessingRequest {
        variables,
        built_in_variables: HashMap::new(),
        variable_configs: HashMap::new(),
        templating_config: None,
    };

    let result = processor
        .process_template(&files, &request, Path::new("."))
        .unwrap();

    assert_eq!(result.files.len(), 2);

    let readme_content = String::from_utf8(result.files[0].1.clone()).unwrap();
    assert_eq!(readme_content, "# Test Project\n\nBy John Doe");

    let main_content = String::from_utf8(result.files[1].1.clone()).unwrap();
    assert_eq!(main_content, "// Test Project by John Doe\nfn main() {}");
}

#[test]
fn test_process_template_with_filtering() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let files = vec![
        ("README.md".to_string(), b"# {{project_name}}".to_vec()),
        ("src/main.rs".to_string(), b"fn main() {}".to_vec()),
        ("target/debug/app".to_string(), b"binary content".to_vec()),
        (
            "tests/test.rs".to_string(),
            b"#[test] fn test() {}".to_vec(),
        ),
    ];

    let mut variables = HashMap::new();
    variables.insert("project_name".to_string(), "Test Project".to_string());

    let templating_config = TemplatingConfig {
        include_patterns: vec!["**/*.md".to_string(), "**/*.rs".to_string()],
        exclude_patterns: vec!["target/**".to_string()],
    };

    let request = TemplateProcessingRequest {
        variables,
        built_in_variables: HashMap::new(),
        variable_configs: HashMap::new(),
        templating_config: Some(templating_config),
    };

    let result = processor
        .process_template(&files, &request, Path::new("."))
        .unwrap();

    // Should include README.md, src/main.rs, tests/test.rs but exclude target/debug/app
    assert_eq!(result.files.len(), 3);

    let file_paths: Vec<&String> = result.files.iter().map(|(path, _)| path).collect();
    assert!(file_paths.contains(&&"README.md".to_string()));
    assert!(file_paths.contains(&&"src/main.rs".to_string()));
    assert!(file_paths.contains(&&"tests/test.rs".to_string()));
    assert!(!file_paths.contains(&&"target/debug/app".to_string()));
}

#[test]
fn test_process_template_removes_template_suffix() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let files = vec![
        (
            "README.md.template".to_string(),
            b"# {{project_name}}".to_vec(),
        ),
        (
            "Cargo.toml.template".to_string(),
            b"[package]\nname = \"{{project_name}}\"".to_vec(),
        ),
        ("src/main.rs".to_string(), b"fn main() {}".to_vec()),
    ];

    let mut variables = HashMap::new();
    variables.insert("project_name".to_string(), "test-app".to_string());

    let request = TemplateProcessingRequest {
        variables,
        built_in_variables: HashMap::new(),
        variable_configs: HashMap::new(),
        templating_config: None,
    };

    let result = processor
        .process_template(&files, &request, Path::new("."))
        .unwrap();

    assert_eq!(result.files.len(), 3);

    let file_paths: Vec<&String> = result.files.iter().map(|(path, _)| path).collect();
    assert!(file_paths.contains(&&"README.md".to_string()));
    assert!(file_paths.contains(&&"Cargo.toml".to_string()));
    assert!(file_paths.contains(&&"src/main.rs".to_string()));
}

#[test]
fn test_simple_glob_match() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    // Test exact matches
    assert!(processor.simple_glob_match("README.md", "README.md"));
    assert!(!processor.simple_glob_match("README.md", "readme.md"));

    // Test ** patterns
    assert!(processor.simple_glob_match("**", "any/path/file.txt"));
    assert!(processor.simple_glob_match("**/*", "any/path/file.txt"));

    // Test recursive patterns
    assert!(processor.simple_glob_match("src/**", "src/main.rs"));
    assert!(processor.simple_glob_match("src/**", "src/lib/mod.rs"));
    assert!(processor.simple_glob_match("**/*.rs", "src/main.rs"));
    assert!(processor.simple_glob_match("**/*.rs", "tests/integration.rs"));
    assert!(processor.simple_glob_match("**/*.rs", "main.rs"));

    // Test single wildcard
    assert!(processor.simple_glob_match("*.rs", "main.rs"));
    assert!(processor.simple_glob_match("test_*", "test_example"));
    assert!(!processor.simple_glob_match("*.rs", "src/main.rs"));
}

#[test]
fn test_validate_variables_required() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let mut variable_configs = HashMap::new();
    variable_configs.insert(
        "required_var".to_string(),
        VariableConfig {
            description: "A required variable".to_string(),
            example: None,
            required: Some(true),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
            default: None,
        },
    );

    // Test missing required variable
    let request = TemplateProcessingRequest {
        variables: HashMap::new(),
        built_in_variables: HashMap::new(),
        variable_configs,
        templating_config: None,
    };

    let result = processor.validate_variables(&request);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::RequiredVariableMissing(_)
    ));
}

#[test]
fn test_validate_variables_pattern() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let mut variable_configs = HashMap::new();
    variable_configs.insert(
        "email".to_string(),
        VariableConfig {
            description: "Email address".to_string(),
            example: None,
            required: Some(true),
            pattern: Some(r"^[^@]+@[^@]+\.[^@]+$".to_string()),
            min_length: None,
            max_length: None,
            options: None,
            default: None,
        },
    );

    let mut variables = HashMap::new();
    variables.insert("email".to_string(), "invalid-email".to_string());

    let request = TemplateProcessingRequest {
        variables,
        built_in_variables: HashMap::new(),
        variable_configs,
        templating_config: None,
    };

    let result = processor.validate_variables(&request);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::PatternValidationFailed { .. }
    ));
}

#[test]
fn test_validate_variables_length() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let mut variable_configs = HashMap::new();
    variable_configs.insert(
        "name".to_string(),
        VariableConfig {
            description: "Project name".to_string(),
            example: None,
            required: Some(true),
            pattern: None,
            min_length: Some(3),
            max_length: Some(10),
            options: None,
            default: None,
        },
    );

    // Test too short
    let mut variables = HashMap::new();
    variables.insert("name".to_string(), "ab".to_string());

    let request = TemplateProcessingRequest {
        variables: variables.clone(),
        built_in_variables: HashMap::new(),
        variable_configs: variable_configs.clone(),
        templating_config: None,
    };

    let result = processor.validate_variables(&request);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::VariableValidation { .. }
    ));

    // Test too long
    variables.insert("name".to_string(), "this-is-too-long".to_string());
    let request = TemplateProcessingRequest {
        variables,
        built_in_variables: HashMap::new(),
        variable_configs,
        templating_config: None,
    };

    let result = processor.validate_variables(&request);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::VariableValidation { .. }
    ));
}

#[test]
fn test_validate_variables_options() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let mut variable_configs = HashMap::new();
    variable_configs.insert(
        "license".to_string(),
        VariableConfig {
            description: "License type".to_string(),
            example: None,
            required: Some(true),
            pattern: None,
            min_length: None,
            max_length: None,
            options: Some(vec![
                "MIT".to_string(),
                "Apache-2.0".to_string(),
                "GPL-3.0".to_string(),
            ]),
            default: None,
        },
    );

    let mut variables = HashMap::new();
    variables.insert("license".to_string(), "BSD".to_string()); // Not in allowed options

    let request = TemplateProcessingRequest {
        variables,
        built_in_variables: HashMap::new(),
        variable_configs,
        templating_config: None,
    };

    let result = processor.validate_variables(&request);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::VariableValidation { .. }
    ));
}

#[test]
fn test_process_template_with_default_values() {
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    let files = vec![
        (
            "README.md".to_string(),
            b"# {{project_name}}\n\nBy {{author}}".to_vec(),
        ),
        (
            "config.yml".to_string(),
            b"version: {{version}}\nenvironment: {{env}}".to_vec(),
        ),
    ];

    // Don't provide any user variables - should use defaults
    let user_variables = HashMap::new();

    // Configure variables with default values
    let mut variable_configs = HashMap::new();
    variable_configs.insert(
        "project_name".to_string(),
        VariableConfig {
            description: "Project name".to_string(),
            example: None,
            required: Some(false),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
            default: Some("Default Project".to_string()),
        },
    );
    variable_configs.insert(
        "author".to_string(),
        VariableConfig {
            description: "Author name".to_string(),
            example: None,
            required: Some(false),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
            default: Some("Default Author".to_string()),
        },
    );
    variable_configs.insert(
        "version".to_string(),
        VariableConfig {
            description: "Version number".to_string(),
            example: None,
            required: Some(false),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
            default: Some("1.0.0".to_string()),
        },
    );
    variable_configs.insert(
        "env".to_string(),
        VariableConfig {
            description: "Environment".to_string(),
            example: None,
            required: Some(false),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
            default: Some("development".to_string()),
        },
    );

    let request = TemplateProcessingRequest {
        variables: user_variables,
        built_in_variables: HashMap::new(),
        variable_configs,
        templating_config: None,
    };

    let result = processor
        .process_template(&files, &request, Path::new("."))
        .unwrap();

    assert_eq!(result.files.len(), 2);

    let readme_content = String::from_utf8(result.files[0].1.clone()).unwrap();
    assert_eq!(readme_content, "# Default Project\n\nBy Default Author");

    let config_content = String::from_utf8(result.files[1].1.clone()).unwrap();
    assert_eq!(config_content, "version: 1.0.0\nenvironment: development");
}

#[test]
fn test_integration_test_variable_substitution_scenario() {
    // This test simulates the exact integration test scenario that was failing
    let processor = TemplateProcessor::new().expect("Failed to create processor");

    // Files from test-variables template (as documented in specs)
    let files = vec![
        (
            "README.md".to_string(),
            b"# {{project_name}}\n\n{{project_description}}".to_vec(),
        ),
        (
            "src/main.rs".to_string(),
            b"fn main() {\n    println!(\"Hello from {{project_name}}!\");\n}".to_vec(),
        ),
        (
            "Cargo.toml".to_string(),
            b"[package]\nname = \"{{project_name}}\"\nversion = \"{{version}}\"".to_vec(),
        ),
    ];

    // Empty user variables (as was happening in integration test)
    let user_variables = HashMap::new();

    // Built-in variables (as provided by repo_roller_core)
    let mut built_in_variables = HashMap::new();
    built_in_variables.insert("repo_name".to_string(), "test-repo-name".to_string());
    built_in_variables.insert("org_name".to_string(), "test-org".to_string());

    // Variable configs with defaults (our fix to integration test)
    let mut variable_configs = HashMap::new();
    variable_configs.insert(
        "project_name".to_string(),
        VariableConfig {
            description: "Name of the project".to_string(),
            example: Some("my-awesome-project".to_string()),
            required: Some(false),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
            default: Some("test-project".to_string()),
        },
    );
    variable_configs.insert(
        "project_description".to_string(),
        VariableConfig {
            description: "Description of the project".to_string(),
            example: Some("A simple test project".to_string()),
            required: Some(false),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
            default: Some("Integration test project for RepoRoller".to_string()),
        },
    );
    variable_configs.insert(
        "version".to_string(),
        VariableConfig {
            description: "Project version".to_string(),
            example: Some("1.0.0".to_string()),
            required: Some(false),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
            default: Some("0.1.0".to_string()),
        },
    );

    let request = TemplateProcessingRequest {
        variables: user_variables,
        built_in_variables,
        variable_configs,
        templating_config: None,
    };

    // This was failing before our fix, should now succeed
    let result = processor
        .process_template(&files, &request, Path::new("."))
        .expect("Template processing should succeed with default values");

    assert_eq!(result.files.len(), 3);

    // Verify defaults were applied and built-ins were preserved
    let readme_content = String::from_utf8(result.files[0].1.clone()).unwrap();
    assert!(readme_content.contains("# test-project"));
    assert!(readme_content.contains("Integration test project for RepoRoller"));

    let src_content = String::from_utf8(result.files[1].1.clone()).unwrap();
    assert!(src_content.contains("Hello from test-project!"));

    let cargo_content = String::from_utf8(result.files[2].1.clone()).unwrap();
    assert!(cargo_content.contains("name = \"test-project\""));
    assert!(cargo_content.contains("version = \"0.1.0\""));
}
