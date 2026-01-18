#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Comprehensive cleanup tool for test repositories in the glitchgrove organization.

.DESCRIPTION
    This script provides multiple cleanup modes:
    - Age-based cleanup: Remove repos older than specified threshold
    - Misnamed cleanup: Remove repos that don't follow naming conventions
    - Inspection mode: List and categorize all repositories

.PARAMETER Mode
    Cleanup mode: 'age', 'misnamed', or 'inspect'
    Default: 'age'

.PARAMETER Organization
    The GitHub organization name.
    Default: glitchgrove

.PARAMETER MaxAgeHours
    For age-based cleanup: Maximum age in hours for repositories to keep.
    Repositories older than this will be deleted.
    Default: 24 hours

.PARAMETER OlderThanDays
    For misnamed cleanup: Only delete repositories older than this many days.
    Default: 1 day

.PARAMETER DryRun
    If specified, only lists repositories that would be deleted without actually deleting them.

.PARAMETER Force
    If specified, skips confirmation prompts.

.EXAMPLE
    # Inspect all repositories
    ./tests/cleanup-repos.ps1 -Mode inspect

.EXAMPLE
    # Preview age-based cleanup (dry run)
    ./tests/cleanup-repos.ps1 -Mode age -MaxAgeHours 24 -DryRun

.EXAMPLE
    # Delete repos older than 24 hours
    ./tests/cleanup-repos.ps1 -Mode age -MaxAgeHours 24

.EXAMPLE
    # Delete misnamed repos older than 7 days
    ./tests/cleanup-repos.ps1 -Mode misnamed -OlderThanDays 7

.EXAMPLE
    # Force cleanup without confirmation
    ./tests/cleanup-repos.ps1 -Mode age -MaxAgeHours 12 -Force
#>

param(
    [ValidateSet('age', 'misnamed', 'inspect')]
    [string]$Mode = 'age',

    [string]$Organization = 'glitchgrove',

    [int]$MaxAgeHours = 24,

    [int]$OlderThanDays = 1,

    [switch]$DryRun,

    [switch]$Force
)

$ErrorActionPreference = 'Stop'

$TEST_PREFIXES = @('test-repo-roller-', 'e2e-repo-roller-')
$MISNAMED_PATTERNS = @('^(template-|test-|integration-|e2e-test-|e2e-|temp-|demo-)')

# ============================================================================
# Helper Functions
# ============================================================================

function Test-GitHubCLI
{
    try
    {
        $null = gh --version
        return $true
    }
    catch
    {
        Write-Error "GitHub CLI (gh) is not installed or not in PATH. Please install it from https://cli.github.com/"
        return $false
    }
}

function Test-GitHubAuth
{
    try
    {
        $authStatus = gh auth status 2>&1
        return $LASTEXITCODE -eq 0
    }
    catch
    {
        Write-Error "Not authenticated with GitHub CLI. Please run: gh auth login"
        return $false
    }
}

function Get-OrganizationRepos
{
    param([string]$Org)

    Write-Host "🔍 Fetching repositories from $Org organization..." -ForegroundColor Cyan

    try
    {
        $repos = gh repo list $Org --limit 1000 --json name, createdAt, isTemplate 2>&1 | ConvertFrom-Json

        if ($LASTEXITCODE -ne 0 -or -not $repos)
        {
            Write-Host ""
            Write-Host "❌ Failed to fetch repositories. Please check:" -ForegroundColor Red
            Write-Host "   - Organization name is correct: '$Org'" -ForegroundColor Gray
            Write-Host "   - You have access to the organization" -ForegroundColor Gray
            Write-Host "   - GitHub CLI is authenticated: gh auth status" -ForegroundColor Gray
            return @()
        }

        Write-Host "   Found $($repos.Count) total repositories" -ForegroundColor Gray
        Write-Host ""
        return $repos
    }
    catch
    {
        Write-Host ""
        Write-Host "❌ Error fetching repositories: $($_.Exception.Message)" -ForegroundColor Red
        return @()
    }
}

function Get-TestRepos
{
    param(
        [array]$AllRepos,
        [string[]]$Prefixes
    )

    return $AllRepos | Where-Object {
        $repoName = $_.name
        $Prefixes | Where-Object { $repoName.StartsWith($_) } | Select-Object -First 1
    }
}

