#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Identifies all test repositories in the glitchgrove organization.

.DESCRIPTION
    Lists all repositories in the organization and categorizes them:
    - Correctly named test repos (test-repo-roller-*, e2e-repo-roller-*)
    - Potentially misnamed test repos (template-*, test-*, other patterns)
    - Regular repositories

.PARAMETER Organization
    The GitHub organization name. Default: glitchgrove

.EXAMPLE
    .\identify-test-repos.ps1
    .\identify-test-repos.ps1 -Organization "myorg"
#>

param(
    [string]$Organization = "glitchgrove"
)

Write-Host "🔍 Analyzing repositories in organization: $Organization" -ForegroundColor Cyan
Write-Host ""

# Get all repositories
$repos = gh repo list $Organization --limit 1000 --json "name,createdAt,updatedAt,isTemplate" | ConvertFrom-Json

if (-not $repos)
{
    Write-Host "❌ No repositories found or error accessing organization" -ForegroundColor Red
    exit 1
}

Write-Host "📊 Total repositories: $($repos.Count)" -ForegroundColor White
Write-Host ""

# Categorize repositories
$correctlyNamed = @()
$misnamed = @()
$regular = @()

foreach ($repo in $repos)
{
    $name = $repo.name

    if ($name -match '^(test-repo-roller-|e2e-repo-roller-)')
    {
        $correctlyNamed += $repo
    }
    elseif ($name -match '^(template-|test-|integration-|e2e-|temp-|demo-)' -and -not $repo.isTemplate)
    {
        $misnamed += $repo
    }
    elseif ($repo.isTemplate)
    {
        # Template repositories are expected, don't flag them
        $regular += $repo
    }
    else
    {
        $regular += $repo
    }
}

# Display results
Write-Host "✅ Correctly named test repositories: $($correctlyNamed.Count)" -ForegroundColor Green
if ($correctlyNamed.Count -gt 0)
{
    $correctlyNamed | Sort-Object createdAt | ForEach-Object {
        $age = (Get-Date) - [DateTime]$_.createdAt
        Write-Host "   - $($_.name) (created $([int]$age.TotalHours)h ago)" -ForegroundColor Gray
    }
    Write-Host ""
}

Write-Host "⚠️  Potentially misnamed test repositories: $($misnamed.Count)" -ForegroundColor Yellow
if ($misnamed.Count -gt 0)
{
    $misnamed | Sort-Object createdAt | ForEach-Object {
        $age = (Get-Date) - [DateTime]$_.createdAt
        Write-Host "   - $($_.name) (created $([int]$age.TotalHours)h ago)" -ForegroundColor Gray
    }
    Write-Host ""
}

Write-Host "📁 Regular repositories: $($regular.Count)" -ForegroundColor White
if ($regular.Count -gt 0 -and $regular.Count -le 10)
{
    $regular | Sort-Object name | ForEach-Object {
        $type = if ($_.isTemplate)
        {
            "(template)"
        }
        else
        {
            ""
        }
        Write-Host "   - $($_.name) $type" -ForegroundColor Gray
    }
    Write-Host ""
}

# Summary
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Summary:" -ForegroundColor Cyan
Write-Host "  Correctly named: $($correctlyNamed.Count)" -ForegroundColor Green
Write-Host "  Misnamed:        $($misnamed.Count)" -ForegroundColor Yellow
Write-Host "  Regular:         $($regular.Count)" -ForegroundColor White
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan

if ($misnamed.Count -gt 0)
{
    Write-Host ""
    Write-Host "💡 Tip: You can clean up misnamed repositories with:" -ForegroundColor Cyan
    Write-Host "   .\cleanup-misnamed-repos.ps1 -Organization $Organization" -ForegroundColor Gray
}
