# Template: Binary Files

## Purpose

This template is used for **integration testing** to verify that binary files (images, PDFs, archives) are copied correctly without corruption during template processing.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_binary_file_preservation()`
- **Validates**:
  - Binary files transferred without corruption
  - No text substitution in binary files
  - Checksum verification
  - Proper handling of various binary formats

## Template Contents

- `test-image.png` - PNG image file
- `test-document.pdf` - PDF document
- `test-archive.zip` - ZIP archive
- `checksums.txt` - SHA256 checksums for validation
- `README.md` - This file

## Usage

This template is automatically used by the integration test suite when testing binary file preservation.

**Note**: This is a test fixture template and should not be used for creating production repositories.
