# Script to replace Error with SystemError in git.rs

$gitFile = "crates/repo_roller_core/src/git.rs"
$content = Get-Content $gitFile -Raw

# Replace all Result<T, Error> with Result<T, SystemError>
$content = $content -replace 'Result<([^,>]+), Error>', 'Result<$1, SystemError>'
$content = $content -replace 'Result<\(\), Error>', 'Result<(), SystemError>'

# Replace Error::GitOperation with SystemError::GitOperation
# Pattern: Error::GitOperation(format!("Failed to <operation>: {}", e))
# Becomes: SystemError::GitOperation { operation: "<operation>".to_string(), reason: e.to_string() }

$content = $content -replace 'Error::GitOperation\(format!\("Failed to get repository index: \{\}", e\)\)', 'SystemError::GitOperation { operation: "get repository index".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to add files to index: \{\}", e\)\)', 'SystemError::GitOperation { operation: "add files to index".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to write tree: \{\}", e\)\)', 'SystemError::GitOperation { operation: "write tree".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to find tree: \{\}", e\)\)', 'SystemError::GitOperation { operation: "find tree".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to create signature: \{\}", e\)\)', 'SystemError::GitOperation { operation: "create signature".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to create commit: \{\}", e\)\)', 'SystemError::GitOperation { operation: "create commit".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to open repository: \{\}", e\)\)', 'SystemError::GitOperation { operation: "open repository".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to initialize git repository: \{\}", e\)\)', 'SystemError::GitOperation { operation: "initialize git repository".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to open git repository: \{\}", e\)\)', 'SystemError::GitOperation { operation: "open git repository".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to delete existing origin remote: \{\}", e\)\)', 'SystemError::GitOperation { operation: "delete existing origin remote".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(format!\("Failed to add remote origin: \{\}", e\)\)', 'SystemError::GitOperation { operation: "add remote origin".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::GitOperation\(detailed_error\)', 'SystemError::GitOperation { operation: "push to origin".to_string(), reason: detailed_error }'

# Save the file
Set-Content $gitFile -Value $content -NoNewline

Write-Host "Updated function signatures and error construction in git.rs"
