# Test Metadata Repository Setup Guide

This document describes how to create the additional metadata repositories needed for integration testing in the `glitchgrove` organization.

## Overview

The integration tests require multiple metadata repositories to test edge cases:

- `.reporoller-test` - Main valid metadata repository (already exists)
- `.reporoller-test-invalid-global` - Contains malformed global TOML
- `.reporoller-test-missing-global` - Missing global defaults file
- `.reporoller-test-conflicting` - Team configuration conflicts with global
- `.reporoller-test-invalid-team` - Contains malformed team TOML
- `.reporoller-test-incomplete` - Missing required directory structure
- `.reporoller-test-duplicates` - Contains duplicate label definitions

---

## 1. .reporoller-test-invalid-global

**Purpose**: Test error handling for malformed global.toml syntax

**Structure**:

```
.reporoller-test-invalid-global/
├── README.md
├── global/
│   └── defaults.toml          # INVALID TOML SYNTAX
├── teams/
└── types/
```

**global/defaults.toml** (intentionally invalid):

```toml
# Invalid TOML - missing closing bracket
[repository_settings
has_issues = true
has_wiki = false

# Missing quote
description = This is invalid
```

**Setup Commands**:

```powershell
# Create repository
gh repo create glitchgrove/.reporoller-test-invalid-global --public --description "Test metadata: invalid global TOML"

# Clone and setup
git clone https://github.com/glitchgrove/.reporoller-test-invalid-global.git
cd .reporoller-test-invalid-global

# Create structure
New-Item -ItemType Directory -Path global, teams, types

# Create invalid TOML
@"
# Invalid TOML - missing closing bracket
[repository_settings
has_issues = true
has_wiki = false

# Missing quote
description = This is invalid
"@ | Out-File -FilePath global/defaults.toml -Encoding utf8

# Create README
@"
# Test Metadata Repository: Invalid Global TOML

This repository contains intentionally invalid TOML syntax in the global defaults file.
Used for testing error handling in RepoRoller integration tests.
"@ | Out-File -FilePath README.md -Encoding utf8

git add .
git commit -m "Initial setup: invalid global TOML for testing"
git push
```

---

## 2. .reporoller-test-missing-global

**Purpose**: Test handling of missing global defaults file

**Structure**:

```
.reporoller-test-missing-global/
├── README.md
├── global/                    # Directory exists but empty
├── teams/
│   └── backend.toml          # Valid team file
└── types/
    └── library.toml          # Valid type file
```

**teams/backend.toml**:

```toml
[repository_settings]
has_issues = true
has_wiki = false
```

**types/library.toml**:

```toml
[repository_settings]
has_projects = false
```

**Setup Commands**:

```powershell
gh repo create glitchgrove/.reporoller-test-missing-global --public --description "Test metadata: missing global defaults"

git clone https://github.com/glitchgrove/.reporoller-test-missing-global.git
cd .reporoller-test-missing-global

# Create structure with empty global directory
New-Item -ItemType Directory -Path global, teams, types

# Create valid team file
@"
[repository_settings]
has_issues = true
has_wiki = false
"@ | Out-File -FilePath teams/backend.toml -Encoding utf8

# Create valid type file
@"
[repository_settings]
has_projects = false
"@ | Out-File -FilePath types/library.toml -Encoding utf8

# Create README
@"
# Test Metadata Repository: Missing Global Defaults

This repository has the correct structure but is missing the global/defaults.toml file.
Used for testing fallback behavior in RepoRoller integration tests.
"@ | Out-File -FilePath README.md -Encoding utf8

# Create .gitkeep to preserve empty global directory
New-Item -ItemType File -Path global/.gitkeep

git add .
git commit -m "Initial setup: missing global defaults for testing"
git push
```

---

## 3. .reporoller-test-conflicting

**Purpose**: Test handling when team configuration conflicts with fixed global values

**Structure**:

```
.reporoller-test-conflicting/
├── README.md
├── global/
│   └── defaults.toml          # Fixed value: has_wiki = false
├── teams/
│   └── conflicting.toml       # Attempts override: has_wiki = true
└── types/
```

**global/defaults.toml**:

```toml
[repository_settings]
has_wiki = { value = false, override_allowed = false }  # Fixed - cannot override
has_issues = { value = true, override_allowed = true }   # Overridable
```

**teams/conflicting.toml**:

```toml
[repository_settings]
has_wiki = true           # This conflicts with global fixed value
has_issues = false        # This is allowed (override_allowed = true)
```

**Setup Commands**:

```powershell
gh repo create glitchgrove/.reporoller-test-conflicting --public --description "Test metadata: conflicting team configuration"

git clone https://github.com/glitchgrove/.reporoller-test-conflicting.git
cd .reporoller-test-conflicting

New-Item -ItemType Directory -Path global, teams, types

@"
[repository_settings]
has_wiki = { value = false, override_allowed = false }
has_issues = { value = true, override_allowed = true }
"@ | Out-File -FilePath global/defaults.toml -Encoding utf8

@"
[repository_settings]
has_wiki = true
has_issues = false
"@ | Out-File -FilePath teams/conflicting.toml -Encoding utf8

@"
# Test Metadata Repository: Conflicting Configuration

This repository contains a team configuration that attempts to override a fixed global value.
Used for testing conflict detection in RepoRoller integration tests.
"@ | Out-File -FilePath README.md -Encoding utf8

git add .
git commit -m "Initial setup: conflicting team configuration for testing"
git push
```

---

## 4. .reporoller-test-invalid-team

**Purpose**: Test error handling for malformed team TOML

**Structure**:

```
.reporoller-test-invalid-team/
├── README.md
├── global/
│   └── defaults.toml          # Valid
├── teams/
│   ├── backend.toml           # Valid
│   └── invalid-syntax.toml    # INVALID TOML
└── types/
```

