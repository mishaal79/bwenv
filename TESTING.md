# Testing Guide for bwenv

This document provides comprehensive information about the testing strategy, test types, and how to run tests for the bwenv project.

## Overview

The project uses a multi-layered testing approach to ensure reliability, correctness, and maintainability:

- **Unit Tests**: Test individual functions and modules in isolation
- **Integration Tests**: Test CLI commands and end-to-end functionality
- **Property-Based Tests**: Test with randomly generated inputs to catch edge cases
- **Mock Tests**: Test Bitwarden integration logic without requiring actual Bitwarden CLI

## Test Structure

```
tests/
├── integration_tests.rs       # CLI command integration tests
├── file_operations_tests.rs   # File I/O and parsing tests
├── property_tests.rs          # Property-based tests using proptest
└── bitwarden_mock_tests.rs    # Mock-based Bitwarden tests

src/
├── lib.rs                     # Library exports for testing
├── main.rs                    # CLI application entry point
├── env_file.rs               # Environment file operations (with unit tests)
├── error.rs                  # Error types (with unit tests)
├── logging.rs                # Logging configuration (with unit tests)
└── bitwarden.rs              # Bitwarden integration (with unit tests)
```

## Running Tests

### Quick Test Run

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

### Comprehensive Test Suite

Use the provided test script for a complete test run:

```bash
./test.sh
```

This script will:
- Check code formatting
- Run clippy linting
- Build the project
- Run all test categories
- Generate coverage reports (if tarpaulin is installed)
- Run security audits (if cargo-audit is installed)

### Individual Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration_tests

# File operation tests
cargo test --test file_operations_tests

# Property-based tests
cargo test --test property_tests

# Mock-based tests
cargo test --test bitwarden_mock_tests
```

### Specific Test Functions

```bash
# Run a specific test
cargo test test_read_env_file_basic

# Run tests matching a pattern
cargo test env_file

# Run tests in a specific module
cargo test env_file::tests
```

## Test Categories

### 1. Unit Tests

Located within each source file using `#[cfg(test)]` modules.

#### env_file.rs Tests
- `test_read_env_file_basic`: Basic .env file parsing
- `test_read_env_file_with_spaces`: Handling whitespace around keys/values
- `test_read_env_file_empty_lines_and_comments`: Comment and empty line handling
- `test_write_env_file_basic`: Basic file writing functionality
- `test_write_env_file_merge_*`: Testing merge vs overwrite behavior
- `test_validate_env_file_*`: File format validation

#### error.rs Tests
- Error type display formatting
- Error conversion from standard types
- Error trait implementations

#### logging.rs Tests
- Verbosity level conversion
- Log directory path generation
- Log file rotation logic
- Cross-platform compatibility

#### bitwarden.rs Tests
- JSON parsing and serialization
- Environment variable format conversion
- Data structure validation

### 2. Integration Tests

Test the complete CLI application functionality.

## Command Line Interface
- Help text generation
- Argument parsing
- Subcommand functionality
- Flag combinations

#### Error Handling
- Invalid file paths
- Missing arguments
- Bitwarden CLI availability

#### File Operations
- Reading various .env file formats
- Writing output files
- Merge operations

### 3. Property-Based Tests

Use `proptest` to generate random inputs and test invariants.

#### Key Features Tested
- **Roundtrip Property**: `write_env_file` → `read_env_file` should preserve data
- **Merge Consistency**: Merging operations should be predictable
- **Sorting Consistency**: Output should be deterministically sorted
- **Validation Invariants**: Generated files should always validate

#### Test Strategies
- Random environment variable generation
- Unicode and special character handling
- Large file handling
- Edge cases with whitespace and formatting

### 4. Mock Tests

Test Bitwarden integration logic without requiring actual Bitwarden CLI.

#### Features Tested
- JSON response parsing
- Command argument construction
- Error response handling
- Data transformation logic

## Test Data

### Sample Environment Files

The tests use various .env file formats:

