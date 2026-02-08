#!/usr/bin/env pwsh
#
# Setup Test Metadata Repositories
#
# This script creates the test metadata repositories needed for RepoRoller integration tests
# in the glitchgrove organization.
#
# Prerequisites:
# - GitHub CLI (gh) installed and authenticated
# - Write access to glitchgrove organization

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$Organization = "glitchgrove",

    [Parameter(Mandatory = $false)]
    [switch]$Force
)

$ErrorActionPreference = "Stop"

function New-TestMetadataRepo
{
    param(
        [string]$RepoName,
        [string]$Description,
        [scriptblock]$SetupScript
    )

    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "Updating $RepoName" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    # Check if repo exists
    $existing = gh repo view "$Organization/$RepoName" 2>$null
    if ($LASTEXITCODE -ne 0)
    {
        # Repository doesn't exist, create it
        Write-Host "Creating GitHub repository..." -ForegroundColor Yellow
        gh repo create "$Organization/$RepoName" --public --description $Description
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to create repository $RepoName"
        }
    }
    else
    {
        Write-Host "Repository exists" -ForegroundColor Gray
        if (-not $Force)
        {
            Write-Host "✓ Skipping (use -Force to update existing repositories)" -ForegroundColor Yellow
            return
        }
        Write-Host "Updating existing repository..." -ForegroundColor Yellow
    }

    # Clone repository
    $tempDir = Join-Path $env:TEMP ([System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try
    {
        Write-Host "Cloning repository..." -ForegroundColor Gray
        Push-Location $tempDir
        git clone "https://github.com/$Organization/$RepoName.git" 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to clone repository"
        }

        Set-Location $RepoName

        # Run setup script
        Write-Host "Setting up repository contents..." -ForegroundColor Gray
        & $SetupScript

        # Check if there are changes to commit
        git add .
        $status = git status --porcelain
        if (-not $status)
        {
            Write-Host "✓ No changes needed" -ForegroundColor Green
        }
        else
        {
            # Commit and push
            Write-Host "Committing changes..." -ForegroundColor Gray
            git commit -m "Update repository configuration for integration testing" 2>&1 | Out-Null

            Write-Host "Pushing to GitHub..." -ForegroundColor Gray
            git push 2>&1 | Out-Null

            Write-Host "✓ Successfully updated $RepoName" -ForegroundColor Green
        }
    }
    finally
    {
        Pop-Location
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Repository 1: Invalid Global TOML
New-TestMetadataRepo -RepoName ".reporoller-test-invalid-global" -Description "Test metadata: invalid global TOML" -SetupScript {
    New-Item -ItemType Directory -Path global, teams, types -Force | Out-Null

    @"
# Invalid TOML - missing closing bracket
[repository_settings
has_issues = true
has_wiki = false

# Missing quote
description = This is invalid
"@ | Out-File -FilePath global/defaults.toml -Encoding utf8

    @"
# Test Metadata Repository: Invalid Global TOML

This repository contains intentionally invalid TOML syntax in the global defaults file.
Used for testing error handling in RepoRoller integration tests.

## Structure

- ``global/defaults.toml`` - Contains invalid TOML syntax
- ``teams/`` - Empty directory
- ``types/`` - Empty directory

## Purpose

Tests that RepoRoller properly handles TOML parsing errors and provides clear error messages.
"@ | Out-File -FilePath README.md -Encoding utf8
}

# Repository 2: Missing Global TOML
New-TestMetadataRepo -RepoName ".reporoller-test-missing-global" -Description "Test metadata: missing global defaults" -SetupScript {
    New-Item -ItemType Directory -Path global, teams, types -Force | Out-Null

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

    # Create .gitkeep to preserve empty global directory
    New-Item -ItemType File -Path global/.gitkeep -Force | Out-Null

    @"
# Test Metadata Repository: Missing Global Defaults

This repository has the correct structure but is missing the ``global/defaults.toml`` file.
Used for testing fallback behavior in RepoRoller integration tests.

## Structure

- ``global/`` - Empty directory (no defaults.toml)
- ``teams/backend.toml`` - Valid team configuration
- ``types/library.toml`` - Valid repository type configuration

## Purpose

Tests that RepoRoller gracefully handles missing global defaults file and documents
whether it uses fallback configuration or reports an error.
"@ | Out-File -FilePath README.md -Encoding utf8
}

# Repository 3: Conflicting Configuration
New-TestMetadataRepo -RepoName ".reporoller-test-conflicting" -Description "Test metadata: conflicting team configuration" -SetupScript {
    New-Item -ItemType Directory -Path global, teams, types -Force | Out-Null

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

## Structure

- ``global/defaults.toml`` - Sets ``has_wiki = false`` with ``override_allowed = false``
- ``teams/conflicting.toml`` - Attempts to set ``has_wiki = true``

## Purpose

Tests that RepoRoller detects and reports conflicts when team configuration
tries to override a fixed global value.
"@ | Out-File -FilePath README.md -Encoding utf8
}

# Repository 4: Invalid Team TOML
New-TestMetadataRepo -RepoName ".reporoller-test-invalid-team" -Description "Test metadata: invalid team TOML" -SetupScript {
    New-Item -ItemType Directory -Path global, teams, types -Force | Out-Null

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

## Structure

- ``global/defaults.toml`` - Valid global configuration
- ``teams/backend.toml`` - Valid team configuration
- ``teams/invalid-syntax.toml`` - Invalid TOML syntax

## Purpose

Tests that RepoRoller properly handles TOML parsing errors in team configuration
files and provides clear error messages indicating which file is problematic.
"@ | Out-File -FilePath README.md -Encoding utf8
}

# Repository 5: Incomplete Structure
New-TestMetadataRepo -RepoName ".reporoller-test-incomplete" -Description "Test metadata: incomplete structure" -SetupScript {
    New-Item -ItemType Directory -Path global -Force | Out-Null

    @"
[repository_settings]
has_issues = true
"@ | Out-File -FilePath global/defaults.toml -Encoding utf8

    @"
# Test Metadata Repository: Incomplete Structure

This repository has global configuration but is missing the ``teams/`` and ``types/`` directories.
Used for testing structure validation in RepoRoller integration tests.

## Structure

- ``global/defaults.toml`` - Valid global configuration
- ``teams/`` - **MISSING**
- ``types/`` - **MISSING**

## Purpose

Tests that RepoRoller gracefully handles incomplete metadata repository structure
and documents whether it tolerates missing directories or reports an error.
"@ | Out-File -FilePath README.md -Encoding utf8
}

# Repository 6: Duplicate Labels
New-TestMetadataRepo -RepoName ".reporoller-test-duplicates" -Description "Test metadata: duplicate labels" -SetupScript {
    New-Item -ItemType Directory -Path global, teams, types -Force | Out-Null

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

## Structure

- ``global/defaults.toml`` - Valid global configuration
- ``global/standard-labels.toml`` - Contains duplicate "bug" label with different colors

## Purpose

Tests how RepoRoller handles duplicate label definitions:
- Does it use the first definition?
- Does it use the last definition?
- Does it report an error?
"@ | Out-File -FilePath README.md -Encoding utf8
}

# Repository 7: Main Test Metadata Repository
New-TestMetadataRepo -RepoName ".reporoller-test" -Description "Primary test metadata repository for RepoRoller integration tests" -SetupScript {
    $metadataSource = "F:\vcs\github\pvandervelde\repo_roller\tests\metadata\.reporoller"

    if (Test-Path $metadataSource)
    {
        # Copy the entire .reporoller directory structure
        Copy-Item -Path "$metadataSource\*" -Destination "." -Recurse -Force
    }
    else
    {
        # Fallback: create basic structure
        New-Item -ItemType Directory -Path global, teams, types -Force | Out-Null

        @"
[repository_settings]
has_issues = true
has_wiki = false
has_projects = false
"@ | Out-File -FilePath global/defaults.toml -Encoding utf8
    }

    @"
# RepoRoller Test Metadata Repository

This is the primary test metadata repository for RepoRoller integration tests.
It contains the standard configuration hierarchy used by most integration tests.

## Structure

- ``global/`` - Global configuration (defaults, labels, webhooks, rulesets)
- ``teams/`` - Team-specific configurations
- ``types/`` - Repository type configurations
- ``templates/`` - Template-specific configurations

## Purpose

This repository serves as the primary configuration source for RepoRoller integration
and E2E tests. It contains realistic configuration examples that demonstrate:

- Configuration hierarchy and merging
- Label definitions
- Webhook configurations
- Repository ruleset policies
- Repository settings with override controls

## Usage

Integration tests reference this repository using the ``GitHubClient`` to retrieve
merged configuration for test repositories. E2E tests verify that repositories
created through the API correctly apply the configuration from this metadata repository.
"@ | Out-File -FilePath README.md -Encoding utf8
}

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "✓ All test metadata repositories created!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green

Write-Host "`nVerifying repositories..." -ForegroundColor Cyan
gh repo list $Organization --limit 50 | Select-String "reporoller-test"

Write-Host "`nSetup complete! You can now run the integration tests." -ForegroundColor Green
