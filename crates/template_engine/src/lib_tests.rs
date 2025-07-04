use super::*;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn test_built_in_variables_generation() {
    let processor = TemplateProcessor::new();
    
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
    assert_eq!(variables.get("template_name"), Some(&"rust-library".to_string()));
    assert_eq!(variables.get("template_repo"), Some(&"templates/rust-library".to_string()));
    assert_eq!(variables.get("user_login"), Some(&"testuser".to_string()));
    assert_eq!(variables.get("user_name"), Some(&"Test User".to_string()));
    assert_eq!(variables.get("default_branch"), Some(&"main".to_string()));
    
    // Check that timestamp variables are present
    assert!(variables.contains_key("timestamp"));
    assert!(variables.contains_key("timestamp_unix"));
}

#[test]
fn test_is_text_file() {
    let processor = TemplateProcessor::new();
    
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
    let processor = TemplateProcessor::new();
    
    let files = vec![
        ("README.md".to_string(), b"# {{project_name}}\n\nBy {{author}}".to_vec()),
        ("src/main.rs".to_string(), b"// {{project_name}} by {{author}}\nfn main() {}".to_vec()),
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
    
    let result = processor.process_template(&files, &request, Path::new(".")).unwrap();
    
    assert_eq!(result.files.len(), 2);
    
    let readme_content = String::from_utf8(result.files[0].1.clone()).unwrap();
    assert_eq!(readme_content, "# Test Project\n\nBy John Doe");
    
    let main_content = String::from_utf8(result.files[1].1.clone()).unwrap();
    assert_eq!(main_content, "// Test Project by John Doe\nfn main() {}");
}

#[test]
fn test_process_template_with_filtering() {
    let processor = TemplateProcessor::new();
    
    let files = vec![
        ("README.md".to_string(), b"# {{project_name}}".to_vec()),
        ("src/main.rs".to_string(), b"fn main() {}".to_vec()),
        ("target/debug/app".to_string(), b"binary content".to_vec()),
        ("tests/test.rs".to_string(), b"#[test] fn test() {}".to_vec()),
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
    
    let result = processor.process_template(&files, &request, Path::new(".")).unwrap();
    
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
    let processor = TemplateProcessor::new();
    
    let files = vec![
        ("README.md.template".to_string(), b"# {{project_name}}".to_vec()),
        ("Cargo.toml.template".to_string(), b"[package]\nname = \"{{project_name}}\"".to_vec()),
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
    
    let result = processor.process_template(&files, &request, Path::new(".")).unwrap();
    
    assert_eq!(result.files.len(), 3);
    
    let file_paths: Vec<&String> = result.files.iter().map(|(path, _)| path).collect();
    assert!(file_paths.contains(&&"README.md".to_string()));
    assert!(file_paths.contains(&&"Cargo.toml".to_string()));
    assert!(file_paths.contains(&&"src/main.rs".to_string()));
}

#[test]
fn test_simple_glob_match() {
    let processor = TemplateProcessor::new();
    
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
fn test_substitute_variables() {
    let processor = TemplateProcessor::new();
    
    let mut variables = HashMap::new();
    variables.insert("name".to_string(), "John".to_string());
    variables.insert("project".to_string(), "MyApp".to_string());
    
    let content = "Hello {{name}}, welcome to {{project}}!";
    let result = processor.substitute_variables(content, &variables);
    assert_eq!(result, "Hello John, welcome to MyApp!");
    
    // Test escaped braces
    let content_with_escaped = "Use \\{{name}} for {{name}} substitution";
    let result = processor.substitute_variables(content_with_escaped, &variables);
    assert_eq!(result, "Use {{name}} for John substitution");
    
    // Test missing variables (should remain unchanged)
    let content_missing = "Hello {{name}}, your age is {{age}}";
    let result = processor.substitute_variables(content_missing, &variables);
    assert_eq!(result, "Hello John, your age is {{age}}");
}

#[test]
fn test_template_processor_default() {
    let processor1 = TemplateProcessor::new();
    let processor2 = TemplateProcessor::default();
    
    // Both should work the same way
    let mut variables = HashMap::new();
    variables.insert("test".to_string(), "value".to_string());
    
    let result1 = processor1.substitute_variables("{{test}}", &variables);
    let result2 = processor2.substitute_variables("{{test}}", &variables);
    
    assert_eq!(result1, result2);
    assert_eq!(result1, "value");
}

#[test]
fn test_validate_variables_required() {
    let processor = TemplateProcessor::new();
    
    let mut variable_configs = HashMap::new();
    variable_configs.insert("required_var".to_string(), VariableConfig {
        description: "A required variable".to_string(),
        example: None,
        required: Some(true),
        pattern: None,
        min_length: None,
        max_length: None,
        options: None,
        default: None,
    });
    
    // Test missing required variable
    let request = TemplateProcessingRequest {
        variables: HashMap::new(),
        built_in_variables: HashMap::new(),
        variable_configs,
        templating_config: None,
    };
    
    let result = processor.validate_variables(&request);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::RequiredVariableMissing(_)));
}

#[test]
fn test_validate_variables_pattern() {
    let processor = TemplateProcessor::new();
    
    let mut variable_configs = HashMap::new();
    variable_configs.insert("email".to_string(), VariableConfig {
        description: "Email address".to_string(),
        example: None,
        required: Some(true),
        pattern: Some(r"^[^@]+@[^@]+\.[^@]+$".to_string()),
        min_length: None,
        max_length: None,
        options: None,
        default: None,
    });
    
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
    assert!(matches!(result.unwrap_err(), Error::PatternValidationFailed { .. }));
}

#[test]
fn test_validate_variables_length() {
    let processor = TemplateProcessor::new();
    
    let mut variable_configs = HashMap::new();
    variable_configs.insert("name".to_string(), VariableConfig {
        description: "Project name".to_string(),
        example: None,
        required: Some(true),
        pattern: None,
        min_length: Some(3),
        max_length: Some(10),
        options: None,
        default: None,
    });
    
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
    assert!(matches!(result.unwrap_err(), Error::VariableValidation { .. }));
    
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
    assert!(matches!(result.unwrap_err(), Error::VariableValidation { .. }));
}

#[test]
fn test_validate_variables_options() {
    let processor = TemplateProcessor::new();
    
    let mut variable_configs = HashMap::new();
    variable_configs.insert("license".to_string(), VariableConfig {
        description: "License type".to_string(),
        example: None,
        required: Some(true),
        pattern: None,
        min_length: None,
        max_length: None,
        options: Some(vec!["MIT".to_string(), "Apache-2.0".to_string(), "GPL-3.0".to_string()]),
        default: None,
    });
    
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
    assert!(matches!(result.unwrap_err(), Error::VariableValidation { .. }));
}
