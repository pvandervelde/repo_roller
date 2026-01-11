# RepoRoller

A GitHub app that creates new repositories from templates or as empty repositories with organization-specific configuration management.

## Features

- **Template-based Creation**: Create repositories from template repositories with variable substitution
- **Empty Repositories**: Create empty repositories for code migration and custom structures
- **Custom Initialization**: Create repositories with just README.md and/or .gitignore
- **Configuration Hierarchy**: Four-level configuration (Template → Team → Type → Global)
- **Organization-wide Policies**: Enforce security and branch protection across all repositories
- **Multiple Interfaces**: CLI, REST API, and MCP server support

## Documentation

- **[Empty Repositories Guide](crates/repo_roller_cli/EMPTY_REPOSITORIES.md)** - Creating repositories without templates
- **[Template Commands](crates/repo_roller_cli/TEMPLATE_COMMANDS.md)** - Working with template repositories
- **[REST API Documentation](crates/repo_roller_api/README.md)** - HTTP API reference
- **[Specifications](specs/README.md)** - Architecture and design specifications

## Quick Start

### CLI

```bash
# Create from template
repo-roller create --org myorg --repo my-repo --template rust-library

# Create empty repository
repo-roller create --org myorg --repo my-repo --empty

# Create with custom initialization
repo-roller create --org myorg --repo my-repo --init-readme --init-gitignore
```

### REST API

```bash
# Create from template
curl -X POST http://localhost:3000/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-repo", "organization": "myorg", "template": "rust-library"}'

# Create empty repository
curl -X POST http://localhost:3000/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-repo", "organization": "myorg", "contentStrategy": "empty"}'
```

## License

MIT - See [LICENSE](LICENSE) for details
