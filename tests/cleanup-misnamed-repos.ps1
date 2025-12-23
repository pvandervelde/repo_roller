#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Cleans up misnamed test repositories in the glitchgrove organization.

.DESCRIPTION
    Identifies and deletes test repositories that don't follow the correct naming convention.
    Requires confirmation before deleting unless -Force is specified.

.PARAMETER Organization
    The GitHub organization name. Default: glitchgrove

.PARAMETER DryRun
    If specified, only shows what would be deleted without actually deleting.

.PARAMETER Force
    If specified, skips confirmation prompts.

.PARAMETER OlderThanDays
    Only delete repositories older than this many days. Default: 1

.EXAMPLE
    # Dry run to see what would be deleted
    .\cleanup-misnamed-repos.ps1 -DryRun

.EXAMPLE
    # Delete misnamed repos older than 7 days with confirmation
    .\cleanup-misnamed-repos.ps1 -OlderThanDays 7

.EXAMPLE
    # Delete all misnamed repos without confirmation
    .\cleanup-misnamed-repos.ps1 -Force
#>

param(
    [string]$Organization = "glitchgrove",
    [switch]$DryRun,
    [switch]$Force,
    [int]$OlderThanDays = 1
)

Write-Host "🧹 Test Repository Cleanup Tool" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

if ($DryRun)
{
    Write-Host "🔍 DRY RUN MODE - No repositories will be deleted" -ForegroundColor Yellow
    Write-Host ""
}

# Get all repositories
Write-Host "📋 Fetching repositories from organization: $Organization..." -ForegroundColor White
$repos = gh repo list $Organization --limit 1000 --json name,createdAt,isTemplate | ConvertFrom-Json

if (-not $repos)
{
    Write-Host "❌ No repositories found or error accessing organization" -ForegroundColor Red
    exit 1
}

Write-Host "   Found $($repos.Count) total repositories" -ForegroundColor Gray
Write-Host ""

# Find misnamed test repositories
$mismatchedRepos = @()
$cutoffDate = (Get-Date).AddDays(-$OlderThanDays)

foreach ($repo in $repos)
{
    $name = $repo.name
    $createdDate = [DateTime]$repo.createdAt

    # Skip if not old enough
    if ($createdDate -gt $cutoffDate)
    {
        continue
    }

    # Identify misnamed test repos (but not correctly named or templates)
    if ($name -match '^(template-|test-|integration-|e2e-|temp-|demo-)' -and
        -not $name -match '^(test-repo-roller-|e2e-repo-roller-)' -and
        -not $repo.isTemplate)
    {
        $mismatchedRepos += $repo
    }
}

if ($mismatchedRepos.Count -eq 0)
{
    Write-Host "✅ No misnamed test repositories found older than $OlderThanDays days" -ForegroundColor Green
    exit 0
}

Write-Host "⚠️  Found $($mismatchedRepos.Count) misnamed test repositories older than $OlderThanDays days:" -ForegroundColor Yellow
Write-Host ""

$mismatchedRepos | Sort-Object createdAt | ForEach-Object {
    $age = (Get-Date) - [DateTime]$_.createdAt
    $ageStr = if ($age.TotalDays -ge 1)
    {
        "$([int]$age.TotalDays)d"
    }
    else
    {
        "$([int]$age.TotalHours)h"
    }
    Write-Host "   📦 $($_.name) (age: $ageStr)" -ForegroundColor Gray
}

Write-Host ""

if ($DryRun)
{
    Write-Host "✅ Dry run complete - no repositories were deleted" -ForegroundColor Green
    exit 0
}

# Confirm deletion
if (-not $Force)
{
    Write-Host "⚠️  WARNING: This will permanently delete $($mismatchedRepos.Count) repositories!" -ForegroundColor Yellow
    $response = Read-Host "Type 'DELETE' to confirm"

    if ($response -ne 'DELETE')
    {
        Write-Host "❌ Deletion cancelled" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
}

# Delete repositories
$deletedCount = 0
$failedCount = 0

Write-Host "🧹 Deleting repositories..." -ForegroundColor Cyan

foreach ($repo in $mismatchedRepos)
{
    $fullName = "$Organization/$($repo.name)"
    Write-Host "   Deleting: $fullName" -ForegroundColor Gray

    try
    {
        gh repo delete $fullName --yes 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0)
        {
            $deletedCount++
            Write-Host "      ✓ Deleted" -ForegroundColor Green
        }
        else
        {
            $failedCount++
            Write-Host "      ✗ Failed" -ForegroundColor Red
        }
    }
    catch
    {
        $failedCount++
        Write-Host "      ✗ Failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Cleanup complete!" -ForegroundColor Green
Write-Host "  Deleted:   $deletedCount" -ForegroundColor Green
Write-Host "  Failed:    $failedCount" -ForegroundColor $(if ($failedCount -gt 0)
    {
        "Red" 
    }
    else
    {
        "Gray" 
    })
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
