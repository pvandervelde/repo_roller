# Creating Empty Repositories

This guide covers creating repositories without templates, including empty repositories and custom initialization options.

## Table of Contents

- [Overview](#overview)
- [Content Strategies](#content-strategies)
- [CLI Usage](#cli-usage)
- [API Usage](#api-usage)
- [Use Cases](#use-cases)
- [Comparison with Template-Based Creation](#comparison-with-template-based-creation)
- [Migration Scenarios](#migration-scenarios)

## Overview

RepoRoller supports three content strategies for repository creation:

1. **Template-based** (default): Create from a template repository
2. **Empty**: Create an empty repository with only organization defaults
3. **Custom Init**: Create with just README.md and/or .gitignore

Empty repository creation is useful when:

- You need a blank slate without template constraints
- You're importing existing code
- You want to start with minimal files
- You're creating infrastructure-only repositories

**Important**: Empty repositories still apply organization-wide settings (security policies, branch protection, etc.) but don't include template files or template-specific configuration.

## Content Strategies

### Template (Default)

**What it includes**:

- All files from template repository
- Template-specific configuration
- Variable substitution in files and paths
- Organization and team settings

**When to use**:

- Starting new projects with standard structure
- Enforcing team conventions
- Rapid project scaffolding

**Example**: Creating a new Rust library with standard CI/CD, docs, and project structure.

### Empty

**What it includes**:

- No files in the repository
- Organization default settings only
- Security policies and branch protection

**When to use**:

- Importing existing codebases
- Creating custom repository structures
- Testing or experimental repositories
- Starting completely from scratch

**Example**: Creating a repository before migrating an existing project from another VCS.

### Custom Init

**What it includes**:

- README.md (optional)
- .gitignore (optional)
- Organization default settings
- Security policies and branch protection

**When to use**:

- Need basic documentation immediately
- Want language-specific .gitignore from start
- Prefer minimal boilerplate
- Starting simple projects

**Example**: Creating a Python project with a Python .gitignore and basic README.

## CLI Usage

### Creating Empty Repositories

#### Basic Empty Repository

Create a completely empty repository with organization defaults:

```bash
repo-roller create \
  --org myorg \
  --repo my-empty-repo \
  --empty \
  --description "Empty repository for code import"
```

**Result**:

- Repository created in GitHub
- Organization security policies applied
- No files in repository
- Ready for initial commit

#### Empty Repository with Visibility

```bash
repo-roller create \
  --org myorg \
  --repo my-private-repo \
  --empty \
  --visibility private \
  --description "Private empty repository"
```

#### Empty Repository with Template Settings

You can still use template settings (team, type) without template files:

```bash
repo-roller create \
  --org myorg \
  --repo my-service-repo \
  --empty \
  --repository-type service \
  --team platform \
  --description "Empty service repository with platform team settings"
```

**Result**:

- Empty repository (no template files)
- Service type configuration applied
- Platform team settings applied
- Branch protection from type/team configs

### Custom Initialization

#### With README Only

Create repository with generated README:

```bash
repo-roller create \
  --org myorg \
  --repo my-readme-repo \
  --init-readme \
  --description "Repository with basic README"
```

**Generated README content**:

```markdown
# my-readme-repo

Repository with basic README
```

#### With .gitignore Only

Create repository with .gitignore (requires specifying content):

```bash
repo-roller create \
  --org myorg \
  --repo my-python-repo \
  --init-gitignore \
  --description "Python repository with .gitignore"
```

**Note**: When using `--init-gitignore`, you'll need to provide the .gitignore content (implementation determines exact mechanism).

#### With Both README and .gitignore

```bash
repo-roller create \
  --org myorg \
  --repo my-full-init-repo \
  --init-readme \
  --init-gitignore \
  --description "Repository with README and .gitignore"
```

#### Custom Init with Settings

Combine custom initialization with template settings:

```bash
repo-roller create \
  --org myorg \
  --repo my-configured-repo \
  --init-readme \
  --init-gitignore \
  --repository-type library \
  --team backend \
  --visibility private \
  --description "Library repository with custom init and team settings"
```

**Result**:

- README.md generated
- .gitignore created
- Library type configuration applied
- Backend team settings applied
- Private visibility set

### Combining with Other Options

All standard creation options work with empty/custom init:

```bash
# With repository type
repo-roller create --org myorg --repo my-repo --empty --repository-type library

# With team
repo-roller create --org myorg --repo my-repo --empty --team platform

# With type and team
repo-roller create --org myorg --repo my-repo --init-readme --repository-type service --team api
```

## API Usage

### Empty Repository Endpoint

#### Request

```bash
POST /api/v1/repositories
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "my-empty-repo",
  "organization": "myorg",
  "contentStrategy": "empty",
  "description": "Empty repository for code import",
  "visibility": "private"
}
```

**Fields**:

- `contentStrategy`: Set to `"empty"` for empty repository
- `template`: Omit or set to `null`
- All other fields work as normal

#### Response

```json
{
  "repository": {
    "name": "my-empty-repo",
    "url": "https://github.com/myorg/my-empty-repo",
    "visibility": "private"
  },
  "appliedConfiguration": {
    "source": "global",
    "settings": {
      "features": {
        "hasIssues": true,
        "hasProjects": false,
        "hasWiki": false
      },
      "security": {
        "requireSignedCommits": true,
        "vulnerabilityAlertsEnabled": true
      }
    }
  }
}
```

### Empty with Template Settings

Use organization settings without template files:

```bash
POST /api/v1/repositories
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "my-configured-repo",
  "organization": "myorg",
  "contentStrategy": "empty",
  "repositoryType": "service",
  "team": "platform",
  "description": "Empty service repository with platform settings",
  "visibility": "private"
}
```

**Result**:

- Empty repository
- Service type settings applied
- Platform team settings applied
- No template files

### Custom Initialization Endpoint

#### With README Only

```bash
POST /api/v1/repositories
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "my-readme-repo",
  "organization": "myorg",
  "contentStrategy": "custom_init",
  "initializeReadme": true,
  "initializeGitignore": false,
  "description": "Repository with README",
  "visibility": "private"
}
```

#### With Both Files

```bash
POST /api/v1/repositories
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "my-init-repo",
  "organization": "myorg",
  "contentStrategy": "custom_init",
  "initializeReadme": true,
  "initializeGitignore": true,
  "description": "Repository with README and .gitignore",
  "visibility": "private"
}
```

#### With Settings

```bash
POST /api/v1/repositories
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "my-full-repo",
  "organization": "myorg",
  "contentStrategy": "custom_init",
  "initializeReadme": true,
  "initializeGitignore": true,
  "repositoryType": "library",
  "team": "backend",
  "description": "Library with init files and team settings",
  "visibility": "private"
}
```

### Field Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Repository name |
| `organization` | string | Yes | Organization name |
| `contentStrategy` | string | No | `"template"` (default), `"empty"`, or `"custom_init"` |
| `template` | string | Conditional | Required if `contentStrategy` is `"template"` or omitted. Ignored for `"empty"` or `"custom_init"` |
| `initializeReadme` | boolean | No | Create README.md (only with `"custom_init"`) |
| `initializeGitignore` | boolean | No | Create .gitignore (only with `"custom_init"`) |
| `repositoryType` | string | No | Repository type configuration |
| `team` | string | No | Team configuration |
| `description` | string | No | Repository description |
| `visibility` | string | No | `"private"` (default) or `"public"` |

## Use Cases

### 1. Migrating Existing Code

**Scenario**: Moving an existing project from GitLab to GitHub.

**Solution**: Create empty repository, then push existing code:

```bash
# Create empty repository
repo-roller create \
  --org myorg \
  --repo migrated-project \
  --empty \
  --repository-type library \
  --team backend \
  --description "Migrated from GitLab"

# In your existing project directory
git remote add origin https://github.com/myorg/migrated-project.git
git push -u origin main
```

**Benefits**:

- Organization security policies applied from start
- Team settings configured
- No template files to remove
- Clean git history preserved

### 2. Infrastructure Repositories

**Scenario**: Creating Terraform/Ansible infrastructure-as-code repository.

**Solution**: Use custom init with .gitignore:

```bash
repo-roller create \
  --org myorg \
  --repo infrastructure-aws \
  --init-readme \
  --init-gitignore \
  --repository-type infrastructure \
  --team platform \
  --description "AWS infrastructure definitions"
```

**Benefits**:

- Basic README for documentation
- .gitignore for Terraform state files
- Infrastructure type settings
- No unnecessary template files

### 3. Experimental Projects

**Scenario**: Creating a research/experimental repository for trying new ideas.

**Solution**: Minimal empty repository:

```bash
repo-roller create \
  --org myorg \
  --repo experiment-new-framework \
  --empty \
  --description "Experimental: testing new framework"
```

**Benefits**:

- Maximum flexibility
- No cleanup needed
- Fast creation
- Still protected by organization policies

### 4. Documentation-Only Repositories

**Scenario**: Creating a repository for documentation or wiki content.

**Solution**: README-only initialization:

```bash
repo-roller create \
  --org myorg \
  --repo team-documentation \
  --init-readme \
  --repository-type documentation \
  --team everyone \
  --description "Team documentation and guides"
```

**Benefits**:

- Starts with README structure
- Documentation type settings
- Team-wide access configured
- No code boilerplate

### 5. Starting from Scratch

**Scenario**: New project that doesn't fit existing templates.

**Solution**: Custom init with project type settings:

```bash
repo-roller create \
  --org myorg \
  --repo novel-project \
  --init-readme \
  --init-gitignore \
  --repository-type library \
  --description "New project with custom structure"
```

**Benefits**:

- Basic files to start
- Library settings applied
- Complete freedom in structure
- Organization policies enforced

## Comparison with Template-Based Creation

### When to Use Templates

✅ **Use templates when**:

- Starting projects with standard structure
- Enforcing team conventions
- Need pre-configured CI/CD
- Want consistent project layout
- Require boilerplate code

**Example**: Creating a new microservice following team standards.

### When to Use Empty/Custom Init

✅ **Use empty/custom init when**:

- Importing existing code
- Need complete flexibility
- Templates are too opinionated
- Creating non-standard repositories
- Want minimal initial files

**Example**: Migrating an existing project from another platform.

### Feature Comparison

| Feature | Template | Empty | Custom Init |
|---------|----------|-------|-------------|
| Initial files | ✅ Many | ❌ None | ⚙️ 0-2 files |
| Variable substitution | ✅ Yes | ❌ No | ❌ No |
| Template config | ✅ Applied | ❌ Not applied | ❌ Not applied |
| Org defaults | ✅ Applied | ✅ Applied | ✅ Applied |
| Type settings | ✅ Applied | ✅ Applied | ✅ Applied |
| Team settings | ✅ Applied | ✅ Applied | ✅ Applied |
| Security policies | ✅ Applied | ✅ Applied | ✅ Applied |
| Setup time | 🐢 Slower | 🚀 Fastest | ⚡ Fast |
| Flexibility | 📦 Structured | 🎨 Maximum | 🎯 High |

## Migration Scenarios

### From Template to Empty

**Scenario**: Organization had template-only policy, now allowing empty repositories.

**Before**:

```bash
# Old: Required template
repo-roller create --org myorg --repo my-repo --template basic
```

**After**:

```bash
# New: Can create empty
repo-roller create --org myorg --repo my-repo --empty
```

**Migration steps**:

1. Update CLI to version supporting `--empty` flag
2. Update API requests to include `"contentStrategy": "empty"`
3. Update documentation and team guides
4. Train teams on new options

### From Manual GitHub to RepoRoller

**Scenario**: Teams manually creating repositories, migrating to RepoRoller.

**Before** (Manual GitHub UI):

1. Click "New Repository"
2. Fill in name and description
3. Choose visibility
4. (Maybe) Initialize with README
5. (Maybe) Add .gitignore
6. Create repository
7. Manually configure settings
8. Manually apply branch protection
9. Manually configure team access

**After** (RepoRoller):

```bash
repo-roller create \
  --org myorg \
  --repo my-repo \
  --init-readme \
  --init-gitignore \
  --repository-type library \
  --team backend \
  --description "My new library"
```

**Benefits**:

- One command vs many manual steps
- Settings applied automatically
- Consistent configuration
- Audit trail
- Faster creation

### From Other Tools to RepoRoller

**Scenario**: Migrating from Terraform or custom scripts.

**Before** (Terraform):

```hcl
resource "github_repository" "my_repo" {
  name        = "my-repo"
  description = "My repository"
  visibility  = "private"

  # Manual configuration
  has_issues      = true
  has_projects    = false
  has_wiki        = false
  # ... many more settings
}
```

**After** (RepoRoller):

```bash
repo-roller create \
  --org myorg \
  --repo my-repo \
  --empty \
  --repository-type library \
  --description "My repository"
```

**Benefits**:

- Organization defaults applied automatically
- Centralized configuration management
- Type-based settings
- Simpler maintenance

## Best Practices

### 1. Choose the Right Strategy

- **Template**: New projects following team standards
- **Empty**: Code migrations, experiments, unique structures
- **Custom Init**: Simple projects needing basic files

### 2. Always Specify Type and Team

Even with empty repositories, specify type and team for proper settings:

```bash
# ✅ Good: Includes type and team
repo-roller create --org myorg --repo my-repo --empty \
  --repository-type library --team backend

# ❌ Less ideal: Missing type/team settings
repo-roller create --org myorg --repo my-repo --empty
```

### 3. Use Descriptive Names and Descriptions

```bash
# ✅ Good: Clear purpose
repo-roller create --org myorg --repo gitlab-migration-projectx --empty \
  --description "Migrated from GitLab gitlab.com/old/projectx"

# ❌ Less clear: Vague purpose
repo-roller create --org myorg --repo empty-repo --empty
```

### 4. Plan Migration Strategy

For existing code migrations:

1. Create empty repository with RepoRoller
2. Clone locally
3. Copy existing code
4. Initial commit and push
5. Configure additional settings if needed

### 5. Document Custom Init Choices

When using custom init, document why in the README:

```markdown
# My Project

This repository was initialized with minimal files to allow
custom project structure development. See ARCHITECTURE.md
for details on the chosen structure.
```

## Troubleshooting

### "Template required when not using --empty"

**Error**:

```
Error: Template is required when content strategy is 'template'
```

**Solution**: Either specify a template or use `--empty`:

```bash
# Option 1: Add template
repo-roller create --org myorg --repo my-repo --template rust-library

# Option 2: Use empty
repo-roller create --org myorg --repo my-repo --empty
```

### "Cannot use template with empty content strategy"

**Error**:

```
Error: Template cannot be specified with empty content strategy
```

**Solution**: Remove either `--template` or `--empty`:

```bash
# Use template (no --empty)
repo-roller create --org myorg --repo my-repo --template rust-library

# OR use empty (no --template)
repo-roller create --org myorg --repo my-repo --empty
```

### "Init flags only valid with custom_init strategy"

**Error** (API):

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "initializeReadme requires contentStrategy 'custom_init'"
  }
}
```

**Solution**: Set `contentStrategy` to `"custom_init"`:

```json
{
  "name": "my-repo",
  "organization": "myorg",
  "contentStrategy": "custom_init",
  "initializeReadme": true
}
```

### Repository Created but Seems Empty

**Issue**: Created repository but can't see any files.

**Expected**: Empty repositories and custom init repositories may have no files or just README/.gitignore. This is correct behavior.

**Verify**: Check repository settings were applied:

1. Go to GitHub repository
2. Check Settings → Branch protection
3. Check Settings → Features (Issues, Wiki, etc.)
4. Settings should match organization/team/type configuration

## See Also

- [Template Commands](./TEMPLATE_COMMANDS.md) - Working with templates
- [API Documentation](../repo_roller_api/README.md) - REST API reference
- [Configuration Guide](../../specs/overview/configuration-hierarchy.md) - Understanding configuration levels
- [Repository Types](../../specs/overview/repository-types.md) - Repository type specifications
