#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Creates GitHub teams and sets up accounts required for RepoRoller permission integration tests.

.DESCRIPTION
    This script creates the GitHub teams needed by the permission integration tests
    in the permission_tests.rs test suite. It also prints the environment variables
    required to run those tests and validates that a collaborator account exists in
    the target GitHub organization.

    Teams created:
    - reporoller-test-permissions  Permissions integration test team
    - reporoller-test-triage       Triage-level integration test team
    - reporoller-test-security     Security/locked team used to verify permission-protection policies

    After running this script:
    1. Set TEST_TEAM_SLUG=reporoller-test-permissions in your CI/CD environment
    2. Set TEST_COLLABORATOR_USERNAME=<GitHub username of an org member> — see -CollaboratorUsername

.PARAMETER Organization
    The GitHub organization. Defaults to 'glitchgrove'.

.PARAMETER CollaboratorUsername
    GitHub username to use as the direct-collaborator test account.
    The user must already be a member of, or have previously interacted with, the org
    so they can receive repository invitations.
    Defaults to 'reporoller-test-user' (this account must already exist on GitHub).

.PARAMETER SourceRepo
    The GitHub repository (owner/name) where the Actions variables will be set.
    Defaults to 'pvandervelde/repo_roller'.
    Must be the repository that contains the integration tests.

.PARAMETER Force
    Re-create teams even if they already exist (deletes and recreates).

.PARAMETER SkipSetVariables
    Skip setting GitHub Actions variables on SourceRepo.
    Use this if you only want to create the teams without modifying the Actions config.

.PARAMETER Verbose
    Enable verbose output for debugging.

.EXAMPLE
    ./tests/create-test-teams.ps1

.EXAMPLE
    ./tests/create-test-teams.ps1 -Organization "glitchgrove" -CollaboratorUsername "some-github-user"

.EXAMPLE
    ./tests/create-test-teams.ps1 -Organization "glitchgrove" -Force -Verbose

.EXAMPLE
    ./tests/create-test-teams.ps1 -SkipSetVariables
#>

param(
    [string]$Organization = "glitchgrove",
    [string]$CollaboratorUsername = "reporoller-test-user",
    [string]$SourceRepo = "pvandervelde/repo_roller",
    [switch]$Force,
    [switch]$SkipSetVariables
)

$ErrorActionPreference = "Stop"

# ── Teams to create ───────────────────────────────────────────────────────────

# Each entry: Name (display), Slug (URL-safe identifier), Description, Privacy
$Teams = @(
    @{
        Name        = "reporoller-test-permissions"
        Slug        = "reporoller-test-permissions"
        Description = "Integration test team for RepoRoller permission system tests"
        Privacy     = "closed"
    },
    @{
        Name        = "reporoller-test-triage"
        Slug        = "reporoller-test-triage"
        Description = "Triage-level integration test team for RepoRoller permission system tests"
        Privacy     = "closed"
    },
    @{
        Name        = "reporoller-test-security"
        Slug        = "reporoller-test-security"
        Description = "Locked security team used to verify permission-protection policies in RepoRoller"
        Privacy     = "closed"
    }
)

# ── Helpers ───────────────────────────────────────────────────────────────────

function Test-GitHubAuth
{
    Write-Verbose "Checking GitHub CLI authentication..."
    $status = gh auth status 2>&1
    if ($LASTEXITCODE -ne 0)
    {
        Write-Error "GitHub CLI is not authenticated. Run 'gh auth login' first."
        exit 1
    }
    Write-Host "✓ GitHub CLI is authenticated" -ForegroundColor Green
}

function Get-OrgTeam
{
    param([string]$Org, [string]$Slug)
    $result = gh api "orgs/$Org/teams/$Slug" 2>&1
    if ($LASTEXITCODE -eq 0)
    {
        return $result | ConvertFrom-Json
    }
    return $null
}

function New-OrgTeam
{
    param(
        [string]$Org,
        [string]$Name,
        [string]$Description,
        [string]$Privacy   # "closed" or "secret"
    )
    $body = @{
        name        = $Name
        description = $Description
        privacy     = $Privacy
    } | ConvertTo-Json -Compress

    $result = $body | gh api "orgs/$Org/teams" --method POST --input - 2>&1
    if ($LASTEXITCODE -ne 0)
    {
        throw "Failed to create team '$Name' in org '$Org': $result"
    }
    return $result | ConvertFrom-Json
}

function Remove-OrgTeam
{
    param([string]$Org, [string]$Slug)
    gh api "orgs/$Org/teams/$Slug" --method DELETE 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0)
    {
        Write-Warning "Could not delete team '$Slug' — it may not exist or we lack permission."
    }
}

function Test-GitHubUserExists
{
    param([string]$Username)
    $result = gh api "users/$Username" 2>&1
    return $LASTEXITCODE -eq 0
}

function Set-ActionsVariable
{
    param(
        [string]$Repo,
        [string]$Name,
        [string]$Value
    )
    Write-Verbose "Setting Actions variable $Name on $Repo"
    $Value | gh variable set $Name --repo $Repo 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0)
    {
        # gh variable set exits non-zero on some versions even on success; re-check.
        $check = gh variable list --repo $Repo --json name 2>&1
        if ($check -match $Name)
        {
            return  # Variable exists, set succeeded despite exit code
        }
        throw "Failed to set Actions variable '$Name' on '$Repo'"
    }
}

