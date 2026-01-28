#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Creates GitHub test metadata repository for RepoRoller integration tests.

.DESCRIPTION
    This script creates a test metadata repository in the specified organization
    with sample configuration files for testing the organization settings system:
    - Global defaults configuration
    - Team-specific configurations
    - Repository type configurations
    - Standard labels

.PARAMETER Organization
    The GitHub organization to create repository in. Defaults to 'glitchgrove'.

.PARAMETER RepositoryName
    Name of the metadata repository. Defaults to '.reporoller-test'.

.PARAMETER Force
    Force recreation of repository if it already exists.

.PARAMETER Verbose
    Enable verbose logging for debugging.

.EXAMPLE
    ./tests/create-test-metadata-repository.ps1

.EXAMPLE
    ./tests/create-test-metadata-repository.ps1 -Organization "glitchgrove" -Force

.EXAMPLE
    ./tests/create-test-metadata-repository.ps1 -Organization "glitchgrove" -Update

.EXAMPLE
    ./tests/create-test-metadata-repository.ps1 -RepositoryName ".reporoller-test" -Verbose
#>

param(
    [string]$Organization = "glitchgrove",
    [string]$RepositoryName = ".reporoller-test",
    [switch]$Force,
    [switch]$Update,
    [switch]$Verbose
)

# Set error action preference
$ErrorActionPreference = "Stop"

# Configuration structure
$ConfigStructure = @{
    "global/defaults.toml"        = @"
# Global default repository settings for test organization
# These settings apply to all repositories unless overridden

[repository]
issues = { value = true, override_allowed = true }
projects = { value = false, override_allowed = true }
discussions = { value = true, override_allowed = true }
wiki = { value = false, override_allowed = true }
security_advisories = { value = true, override_allowed = false }
vulnerability_reporting = { value = true, override_allowed = false }

[pull_requests]
allow_merge_commit = { value = false, override_allowed = true }
allow_squash_merge = { value = true, override_allowed = true }
allow_rebase_merge = { value = true, override_allowed = true }
delete_branch_on_merge = { value = true, override_allowed = true }
require_conversation_resolution = { value = true, override_allowed = false }

[branch_protection]
default_branch = { value = "main", override_allowed = true }
require_pull_request_reviews = { value = true, override_allowed = false }
required_approving_review_count = { value = 1, override_allowed = true }

[actions]
enabled = { value = true, override_allowed = true }
"@

    "global/standard-labels.toml" = @"
# Standard labels available to all repositories

[bug]
color = "d73a4a"
description = "Something isn't working"

[enhancement]
color = "a2eeef"
description = "New feature or request"

[documentation]
color = "0075ca"
description = "Improvements or additions to documentation"

[good-first-issue]
color = "7057ff"
description = "Good for newcomers"

[help-wanted]
color = "008672"
description = "Extra attention is needed"

[question]
color = "d876e3"
description = "Further information is requested"

[wontfix]
color = "ffffff"
description = "This will not be worked on"

[duplicate]
color = "cfd3d7"
description = "This issue or pull request already exists"

[invalid]
color = "e4e669"
description = "This doesn't seem right"

[dependencies]
color = "0366d6"
description = "Pull requests that update a dependency file"
"@

    "global/webhooks.toml"        = @"
# Global webhook configurations for all repositories

[[webhook]]
url = "https://httpbin.org/global-webhook"
events = ["push", "pull_request"]
active = true
content_type = "json"
insecure_ssl = false
"@

    "teams/platform/config.toml"  = @"
# Platform team configuration overrides

[repository]
discussions = false  # Platform team doesn't use discussions
wiki = true         # Platform team uses wiki for documentation

[pull_requests]
required_approving_review_count = 2  # Platform requires 2 reviewers

# Platform team custom properties
[[custom_properties]]
name = "team"
value = "platform"

[[custom_properties]]
name = "criticality"
value = "high"
"@

    "teams/platform/labels.toml"  = @"
# Platform team-specific labels

[platform-infrastructure]
color = "1d76db"
description = "Platform infrastructure changes"

[security-critical]
color = "b60205"
description = "Security-critical changes requiring extra review"

[performance]
color = "0e8a16"
description = "Performance improvements"
"@

    "teams/backend/config.toml"   = @"
# Backend team configuration overrides

[repository]
projects = true  # Backend team uses projects for tracking

[pull_requests]
allow_auto_merge = true  # Backend team allows auto-merge

[[custom_properties]]
name = "team"
value = "backend"

[[custom_properties]]
name = "service_tier"
value = "tier-2"
"@

    "teams/backend/labels.toml"   = @"
# Backend team-specific labels

[api-breaking]
color = "b60205"
description = "Breaking API changes"

[database]
color = "1d76db"
description = "Database-related changes"

[microservice]
color = "0e8a16"
description = "Microservice changes"
"@

    "types/library/config.toml"   = @"
# Configuration for library-type repositories

[repository]
wiki = false  # Libraries typically don't need wikis
discussions = true  # Libraries use discussions for community

[pull_requests]
require_code_owner_reviews = true  # Libraries require code owner review

[[custom_properties]]
name = "repo_type"
value = "library"

[[custom_properties]]
name = "visibility"
value = "public"
"@

    "types/service/config.toml"   = @"
# Configuration for service-type repositories

[repository]
wiki = true  # Services need wikis for operations documentation
discussions = false  # Services use issues instead of discussions

[pull_requests]
required_approving_review_count = 2  # Services require 2 reviewers

[[custom_properties]]
name = "repo_type"
value = "service"

[[custom_properties]]
name = "deployment_type"
value = "kubernetes"

[[environments]]
name = "development"
wait_timer = 0

[[environments]]
name = "staging"
wait_timer = 300

[[environments]]
name = "production"
wait_timer = 600
"@

    "README.md"                   = @"

# RepoRoller Test Metadata Repository

This repository contains test configuration data for RepoRoller integration tests.

## Structure

- 'global/' - Organization-wide defaults and standard labels
- 'teams/' - Team-specific configuration overrides
- 'types/' - Repository type-specific configurations

## Teams

### Platform Team
- Higher approval requirements (2 reviewers)
- Custom labels for infrastructure work
- Wiki enabled for documentation

### Backend Team
- Auto-merge enabled
- Custom labels for API and database work
- Projects enabled for tracking

## Repository Types

### Library
- Public visibility
- Discussions enabled
- Code owner reviews required

### Service
- Kubernetes deployment
- Multiple environments (dev, staging, production)
- Higher approval requirements

## Usage

This is a test repository for RepoRoller integration tests. It is automatically
created and managed by the test setup scripts.

Do not use this repository for production configuration.
"@
}