function Get-MisnamedRepos
{
    param(
        [array]$AllRepos,
        [string[]]$Prefixes,
        [string[]]$MisnamedPatterns
    )

    $misnamed = @()

    foreach ($repo in $AllRepos)
    {
        $name = $repo.name
        $isCorrectlyNamed = $Prefixes | Where-Object { $name.StartsWith($_) } | Select-Object -First 1
        $isMisnamedPattern = $false

        foreach ($pattern in $MisnamedPatterns)
        {
            if ($name -match $pattern)
            {
                $isMisnamedPattern = $true
                break
            }
        }

        if ($isMisnamedPattern -and -not $isCorrectlyNamed -and -not $repo.isTemplate)
        {
            $misnamed += $repo
        }
    }

    return $misnamed
}

function Format-Age
{
    param([TimeSpan]$Age)

    if ($Age.TotalDays -ge 1)
    {
        return "$([int]$Age.TotalDays)d"
    }
    elseif ($Age.TotalHours -ge 1)
    {
        return "$([math]::Round($Age.TotalHours, 1))h"
    }
    else
    {
        return "$([int]$Age.TotalMinutes)m"
    }
}

function Remove-Repositories
{
    param(
        [array]$Repos,
        [string]$Org,
        [bool]$IsDryRun,
        [bool]$SkipConfirm
    )

    if ($Repos.Count -eq 0)
    {
        Write-Host "✅ No repositories to delete" -ForegroundColor Green
        return @{ Deleted = 0; Failed = 0 }
    }

    # Display repositories
    Write-Host "📋 Repositories to delete ($($Repos.Count) total):" -ForegroundColor Yellow
    Write-Host ""

    $now = Get-Date
    $Repos | Sort-Object createdAt | ForEach-Object {
        $age = $now - [DateTime]$_.createdAt
        Write-Host "  - $($_.name) (age: $(Format-Age $age))" -ForegroundColor Gray
    }
    Write-Host ""

    if ($IsDryRun)
    {
        Write-Host "🔍 DRY RUN MODE - No repositories will be deleted" -ForegroundColor Yellow
        return @{ Deleted = 0; Failed = 0 }
    }

    # Confirm deletion
    if (-not $SkipConfirm)
    {
        Write-Host "⚠️  WARNING: This will permanently delete $($Repos.Count) repositories!" -ForegroundColor Red
        Write-Host ""
        $confirmation = Read-Host "Type 'DELETE' to confirm deletion"

        if ($confirmation -ne 'DELETE')
        {
            Write-Host ""
            Write-Host "❌ Deletion cancelled" -ForegroundColor Yellow
            return @{ Deleted = 0; Failed = 0 }
        }
        Write-Host ""
    }

    Write-Host "🗑️  Deleting repositories..." -ForegroundColor Red

    $deleted = 0
    $failed = 0
    $errors = @()

    foreach ($repo in $Repos)
    {
        $repoFullName = "$Org/$($repo.name)"

        try
        {
            Write-Host "   Deleting $repoFullName..." -ForegroundColor Gray
            gh repo delete $repoFullName --yes 2>&1 | Out-Null

            if ($LASTEXITCODE -eq 0)
            {
                $deleted++
                Write-Host "   ✓ Deleted $($repo.name)" -ForegroundColor Green
            }
            else
            {
                $failed++
                $errors += "Failed to delete $($repo.name): Exit code $LASTEXITCODE"
                Write-Host "   ✗ Failed to delete $($repo.name)" -ForegroundColor Red
            }
        }
        catch
        {
            $failed++
            $errors += "Failed to delete $($repo.name): $($_.Exception.Message)"
            Write-Host "   ✗ Failed to delete $($repo.name): $($_.Exception.Message)" -ForegroundColor Red
        }
    }

    return @{
        Deleted = $deleted
        Failed  = $failed
        Errors  = $errors
    }
}

# ============================================================================
# Mode: Inspect
# ============================================================================

