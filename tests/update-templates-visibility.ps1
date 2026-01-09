#!/usr/bin/env pwsh
#
# Update Test Templates with Visibility Configuration
#
# This script updates test template repositories in glitchgrove with
# default_visibility settings in their template.toml files.
#
# Prerequisites:
# - GitHub CLI (gh) installed and authenticated
# - Write access to glitchgrove organization

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$Organization = "glitchgrove"
)

$ErrorActionPreference = "Stop"
$templatesDir = Join-Path $PSScriptRoot "templates"

function Update-Template
{
    param(
        [string]$TemplateName,
        [string]$SourceDir
    )

    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "Updating $TemplateName" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    # Check if template exists
    $existing = gh repo view "$Organization/$TemplateName" 2>$null
    if ($LASTEXITCODE -ne 0)
    {
        Write-Host "Creating template repository..." -ForegroundColor Yellow
        gh repo create "$Organization/$TemplateName" --public --description "Test template: $TemplateName"
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to create template $TemplateName"
        }
    }
    else
    {
        Write-Host "Template exists" -ForegroundColor Gray
    }

    # Clone or update
    $tempDir = Join-Path $env:TEMP (New-Guid).ToString()
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try
    {
        Push-Location $tempDir

        Write-Host "Cloning template..." -ForegroundColor Gray
        git clone "https://github.com/$Organization/$TemplateName.git" 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to clone template"
        }

        Set-Location $TemplateName

        # Copy template files
        Write-Host "Copying template files..." -ForegroundColor Gray
        if (Test-Path $SourceDir)
        {
            # Copy all files, preserving structure
            Get-ChildItem -Path $SourceDir -Recurse | ForEach-Object {
                $relativePath = $_.FullName.Substring($SourceDir.Length + 1)
                $targetPath = Join-Path "." $relativePath

                if ($_.PSIsContainer)
                {
                    New-Item -ItemType Directory -Path $targetPath -Force | Out-Null
                }
                else
                {
                    $targetDir = Split-Path $targetPath -Parent
                    if ($targetDir -and -not (Test-Path $targetDir))
                    {
                        New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
                    }
                    Copy-Item -Path $_.FullName -Destination $targetPath -Force
                }
            }
        }
        else
        {
            throw "Source directory not found: $SourceDir"
        }

        # Check for changes
        git add -A
        $status = git status --porcelain

        if ($status)
        {
            Write-Host "Committing changes..." -ForegroundColor Gray
            git commit -m "Update template with visibility configuration" 2>&1 | Out-Null

            Write-Host "Pushing to GitHub..." -ForegroundColor Gray
            git push 2>&1 | Out-Null

            Write-Host "✓ Successfully updated $TemplateName" -ForegroundColor Green
        }
        else
        {
            Write-Host "✓ No changes needed for $TemplateName" -ForegroundColor Green
        }
    }
    finally
    {
        Pop-Location
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Update templates with visibility settings
Write-Host "Updating templates with default_visibility configuration..." -ForegroundColor Cyan

Update-Template `
    -TemplateName "template-test-basic" `
    -SourceDir (Join-Path $templatesDir "template-test-basic")

Update-Template `
    -TemplateName "template-test-variables" `
    -SourceDir (Join-Path $templatesDir "template-test-variables")

Update-Template `
    -TemplateName "template-test-internal" `
    -SourceDir (Join-Path $templatesDir "template-test-internal")

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "All templates updated!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