function Write-Log
{
    param(
        [string]$Message,
        [string]$Level = "Info"
    )

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $color = switch ($Level)
    {
        "Success"
        {
            "Green"
        }
        "Warning"
        {
            "Yellow"
        }
        "Error"
        {
            "Red"
        }
        default
        {
            "White"
        }
    }

    Write-Host "[$timestamp] $Message" -ForegroundColor $color
}

function Test-GitHubCLI
{
    try
    {
        $null = gh --version 2>&1
        Write-Log "✓ GitHub CLI is available" -Level "Success"
        return $true
    }
    catch
    {
        Write-Log "✗ GitHub CLI (gh) is not installed or not in PATH" -Level "Error"
        Write-Log "  Please install it from https://cli.github.com/" -Level "Error"
        return $false
    }
}

function Test-GitHubAuth
{
    try
    {
        $null = gh auth status 2>&1
        Write-Log "✓ GitHub CLI is authenticated" -Level "Success"
        return $true
    }
    catch
    {
        Write-Log "✗ GitHub CLI is not authenticated" -Level "Error"
        Write-Log "  Please run 'gh auth login' first" -Level "Error"
        return $false
    }
}

function Test-RepositoryExists
{
    param(
        [string]$Owner,
        [string]$Repo
    )

    $result = gh repo view "$Owner/$Repo" 2>&1
    if ($LASTEXITCODE -eq 0)
    {
        return $true
    }
    else
    {
        return $false
    }
}

function Remove-Repository
{
    param(
        [string]$Owner,
        [string]$Repo
    )

    Write-Log "Deleting existing repository $Owner/$Repo" -Level "Warning"

    $output = gh repo delete "$Owner/$Repo" --yes 2>&1
    if ($LASTEXITCODE -ne 0)
    {
        Write-Log "✗ Failed to delete repository: $output" -Level "Error"
        return $false
    }

    Write-Log "✓ Repository deleted" -Level "Success"
    Start-Sleep -Seconds 3  # Give GitHub API time to process deletion
    return $true
}

