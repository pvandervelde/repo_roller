# Template Processing Design

## Overview

The template processing system is responsible for taking template repositories and customizing them based on user-provided variables to create new repositories. This system uses Handlebars templating engine to provide powerful and flexible template processing capabilities.

## Architecture

### Template Engine Design

The template processing is built around the `template_engine` crate with the following key components:

```rust
pub struct HandlebarsTemplateEngine {
    handlebars: handlebars::Handlebars<'static>,
}

pub struct TemplateContext {
    variables: HashMap<String, serde_json::Value>,
    helpers: Vec<Box<dyn handlebars::HelperDef + Send + Sync>>,
}
```

### Core Capabilities

#### 1. Content Templating

**Variable Substitution:**

- Basic variables: `{{variable_name}}`
- Nested objects: `{{object.property}}`
- Array access: `{{array.[0]}}`

**Control Structures:**

- Conditionals: `{{#if condition}}...{{/if}}`
- Loops: `{{#each items}}...{{/each}}`
- Context switching: `{{#with object}}...{{/with}}`

**Built-in Helpers:**

- `{{#unless condition}}...{{/unless}}`
- `{{#each items}}{{@index}}: {{this}}{{/each}}`

#### 2. File Path Templating

**Capability:**

- Process file paths and directory names through Handlebars
- Support complex path templating: `{{repo_name}}/{{environment}}/config.yml`
- Handle special characters and path separators correctly
- Validate resulting file paths for security (no path traversal)

**Implementation:**

- `template_file_path()` method processes file names
- `process_files()` applies templating to both content and paths
- Path normalization and security validation

#### 3. Custom Handlebars Helpers

**Repository-Specific Helpers:**

- `{{snake_case variable}}` - Convert to snake_case
- `{{kebab_case variable}}` - Convert to kebab-case
- `{{upper_case variable}}` - Convert to UPPER_CASE
- `{{lower_case variable}}` - Convert to lowercase
- `{{capitalize variable}}` - Capitalize first letter
- `{{timestamp format}}` - Format timestamps
- `{{default value fallback}}` - Provide default values

### Processing Workflow

1. **Template Repository Cloning**
   - Clone the specified template repository
   - Validate repository structure and permissions

2. **Context Preparation**
   - Build template context from user variables
   - Register custom helpers
   - Validate variable types and constraints

3. **File Discovery**
   - Scan template repository for processable files
   - Identify binary files to skip
   - Build file processing queue

4. **Path Templating**
   - Process directory names through Handlebars
   - Process file names through Handlebars
   - Validate resulting paths for security
   - Create target directory structure

5. **Content Templating**
   - Process file contents through Handlebars
   - Apply variable substitution and control structures
   - Execute custom helpers as needed

6. **Output Generation**
   - Write processed files to target location
   - Preserve file permissions and metadata
   - Generate initial Git commit

## Implementation Details

### Dependencies

```toml
[dependencies]
handlebars = "4.4"
serde_json = "1.0"
```

### Security Considerations

**File Path Validation:**

- Prevent directory traversal attacks (../../../etc/passwd)
- Validate file names for filesystem compatibility
- Ensure paths stay within target directory bounds

**Template Sandboxing:**

- Template helpers cannot access filesystem directly
- No network access from template context
- Input validation for all template variables

**Resource Limits:**

- Template processing timeout (< 30 seconds)
- Memory usage limits (< 100MB for context)
- File size limits for template processing

### Performance Optimizations

**Template Compilation Caching:**

- Cache compiled Handlebars templates for reuse
- Template compilation is expensive, rendering is fast
- Clear cache when template changes detected

**Context Building:**

- Optimize context building for common use cases
- Lazy evaluation of complex variables
- Efficient JSON serialization for variable passing

### Error Handling

**Template Syntax Errors:**

- Comprehensive error types for different syntax issues
- Clear error messages with line numbers
- Context preservation through processing pipeline

**Runtime Errors:**

- Graceful handling of missing variables
- Type mismatch error reporting
- Timeout and resource limit enforcement

**Validation Errors:**

- File path validation failure reporting
- Variable constraint violation messages
- Template helper execution error handling

## Edge Cases and Considerations

### File Name Templating Edge Cases

1. **Invalid Characters:** Template results containing filesystem-invalid characters
   - Solution: Validate and sanitize generated file names
   - Fallback to safe character substitution

