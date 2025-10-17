# E2E Testing for bwenv

## Quick Start

```bash
# 1. Setup (one-time)
cp ../../.env.test.example ../../.env.test
# Edit .env.test with your Bitwarden access token

# 2. Create test project in Bitwarden web UI
# Project name: "E2E-Test" (or customize in .env.test)

# 3. Run tests
../../scripts/run-e2e-tests.sh
```

## What Gets Tested

These E2E tests execute the **real bwenv binary** against a **real Bitwarden Secrets Manager instance**. They verify:

- ✅ **Push**: Upload .env secrets to Bitwarden
- ✅ **Pull**: Download secrets from Bitwarden to .env
- ✅ **List**: Display projects and secrets
- ✅ **Validate**: Check .env file format
- ✅ **Roundtrip**: Data integrity (push → pull → compare)
- ✅ **Error Handling**: Invalid tokens, missing projects, file errors

## Test Coverage

**20 comprehensive test scenarios:**

| Category | Tests | Description |
|----------|-------|-------------|
| Push | 5 | Basic push, overwrite, empty files, error cases |
| Pull | 4 | Basic pull, force overwrite, empty projects |
| Roundtrip | 2 | Data integrity, sync workflows |
| List | 2 | Project listing, secret enumeration |
| Validate | 3 | Format validation, error detection |
| Errors | 4 | Auth failures, network errors, invalid input |

## Architecture

```
TestContext (setup.rs)
    ↓
Real Bitwarden SDK
    ↓
Bitwarden Cloud API
    ↓
Your Test Project
```

**Key Components:**

1. **TestContext**: Manages test lifecycle
   - Creates unique test data
   - Provides helper methods
   - Handles cleanup

2. **CLI Execution**: Runs actual binary
   ```rust
   Command::new("./target/release/bwenv")
       .env("BITWARDEN_ACCESS_TOKEN", token)
       .args(&["push", ...])
   ```

3. **Assertions**: Verify results
   - File content comparison
   - API data verification
   - Error message validation

## File Structure

```
tests/e2e/
├── README.md           # This file
├── mod.rs              # Module exports
├── setup.rs            # TestContext implementation
├── cli_tests.rs        # 20 E2E test scenarios
└── fixtures/           # Test data
    ├── basic.env       # Simple variables
    ├── complex.env     # Special characters, JSON
    └── empty.env       # Edge case testing
```

## Running Tests

### Local (Fast Iteration)

```bash
# Build once
cargo build --release

# Run tests
./scripts/run-e2e-tests.sh --local

# Run specific test
cargo test --test e2e test_push_basic_secrets -- --nocapture
```

### Docker (CI-like Environment)

```bash
# Full Docker execution
./scripts/run-e2e-tests.sh --docker

# Uses docker-compose to:
# 1. Build test container
# 2. Run cargo test
# 3. Clean up resources
```

### Cleanup Only

```bash
# Delete all test secrets from Bitwarden
./scripts/run-e2e-tests.sh --cleanup
```

## Writing New E2E Tests

```rust
#[tokio::test]
async fn test_my_feature() -> TestResult<()> {
    // 1. Setup
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // 2. Prepare test data
    let mut vars = HashMap::new();
    vars.insert("MY_KEY".to_string(), "my_value".to_string());
    let env_path = ctx.create_test_env("test.env", &vars)?;

    // 3. Execute CLI command
    let output = run_bwenv(&ctx, &[
        "push",
        "--project", ctx.project_name(),
        "--input", env_path.to_str().unwrap(),
    ])?;

    // 4. Assert results
    assert!(output.status.success());
    let secrets = ctx.provider().get_secrets_map(ctx.project_id()?).await?;
    assert_eq!(secrets.get("MY_KEY").unwrap(), "my_value");

    // 5. Cleanup
    ctx.cleanup().await?;
    Ok(())
}
```

## Troubleshooting

### "BITWARDEN_ACCESS_TOKEN not set"

**Solution**: Create `.env.test` with your token:
```bash
cp .env.test.example .env.test
# Edit with real token
```

### "Test project 'E2E-Test' not found"

**Solution**: Create project in Bitwarden Secrets Manager web UI, or customize name:
```bash
# In .env.test
BITWARDEN_TEST_PROJECT=YourProjectName
```

### "Authentication failed"

**Solution**: Generate new access token in Bitwarden:
1. Go to Settings → Security → Access Tokens
2. Create new token with Secrets Manager access
3. Update `.env.test`

### Tests Leave Secrets in Bitwarden

**Solution**: Run cleanup manually:
```bash
./scripts/run-e2e-tests.sh --cleanup
```

Or use Bitwarden web UI to delete test secrets.

## Best Practices

1. **Always cleanup**: Call `ctx.cleanup().await?` in every test
2. **Unique data**: Use timestamps or UUIDs for test keys
3. **Test isolation**: Don't depend on test execution order
4. **Realistic scenarios**: Test complex .env files with special characters
5. **Error paths**: Verify error messages, not just exit codes
6. **Separate projects**: Use different projects for dev/CI/local

## Performance

- **Single test**: ~2-5 seconds (includes network latency)
- **Full suite (20 tests)**: ~60-90 seconds
- **Docker build (first time)**: ~2 minutes
- **Docker build (cached)**: ~30 seconds

**Optimization**: Use `--local` mode for faster development iteration.

## CI/CD Integration

See `TESTING.md` for GitHub Actions configuration example.

**Key points:**
- Store `BITWARDEN_ACCESS_TOKEN` in CI secrets
- Use separate test project for CI
- Run in Docker for consistency

## Security

- ⚠️ **Never commit `.env.test`** (contains access token)
- ✅ `.env.test` is gitignored automatically
- ✅ Test cleanup prevents data leakage
- ✅ Use separate project for testing (isolate from production)

## Support

For issues or questions:
- Check `TESTING.md` for comprehensive documentation
- Open issue on GitHub
- Review test logs with `--nocapture` flag