function New-MetadataRepository
{
    param(
        [string]$Owner,
        [string]$Repo
    )

    Write-Log "Creating repository $Owner/$Repo"

    try
    {
        # Create repository
        gh repo create "$Owner/$Repo" `
            --private `
            --description "Test metadata repository for RepoRoller integration tests" `
            2>&1 | Out-Null

        Write-Log "✓ Repository created" -Level "Success"
        return $true
    }
    catch
    {
        Write-Log "✗ Failed to create repository: $_" -Level "Error"
        return $false
    }
}

function Initialize-LocalRepository
{
    param(
        [string]$Path
    )

    Write-Log "Initializing local repository at $Path"

    try
    {
        # Create directory (remove if exists)
        if (Test-Path $Path)
        {
            Write-Log "Cleaning up existing temporary directory" -Level "Warning"
            # Try to remove, but don't fail if it's locked
            try
            {
                Remove-Item -Path $Path -Recurse -Force -ErrorAction Stop
                Start-Sleep -Milliseconds 500  # Give filesystem time to release
            }
            catch
            {
                Write-Log "Could not remove existing directory, using alternate path" -Level "Warning"
                # Generate a new unique path
                $timestamp = Get-Date -Format "yyyyMMddHHmmss"
                $Path = Join-Path $env:TEMP "reporoller-test-metadata-$timestamp"
            }
        }

        New-Item -ItemType Directory -Path $Path -Force | Out-Null

        # Initialize git
        Push-Location $Path
        try
        {
            git init 2>&1 | Out-Null
            git config user.name "RepoRoller Test" 2>&1 | Out-Null
            git config user.email "test@reporoller.test" 2>&1 | Out-Null
            Write-Log "✓ Git repository initialized" -Level "Success"
            return $true
        }
        finally
        {
            Pop-Location
        }
    }
    catch
    {
        Write-Log "✗ Failed to initialize repository: $_" -Level "Error"
        return $false
    }
}

function Add-ConfigurationFiles
{
    param(
        [string]$BasePath,
        [hashtable]$Structure
    )

    Write-Log "Creating configuration files"

    try
    {
        foreach ($file in $Structure.Keys)
        {
            $filePath = Join-Path $BasePath $file
            $directory = Split-Path $filePath -Parent

            # Create directory if it doesn't exist
            if (-not (Test-Path $directory))
            {
                New-Item -ItemType Directory -Path $directory -Force | Out-Null
            }

            # Write file content
            $Structure[$file] | Out-File -FilePath $filePath -Encoding UTF8 -NoNewline

            if ($Verbose)
            {
                Write-Log "  Created $file" -Level "Info"
            }
        }

        Write-Log "✓ Configuration files created ($($Structure.Keys.Count) files)" -Level "Success"
        return $true
    }
    catch
    {
        Write-Log "✗ Failed to create configuration files: $_" -Level "Error"
        return $false
    }
}

function Publish-Repository
{
    param(
        [string]$Path,
        [string]$Owner,
        [string]$Repo,
        [bool]$IsUpdate = $false
    )

    Write-Log "Publishing repository to GitHub"

    try
    {
        Push-Location $Path
        try
        {
            # Stage all files
            git add . 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0)
            {
                throw "Failed to stage files"
            }

            # Check if there are changes to commit
            $status = git status --porcelain 2>&1
            if (-not $status)
            {
                Write-Log "No changes to commit" -Level "Info"
                return $true
            }

            # Commit
            $commitMsg = if ($IsUpdate)
            {
                "Update test configuration files" 
            }
            else
            {
                "Initial commit: Add test configuration files" 
            }
            git commit -m $commitMsg 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0)
            {
                throw "Failed to commit changes"
            }

            # Add remote if it doesn't exist
            $remoteUrl = "https://github.com/$Owner/$Repo.git"
            $existingRemote = git remote get-url origin 2>$null
            if ($LASTEXITCODE -ne 0)
            {
                git remote add origin $remoteUrl 2>&1 | Out-Null
                if ($LASTEXITCODE -ne 0)
                {
                    throw "Failed to add remote"
                }
            }

            # Set default branch to main
            git branch -M main 2>&1 | Out-Null

            # Push (force if updating to ensure we overwrite)
            if ($IsUpdate)
            {
                git push -f origin main 2>&1 | Out-Null
            }
            else
            {
                git push -u origin main 2>&1 | Out-Null
            }

            if ($LASTEXITCODE -ne 0)
            {
                throw "Failed to push to GitHub"
            }

            Write-Log "✓ Repository published to GitHub" -Level "Success"
            return $true
        }
        finally
        {
            Pop-Location
        }
    }
    catch
    {
        Write-Log "✗ Failed to publish repository: $_" -Level "Error"
        return $false
    }
}

# Main execution
Write-Log "=== RepoRoller Test Metadata Repository Setup ===" -Level "Info"
Write-Log ""

# Check prerequisites
if (-not (Test-GitHubCLI))
{
    exit 1
}

if (-not (Test-GitHubAuth))
{
    exit 1
}

# Check if repository exists
$repoExists = Test-RepositoryExists -Owner $Organization -Repo $RepositoryName

if ($repoExists)
{
    if ($Force)
    {
        Write-Log "Repository exists and -Force specified" -Level "Warning"
        Write-Log "Note: Deletion requires 'delete_repo' scope. Run: gh auth refresh -h github.com -s delete_repo" -Level "Info"
        if (-not (Remove-Repository -Owner $Organization -Repo $RepositoryName))
        {
            Write-Log "Failed to delete repository. Try using -Update instead to update existing repository." -Level "Error"
            exit 1
        }
    }
    elseif ($Update)
    {
        Write-Log "Repository exists and -Update specified - will update content" -Level "Info"
        # Continue to update existing repository
    }
    else
    {
        Write-Log "Repository $Organization/$RepositoryName already exists" -Level "Warning"
        Write-Log "Use -Force to recreate it (requires delete_repo scope) or -Update to update content" -Level "Info"
        exit 0
    }
}

# Create repository if it doesn't exist (or was deleted)
if (-not $repoExists -or $Force)
{
    if (-not (New-MetadataRepository -Owner $Organization -Repo $RepositoryName))
    {
        exit 1
    }
}

# Initialize local repository with unique path
$timestamp = Get-Date -Format "yyyyMMddHHmmss"
$tempPath = Join-Path $env:TEMP "reporoller-test-metadata-$timestamp"
if (-not (Initialize-LocalRepository -Path $tempPath))
{
    exit 1
}

# Add configuration files
if (-not (Add-ConfigurationFiles -BasePath $tempPath -Structure $ConfigStructure))
{
    exit 1
}

# Publish to GitHub
if (-not (Publish-Repository -Path $tempPath -Owner $Organization -Repo $RepositoryName -IsUpdate $Update))
{
    exit 1
}

# Cleanup (wait a bit for git to release file handles)
Write-Log "Cleaning up temporary directory" -Level "Info"
Start-Sleep -Seconds 2
$retryCount = 0
$maxRetries = 3
$cleaned = $false

while (-not $cleaned -and $retryCount -lt $maxRetries)
{
    try
    {
        if (Test-Path $tempPath)
        {
            Remove-Item -Path $tempPath -Recurse -Force -ErrorAction Stop
            $cleaned = $true
            Write-Log "✓ Temporary directory cleaned up" -Level "Success"
        }
        else
        {
            $cleaned = $true
        }
    }
    catch
    {
        $retryCount++
        if ($retryCount -lt $maxRetries)
        {
            Write-Log "Cleanup attempt $retryCount failed, retrying..." -Level "Warning"
            Start-Sleep -Seconds 2
        }
        else
        {
            Write-Log "Note: Temporary directory cleanup will be handled by system" -Level "Warning"
        }
    }
}

Write-Log ""
Write-Log "=== Setup Complete ===" -Level "Success"
Write-Log "Metadata repository created: https://github.com/$Organization/$RepositoryName" -Level "Success"
Write-Log ""
Write-Log "Repository structure:" -Level "Info"
Write-Log "  - global/defaults.toml: Organization-wide defaults" -Level "Info"
Write-Log "  - global/standard-labels.toml: Standard labels" -Level "Info"
Write-Log "  - 'teams/platform/': Platform team configuration" -Level "Info"
Write-Log "  - 'teams/backend/': Backend team configuration" -Level "Info"
Write-Log "  - 'types/library/': Library repository type configuration" -Level "Info"
Write-Log "  - 'types/service/': Service repository type configuration" -Level "Info"