function Invoke-InspectMode
{
    param(
        [array]$AllRepos,
        [string]$Org
    )

    Write-Host "🔍 Repository Analysis - $Org" -ForegroundColor Magenta
    Write-Host "══════════════════════════════════════════" -ForegroundColor Magenta
    Write-Host ""

    # Categorize repositories
    $correctlyNamed = Get-TestRepos -AllRepos $AllRepos -Prefixes $TEST_PREFIXES
    $misnamed = Get-MisnamedRepos -AllRepos $AllRepos -Prefixes $TEST_PREFIXES -MisnamedPatterns $MISNAMED_PATTERNS
    $templates = $AllRepos | Where-Object { $_.isTemplate }
    $regular = $AllRepos | Where-Object {
        $repo = $_
        -not ($correctlyNamed -contains $repo) -and
        -not ($misnamed -contains $repo) -and
        -not ($templates -contains $repo)
    }

    Write-Host "📊 Summary:" -ForegroundColor Cyan
    Write-Host "   Total repositories:     $($AllRepos.Count)" -ForegroundColor White
    Write-Host "   Correctly named tests:  $($correctlyNamed.Count)" -ForegroundColor Green
    Write-Host "   Misnamed tests:         $($misnamed.Count)" -ForegroundColor Yellow
    Write-Host "   Template repositories:  $($templates.Count)" -ForegroundColor White
    Write-Host "   Other repositories:     $($regular.Count)" -ForegroundColor White
    Write-Host ""

    # Show correctly named
    if ($correctlyNamed.Count -gt 0)
    {
        Write-Host "✅ Correctly named test repositories:" -ForegroundColor Green
        $now = Get-Date
        $correctlyNamed | Sort-Object createdAt | ForEach-Object {
            $age = $now - [DateTime]$_.createdAt
            Write-Host "   - $($_.name) (age: $(Format-Age $age))" -ForegroundColor Gray
        }
        Write-Host ""
    }

    # Show misnamed
    if ($misnamed.Count -gt 0)
    {
        Write-Host "⚠️  Misnamed test repositories:" -ForegroundColor Yellow
        $now = Get-Date
        $misnamed | Sort-Object createdAt | ForEach-Object {
            $age = $now - [DateTime]$_.createdAt
            Write-Host "   - $($_.name) (age: $(Format-Age $age))" -ForegroundColor Gray
        }
        Write-Host ""
        Write-Host "💡 Tip: Clean up with: .\cleanup-repos.ps1 -Mode misnamed" -ForegroundColor Cyan
        Write-Host ""
    }

    # Show templates (limited)
    if ($templates.Count -gt 0 -and $templates.Count -le 20)
    {
        Write-Host "📦 Template repositories:" -ForegroundColor White
        $templates | Sort-Object name | ForEach-Object {
            Write-Host "   - $($_.name)" -ForegroundColor Gray
        }
        Write-Host ""
    }
}

# ============================================================================
# Mode: Age-based Cleanup
# ============================================================================

function Invoke-AgeCleanup
{
    param(
        [array]$AllRepos,
        [string]$Org,
        [int]$MaxAge,
        [bool]$IsDryRun,
        [bool]$SkipConfirm
    )

    Write-Host "🧹 Age-Based Repository Cleanup" -ForegroundColor Magenta
    Write-Host "══════════════════════════════════════════" -ForegroundColor Magenta
    Write-Host ""

    # Get test repositories
    $testRepos = Get-TestRepos -AllRepos $AllRepos -Prefixes $TEST_PREFIXES
    Write-Host "   Found $($testRepos.Count) test repositories" -ForegroundColor Gray

    # Calculate cutoff time
    $cutoffTime = (Get-Date).AddHours(-$MaxAge)
    Write-Host "   Cutoff time: $($cutoffTime.ToString('yyyy-MM-dd HH:mm:ss'))" -ForegroundColor Gray
    Write-Host "   (deleting repos older than $MaxAge hours)" -ForegroundColor Gray
    Write-Host ""

    # Filter old repositories
    $oldRepos = $testRepos | Where-Object {
        $createdAt = [DateTime]$_.createdAt
        $createdAt -lt $cutoffTime
    }

    if ($oldRepos.Count -eq 0)
    {
        Write-Host "✅ No repositories older than $MaxAge hours found" -ForegroundColor Green
        return
    }

    # Delete repositories
    $result = Remove-Repositories -Repos $oldRepos -Org $Org -IsDryRun $IsDryRun -SkipConfirm $SkipConfirm

    # Show summary
    if (-not $IsDryRun)
    {
        Write-Host ""
        Write-Host "📊 Cleanup Summary:" -ForegroundColor Cyan
        Write-Host "   Successfully deleted: $($result.Deleted)" -ForegroundColor $(if ($result.Deleted -gt 0)
            {
                'Green'
            }
            else
            {
                'Gray'
            })
        Write-Host "   Failed:              $($result.Failed)" -ForegroundColor $(if ($result.Failed -gt 0)
            {
                'Red'
            }
            else
            {
                'Gray'
            })

        if ($result.Errors.Count -gt 0)
        {
            Write-Host ""
            Write-Host "❌ Errors encountered:" -ForegroundColor Red
            foreach ($error in $result.Errors)
            {
                Write-Host "   $error" -ForegroundColor DarkRed
            }
        }

        Write-Host ""
        if ($result.Failed -eq 0)
        {
            Write-Host "✅ Cleanup completed successfully!" -ForegroundColor Green
        }
        else
        {
            Write-Host "⚠️  Cleanup completed with $($result.Failed) failures" -ForegroundColor Yellow
        }
    }
}

