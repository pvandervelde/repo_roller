#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Creates GitHub test template repositories for RepoRoller integration tests.

.DESCRIPTION
    This script creates the four test template repositories required for RepoRoller integration tests:
    - test-basic: Basic repository creation testing
    - test-variables: Variable substitution testing
    - test-filtering: File filtering testing
    - test-invalid: Error handling testing

.PARAMETER Organization
    The GitHub organization to create repositories in. Defaults to 'pvandervelde'.

.PARAMETER Force
    Force recreation of repositories if they already exist.

.EXAMPLE
    ./scripts/create-test-repositories.ps1

.EXAMPLE
    ./scripts/create-test-repositories.ps1 -Organization "myorg" -Force
#>

param(
    [string]$Organization = "pvandervelde",
    [switch]$Force
)

# Set error action preference
$ErrorActionPreference = "Stop"

# Template repositories to create
$Templates = @(
    @{
        Name        = "test-basic"
        Description = "Basic repository template for RepoRoller integration tests"
        Path        = "tests/templates/test-basic"
    },
    @{
        Name        = "test-variables"
        Description = "Variable substitution template for RepoRoller integration tests"
        Path        = "tests/templates/test-variables"
    },
    @{
        Name        = "test-filtering"
        Description = "File filtering template for RepoRoller integration tests"
        Path        = "tests/templates/test-filtering"
    },
    @{
        Name        = "test-invalid"
        Description = "Error handling template for RepoRoller integration tests"
        Path        = "tests/templates/test-invalid"
    }
)

function Test-GitHubCLI
{
    try
    {
        $null = gh --version
        Write-Host "✓ GitHub CLI is available" -ForegroundColor Green
    }
    catch
    {
        Write-Error "GitHub CLI (gh) is not installed or not in PATH. Please install it from https://cli.github.com/"
        exit 1
    }
}

function Test-GitHubAuth
{
    try
    {
        $null = gh auth status
        Write-Host "✓ GitHub CLI is authenticated" -ForegroundColor Green
    }
    catch
    {
        Write-Error "GitHub CLI is not authenticated. Please run 'gh auth login' first."
        exit 1
    }
}

function Test-TemplateDirectory
{
    param([string]$Path)

    if (-not (Test-Path $Path))
    {
        Write-Error "Template directory not found: $Path"
        return $false
    }

    $files = Get-ChildItem -Path $Path -Recurse -File
    if ($files.Count -eq 0)
    {
        Write-Error "Template directory is empty: $Path"
        return $false
    }

    Write-Host "✓ Template directory validated: $Path ($($files.Count) files)" -ForegroundColor Green
    return $true
}

function Test-RepositoryExists
{
    param([string]$Organization, [string]$Name)

    try
    {
        $null = gh repo view "$Organization/$Name" 2>$null
        return $true
    }
    catch
    {
        return $false
    }
}

function Remove-Repository
{
    param([string]$Organization, [string]$Name)

    Write-Host "Removing existing repository: $Organization/$Name" -ForegroundColor Yellow
    try
    {
        gh repo delete "$Organization/$Name" --yes
        Write-Host "✓ Repository removed: $Organization/$Name" -ForegroundColor Green
    }
    catch
    {
        Write-Error "Failed to remove repository: $Organization/$Name. Error: $_"
    }
}

function New-Repository
{
    param([string]$Organization, [string]$Name, [string]$Description, [string]$TemplatePath)

    Write-Host "Creating repository: $Organization/$Name" -ForegroundColor Cyan

    # Create temporary directory for git operations
    $tempDir = Join-Path $env:TEMP "repo-roller-$Name-$(Get-Random)"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try
    {
        # Copy template files to temp directory
        Copy-Item -Path "$TemplatePath/*" -Destination $tempDir -Recurse -Force

        # Initialize git repository
        Push-Location $tempDir
        git init
        git add .
        git commit -m "Initial commit: $Description"

        # Create GitHub repository
        gh repo create "$Organization/$Name" --public --description $Description --template

        # Push to GitHub
        git remote add origin "https://github.com/$Organization/$Name.git"
        git branch -M main
        git push -u origin main

        Write-Host "✓ Repository created: $Organization/$Name" -ForegroundColor Green
    }
    catch
    {
        Write-Error "Failed to create repository: $Organization/$Name. Error: $_"
    }
    finally
    {
        Pop-Location
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Main execution
Write-Host "RepoRoller Test Repository Creator" -ForegroundColor Magenta
Write-Host "=================================" -ForegroundColor Magenta

# Validate prerequisites
Test-GitHubCLI
Test-GitHubAuth

# Validate template directories
$allValid = $true
foreach ($template in $Templates)
{
    if (-not (Test-TemplateDirectory $template.Path))
    {
        $allValid = $false
    }
}

if (-not $allValid)
{
    Write-Error "One or more template directories are invalid. Aborting."
    exit 1
}

# Process each template
foreach ($template in $Templates)
{
    $repoName = $template.Name
    $repoExists = Test-RepositoryExists $Organization $repoName

    if ($repoExists)
    {
        if ($Force)
        {
            Remove-Repository $Organization $repoName
            Start-Sleep -Seconds 2  # Give GitHub time to process deletion
        }
        else
        {
            Write-Host "Repository already exists: $Organization/$repoName (use -Force to recreate)" -ForegroundColor Yellow
            continue
        }
    }

    New-Repository $Organization $repoName $template.Description $template.Path
}

Write-Host ""
Write-Host "✓ Test repository creation completed!" -ForegroundColor Green
Write-Host "The following repositories are now available:" -ForegroundColor Cyan
foreach ($template in $Templates)
{
    Write-Host "  - https://github.com/$Organization/$($template.Name)" -ForegroundColor White
}
