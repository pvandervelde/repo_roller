# Script to replace Error with SystemError/TemplateError in template_processing.rs

$file = "crates/repo_roller_core/src/template_processing.rs"
$content = Get-Content $file -Raw

# Update import
$content = $content -replace 'use crate::errors::Error;', 'use crate::errors::{SystemError, TemplateError};'

# Replace all Result<T, Error> with Result<T, SystemError>
$content = $content -replace 'Result<([^,>]+), Error>', 'Result<$1, SystemError>'
$content = $content -replace 'Result<\(\), Error>', 'Result<(), SystemError>'

# Replace Error::FileSystem with SystemError::FileSystem
# Most are single-line format patterns
$content = $content -replace 'Error::FileSystem\(\s*"([^"]+)"\s*\)', 'SystemError::FileSystem { operation: "file validation".to_string(), reason: "$1".to_string() }'

# Replace multiline FileSystem errors with format!
$content = $content -replace 'Error::FileSystem\(format!\("Failed to create directory \{:?\}: \{\}", parent, e\)\)', 'SystemError::FileSystem { operation: "create directory".to_string(), reason: format!("{:?}: {}", parent, e) }'
$content = $content -replace 'Error::FileSystem\(format!\("Failed to create file \{:?\}: \{\}", target_path, e\)\)', 'SystemError::FileSystem { operation: "create file".to_string(), reason: format!("{:?}: {}", target_path, e) }'
$content = $content -replace 'Error::FileSystem\(format!\("Failed to write to file \{:?\}: \{\}", target_path, e\)\)', 'SystemError::FileSystem { operation: "write file".to_string(), reason: format!("{:?}: {}", target_path, e) }'
$content = $content -replace 'Error::FileSystem\(format!\("Failed to create README\.md: \{\}", e\)\)', 'SystemError::FileSystem { operation: "create README.md".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::FileSystem\(format!\("Failed to create \.gitignore: \{\}", e\)\)', 'SystemError::FileSystem { operation: "create .gitignore".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::FileSystem\(format!\("Failed to read directory entry: \{\}", e\)\)', 'SystemError::FileSystem { operation: "read directory entry".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::FileSystem\(format!\("Failed to get relative path: \{\}", e\)\)', 'SystemError::FileSystem { operation: "get relative path".to_string(), reason: e.to_string() }'
$content = $content -replace 'Error::FileSystem\(format!\("Failed to read file \{:?\}: \{\}", file_path, e\)\)', 'SystemError::FileSystem { operation: "read file".to_string(), reason: format!("{:?}: {}", file_path, e) }'
$content = $content -replace 'Error::FileSystem\(format!\("Failed to remove file \{:?\}: \{\}", entry\.path\(\), e\)\)', 'SystemError::FileSystem { operation: "remove file".to_string(), reason: format!("{:?}: {}", entry.path(), e) }'

# Replace Error::TemplateProcessing with TemplateError
# For template errors, we use a generic "unknown" variable since we don't have the specific variable name
$content = $content -replace 'Error::TemplateProcessing\(format!\("Failed to create template processor: \{\}", e\)\)', 'TemplateError::SubstitutionFailed { variable: "template".to_string(), reason: format!("Failed to create template processor: {}", e) }'
$content = $content -replace 'Error::TemplateProcessing\(format!\("Template processing failed: \{\}", e\)\)', 'TemplateError::SubstitutionFailed { variable: "template".to_string(), reason: format!("Template processing failed: {}", e) }'

# Save the file
Set-Content $file -Value $content -NoNewline

Write-Host "Updated template_processing.rs"