# ============================================================================
# Mode: Misnamed Cleanup
# ============================================================================

function Invoke-MisnamedCleanup
{
    param(
        [array]$AllRepos,
        [string]$Org,
        [int]$OlderThan,
        [bool]$IsDryRun,
        [bool]$SkipConfirm
    )

    Write-Host "🧹 Misnamed Repository Cleanup" -ForegroundColor Magenta
    Write-Host "══════════════════════════════════════════" -ForegroundColor Magenta
    Write-Host ""

    # Get misnamed repositories
    $misnamedRepos = Get-MisnamedRepos -AllRepos $AllRepos -Prefixes $TEST_PREFIXES -MisnamedPatterns $MISNAMED_PATTERNS

    # Apply age filter
    $cutoffDate = (Get-Date).AddDays(-$OlderThan)
    Write-Host "   Cutoff date: $($cutoffDate.ToString('yyyy-MM-dd HH:mm:ss'))" -ForegroundColor Gray
    Write-Host "   (deleting repos older than $OlderThan days)" -ForegroundColor Gray
    Write-Host ""

    $oldMisnamed = $misnamedRepos | Where-Object {
        [DateTime]$_.createdAt -lt $cutoffDate
    }

    if ($oldMisnamed.Count -eq 0)
    {
        Write-Host "✅ No misnamed repositories older than $OlderThan days found" -ForegroundColor Green
        return
    }

    # Delete repositories
    $result = Remove-Repositories -Repos $oldMisnamed -Org $Org -IsDryRun $IsDryRun -SkipConfirm $SkipConfirm

    # Show summary
    if (-not $IsDryRun)
    {
        Write-Host ""
        Write-Host "📊 Cleanup Summary:" -ForegroundColor Cyan
        Write-Host "   Successfully deleted: $($result.Deleted)" -ForegroundColor $(if ($result.Deleted -gt 0)
            {
                'Green'
            }
            else
            {
                'Gray'
            })
        Write-Host "   Failed:              $($result.Failed)" -ForegroundColor $(if ($result.Failed -gt 0)
            {
                'Red'
            }
            else
            {
                'Gray'
            })

        if ($result.Errors.Count -gt 0)
        {
            Write-Host ""
            Write-Host "❌ Errors encountered:" -ForegroundColor Red
            foreach ($error in $result.Errors)
            {
                Write-Host "   $error" -ForegroundColor DarkRed
            }
        }

        Write-Host ""
        if ($result.Failed -eq 0)
        {
            Write-Host "✅ Cleanup completed successfully!" -ForegroundColor Green
        }
        else
        {
            Write-Host "⚠️  Cleanup completed with $($result.Failed) failures" -ForegroundColor Yellow
        }
    }
}

# ============================================================================
# Main Execution
# ============================================================================

# Validate prerequisites
if (-not (Test-GitHubCLI))
{
    exit 1
}

if (-not (Test-GitHubAuth))
{
    exit 1
}

# Fetch repositories
$allRepos = Get-OrganizationRepos -Org $Organization

if ($allRepos.Count -eq 0)
{
    exit 1
}

# Execute appropriate mode
switch ($Mode)
{
    'inspect'
    {
        Invoke-InspectMode -AllRepos $allRepos -Org $Organization
    }
    'age'
    {
        Invoke-AgeCleanup -AllRepos $allRepos -Org $Organization -MaxAge $MaxAgeHours -IsDryRun:$DryRun -SkipConfirm:$Force
    }
    'misnamed'
    {
        Invoke-MisnamedCleanup -AllRepos $allRepos -Org $Organization -OlderThan $OlderThanDays -IsDryRun:$DryRun -SkipConfirm:$Force
    }
}
