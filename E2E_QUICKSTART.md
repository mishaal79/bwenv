# E2E Testing Quick Start Guide

## 🎯 Goal

Run comprehensive end-to-end tests against **real Bitwarden Secrets Manager** to verify bwenv works correctly in production-like conditions.

## ⚡ 5-Minute Setup

### Step 1: Create Bitwarden Account (if needed)

```bash
# 1. Go to https://vault.bitwarden.com
# 2. Sign up or log in
# 3. Enable Secrets Manager (free tier available)
```

### Step 2: Create Test Project

```bash
# In Bitwarden Secrets Manager web UI:
# 1. Click "New Project"
# 2. Name it "E2E-Test"
# 3. Save
```

### Step 3: Generate Access Token

```bash
# In Bitwarden:
# 1. Go to Settings → Security → Access Tokens
# 2. Click "New Access Token"
# 3. Name: "bwenv-e2e-tests"
# 4. Select your organization
# 5. Check "Secrets Manager" permission
# 6. Copy the token (only shown once!)
```

### Step 4: Configure bwenv

```bash
# In your bwenv project directory:
cp .env.test.example .env.test

# Edit .env.test:
BITWARDEN_ACCESS_TOKEN=0.48b4774c-xxxx-xxxx-xxxx-xxxxxxxxxxxx.your_token_data
BITWARDEN_TEST_PROJECT=E2E-Test
```

### Step 5: Run Tests

```bash
# Option A: Docker (recommended for CI)
./scripts/run-e2e-tests.sh

# Option B: Local (faster for development)
./scripts/run-e2e-tests.sh --local
```

## ✅ Expected Output

```
=== bwenv E2E Test Runner ===

Mode: docker
Project: E2E-Test

Building and running E2E tests in Docker...

=== Building bwenv release binary ===
    Finished release [optimized] target(s) in 38.42s

=== Running E2E tests ===

running 20 tests
test test_push_basic_secrets ... ok
test test_pull_basic_secrets ... ok
test test_roundtrip_push_pull_integrity ... ok
test test_push_overwrite_existing_secrets ... ok
test test_pull_force_overwrite ... ok
test test_push_empty_env_file ... ok
test test_pull_empty_project ... ok
test test_push_nonexistent_file ... ok
test test_push_nonexistent_project ... ok
test test_pull_nonexistent_project ... ok
test test_list_projects ... ok
test test_list_specific_project ... ok
test test_validate_valid_env_file ... ok
test test_validate_invalid_env_file ... ok
test test_validate_nonexistent_file ... ok
test test_invalid_access_token ... ok
test test_missing_access_token ... ok
test test_roundtrip_update_workflow ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

✓ All E2E tests passed!
```

## 🧪 What Gets Tested

### CLI Commands (100% Coverage)

| Command | Tests | Status |
|---------|-------|--------|
| `push` | 5 scenarios | ✅ |
| `pull` | 4 scenarios | ✅ |
| `list` | 2 scenarios | ✅ |
| `validate` | 3 scenarios | ✅ |
| Roundtrip | 2 workflows | ✅ |
| Error handling | 4 cases | ✅ |

### Real-World Scenarios

- ✅ Upload secrets from `.env` to Bitwarden
- ✅ Download secrets from Bitwarden to `.env`
- ✅ Sync workflow (push → modify → push → pull)
- ✅ Data integrity (push → pull → verify byte-for-byte)
- ✅ Complex values (JSON, base64, special chars, URLs)
- ✅ Error handling (auth failures, missing files, network errors)

## 🐳 Docker vs Local

### Docker Mode (Default)

```bash
./scripts/run-e2e-tests.sh
```

**Pros:**
- ✅ Consistent environment (CI-like)
- ✅ Automatic cleanup
- ✅ No local Rust setup needed

**Cons:**
- ⏱️ Slower (rebuilds container)
- 💾 Requires Docker

**Best for:** CI/CD, final validation before commit

### Local Mode

```bash
./scripts/run-e2e-tests.sh --local
```

**Pros:**
- ⚡ Fast iteration (no container rebuild)
- 🔍 Better debugging output
- 💻 Direct binary execution

**Cons:**
- 📦 Requires local Rust toolchain
- 🧹 Manual cleanup if tests fail

**Best for:** Development, debugging, rapid testing

## 🛠️ Troubleshooting

### Error: "BITWARDEN_ACCESS_TOKEN not set"

**Cause**: Missing or empty `.env.test` file

**Fix**:
```bash
# Check if .env.test exists
ls -la .env.test

# If missing, copy template
cp .env.test.example .env.test

# Edit with real token
vim .env.test
```