```env
# Basic format
KEY1=value1
KEY2=value2

# With comments and empty lines
# Database configuration
DB_HOST=localhost
DB_PORT=5432

# API settings
API_KEY=secret123

# Complex values
URL=https://api.example.com/v1?key=value&other=data
JSON_CONFIG={"key":"value","nested":{"inner":"data"}}
```

### Mock Bitwarden Responses

```json
{
  "id": "item-123",
  "name": "Environment Variables",
  "notes": "DB_HOST=localhost\nDB_PORT=5432\nAPI_KEY=secret123",
  "type": 2,
  "folderId": "folder-456"
}
```

## Coverage

To generate coverage reports, install and run tarpaulin:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/tarpaulin-report.html
```

Current coverage targets:
- **Unit Tests**: >90% line coverage
- **Integration Tests**: All CLI commands and major error paths
- **Property Tests**: Edge cases and invariant validation

## Testing Best Practices

### 1. Test Isolation
- Each test should be independent
- Use `tempfile` for temporary files
- Clean up resources after tests

### 2. Descriptive Names
- Test names should describe what is being tested
- Include expected behavior in test names
- Group related tests in modules

### 3. Comprehensive Assertions
- Test both success and failure cases
- Verify error messages and types
- Check edge cases and boundary conditions

### 4. Mock External Dependencies
- Mock Bitwarden CLI calls
- Use dependency injection where possible
- Test error conditions from external services

## Continuous Integration

### GitHub Actions Workflow

```yaml
- name: Run Tests
  run: |
    cargo test --verbose
    cargo test --test integration_tests
    cargo test --test file_operations_tests
    cargo test --test property_tests
    cargo test --test bitwarden_mock_tests
```

### Test Matrix
- **Rust Versions**: stable, beta, nightly
- **Operating Systems**: Ubuntu, macOS, Windows
- **Features**: default, all features

## Debugging Tests

### Running with Debug Output

```bash
# Show println! output
cargo test -- --nocapture

# Show log output
RUST_LOG=debug cargo test

# Run specific test with output
cargo test test_name -- --nocapture
```

### Test Debugging Tips

1. **Use `dbg!()` macro** for temporary debugging
2. **Add temporary `println!()` statements**
3. **Run single tests** to isolate issues
4. **Check test file locations** with `tempfile` debug output

## Performance Testing

### Benchmarks

While not included in the current test suite, consider adding benchmarks for:
- Large .env file parsing
- Bitwarden API response processing
- File I/O operations

```bash
# Future benchmark command
cargo bench
```

## Security Testing

### Audit Dependencies

```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit
```

### Testing Security Features

- Verify sensitive data is not logged
- Test file permission handling
- Validate input sanitization

## Contributing Tests

When adding new features:

1. **Write tests first** (TDD approach)
2. **Add unit tests** for new functions
3. **Update integration tests** for CLI changes
4. **Add property tests** for data transformations
5. **Update mock tests** for Bitwarden integration
6. **Document test cases** in this file

### Test Review Checklist

- [ ] All new code has corresponding tests
- [ ] Tests cover both success and failure cases
- [ ] Property-based tests cover edge cases
- [ ] Integration tests verify CLI behavior
- [ ] Mock tests don't require external dependencies
- [ ] Tests are properly documented
- [ ] Test names are descriptive and clear

## Troubleshooting

### Common Test Failures

#### "Bitwarden CLI not found"
- Expected for integration tests without Bitwarden CLI
- Install Bitwarden CLI or skip integration tests
- Use mock tests for CI/CD environments

#### Permission Denied Errors
- Check file permissions in test directories
- Use `tempfile` for test file creation
- Run with appropriate user permissions

#### Flaky Tests
- Check for timing dependencies
- Ensure test isolation
- Use deterministic test data

### Getting Help

1. Check test output for specific error messages
2. Run individual test categories to isolate issues
3. Use `--nocapture` flag for detailed output
4. Review this documentation for guidance
5. Check the project's issue tracker for known problems

## Future Improvements

- Add benchmark tests for performance regression detection
- Implement mutation testing for test quality validation
- Add end-to-end tests with real Bitwarden vault
- Create visual test reports and dashboards
- Implement parallel test execution optimization