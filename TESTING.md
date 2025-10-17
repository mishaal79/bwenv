# Testing Guide for bwenv

This document provides comprehensive guidance on testing the bwenv project, including architecture, best practices, and troubleshooting.

## Table of Contents

- [Test Architecture](#test-architecture)
- [Running Tests](#running-tests)
- [Test Organization](#test-organization)
- [Writing Tests](#writing-tests)
- [Trait-Based Testing](#trait-based-testing)
- [Security Testing](#security-testing)
- [Property-Based Testing](#property-based-testing)
- [CI/CD Integration](#cicd-integration)
- [Troubleshooting](#troubleshooting)

## Test Architecture

### Trait-Based Dependency Injection

The testing architecture uses the `SecretsProvider` trait to abstract interactions with Bitwarden Secrets Manager. This enables:

- **Production Code**: Uses `SdkProvider` to interact with the real Bitwarden SDK
- **Test Code**: Uses `MockProvider` for fast, deterministic, in-memory testing
- **Future Flexibility**: Easy to add HTTP mocking, database testing, etc.

```rust
#[async_trait]
pub trait SecretsProvider: Send + Sync {
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn get_secrets_map(&self, project_id: &str) -> Result<HashMap<String, String>>;
    async fn sync_secrets(&self, project_id: &str, secrets: &HashMap<String, String>, overwrite: bool) -> Result<Vec<Secret>>;
    // ... other methods
}
```

### Test Pyramid

1. **Unit Tests** (70%): Test individual functions and modules in isolation
   - Located in `src/` files with `#[cfg(test)]`
   - Fast, focused, no I/O

2. **Integration Tests** (25%): Test component interactions
   - Located in `tests/integration/`
   - Uses `MockProvider` for Bitwarden interactions
   - Tests real .env file operations

3. **Property-Based Tests** (5%): Verify invariants across random inputs
   - Located in `tests/property_tests.rs`
   - Uses `proptest` for exhaustive testing

4. **Security Tests**: Specialized tests for secrets leakage
   - Located in `tests/security/`
   - Critical for secrets management tool

## Running Tests

### Quick Test Commands

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run specific test file
cargo test --test push_tests

# Run with output
cargo test -- --nocapture

# Run single test
cargo test test_push_creates_new_secrets

# Run property tests (slower)
cargo test --test property_tests -- --test-threads=1
```

### Advanced Testing

```bash
# Run with nextest (faster parallel execution)
cargo nextest run

# Generate coverage report
cargo llvm-cov --all-features --html

# Run security audit
cargo audit

# Run benchmarks
cargo bench

# Check for test compilation errors without running
cargo test --no-run
```

### Test Filtering

```bash
# Run only push tests
cargo test push

# Run only security tests
cargo test --test security

# Run tests matching pattern
cargo test roundtrip

# Exclude slow tests
cargo test --exclude-tags=slow
```

## Test Organization

### Directory Structure

```
tests/
├── common/                    # Shared test utilities
│   ├── mod.rs                # Re-exports and type aliases
│   ├── fixtures.rs           # Test data builders and fixtures
│   └── helpers.rs            # Custom assertions and utilities
├── fixtures/                  # Static test .env files
│   ├── simple.env
│   ├── complex.env
│   ├── production.env
│   └── empty_values.env
├── integration/               # Integration test suites
│   ├── mod.rs
│   ├── push_tests.rs         # Push command tests
│   ├── pull_tests.rs         # Pull command tests
│   └── status_tests.rs       # Status/drift detection tests
├── security/                  # Security-focused tests
│   ├── mod.rs
│   └── secrets_leakage_tests.rs
├── property_tests.rs          # Property-based tests
├── file_operations_tests.rs   # File I/O tests
└── bitwarden_mock_tests.rs    # Mock provider tests
```

### Test File Naming

- Unit tests: `#[cfg(test)] mod tests { ... }` in source files
- Integration tests: `*_tests.rs` in `tests/` directory
- Fixtures: `*.env` in `tests/fixtures/`
- Test helpers: `common/*.rs`

## Writing Tests

### Using Test Fixtures

#### EnvFileBuilder Pattern

```rust
use common::EnvFileBuilder;

#[test]
fn test_example() {
    let (path, _temp_dir) = EnvFileBuilder::new()
        .entry("API_KEY", "secret123")
        .entry("DB_HOST", "localhost")
        .with_header(true)
        .build_temp()
        .unwrap();

    // Use path for testing
    // _temp_dir automatically cleans up when dropped
}
```

#### TestProject Pattern

```rust
use common::TestProject;

#[tokio::test]
async fn test_example() {
    let project = TestProject::new("My Project")
        .with_secret("API_KEY", "secret123")
        .with_secret("DB_PASSWORD", "pass456");

    let provider = project.provider();
    // Use provider for testing
}
```

### Custom Assertions

```rust
use common::{assert_env_files_equivalent, assert_no_secrets_leaked, assert_map_contains};

// Compare .env files ignoring order and comments
assert_env_files_equivalent(path1, path2);

// Verify secrets don't leak in output
assert_no_secrets_leaked(&error_message, &["secret123", "password456"]);

// Check HashMap contains expected values
assert_map_contains(&map, &[("KEY1", "value1"), ("KEY2", "value2")]);
```

### Environment Variable Testing

```rust
use common::EnvGuard;

#[test]
fn test_with_env_vars() {
    let guard = EnvGuard::new(&["TEST_VAR", "ANOTHER_VAR"]);

    // Modify environment
    guard.set("TEST_VAR", "test_value");

    // Test code here

    // Environment automatically restored when guard drops
}
```

## Trait-Based Testing

### Creating Mock Providers

```rust
use bwenv::bitwarden::{MockProvider, Project, Secret};

#[tokio::test]
async fn test_with_mock() {
    // Empty provider
    let provider = MockProvider::new();

    // Provider with data
    let project = Project {
        id: "proj_1".to_string(),
        name: "Test Project".to_string(),
        organization_id: "org_1".to_string(),
    };

    let secret = Secret {
        id: "sec_1".to_string(),
        key: "API_KEY".to_string(),
        value: "secret123".to_string(),
        note: None,
        project_id: project.id.clone(),
    };

    let provider = MockProvider::with_data(vec![project], vec![secret]);
}
```

### Testing Against the Trait

```rust
use bwenv::bitwarden::SecretsProvider;

async fn test_any_provider<P: SecretsProvider>(provider: &P, project_id: &str) {
    let secrets = provider.get_secrets_map(project_id).await.unwrap();
    assert_eq!(secrets.len(), 2);
}

#[tokio::test]
async fn test_with_mock_provider() {
    let project = TestProject::new("Test").with_secret("KEY", "value");
    test_any_provider(project.provider(), &project.project.id).await;
}
```

## Security Testing

### Testing for Secrets Leakage

Security tests ensure secrets never appear in:
- Debug output (`{:?}`)
- Display output (`{}`)
- Error messages
- Log output
- Temporary files (after cleanup)

```rust
#[tokio::test]
async fn test_no_secret_leakage() {
    let provider = MockProvider::new();
    let mut secrets = HashMap::new();
    secrets.insert("API_KEY".to_string(), "super_secret".to_string());

    let result = provider.sync_secrets("nonexistent", &secrets, false).await;

    let error_message = result.unwrap_err().to_string();
    assert_no_secrets_leaked(&error_message, &["super_secret"]);
}
```

### Best Practices

1. **Never log secret values**: Log only key names, counts, or metadata
2. **Use custom Debug traits**: Implement `Debug` to redact sensitive fields
3. **Clean up temp files**: Use `tempfile::TempDir` for automatic cleanup
4. **Test error messages**: Ensure errors don't expose secrets
5. **Audit dependencies**: Run `cargo audit` regularly

## Property-Based Testing

Property-based tests verify that invariants hold across thousands of randomly generated inputs.

### Key Properties Tested

1. **Roundtrip Property**: Write then read preserves data
2. **Merge Preserves Data**: Merging preserves both old and new keys
3. **Overwrite Correctness**: New values overwrite old ones in merge
4. **Parser Robustness**: Parser never panics on any input
5. **Provider Idempotency**: Operations can be repeated safely

### Writing Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_my_property(input in prop::string::string_regex("[A-Z]+").unwrap()) {
        // Test that property holds for all generated inputs
        let result = my_function(&input);
        prop_assert!(result.is_ok());
    }
}
```

### Running Property Tests

```bash
# Run with default 256 cases
cargo test --test property_tests

# Run with more cases (slower but more thorough)
PROPTEST_CASES=10000 cargo test --test property_tests

# Run with single thread (required for some tests)
cargo test --test property_tests -- --test-threads=1
```

## CI/CD Integration

### GitHub Actions Workflow

The `.github/workflows/test-enhanced.yml` workflow provides:

- **Matrix Testing**: Tests on Ubuntu, macOS, Windows
- **Rust Version Testing**: Stable, MSRV (1.85.0), Beta, Nightly
- **Fast Execution**: Uses `cargo-nextest` for parallel testing
- **Coverage Reports**: Generates code coverage with `cargo-llvm-cov`
- **Security Audits**: Runs `cargo-audit` on dependencies
- **Benchmarks**: Performance tracking on main branch

### Coverage Requirements

- **Unit Tests**: >80% coverage
- **Integration Tests**: Cover all public APIs
- **Critical Paths**: 100% coverage for security-sensitive code

### Viewing Coverage Reports

After CI runs:
1. Download coverage artifacts from GitHub Actions
2. Open `coverage/index.html` in browser
3. Review uncovered lines and add tests

## Troubleshooting

### Common Issues

#### Tests Failing with "Connection Refused"

**Cause**: Tests trying to connect to real Bitwarden server instead of mock.

**Solution**: Ensure tests use `MockProvider`:
```rust
let provider = MockProvider::new(); // Not SdkProvider
```

#### Flaky Tests on CI

**Cause**: Race conditions, timing issues, or shared state.

**Solutions**:
- Use `serial_test` crate for tests that must run sequentially
- Avoid shared global state
- Use unique temp directories for each test

```rust
#[tokio::test]
#[serial] // Ensures sequential execution
async fn test_that_needs_isolation() {
    // test code
}
```

#### Property Tests Taking Too Long

**Cause**: Default number of test cases (256) with complex properties.

**Solutions**:
```bash
# Reduce cases for quick iteration
PROPTEST_CASES=50 cargo test property_tests

# Or configure in test file
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn my_test(...) { }
}
```

#### "Too Many Open Files" Error

**Cause**: Tests creating many temp files/directories.

**Solution**: Ensure proper cleanup with `tempfile`:
```rust
let temp_dir = tempdir().unwrap(); // Auto-cleanup on drop
```

#### Tests Pass Locally but Fail on CI

**Causes**:
- Platform-specific behavior (Windows vs Unix line endings)
- Different Rust versions
- Missing dependencies

**Solutions**:
- Test on multiple platforms locally using Docker
- Pin Rust version in `rust-toolchain.toml`
- Run `cargo check --all-targets` before pushing

### Debugging Test Failures

```bash
# Show full output
cargo test -- --nocapture

# Show backtrace
RUST_BACKTRACE=1 cargo test

# Run specific test with logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Run tests in single thread for easier debugging
cargo test -- --test-threads=1
```

### Performance Profiling

```bash
# Profile test execution
cargo test --release -- --bench

# Generate flamegraph
cargo flamegraph --test integration_tests
```

## Best Practices Summary

1. ✅ **Use MockProvider for tests** - Fast, deterministic, no network
2. ✅ **Write focused tests** - 30-50 named tests, not 200 parameterized
3. ✅ **Test security** - Verify secrets never leak
4. ✅ **Use property tests** - Find edge cases automatically
5. ✅ **Clean up resources** - Use RAII patterns (tempfile, guards)
6. ✅ **Test error paths** - Don't just test happy path
7. ✅ **Keep tests fast** - Unit tests < 100ms, integration tests < 1s
8. ✅ **Document test intent** - Use descriptive names and comments
9. ✅ **Review coverage** - Aim for >80% overall coverage
10. ✅ **Run tests before commit** - `cargo test && cargo clippy`

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Proptest Documentation](https://docs.rs/proptest/)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [cargo-nextest](https://nexte.st/)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)

## End-to-End Testing

### Overview

E2E tests verify the complete bwenv workflow against a **real Bitwarden Secrets Manager instance**. Unlike unit tests (MockProvider) or integration tests (CLI parsing), E2E tests execute the actual binary and communicate with Bitwarden Cloud.

### Architecture

**Test Environment:**
- **Real Bitwarden Cloud**: Free tier Secrets Manager account
- **Docker Compose**: Orchestrates test execution and cleanup
- **TestContext**: Manages project lifecycle and test data

**Why Bitwarden Cloud (not self-hosted)?**
- ✅ Zero infrastructure overhead
- ✅ Tests production API (real-world conditions)
- ✅ Free tier available (no cost)
- ✅ Simple setup (one access token)
- ❌ Vaultwarden doesn't support Secrets Manager
- ❌ Self-hosted requires enterprise license + SQL Server

### Setup

#### 1. Create Bitwarden Account

```bash
# Sign up at https://vault.bitwarden.com
# Enable Secrets Manager (free tier)
# Create a test project named "E2E-Test"
```

#### 2. Generate Access Token

```bash
# Go to: Settings → Security → Access Tokens
# Create new token with Secrets Manager access
# Copy the token (only shown once)
```

#### 3. Configure Environment

```bash
# Copy template
cp .env.test.example .env.test

# Edit .env.test with your token
BITWARDEN_ACCESS_TOKEN=0.48b4774c-68ca-4539-a3d7-ac00018b4377.your_token_data_here
BITWARDEN_TEST_PROJECT=E2E-Test
```

**Important**: `.env.test` is gitignored. Never commit your access token.

### Running E2E Tests

#### Quick Start (Docker)

```bash
# Run all E2E tests in Docker
./scripts/run-e2e-tests.sh

# Output:
# === bwenv E2E Test Runner ===
# Mode: docker
# Project: E2E-Test
#
# Building and running E2E tests in Docker...
# [+] Running 2/2
#  ✔ Container bwenv-e2e-tests  Started
#  ✔ Container bwenv-cleanup    Started
#
# === Building bwenv release binary ===
# === Running E2E tests ===
#
# running 20 tests
# test test_push_basic_secrets ... ok
# test test_pull_basic_secrets ... ok
# test test_roundtrip_push_pull_integrity ... ok
# ...
# ✓ All E2E tests passed!
```

#### Local Execution (No Docker)

```bash
# Run tests locally (faster iteration)
./scripts/run-e2e-tests.sh --local

# Behind the scenes:
# 1. Builds release binary: cargo build --release
# 2. Exports BWENV_BINARY=./target/release/bwenv
# 3. Runs: cargo test --test e2e -- --test-threads=1 --nocapture
```

#### Cleanup Only

```bash
# Delete all test secrets from Bitwarden project
./scripts/run-e2e-tests.sh --cleanup
```

### Test Coverage

E2E tests verify **20 real-world scenarios** across all CLI commands:

#### Push Command (5 tests)
- ✅ Push basic secrets to new project
- ✅ Push with --overwrite (update existing)
- ✅ Push empty .env file (edge case)
- ✅ Push nonexistent file (error handling)
- ✅ Push to nonexistent project (error handling)

#### Pull Command (4 tests)
- ✅ Pull secrets to new .env file
- ✅ Pull with --force (overwrite existing file)
- ✅ Pull from empty project (edge case)
- ✅ Pull nonexistent project (error handling)

#### Roundtrip Tests (2 tests)
- ✅ Push → Pull → Verify data integrity
- ✅ Push → Update → Push → Pull (sync workflow)

#### List Command (2 tests)
- ✅ List all projects
- ✅ List specific project secrets

#### Validate Command (3 tests)
- ✅ Validate correct .env format
- ✅ Validate malformed .env (error detection)
- ✅ Validate nonexistent file (error handling)

#### Error Scenarios (4 tests)
- ✅ Invalid access token authentication
- ✅ Missing access token
- ✅ Network/API errors
- ✅ File permission errors

### Test Data Management

#### Setup (Before Each Test)

```rust
let mut ctx = TestContext::new().await?;
ctx.setup_project().await?;
// Creates unique test project: "bwenv-e2e-test-{timestamp}"
```

#### Teardown (After Each Test)

```rust
ctx.cleanup().await?;
// Deletes all secrets from test project
// Leaves project intact for reuse
```

#### Test Isolation

Each test:
1. Uses shared test project (manual creation required)
2. Creates unique secrets (timestamped keys)
3. Cleans up ALL secrets after test
4. No cross-test pollution

### File Structure

```
tests/
├── e2e/
│   ├── mod.rs              # Module exports
│   ├── setup.rs            # TestContext, setup/teardown (200 lines)
│   ├── cli_tests.rs        # 20 E2E test scenarios (400 lines)
│   └── fixtures/
│       ├── basic.env       # Simple test data
│       ├── complex.env     # Special chars, JSON, base64
│       └── empty.env       # Edge case testing
├── docker/
│   ├── docker-compose.e2e.yml  # Test orchestration (50 lines)
│   └── Dockerfile.e2e          # Test runner image (25 lines)
└── scripts/
    └── run-e2e-tests.sh        # Execution script (150 lines)
```

### TestContext API

```rust
use tests::e2e::setup::{TestContext, assert_env_files_equivalent};

#[tokio::test]
async fn test_example() -> TestResult<()> {
    // Setup
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // Create test data
    let mut vars = HashMap::new();
    vars.insert("KEY".to_string(), "value".to_string());
    let env_path = ctx.create_test_env("test.env", &vars)?;

    // Execute CLI command
    let output = Command::new("./target/release/bwenv")
        .env("BITWARDEN_ACCESS_TOKEN", ctx.access_token())
        .args(&["push", "--project", ctx.project_name(), "--input", env_path.to_str().unwrap()])
        .output()?;

    assert!(output.status.success());

    // Verify results
    let secrets = ctx.provider().get_secrets_map(ctx.project_id()?).await?;
    assert_eq!(secrets.get("KEY").unwrap(), "value");

    // Cleanup
    ctx.cleanup().await?;
    Ok(())
}
```

### Docker Compose Architecture

```yaml
services:
  e2e-tests:
    # Builds bwenv binary
    # Runs: cargo test --test e2e
    # Mounts project directory
    # Uses Bitwarden Cloud API

  cleanup:
    # Runs after tests (success or failure)
    # Deletes test data from Bitwarden
    # Ensures no pollution
```

**Volumes:**
- `cargo-cache`: Speeds up rebuilds (dependency caching)
- Project directory: Access to source code and binary

### Troubleshooting

#### Tests Fail: "BITWARDEN_ACCESS_TOKEN not set"

**Cause**: Missing or incorrect `.env.test` file

**Fix**:
```bash
cp .env.test.example .env.test
# Edit .env.test with real token
```

#### Tests Fail: "Test project 'E2E-Test' not found"

**Cause**: Test project doesn't exist in Bitwarden

**Fix**:
```bash
# 1. Log into Bitwarden Secrets Manager
# 2. Create project named "E2E-Test"
# 3. Or set custom name in .env.test:
BITWARDEN_TEST_PROJECT=YourProjectName
```

#### Tests Fail: "Authentication failed"

**Cause**: Invalid or expired access token

**Fix**:
```bash
# 1. Generate new access token in Bitwarden
# 2. Update .env.test with new token
# 3. Ensure token has Secrets Manager permissions
```

#### Docker Tests Slow

**Cause**: Rebuilding dependencies on each run

**Fix**:
```bash
# Use local mode for faster iteration
./scripts/run-e2e-tests.sh --local

# Or keep Docker containers running
docker-compose -f tests/docker/docker-compose.e2e.yml up --build
```

#### Test Data Not Cleaned Up

**Cause**: Test failure before cleanup() call

**Fix**:
```bash
# Manual cleanup
./scripts/run-e2e-tests.sh --cleanup

# Or use Bitwarden web UI to delete secrets
```

### CI/CD Integration

**GitHub Actions Example:**

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Create .env.test
        run: |
          echo "BITWARDEN_ACCESS_TOKEN=${{ secrets.BITWARDEN_TOKEN }}" > .env.test
          echo "BITWARDEN_TEST_PROJECT=CI-E2E-Test" >> .env.test

      - name: Run E2E Tests
        run: ./scripts/run-e2e-tests.sh --docker
```

**Secrets Configuration:**
- Store `BITWARDEN_ACCESS_TOKEN` in GitHub Secrets
- Use separate test project for CI (e.g., "CI-E2E-Test")
- Cleanup runs automatically via docker-compose

### Best Practices

1. **Always Cleanup**: Call `ctx.cleanup().await?` in every test
2. **Use TestContext**: Don't create manual SDK providers
3. **Unique Test Data**: Use timestamps or unique IDs for keys
4. **Test Isolation**: Don't rely on test execution order
5. **Error Assertions**: Verify error messages, not just status codes
6. **Realistic Data**: Use complex .env files (special chars, JSON, etc.)
7. **Separate Projects**: Use different projects for dev/CI/local testing

### Performance

**Typical Execution Times:**
- Single E2E test: ~2-5 seconds (network latency)
- Full suite (20 tests): ~60-90 seconds
- Docker build (first time): ~2 minutes
- Docker build (cached): ~30 seconds

**Optimization Tips:**
- Run `--local` for faster iteration during development
- Use `--test-threads=1` to avoid rate limiting
- Keep test data small (< 20 secrets per test)

## Contributing

When adding new features:

1. Write tests first (TDD)
2. Ensure >80% coverage for new code
3. Add integration tests for user-facing features
4. **Add E2E tests for new CLI commands**
5. Update this documentation if adding new test patterns
6. Run full test suite before submitting PR

For questions or issues with testing, please open an issue on GitHub.
