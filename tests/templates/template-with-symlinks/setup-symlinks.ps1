# Setup Symbolic Links for Testing (Windows)
# NOTE: Requires Administrator privileges

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# Check for admin privileges
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Error "This script requires Administrator privileges to create symlinks on Windows."
    Write-Host "Please run PowerShell as Administrator and try again."
    exit 1
}

# Create symlink to file
New-Item -ItemType SymbolicLink -Path "link-to-file.txt" -Target "real-file.txt" -Force | Out-Null

# Create directory and symlink to it
New-Item -ItemType Directory -Path "actual-dir" -Force | Out-Null
"Content in actual directory" | Out-File -FilePath "actual-dir\content.txt"
New-Item -ItemType SymbolicLink -Path "link-to-dir" -Target "actual-dir" -Force | Out-Null

Write-Host "✓ Symlinks created successfully"