# ── Main logic ────────────────────────────────────────────────────────────────

Test-GitHubAuth

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  RepoRoller Permission Test Team Setup" -ForegroundColor Cyan
Write-Host "  Organization: $Organization" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# ── 1. Create test teams ──────────────────────────────────────────────────────

$createdSlugs = @()

foreach ($team in $Teams)
{
    Write-Host ""
    Write-Host "── Team: $($team.Name)" -ForegroundColor White

    $existing = Get-OrgTeam -Org $Organization -Slug $team.Slug

    if ($existing)
    {
        if ($Force)
        {
            Write-Host "  Team exists — deleting for recreation (-Force)..." -ForegroundColor Yellow
            Remove-OrgTeam -Org $Organization -Slug $team.Slug
            $existing = $null
        }
        else
        {
            Write-Host "  ✓ Team already exists (slug: $($team.Slug)) — skipping (use -Force to recreate)" -ForegroundColor Green
            $createdSlugs += $team.Slug
            continue
        }
    }

    Write-Host "  Creating team '$($team.Name)' in '$Organization'..." -ForegroundColor Gray
    $created = New-OrgTeam `
        -Org        $Organization `
        -Name       $team.Name `
        -Description $team.Description `
        -Privacy    $team.Privacy

    Write-Host "  ✓ Created team '$($created.name)' (slug: $($created.slug), id: $($created.id))" -ForegroundColor Green
    $createdSlugs += $created.slug
}

# ── 2. Validate collaborator account ─────────────────────────────────────────

Write-Host ""
Write-Host "── Collaborator account check" -ForegroundColor White

if (Test-GitHubUserExists -Username $CollaboratorUsername)
{
    Write-Host "  ✓ GitHub user '$CollaboratorUsername' exists" -ForegroundColor Green
}
else
{
    Write-Warning "  GitHub user '$CollaboratorUsername' was not found."
    Write-Host "  Create or choose an existing GitHub user and re-run with:" -ForegroundColor Yellow
    Write-Host "    ./tests/create-test-teams.ps1 -CollaboratorUsername <username>" -ForegroundColor Yellow
}

# ── 3. Set GitHub Actions variables on the source repository ─────────────────

$primaryTeamSlug = $createdSlugs | Select-Object -First 1

if ($SkipSetVariables)
{
    Write-Host ""
    Write-Host "── GitHub Actions variables (skipped — -SkipSetVariables)" -ForegroundColor White
    Write-Host "  Would have set on '$SourceRepo':" -ForegroundColor Gray
    Write-Host "    TEST_TEAM_SLUG            = $primaryTeamSlug" -ForegroundColor Yellow
    Write-Host "    TEST_COLLABORATOR_USERNAME = $CollaboratorUsername" -ForegroundColor Yellow
}
else
{
    Write-Host ""
    Write-Host "── Setting GitHub Actions variables on '$SourceRepo'" -ForegroundColor White

    Write-Host "  Setting TEST_TEAM_SLUG = '$primaryTeamSlug'..." -ForegroundColor Gray
    Set-ActionsVariable -Repo $SourceRepo -Name "TEST_TEAM_SLUG" -Value $primaryTeamSlug
    Write-Host "  ✓ TEST_TEAM_SLUG set" -ForegroundColor Green

    Write-Host "  Setting TEST_COLLABORATOR_USERNAME = '$CollaboratorUsername'..." -ForegroundColor Gray
    Set-ActionsVariable -Repo $SourceRepo -Name "TEST_COLLABORATOR_USERNAME" -Value $CollaboratorUsername
    Write-Host "  ✓ TEST_COLLABORATOR_USERNAME set" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Reference: all required variables" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  # Set as Actions secrets (done separately — contain sensitive data):" -ForegroundColor Gray
Write-Host "  GITHUB_APP_ID=<your-app-id>" -ForegroundColor White
Write-Host "  GITHUB_APP_PRIVATE_KEY=<pem-contents-without-newlines>" -ForegroundColor White
Write-Host ""
Write-Host "  # Set as Actions variables by this script on ${SourceRepo}:" -ForegroundColor Gray
Write-Host "  TEST_ORG=$Organization                        (set manually — org name)" -ForegroundColor White
Write-Host "  TEST_TEAM_SLUG=$primaryTeamSlug" -ForegroundColor Yellow
Write-Host "  TEST_COLLABORATOR_USERNAME=$CollaboratorUsername" -ForegroundColor Yellow
Write-Host ""
Write-Host "  # To run permission integration tests:" -ForegroundColor Gray
Write-Host "  cargo test --test permission_tests -p integration_tests -- --ignored" -ForegroundColor White
Write-Host ""

# ── 4. Summary ────────────────────────────────────────────────────────────────

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Summary" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Teams available in '$Organization':" -ForegroundColor White
foreach ($slug in $createdSlugs)
{
    Write-Host "    - $slug" -ForegroundColor Green
}
Write-Host ""
Write-Host "  Collaborator: $CollaboratorUsername" -ForegroundColor White
Write-Host ""
Write-Host "✓ Permission test prerequisites are set up." -ForegroundColor Green
Write-Host ""
Write-Host "NOTE: The collaborator user must have a GitHub account and the" -ForegroundColor Yellow
Write-Host "      GitHub App must have permission to invite users to repositories" -ForegroundColor Yellow
Write-Host "      in the '$Organization' organization." -ForegroundColor Yellow
