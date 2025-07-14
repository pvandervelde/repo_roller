#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Runs RepoRoller integration tests locally with proper environment setup.

.DESCRIPTION
    This script sets up the required environment variables and runs the integration tests locally.
    It can run individual test scenarios or the full integration test suite.

.PARAMETER TestScenario
    Specific test scenario to run. Options: basic, variables, filtering, errors, all
    Default: all

.PARAMETER CleanupOrphans
    Clean up orphaned test repositories before running tests.

.PARAMETER MaxAgeHours
    Maximum age in hours for orphaned repositories to clean up (default: 24).

.PARAMETER Verbose
    Enable verbose logging.

.EXAMPLE
    ./tests/run-integration-tests-local.ps1

.EXAMPLE
    ./tests/run-integration-tests-local.ps1 -TestScenario basic -Verbose

.EXAMPLE
    ./tests/run-integration-tests-local.ps1 -CleanupOrphans -MaxAgeHours 12
#>

param(
    [ValidateSet("basic", "variables", "filtering", "errors", "all")]
    [string]$TestScenario = "all",
    [switch]$CleanupOrphans,
    [int]$MaxAgeHours = 24,
    [switch]$Verbose
)

# Set error action preference
$ErrorActionPreference = "Stop"

function Test-Prerequisites
{
    Write-Host "🔍 Checking prerequisites..." -ForegroundColor Cyan

    # Check if we're in the right directory
    if (-not (Test-Path "Cargo.toml"))
    {
        Write-Error "Please run this script from the root of the repo_roller project"
        exit 1
    }

    # Check if integration_tests crate exists
    if (-not (Test-Path "crates/integration_tests"))
    {
        Write-Error "Integration tests crate not found. Please ensure the project structure is correct."
        exit 1
    }

    Write-Host "✅ Prerequisites check passed" -ForegroundColor Green
}

function Set-Environment
{
    Write-Host "🔧 Setting up environment variables..." -ForegroundColor Cyan

    # These should match the GitHub secrets configuration
    $env:GITHUB_APP_ID = "1442780"
    $env:TEST_ORG = "glitchgrove"

    # Check if GITHUB_APP_PRIVATE_KEY is already set
    if (-not $env:GITHUB_APP_PRIVATE_KEY)
    {
        Write-Host "⚠️  GITHUB_APP_PRIVATE_KEY environment variable not set" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "To run integration tests locally, you need to set the GitHub App private key:" -ForegroundColor White
        Write-Host "  1. Get the private key from the GitHub App settings" -ForegroundColor White
        Write-Host "  2. Set the environment variable:" -ForegroundColor White
        Write-Host "     `$env:GITHUB_APP_PRIVATE_KEY = 'YOUR_PRIVATE_KEY_HERE'" -ForegroundColor Gray
        Write-Host ""
        Write-Host "Or create a .env file in the project root with:" -ForegroundColor White
        Write-Host "  GITHUB_APP_PRIVATE_KEY=YOUR_PRIVATE_KEY_HERE" -ForegroundColor Gray
        Write-Host ""

        # Check if .env file exists
        if (Test-Path ".env")
        {
            Write-Host "📄 Found .env file, loading environment variables..." -ForegroundColor Cyan
            Get-Content ".env" | ForEach-Object {
                if ($_ -match "^([^=]+)=(.*)$")
                {
                    [System.Environment]::SetEnvironmentVariable($Matches[1], $Matches[2])
                }
            }

            if ($env:GITHUB_APP_PRIVATE_KEY)
            {
                Write-Host "✅ GITHUB_APP_PRIVATE_KEY loaded from .env file" -ForegroundColor Green
            }
            else
            {
                Write-Error "GITHUB_APP_PRIVATE_KEY not found in .env file"
                exit 1
            }
        }
        else
        {
            Write-Error "GITHUB_APP_PRIVATE_KEY environment variable is required"
            exit 1
        }
    }
    else
    {
        Write-Host "✅ GITHUB_APP_PRIVATE_KEY is set" -ForegroundColor Green
    }

    # Set logging level
    if ($Verbose)
    {
        $env:RUST_LOG = "debug"
    }
    else
    {
        $env:RUST_LOG = "info"
    }

    Write-Host "✅ Environment configured:" -ForegroundColor Green
    Write-Host "  GITHUB_APP_ID: $env:GITHUB_APP_ID" -ForegroundColor Gray
    Write-Host "  TEST_ORG: $env:TEST_ORG" -ForegroundColor Gray
    Write-Host "  RUST_LOG: $env:RUST_LOG" -ForegroundColor Gray
}

function Build-IntegrationTests
{
    Write-Host "🔨 Building integration tests..." -ForegroundColor Cyan

    try
    {
        cargo build --bin integration_tests
        Write-Host "✅ Integration tests built successfully" -ForegroundColor Green
    }
    catch
    {
        Write-Error "Failed to build integration tests: $_"
        exit 1
    }
}

function Invoke-IntegrationTests
{
    param([string]$Scenario, [switch]$Cleanup, [int]$MaxAge)

    Write-Host "🚀 Running integration tests..." -ForegroundColor Cyan

    $testArgs = @()

    if ($Cleanup)
    {
        $testArgs += "--cleanup-orphans"
        $testArgs += "--max-age-hours"
        $testArgs += $MaxAge.ToString()
    }

    try
    {
        if ($Scenario -eq "all")
        {
            Write-Host "📊 Running full integration test suite..." -ForegroundColor Yellow
            cargo run --bin integration_tests -- @testArgs
        }
        else
        {
            Write-Host "📊 Running individual test scenario: $Scenario..." -ForegroundColor Yellow

            # Map scenario names to test function names
            $testMap = @{
                "basic"     = "test_basic_repository_creation"
                "variables" = "test_variable_substitution"
                "filtering" = "test_file_filtering"
                "errors"    = "test_error_handling"
            }

            $testFunction = $testMap[$Scenario]
            if ($testFunction)
            {
                cargo test -p integration_tests $testFunction
            }
            else
            {
                Write-Error "Unknown test scenario: $Scenario"
                exit 1
            }
        }

        Write-Host "✅ Integration tests completed successfully" -ForegroundColor Green
    }
    catch
    {
        Write-Host "❌ Integration tests failed: $_" -ForegroundColor Red
        exit 1
    }
}

# Main execution
Write-Host "RepoRoller Integration Test Runner" -ForegroundColor Magenta
Write-Host "==================================" -ForegroundColor Magenta
Write-Host ""

if ($Verbose)
{
    Write-Host "🔍 Verbose mode enabled" -ForegroundColor Gray
    Write-Host "Test scenario: $TestScenario" -ForegroundColor Gray
    Write-Host "Cleanup orphans: $CleanupOrphans" -ForegroundColor Gray
    Write-Host "Max age hours: $MaxAgeHours" -ForegroundColor Gray
    Write-Host ""
}

try
{
    Test-Prerequisites
    Set-Environment
    Build-IntegrationTests
    Invoke-IntegrationTests -Scenario $TestScenario -Cleanup:$CleanupOrphans -MaxAge $MaxAgeHours

    Write-Host ""
    Write-Host "🎉 All operations completed successfully!" -ForegroundColor Green
}
catch
{
    Write-Host ""
    Write-Host "💥 Script failed: $_" -ForegroundColor Red
    exit 1
}
