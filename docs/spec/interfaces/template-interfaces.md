# Template Processing Interfaces

**Architectural Layer**: Template processing
**Crate**: `template_engine`
**Responsibilities**:

- **Knows**: Template file formats, variable substitution, and validation rules
- **Does**: Fetches template files and renders content for repository creation

---

## Overview

Template processing is split across configuration loading and content
rendering. `config_manager::TemplateLoader` loads and caches template
configuration; `template_engine::TemplateFetcher` retrieves template files,
and `TemplateProcessor` renders eligible text files using Handlebars.

## Template file fetching

```rust
#[async_trait]
pub trait TemplateFetcher: Send + Sync {
    async fn fetch_template_files(
        &self,
        source_repo: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, String>;
}
```

The fetcher accepts a string repository reference and returns relative file
paths with their contents. The `template_engine` crate does not depend on
`repo_roller_core`, so it intentionally does not accept core domain newtypes.
Callers pass validated values as strings at this crate boundary. No separate
`TemplateSource` type is part of this interface.

## Template processor

`TemplateProcessor` renders fetched files using a
`TemplateProcessingRequest`, which carries user variables, built-in
variables, variable configuration, and optional templating configuration.
`BuiltInVariablesParams` also borrows strings for repository, organization,
template, user, and branch data for the same crate-boundary reason.

The core domain uses `RepositoryName`, `OrganizationName`, and `TemplateName`
to validate names before they reach the template engine. The processor
receives their string representations; it does not revalidate those domain
names.

Processing validates required variables and configured constraints before
rendering. Binary files are preserved without text substitution. Rendering
and validation failures are returned as template-processing errors to the
repository-creation workflow.

## Variable configuration

Variable configuration describes whether a value is required and may provide
a description, example, default, pattern, minimum or maximum length, and
allowed options. The processor applies the constraints supplied for each
variable; the template configuration schema is defined by
`config_manager::TemplateVariable`.

## Built-in variable boundary

The parameters are borrowed strings rather than branded types:

```rust
pub struct BuiltInVariablesParams<'a> {
    pub repo_name: &'a str,
    pub org_name: &'a str,
    pub template_name: &'a str,
    pub template_repo: &'a str,
    pub user_login: &'a str,
    pub user_name: &'a str,
    pub default_branch: &'a str,
}
```

This keeps `template_engine` independent of `repo_roller_core`. Core callers
can pass a domain value using `as_ref()` or `as_str()` where available.

## Related Specifications

- [Configuration Management](configuration-interfaces.md) — Template configuration resolution
- [Template Configuration Loading](template-loading.md) — Loading template metadata
- [Shared Types](shared-types.md) — Core validated names
