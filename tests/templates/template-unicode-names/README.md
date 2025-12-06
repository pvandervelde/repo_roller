# Template: Unicode Filenames

## Purpose

This template is used for **integration testing** to verify that filenames with Unicode characters, emojis, and special characters are handled correctly.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_unicode_filenames()`
- **Validates**:
  - Unicode characters in filenames
  - Emoji in filenames
  - Various script systems (Japanese, Cyrillic, etc.)
  - Accented characters
  - File accessibility after creation

## Template Contents

- `日本語.txt` - Japanese characters
- `файл.txt` - Cyrillic characters
- `test-😀-emoji.txt` - Emoji in filename
- `spëcîål-çhãrs.txt` - Accented characters
- `中文文件.txt` - Chinese characters
- `README.md` - This file

## Usage

This template is automatically used by the integration test suite when testing Unicode filename handling.

**Note**: This is a test fixture template and should not be used for creating production repositories.
