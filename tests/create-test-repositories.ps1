#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Creates GitHub test template repositories for RepoRoller integration tests.

.DESCRIPTION
    This script creates test template repositories required for RepoRoller integration tests.

    Basic templates (4):
    - test-basic: Basic repository creation testing
    - test-variables: Variable substitution testing
    - test-filtering: File filtering testing
    - test-invalid: Error handling testing

    Template processing edge cases (10):
    - template-large-files: Large file handling (>10MB)
    - template-binary-files: Binary file preservation
    - template-deep-nesting: Deep directory nesting (>10 levels)
    - template-many-files: Many files handling (>1000 files)
    - template-unicode-names: Unicode characters in filenames
    - template-with-symlinks: Symbolic link handling
    - template-with-scripts: Executable permission preservation
    - template-with-dotfiles: Hidden file processing
    - template-empty-dirs: Empty directory handling
    - template-no-extensions: Extensionless file processing

    Variable substitution edge cases (2):
    - template-nested-variables: Nested variable substitution
    - template-variable-paths: Variables in file/directory names

.PARAMETER Organization
    The GitHub organization to create repositories in. Defaults to 'pvandervelde'.

.PARAMETER Force
    Force recreation of repositories if they already exist.

.PARAMETER Verbose
    Enable verbose logging for debugging.

.EXAMPLE
    ./scripts/create-test-repositories.ps1

.EXAMPLE
    ./scripts/create-test-repositories.ps1 -Organization "myorg" -Force

.EXAMPLE
    ./scripts/create-test-repositories.ps1 -Organization "myorg" -Verbose
#>

param(
    [string]$Organization = "pvandervelde",
    [switch]$Force,
    [switch]$Verbose
)

# Set error action preference
$ErrorActionPreference = "Stop"

