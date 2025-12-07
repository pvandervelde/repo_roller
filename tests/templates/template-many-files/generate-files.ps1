# Generate Many Test Files
# This script creates 1000+ files for testing template processing performance

$targetDir = Join-Path $PSScriptRoot "files"
New-Item -ItemType Directory -Path $targetDir -Force | Out-Null

Write-Host "Generating 1200 test files..."

for ($i = 1; $i -le 1200; $i++)
{
    $fileName = "test-file-{0:D4}.txt" -f $i
    $filePath = Join-Path $targetDir $fileName

    $content = @"
Test File #$i
Purpose: Template processing performance test
Generated: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

This file is part of a collection of 1200+ files used to test
template processing with large numbers of files.

Content Index: $i/1200
"@

    Set-Content -Path $filePath -Value $content

    if ($i % 100 -eq 0)
    {
        Write-Host "  Created $i files..."
    }
}

Write-Host "✓ Successfully generated 1200 test files in $targetDir"
