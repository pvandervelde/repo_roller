#!/usr/bin/env pwsh
#
# Update Test Metadata Repositories with Visibility Policy Configuration
#
# This script creates/updates metadata repositories in glitchgrove with visibility
# policy configurations for testing the visibility feature.
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
$metadataSourceDir = Join-Path $PSScriptRoot "metadata"

function Update-MetadataRepo
{
    param(
        [string]$RepoName,
        [string]$SourceDir,
        [string]$Description
    )

    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "Updating $RepoName" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    # Check if repo exists
    $existing = gh repo view "$Organization/$RepoName" 2>$null
    if ($LASTEXITCODE -ne 0)
    {
        Write-Host "Creating repository..." -ForegroundColor Yellow
        gh repo create "$Organization/$RepoName" --public --description $Description
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to create repository $RepoName"
        }
    }
    else
    {
        Write-Host "Repository exists" -ForegroundColor Gray
    }

    # Clone or update
    $tempDir = Join-Path $env:TEMP (New-Guid).ToString()
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try
    {
        Push-Location $tempDir

        Write-Host "Cloning repository..." -ForegroundColor Gray
        git clone "https://github.com/$Organization/$RepoName.git" 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to clone repository"
        }

        Set-Location $RepoName

        # Copy files from source
        Write-Host "Copying configuration files..." -ForegroundColor Gray
        if (Test-Path $SourceDir)
        {
            Copy-Item -Path "$SourceDir\*" -Destination "." -Recurse -Force
        }
        else
        {
            throw "Source directory not found: $SourceDir"
        }

        # Create README if it doesn't exist
        if (-not (Test-Path "README.md"))
        {
            @"
# $RepoName

Test metadata repository for RepoRoller visibility policy testing.

## Purpose

$Description

## Configuration

See `global/defaults.toml` for visibility policy configuration.
"@ | Out-File -FilePath "README.md" -Encoding utf8
        }

        # Check for changes
        git add -A
        $status = git status --porcelain

        if ($status)
        {
            Write-Host "Committing changes..." -ForegroundColor Gray
            git commit -m "Update visibility policy configuration for testing" 2>&1 | Out-Null

            Write-Host "Pushing to GitHub..." -ForegroundColor Gray
            git push 2>&1 | Out-Null

            Write-Host "✓ Successfully updated $RepoName" -ForegroundColor Green
        }
        else
        {
            Write-Host "✓ No changes needed for $RepoName" -ForegroundColor Green
        }
    }
    finally
    {
        Pop-Location
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Update metadata repositories
Write-Host "Updating metadata repositories with visibility policies..." -ForegroundColor Cyan

Update-MetadataRepo `
    -RepoName ".reporoller" `
    -SourceDir (Join-Path $metadataSourceDir ".reporoller") `
    -Description "Test metadata: unrestricted visibility policy"

Update-MetadataRepo `
    -RepoName ".reporoller-restricted" `
    -SourceDir (Join-Path $metadataSourceDir ".reporoller-restricted") `
    -Description "Test metadata: restricted visibility policy (prohibits public)"

Update-MetadataRepo `
    -RepoName ".reporoller-required" `
    -SourceDir (Join-Path $metadataSourceDir ".reporoller-required") `
    -Description "Test metadata: required visibility policy (requires private)"

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "All metadata repositories updated!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
