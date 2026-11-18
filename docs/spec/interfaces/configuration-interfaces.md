# Configuration Management Interfaces

**Architectural Layer**: Configuration orchestration and infrastructure
**Crate**: `config_manager`
**Responsibilities**:

- **Knows**: Metadata repository layout, configuration schemas, and merge precedence
- **Does**: Discovers and loads configuration, resolves templates, merges settings, and validates the result

---

## Overview

Configuration is resolved for each repository creation request by
`OrganizationSettingsManager`. It loads applicable settings from the metadata
repository, loads an optional template configuration from the template
repository, merges the values, and validates the merged configuration before
the repository is created.

The configuration model is implemented with concrete types and services; it
does not expose a `ConfigurationManager` trait.

## Configuration sources

The merger considers the applicable sources in this order:

1. Organization-wide `GlobalDefaults`
2. Repository-type configuration
3. Team configuration
4. Template configuration

Later levels override earlier values where a setting is overridable. Settings
documented as additive (including naming rules, labels, webhooks, rulesets,
and notification endpoints) are accumulated according to their type-specific
merge behavior.

## Main components

### OrganizationSettingsManager

`OrganizationSettingsManager` is the configuration-resolution entry point. It
uses a `MetadataRepositoryProvider` to discover and load metadata, a
`TemplateLoader` to load optional template configuration, a
`ConfigurationMerger` to combine sources, and a validator to check the merged
result. Callers provide a `ConfigurationContext` identifying the organization
and any applicable team, repository type, and template.

### Configuration source types

- `GlobalDefaults` contains organization-wide settings.
- `TeamConfig` contains team-level overrides and additive settings.
- `RepositoryTypeConfig` contains repository-type settings.
- `TemplateConfig` contains template metadata, variables, and template-level
  settings.
- `MergedConfiguration` is the resolved result consumed by repository
  creation.

### Template loading

`TemplateRepository` abstracts retrieval of template configuration.
`GitHubTemplateRepository` reads `.reporoller/template.toml` from GitHub
repositories, and `TemplateLoader` parses and caches the resulting
`TemplateConfig`. See [Template Configuration Loading](template-loading.md).

## Naming rules

Repository naming rules are represented by `RepositoryNamingRulesConfig`, not
by one organization-level regular expression. A rule can specify an allowed
pattern, forbidden patterns, reserved words, a required prefix or suffix, and
minimum or maximum length. Rules from applicable sources are additive, and a
repository name must satisfy every merged rule.

## Error handling

Configuration operations return `ConfigurationResult<T>`. Failures include
metadata discovery or loading errors, invalid configuration, template lookup
or parsing errors, and validation failures. The repository-creation
orchestrator converts configuration errors into `RepoRollerError` for its
public result.

## Related Specifications

- [Repository Domain](repository-domain.md) — Configuration use during repository creation
- [Shared Types](shared-types.md) — Domain value validation
- [Template Configuration Loading](template-loading.md) — Template source and cache behavior
