#Requires -Version 5.1
<#
.SYNOPSIS
    Start the RepoRoller local Docker Compose stack.

.DESCRIPTION
    On first run, copies .env.example to .env and exits so you can fill in
    the GitHub credentials. On subsequent runs, automatically generates
    SESSION_SECRET and JWT_SECRET if they are empty, validates that all other
    required variables are present, then launches both containers.

.PARAMETER NoBuild
    Skip rebuilding container images (pass --no-build to docker compose).

.PARAMETER Detach
    Run containers in the background (pass --detach to docker compose).

.EXAMPLE
    .\start.ps1

.EXAMPLE
    .\start.ps1 -Detach

.PARAMETER EnvFile
    Path to the env file to use. Defaults to .env in the script directory.
    Use this to point at a differently-named file, e.g. .env.local.

.EXAMPLE
    .\start.ps1 -NoBuild -Detach

.EXAMPLE
    .\start.ps1 -EnvFile .env.local
#>
[CmdletBinding()]
param (
    [switch] $NoBuild,
    [switch] $Detach,
    [string] $EnvFile = '.env'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Run everything relative to this script so docker compose can find its files.
Push-Location $PSScriptRoot
try
{

    # ------------------------------------------------------------------ #
    # 1. First-run bootstrap: create .env from .env.example               #
    # ------------------------------------------------------------------ #
    if (-not (Test-Path $EnvFile))
    {
        if (-not (Test-Path '.env.example'))
        {
            Write-Error "Neither .env nor .env.example found in $PSScriptRoot"
            exit 1
        }
        Copy-Item '.env.example' $EnvFile
        Write-Host ''
        Write-Host "Created $EnvFile from .env.example." -ForegroundColor Green
        Write-Host ''
        Write-Host "ACTION REQUIRED: open $EnvFile and fill in these variables:" -ForegroundColor Yellow
        Write-Host '  GITHUB_APP_ID            Numeric App ID from your GitHub App settings page' -ForegroundColor Yellow
        Write-Host '  GITHUB_APP_PRIVATE_KEY   Full PEM private key (keep newlines, or collapse to \n)' -ForegroundColor Yellow
        Write-Host '  GITHUB_CLIENT_ID         OAuth App client ID' -ForegroundColor Yellow
        Write-Host '  GITHUB_CLIENT_SECRET     OAuth App client secret' -ForegroundColor Yellow
        Write-Host '  GITHUB_ORG               Your GitHub organization slug (e.g. acme-corp)' -ForegroundColor Yellow
        Write-Host ''
        Write-Host 'SESSION_SECRET and JWT_SECRET will be generated automatically on the next run.' -ForegroundColor Cyan
        Write-Host 'See docs/configuration-guide.md#prerequisites for full setup instructions.' -ForegroundColor Cyan
        Write-Host ''
        exit 0
    }

    # ------------------------------------------------------------------ #
    # 2. Read env file into an in-memory line array                       #
    # ------------------------------------------------------------------ #
    [string[]] $script:envLines = Get-Content $EnvFile -Encoding utf8

    function Get-EnvValue ([string] $Name)
    {
        $match = $script:envLines |
        Where-Object { $_ -match "^${Name}\s*=(.*)$" } |
        Select-Object -Last 1
        if ($null -eq $match)
        {
            return ''
        }
        if ($match -match "^${Name}\s*=(.*)$")
        {
            return $Matches[1].Trim()
        }
        return ''
    }

    function Set-EnvValue ([string] $Name, [string] $Value)
    {
        $found = $false
        $updated = [System.Collections.Generic.List[string]]::new()
        foreach ($line in $script:envLines)
        {
            if ($line -match "^${Name}\s*=")
            {
                $updated.Add("${Name}=${Value}")
                $found = $true
            }
            else
            {
                $updated.Add($line)
            }
        }
        if (-not $found)
        {
            $updated.Add("${Name}=${Value}")
        }
        $script:envLines = $updated.ToArray()
    }

    function New-SecretHex
    {
        # 48 random bytes → 96 lowercase hex characters (well above the 32-char minimum)
        $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
        $bytes = New-Object byte[] 48
        $rng.GetBytes($bytes)
        $rng.Dispose()
        return [System.BitConverter]::ToString($bytes).Replace('-', '').ToLower()
    }

    # ------------------------------------------------------------------ #
    # 3. Auto-generate missing secrets                                    #
    # ------------------------------------------------------------------ #
    $generated = [System.Collections.Generic.List[string]]::new()

    if ([string]::IsNullOrWhiteSpace((Get-EnvValue 'SESSION_SECRET')))
    {
        Set-EnvValue 'SESSION_SECRET' (New-SecretHex)
        $generated.Add('SESSION_SECRET')
    }

    if ([string]::IsNullOrWhiteSpace((Get-EnvValue 'JWT_SECRET')))
    {
        Set-EnvValue 'JWT_SECRET' (New-SecretHex)
        $generated.Add('JWT_SECRET')
    }

    if ($generated.Count -gt 0)
    {
        $script:envLines | Set-Content $EnvFile -Encoding utf8
        Write-Host "Generated and saved to ${EnvFile}: $($generated -join ', ')" -ForegroundColor Green
    }

    # ------------------------------------------------------------------ #
    # 4. Validate all required variables are present                      #
    # ------------------------------------------------------------------ #
    $required = @(
        'GITHUB_APP_ID',
        'GITHUB_APP_PRIVATE_KEY',
        'GITHUB_CLIENT_ID',
        'GITHUB_CLIENT_SECRET',
        'GITHUB_ORG'
    )

    # Also accept values already present in the shell environment so that
    # secrets like GITHUB_APP_PRIVATE_KEY can be passed via $env: rather than
    # written to the file.
    $missing = $required | Where-Object {
        [string]::IsNullOrWhiteSpace((Get-EnvValue $_)) -and
        [string]::IsNullOrWhiteSpace([System.Environment]::GetEnvironmentVariable($_))
    }

    if ($missing)
    {
        Write-Host ''
        Write-Host 'The following required variables are not set in .env:' -ForegroundColor Red
        $missing | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
        Write-Host ''
        Write-Host 'Fill in the missing values and run this script again.' -ForegroundColor Yellow
        Write-Host 'See docs/configuration-guide.md#prerequisites for setup instructions.' -ForegroundColor Yellow
        exit 1
    }

    # ------------------------------------------------------------------ #
    # 5. Launch the stack                                                 #
    # ------------------------------------------------------------------ #
    $composeArgs = @('compose', '--env-file', $EnvFile, 'up')
    if (-not $NoBuild)
    {
        $composeArgs += '--build'
    }
    if ($Detach)
    {
        $composeArgs += '--detach'
    }

    Write-Host ''
    Write-Host 'Starting RepoRoller stack...' -ForegroundColor Cyan
    Write-Host '  Frontend : http://localhost:3001' -ForegroundColor Cyan
    Write-Host '  Backend  : http://localhost:8080/health' -ForegroundColor Cyan
    Write-Host ''

    & docker @composeArgs
    exit $LASTEXITCODE

}
finally
{
    Pop-Location
}
