#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Cleans up orphaned test repositories in the glitchgrove organization.

.DESCRIPTION
    This script uses GitHub CLI to delete test repositories older than the specified age.
    It's a standalone cleanup utility that doesn't require building the Rust code.

.PARAMETER MaxAgeHours
    Maximum age in hours for repositories to keep. Repositories older than this will be deleted.
    Default: 24 hours

.PARAMETER DryRun
    If specified, only lists repositories that would be deleted without actually deleting them.

.PARAMETER Force
    If specified, skips confirmation prompts.

.EXAMPLE
    ./tests/cleanup-test-repos.ps1 -MaxAgeHours 24

.EXAMPLE
    ./tests/cleanup-test-repos.ps1 -MaxAgeHours 48 -DryRun

.EXAMPLE
    ./tests/cleanup-test-repos.ps1 -MaxAgeHours 12 -Force
#>

param(
    [int]$MaxAgeHours = 24,
    [switch]$DryRun,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$ORG = "glitchgrove"
$PREFIX = "test-repo-roller-"

Write-Host "RepoRoller Test Repository Cleanup" -ForegroundColor Magenta
Write-Host "===================================" -ForegroundColor Magenta
Write-Host ""

# Check if gh CLI is available
try {
    $null = gh --version
}
catch {
    Write-Error "GitHub CLI (gh) is not installed or not in PATH. Please install it from https://cli.github.com/"
    exit 1
}

# Check if authenticated
try {
    $authStatus = gh auth status 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Not authenticated with GitHub CLI. Please run: gh auth login"
        exit 1
    }
}
catch {
    Write-Error "Failed to check GitHub CLI authentication status"
    exit 1
}

Write-Host "🔍 Fetching repositories from $ORG organization..." -ForegroundColor Cyan

# Fetch all repositories
$repos = gh repo list $ORG --limit 300 --json name,createdAt | ConvertFrom-Json

# Filter for test repositories
$testRepos = $repos | Where-Object { $_.name -like "$PREFIX*" }

Write-Host "   Found $($testRepos.Count) test repositories total" -ForegroundColor Gray

if ($testRepos.Count -eq 0) {
    Write-Host "✅ No test repositories found. Nothing to clean up!" -ForegroundColor Green
    exit 0
}

# Calculate cutoff time
$cutoffTime = (Get-Date).AddHours(-$MaxAgeHours)
Write-Host "   Cutoff time: $($cutoffTime.ToString('yyyy-MM-dd HH:mm:ss')) (repos older than $MaxAgeHours hours)" -ForegroundColor Gray
Write-Host ""

# Filter repositories older than cutoff
$oldRepos = $testRepos | Where-Object {
    $createdAt = [DateTime]$_.createdAt
    $createdAt -lt $cutoffTime
} | Sort-Object createdAt

if ($oldRepos.Count -eq 0) {
    Write-Host "✅ No repositories older than $MaxAgeHours hours found. Nothing to clean up!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Age distribution of test repositories:" -ForegroundColor Cyan
    $now = Get-Date
    $testRepos | ForEach-Object {
        $age = ($now - [DateTime]$_.createdAt).TotalHours
        [PSCustomObject]@{
            Name    = $_.name
            Age     = "$([math]::Round($age, 1))h"
            Created = ([DateTime]$_.createdAt).ToString('yyyy-MM-dd HH:mm')
        }
    } | Format-Table -AutoSize
    exit 0
}

Write-Host "📋 Repositories to delete ($($oldRepos.Count) total):" -ForegroundColor Yellow
Write-Host ""

# Display repositories that will be deleted
$now = Get-Date
$oldRepos | ForEach-Object {
    $age = ($now - [DateTime]$_.createdAt).TotalHours
    $ageFormatted = "$([math]::Round($age, 1))h"
    $created = ([DateTime]$_.createdAt).ToString('yyyy-MM-dd HH:mm:ss')
    Write-Host "  - $($_.name)" -ForegroundColor Gray
    Write-Host "    Created: $created (Age: $ageFormatted)" -ForegroundColor DarkGray
}

Write-Host ""

if ($DryRun) {
    Write-Host "🔍 DRY RUN MODE - No repositories will be deleted" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To actually delete these repositories, run without -DryRun flag:" -ForegroundColor White
    Write-Host "  ./tests/cleanup-test-repos.ps1 -MaxAgeHours $MaxAgeHours" -ForegroundColor Gray
    exit 0
}

# Confirm deletion
if (-not $Force) {
    Write-Host "⚠️  WARNING: This will permanently delete $($oldRepos.Count) repositories!" -ForegroundColor Red
    Write-Host ""
    $confirmation = Read-Host "Type 'DELETE' to confirm deletion"

    if ($confirmation -ne "DELETE") {
        Write-Host ""
        Write-Host "❌ Deletion cancelled" -ForegroundColor Yellow
        exit 0
    }
}

Write-Host ""
Write-Host "🗑️  Deleting repositories..." -ForegroundColor Red

$deleted = 0
$failed = 0
$errors = @()

foreach ($repo in $oldRepos) {
    $repoFullName = "$ORG/$($repo.name)"

    try {
        Write-Host "   Deleting $repoFullName..." -ForegroundColor Gray
        gh repo delete $repoFullName --yes 2>&1 | Out-Null

        if ($LASTEXITCODE -eq 0) {
            $deleted++
            Write-Host "   ✓ Deleted $($repo.name)" -ForegroundColor Green
        }
        else {
            $failed++
            $errors += "Failed to delete $($repo.name): Exit code $LASTEXITCODE"
            Write-Host "   ✗ Failed to delete $($repo.name)" -ForegroundColor Red
        }
    }
    catch {
        $failed++
        $errors += "Failed to delete $($repo.name): $($_.Exception.Message)"
        Write-Host "   ✗ Failed to delete $($repo.name): $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "📊 Cleanup Summary:" -ForegroundColor Cyan
Write-Host "   Successfully deleted: $deleted" -ForegroundColor $(if ($deleted -gt 0) { "Green" } else { "Gray" })
Write-Host "   Failed: $failed" -ForegroundColor $(if ($failed -gt 0) { "Red" } else { "Gray" })

if ($errors.Count -gt 0) {
    Write-Host ""
    Write-Host "❌ Errors encountered:" -ForegroundColor Red
    foreach ($error in $errors) {
        Write-Host "   $error" -ForegroundColor DarkRed
    }
}

Write-Host ""
if ($failed -eq 0) {
    Write-Host "✅ Cleanup completed successfully!" -ForegroundColor Green
}
else {
    Write-Host "⚠️  Cleanup completed with $failed failures" -ForegroundColor Yellow
    exit 1
}