**global/defaults.toml** (valid):

```toml
[repository_settings]
has_issues = true
```

**teams/invalid-syntax.toml** (intentionally invalid):

```toml
# Invalid TOML - syntax errors
[repository_settings
has_wiki = "not a boolean
has_projects = [unclosed array
```

**Setup Commands**:

```powershell
gh repo create glitchgrove/.reporoller-test-invalid-team --public --description "Test metadata: invalid team TOML"

git clone https://github.com/glitchgrove/.reporoller-test-invalid-team.git
cd .reporoller-test-invalid-team

New-Item -ItemType Directory -Path global, teams, types

@"
[repository_settings]
has_issues = true
"@ | Out-File -FilePath global/defaults.toml -Encoding utf8

@"
[repository_settings]
has_wiki = false
"@ | Out-File -FilePath teams/backend.toml -Encoding utf8

@"
# Invalid TOML - syntax errors
[repository_settings
has_wiki = "not a boolean
has_projects = [unclosed array
"@ | Out-File -FilePath teams/invalid-syntax.toml -Encoding utf8

@"
# Test Metadata Repository: Invalid Team TOML

This repository contains a team configuration file with invalid TOML syntax.
Used for testing error handling in RepoRoller integration tests.
"@ | Out-File -FilePath README.md -Encoding utf8

git add .
git commit -m "Initial setup: invalid team TOML for testing"
git push
```

---

## 5. .reporoller-test-incomplete

**Purpose**: Test handling of incomplete directory structure

**Structure**:

```
.reporoller-test-incomplete/
├── README.md
└── global/
    └── defaults.toml          # Valid but missing teams/ and types/ directories
```

**global/defaults.toml**:

```toml
[repository_settings]
has_issues = true
```

**Setup Commands**:

```powershell
gh repo create glitchgrove/.reporoller-test-incomplete --public --description "Test metadata: incomplete structure"

git clone https://github.com/glitchgrove/.reporoller-test-incomplete.git
cd .reporoller-test-incomplete

# Only create global directory - missing teams/ and types/
New-Item -ItemType Directory -Path global

@"
[repository_settings]
has_issues = true
"@ | Out-File -FilePath global/defaults.toml -Encoding utf8

@"
# Test Metadata Repository: Incomplete Structure

This repository has global configuration but is missing the teams/ and types/ directories.
Used for testing structure validation in RepoRoller integration tests.
"@ | Out-File -FilePath README.md -Encoding utf8

git add .
git commit -m "Initial setup: incomplete structure for testing"
git push
```

---

## 6. .reporoller-test-duplicates

**Purpose**: Test handling of duplicate label definitions

**Structure**:

```
.reporoller-test-duplicates/
├── README.md
├── global/
│   ├── defaults.toml
│   └── standard-labels.toml   # Contains duplicate labels
├── teams/
└── types/
```

**global/defaults.toml**:

```toml
[repository_settings]
has_issues = true
```

**global/standard-labels.toml** (with duplicates):

```toml
# Duplicate label definitions - same name, different colors
[[labels]]
name = "bug"
color = "FF0000"
description = "Something isn't working"

[[labels]]
name = "enhancement"
color = "00FF00"
description = "New feature or request"

[[labels]]
name = "bug"              # DUPLICATE - different color
color = "AA0000"
description = "Bug (duplicate definition)"

[[labels]]
name = "documentation"
color = "0000FF"
description = "Documentation improvements"
```

**Setup Commands**:

```powershell
gh repo create glitchgrove/.reporoller-test-duplicates --public --description "Test metadata: duplicate labels"

git clone https://github.com/glitchgrove/.reporoller-test-duplicates.git
cd .reporoller-test-duplicates

New-Item -ItemType Directory -Path global, teams, types

@"
[repository_settings]
has_issues = true
"@ | Out-File -FilePath global/defaults.toml -Encoding utf8

@"
[[labels]]
name = "bug"
color = "FF0000"
description = "Something isn't working"

[[labels]]
name = "enhancement"
color = "00FF00"
description = "New feature or request"

[[labels]]
name = "bug"
color = "AA0000"
description = "Bug (duplicate definition)"

[[labels]]
name = "documentation"
color = "0000FF"
description = "Documentation improvements"
"@ | Out-File -FilePath global/standard-labels.toml -Encoding utf8

@"
# Test Metadata Repository: Duplicate Labels

This repository contains duplicate label definitions with different configurations.
Used for testing duplicate handling in RepoRoller integration tests.
"@ | Out-File -FilePath README.md -Encoding utf8

git add .
git commit -m "Initial setup: duplicate labels for testing"
git push
```

---

## Quick Setup Script

Run all repository setups in sequence:

```powershell
# Save this as setup-test-metadata-repos.ps1

$repos = @(
    ".reporoller-test-invalid-global",
    ".reporoller-test-missing-global",
    ".reporoller-test-conflicting",
    ".reporoller-test-invalid-team",
    ".reporoller-test-incomplete",
    ".reporoller-test-duplicates"
)

foreach ($repo in $repos) {
    Write-Host "Setting up $repo..." -ForegroundColor Cyan
    # Follow the individual setup commands above for each repository
}

Write-Host "All test metadata repositories created!" -ForegroundColor Green
```

---

## Verification

After creating all repositories, verify they exist:

```powershell
gh repo list glitchgrove --limit 50 | Select-String "reporoller-test"
```

Expected output should show all 7 metadata repositories:

- `.reporoller-test` (existing)
- `.reporoller-test-invalid-global`
- `.reporoller-test-missing-global`
- `.reporoller-test-conflicting`
- `.reporoller-test-invalid-team`
- `.reporoller-test-incomplete`
- `.reporoller-test-duplicates`