### Error: "Test project 'E2E-Test' not found"

**Cause**: Project doesn't exist in Bitwarden

**Fix**:
```bash
# Option 1: Create project in Bitwarden web UI named "E2E-Test"

# Option 2: Use existing project name
echo "BITWARDEN_TEST_PROJECT=YourExistingProject" >> .env.test
```

### Error: "Authentication failed"

**Cause**: Invalid or expired access token

**Fix**:
```bash
# Generate new token in Bitwarden:
# Settings → Security → Access Tokens → New Access Token

# Update .env.test with new token
```

### Error: "Permission denied: ./scripts/run-e2e-tests.sh"

**Cause**: Script not executable

**Fix**:
```bash
chmod +x scripts/run-e2e-tests.sh
```

### Docker build fails

**Cause**: Docker not running or insufficient resources

**Fix**:
```bash
# Check Docker status
docker ps

# Start Docker Desktop (macOS/Windows)
# Or start Docker daemon (Linux)

# Try local mode instead
./scripts/run-e2e-tests.sh --local
```

### Tests pass but secrets remain in Bitwarden

**Cause**: Cleanup didn't run (test failure or interruption)

**Fix**:
```bash
# Run cleanup manually
./scripts/run-e2e-tests.sh --cleanup

# Or delete manually in Bitwarden web UI
```

## 📊 Performance

| Metric | Docker | Local |
|--------|--------|-------|
| First run | ~3-4 min | ~2 min |
| Subsequent runs | ~1.5 min | ~1 min |
| Single test | ~5 sec | ~3 sec |

**Tips for Speed:**
- Use `--local` for development
- Run specific tests: `cargo test --test e2e test_push_basic_secrets`
- Keep Docker volumes (automatic caching)

## 🔒 Security

### What's Safe

- ✅ `.env.test` is gitignored (never committed)
- ✅ Test secrets are isolated in dedicated project
- ✅ Automatic cleanup prevents data leakage
- ✅ Access tokens are scoped to Secrets Manager only

### Best Practices

1. **Use dedicated test account** (not production Bitwarden account)
2. **Use separate test project** (never use production project)
3. **Rotate tokens regularly** (generate new token monthly)
4. **Limit token permissions** (Secrets Manager only, specific organization)
5. **Clean up after failures** (run `--cleanup` if tests interrupted)

### What NOT to Do

- ❌ Don't commit `.env.test` (contains access token)
- ❌ Don't use production Bitwarden project
- ❌ Don't share access tokens (each dev should generate own)
- ❌ Don't run E2E tests against production data

## 📚 Next Steps

### Run Specific Test

```bash
# Build first
cargo build --release

# Run single test
cargo test --test e2e test_roundtrip_push_pull_integrity -- --nocapture
```

### Debug Test Failure

```bash
# Enable verbose logging
export RUST_LOG=debug

# Run with full output
cargo test --test e2e -- --nocapture

# Or in Docker
RUST_LOG=debug ./scripts/run-e2e-tests.sh
```

### Add New E2E Test

See `tests/e2e/README.md` for guide on writing new tests.

Example:
```rust
#[tokio::test]
async fn test_my_feature() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // Your test logic here

    ctx.cleanup().await?;
    Ok(())
}
```

### CI/CD Integration

See `TESTING.md` for GitHub Actions configuration.

**Key steps:**
1. Add `BITWARDEN_ACCESS_TOKEN` to GitHub Secrets
2. Create dedicated CI test project
3. Run in Docker for consistency

## 🆘 Support

### Documentation

- **Quick Start**: `E2E_QUICKSTART.md` (this file)
- **Comprehensive Guide**: `TESTING.md` (E2E section)
- **E2E Details**: `tests/e2e/README.md`

### Getting Help

1. Check `TESTING.md` troubleshooting section
2. Run with `--nocapture` for detailed output
3. Enable debug logging: `RUST_LOG=debug`
4. Open GitHub issue with:
   - Error message
   - Test output
   - Environment (Docker/local, OS, Rust version)

## ✨ Success Checklist

- [ ] Created Bitwarden account with Secrets Manager
- [ ] Created test project "E2E-Test"
- [ ] Generated access token
- [ ] Created `.env.test` with token
- [ ] Ran `./scripts/run-e2e-tests.sh` successfully
- [ ] All 20 tests passed
- [ ] Test secrets cleaned up from Bitwarden

**If all checked**: Congratulations! Your E2E testing is fully configured. 🎉