2. **Path Traversal:** Templates attempting to create paths outside target directory
   - Solution: Strict path validation and normalization
   - Reject any paths containing `..` or absolute paths

3. **Name Collisions:** Multiple template files resolving to same final path
   - Solution: Detect collisions and report as error
   - Provide clear conflict resolution guidance

4. **Empty Names:** Templates that resolve to empty file names
   - Solution: Validate for non-empty results
   - Require minimum viable file name length

5. **Very Long Paths:** Handling platform-specific path length limits
   - Solution: Validate against OS path length limits
   - Provide truncation strategies if needed

### Content Templating Edge Cases

1. **Binary Files:** Ensure binary files are not processed as templates
   - Solution: File type detection using magic numbers
   - Maintain whitelist of processable file extensions

2. **Large Files:** Memory usage with very large template files
   - Solution: Streaming template processing for large files
   - File size limits and progress reporting

3. **Malformed Templates:** Graceful handling of syntax errors
   - Solution: Pre-validation of template syntax
   - Detailed error reporting with line numbers

4. **Circular References:** Prevention of infinite template expansion
   - Solution: Recursion depth limits
   - Cycle detection in template includes

5. **Special Characters:** Proper handling of Unicode and escape sequences
   - Solution: UTF-8 compliant processing throughout
   - Proper escaping in generated content

### Variable Resolution Edge Cases

1. **Missing Variables:** Behavior when referenced variables are undefined
   - Solution: Configurable handling (error, warning, or empty)
   - Clear reporting of undefined variable usage

2. **Type Mismatches:** Handling when variables have unexpected types
   - Solution: Type coercion where safe
   - Clear error messages for incompatible operations

3. **Nested Objects:** Complex variable structures in templates
   - Solution: Full JSON object navigation support
   - Safe property access with existence checking

4. **Case Sensitivity:** Consistent variable name handling
   - Solution: Case-sensitive variable matching
   - Clear documentation of naming conventions

## Migration Strategy

### Phase 1: Infrastructure Setup

1. Add Handlebars dependencies to Cargo.toml
2. Create `HandlebarsTemplateEngine` alongside existing implementation
3. Implement basic variable substitution with Handlebars
4. Add comprehensive unit tests

### Phase 2: Feature Enhancement

1. Implement custom helpers
2. Add file name templating functionality
3. Enhanced error handling and validation
4. Integration tests with complex templates

### Phase 3: Migration and Cleanup

1. Replace `substitute_variables()` calls with new Handlebars implementation
2. Update documentation and examples
3. Remove old string replacement code
4. Performance testing and optimization

## Acceptance Criteria

### Core Functionality

1. **Handlebars Integration:** Template content is processed using Handlebars instead of string replacement
2. **Variable Substitution:** All existing `{{variable}}` patterns work correctly
3. **Advanced Features:** Support for conditionals (`{{#if}}`), loops (`{{#each}}`), and helpers
4. **File Name Templating:** File and directory names can contain template variables
5. **Custom Helpers:** Repository-specific helpers are available (snake_case, kebab_case, etc.)

### Quality and Safety

1. **Error Handling:** Clear, actionable error messages for template syntax errors
2. **Security:** File path validation prevents directory traversal
3. **Performance:** Template processing performance is comparable or better than string replacement
4. **Backward Compatibility:** Existing templates continue to work without modification

### Testing

1. **Unit Tests:** Comprehensive coverage of all new templating features
2. **Integration Tests:** End-to-end testing with realistic template repositories
3. **Edge Case Tests:** All identified edge cases have corresponding test coverage
4. **Performance Tests:** Benchmarks for template processing performance

## Behavioral Assertions

1. Template processing must be deterministic - same inputs produce same outputs
2. File path templating must never allow creation of files outside the target directory
3. Template errors must not cause crashes - all errors must be handled gracefully
4. Binary files must never be processed as templates, regardless of configuration
5. Template variable names are case-sensitive and follow exact matching rules
6. Custom helpers must be isolated and cannot access filesystem or network resources
7. Template processing must complete within reasonable time bounds (< 30s for typical repositories)
8. All generated file paths must be valid for the target operating system
9. Template expansion must not exceed reasonable memory limits (< 100MB for context)
10. Handlebars template compilation errors must be caught and reported with line numbers