# Template repositories to create
$Templates = @(
    # Basic templates
    @{
        Name        = "template-test-basic"
        Description = "Basic repository template for RepoRoller integration tests"
        Path        = "tests/templates/test-basic"
    },
    @{
        Name        = "template-test-variables"
        Description = "Variable substitution template for RepoRoller integration tests"
        Path        = "tests/templates/test-variables"
    },
    @{
        Name        = "template-test-filtering"
        Description = "File filtering template for RepoRoller integration tests"
        Path        = "tests/templates/test-filtering"
    },
    @{
        Name        = "template-test-invalid"
        Description = "Error handling template for RepoRoller integration tests"
        Path        = "tests/templates/test-invalid"
    },

    # Template processing edge cases
    @{
        Name        = "template-large-files"
        Description = "Large file handling test template (>10MB files) for RepoRoller integration tests"
        Path        = "tests/templates/template-large-files"
    },
    @{
        Name        = "template-binary-files"
        Description = "Binary file preservation test template (PNG, PDF, ZIP) for RepoRoller integration tests"
        Path        = "tests/templates/template-binary-files"
    },
    @{
        Name        = "template-deep-nesting"
        Description = "Deep directory nesting test template (>10 levels) for RepoRoller integration tests"
        Path        = "tests/templates/template-deep-nesting"
    },
    @{
        Name        = "template-many-files"
        Description = "Many files test template (>1000 files) for RepoRoller integration tests"
        Path        = "tests/templates/template-many-files"
    },
    @{
        Name        = "template-unicode-names"
        Description = "Unicode filename test template (Japanese, Cyrillic, emoji) for RepoRoller integration tests"
        Path        = "tests/templates/template-unicode-names"
    },
    @{
        Name        = "template-with-symlinks"
        Description = "Symbolic link handling test template for RepoRoller integration tests"
        Path        = "tests/templates/template-with-symlinks"
    },
    @{
        Name        = "template-with-scripts"
        Description = "Executable permission preservation test template for RepoRoller integration tests"
        Path        = "tests/templates/template-with-scripts"
    },
    @{
        Name        = "template-with-dotfiles"
        Description = "Hidden file processing test template (.gitignore, .env, etc.) for RepoRoller integration tests"
        Path        = "tests/templates/template-with-dotfiles"
    },
    @{
        Name        = "template-empty-dirs"
        Description = "Empty directory handling test template (.gitkeep) for RepoRoller integration tests"
        Path        = "tests/templates/template-empty-dirs"
    },
    @{
        Name        = "template-no-extensions"
        Description = "Extensionless file processing test template (Dockerfile, Makefile, LICENSE) for RepoRoller integration tests"
        Path        = "tests/templates/template-no-extensions"
    },

    # Variable substitution edge cases
    @{
        Name        = "template-nested-variables"
        Description = "Nested variable substitution test template for RepoRoller integration tests"
        Path        = "tests/templates/template-nested-variables"
    },
    @{
        Name        = "template-variable-paths"
        Description = "Variables in file/directory names test template for RepoRoller integration tests"
        Path        = "tests/templates/template-variable-paths"
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

    if ($Verbose)
    {
        Write-Host "Checking if repository exists: $Organization/$Name" -ForegroundColor Gray
    }

    try
    {
        # Try to get repository information using GitHub CLI
        # Suppress stderr but capture stdout
        $output = gh repo view "$Organization/$Name" --json name, owner 2>&1

        # Check if the command failed (exitcode or error message)
        if ($LASTEXITCODE -ne 0)
        {
            if ($Verbose)
            {
                Write-Host "Repository does not exist (gh command failed with exit code $LASTEXITCODE)" -ForegroundColor Gray
            }
            return $false
        }

        if ($Verbose)
        {
            Write-Host "Raw output: '$output'" -ForegroundColor Gray
        }

        # Check if we got any output
        if ([string]::IsNullOrWhiteSpace($output))
        {
            if ($Verbose)
            {
                Write-Host "No output received - repository does not exist" -ForegroundColor Gray
            }
            return $false
        }

        # Try to parse JSON
        try
        {
            $repoInfo = $output | ConvertFrom-Json
        }
        catch
        {
            if ($Verbose)
            {
                Write-Host "Failed to parse JSON output - repository likely doesn't exist" -ForegroundColor Gray
                Write-Host "Output was: $output" -ForegroundColor Gray
            }
            return $false
        }

        # Check if we have valid repository info
        if (-not $repoInfo -or -not $repoInfo.owner -or -not $repoInfo.name)
        {
            if ($Verbose)
            {
                Write-Host "Invalid repository information - repository does not exist" -ForegroundColor Gray
            }
            return $false
        }

        if ($Verbose)
        {
            Write-Host "Repository found: $($repoInfo.owner.login)/$($repoInfo.name)" -ForegroundColor Gray
        }

        # Verify exact match (case-insensitive for organization, exact for repo name)
        $exactMatch = ($repoInfo.owner.login -ieq $Organization) -and ($repoInfo.name -eq $Name)

        if ($Verbose)
        {
            if ($exactMatch)
            {
                Write-Host "Exact match confirmed" -ForegroundColor Green
            }
            else
            {
                Write-Host "Repository name mismatch. Expected: $Organization/$Name, Found: $($repoInfo.owner.login)/$($repoInfo.name)" -ForegroundColor Yellow
            }
        }

        return $exactMatch
    }
    catch
    {
        if ($Verbose)
        {
            Write-Host "Repository not found or error occurred: $Organization/$Name" -ForegroundColor Gray
            Write-Host "Error details: $($_.Exception.Message)" -ForegroundColor Gray
        }
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
    param([string]$Organization, [string]$Name, [string]$Description, [string]$TemplatePath, [bool]$RepositoryExists)

    if ($RepositoryExists)
    {
        Write-Host "Updating repository: $Organization/$Name" -ForegroundColor Cyan
    }
    else
    {
        Write-Host "Creating repository: $Organization/$Name" -ForegroundColor Cyan
    }

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
        git commit -m "Update template: $Description"

        if (-not $RepositoryExists)
        {
            # Create GitHub repository
            gh repo create "$Organization/$Name" --public --description "$Description"

            # Enable template repository feature after creation
            gh api repos/$Organization/$Name --method PATCH --field is_template=true

            # Add reporoller-template topic
            gh api repos/$Organization/$Name/topics --method PUT --field names[]='reporoller-template'
        }
        else
        {
            # Update existing repository settings
            gh api repos/$Organization/$Name --method PATCH --field description="$Description" --field is_template=true

            # Add/update reporoller-template topic
            gh api repos/$Organization/$Name/topics --method PUT --field names[]='reporoller-template'
        }

        # Push to GitHub (force push to overwrite main branch)
        git remote add origin "https://github.com/$Organization/$Name.git"
        git branch -M main
        git push -u origin main --force

        if ($RepositoryExists)
        {
            Write-Host "✓ Repository updated: $Organization/$Name" -ForegroundColor Green
        }
        else
        {
            Write-Host "✓ Repository created: $Organization/$Name" -ForegroundColor Green
        }
    }
    catch
    {
        Write-Error "Failed to create/update repository: $Organization/$Name. Error: $_"
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

if ($Verbose)
{
    Write-Host "Verbose mode enabled" -ForegroundColor Gray
    Write-Host "Organization: $Organization" -ForegroundColor Gray
    Write-Host "Force recreation: $Force" -ForegroundColor Gray
    Write-Host ""
}

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

    if ($Verbose)
    {
        Write-Host "Processing template: $($template.Name)" -ForegroundColor Gray
        Write-Host "Description: $($template.Description)" -ForegroundColor Gray
        Write-Host "Path: $($template.Path)" -ForegroundColor Gray
    }

    $repoExists = Test-RepositoryExists $Organization $repoName

    if ($repoExists)
    {
        if ($Force)
        {
            Write-Host "Repository exists - will update content" -ForegroundColor Yellow
            New-Repository $Organization $repoName $template.Description $template.Path -RepositoryExists $true
        }
        else
        {
            Write-Host "Repository already exists: $Organization/$repoName (use -Force to update)" -ForegroundColor Yellow
            continue
        }
    }
    else
    {
        New-Repository $Organization $repoName $template.Description $template.Path -RepositoryExists $false
    }
}

Write-Host ""
Write-Host "✓ Test repository creation completed!" -ForegroundColor Green
Write-Host "The following repositories are now available:" -ForegroundColor Cyan
foreach ($template in $Templates)
{
    Write-Host "  - https://github.com/$Organization/$($template.Name)" -ForegroundColor White
}

Write-Host ""
Write-Host "📝 Next Steps:" -ForegroundColor Yellow
Write-Host "1. Update the metadata repository (.reporoller-test) with template definitions" -ForegroundColor White
Write-Host "2. Configure global.toml with template-to-repository mappings" -ForegroundColor White
Write-Host "3. Add any repository-type-specific configurations" -ForegroundColor White
Write-Host "4. Run integration tests to verify templates work correctly" -ForegroundColor White
Write-Host ""
Write-Host "Example metadata repository structure:" -ForegroundColor Gray
Write-Host "  .reporoller-test/" -ForegroundColor Gray
Write-Host "    global.toml          # Template mappings" -ForegroundColor Gray
Write-Host "    repository-types/    # Type definitions" -ForegroundColor Gray
Write-Host "    team-configs/        # Team-specific overrides" -ForegroundColor Gray
