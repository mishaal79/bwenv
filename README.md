# bwenv

**Secure `.env` file management for developers who hate complexity.**

A fast, simple CLI built on Bitwarden Secrets Manager SDK. Sync your environment variables across teams without the enterprise overhead.

[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Bitwarden SDK](https://img.shields.io/badge/powered%20by-Bitwarden%20SDK-175DDC)](https://bitwarden.com/products/secrets-manager/)

## Why bwenv?

- 🚀 **5-minute setup** (vs 30min - 4 hours for competitors)
- 🆓 **Free tier that works** (full features, forever)
- 🔍 **Drift detection** (see exactly what's out of sync)
- 🎯 **6 commands** (vs 15-20 for enterprise tools)
- 🔒 **Zero-knowledge encryption** (via Bitwarden)

## Features

### ✅ Simple .env Workflow
- **Push** local `.env` to Bitwarden
- **Pull** secrets from Bitwarden to `.env`
- **Status** command shows drift (local vs remote)
- **Validate** .env format before pushing

### ✅ Built on Bitwarden Secrets Manager
- Uses official Rust SDK (not CLI wrapper)
- Leverages Bitwarden's battle-tested security
- Works with Bitwarden Cloud or self-hosted
- End-to-end encryption (zero-knowledge)

### ✅ Developer Experience
- Single 3.3MB binary (no dependencies)
- Works offline (after initial pull)
- Git-friendly (never commit secrets)
- Drift detection (know what changed)
- 72 unit tests + 20 E2E tests

### ✅ Team Collaboration
- Share secrets via Bitwarden projects
- Onboard teammates with one command
- Organization support (via Bitwarden orgs)
- RBAC through Bitwarden (if self-hosted)

## Prerequisites

- **Bitwarden Account** - [Sign up free](https://vault.bitwarden.com)
- **Secrets Manager Access** - [Enable free tier](https://bitwarden.com/products/secrets-manager/)
- **Access Token** - Generate from Bitwarden Settings → Security

**No Bitwarden CLI needed!** bwenv uses the official Rust SDK directly.

## Installation

### Cargo (Recommended)

```bash
cargo install bwenv
```

### Homebrew (macOS/Linux) - Coming Soon

```bash
brew install bwenv
```

### From Source

```bash
git clone https://github.com/yourusername/bwenv.git
cd bwenv
cargo build --release
sudo cp target/release/bwenv /usr/local/bin/
```

### Binary Releases

Download from [GitHub Releases](https://github.com/yourusername/bwenv/releases)

## Quick Start (5 minutes)

### 1. Get Bitwarden Access Token

```bash
# Go to: https://vault.bitwarden.com
# Settings → Security → Access Tokens → New Access Token
# Name: "bwenv-dev"
# Permissions: ✓ Secrets Manager
# Copy the token (shown only once!)
```

### 2. Export Token

```bash
export BITWARDEN_ACCESS_TOKEN="0.48b4774c-xxxx-xxxx-xxxx.your_token_data_here"

# Add to shell profile for persistence
echo 'export BITWARDEN_ACCESS_TOKEN="your_token"' >> ~/.bashrc  # or ~/.zshrc
```

### 3. Create Project in Bitwarden

1. Go to [Bitwarden Secrets Manager](https://vault.bitwarden.com)
2. Click **"New Project"**
3. Name it (e.g., "MyApp")
4. Save

### 4. Push Your Secrets

```bash
# Upload your .env to Bitwarden
bwenv push --project MyApp

# Output:
# ✓ Successfully pushed 12 secrets to project 'MyApp'
```

### 5. Team Onboarding (One Command!)

```bash
# New teammate pulls secrets
bwenv pull --project MyApp

# Done! They have the .env file with all secrets
```

## Commands

### `pull` - Download Secrets

Pull secrets from Bitwarden to local `.env` file:

```bash
# Basic usage
bwenv pull --project MyApp

# Custom output file
bwenv pull --project MyApp --output .env.production

# Overwrite existing file (skip confirmation)
bwenv pull --project MyApp --force
```

**Options:**
- `-p, --project <PROJECT>` - Project name or ID (required)
- `-o, --output <OUTPUT>` - Output file path (default: `.env`)
- `--force` - Overwrite existing file without prompting

---

### `push` - Upload Secrets

Push local `.env` file to Bitwarden:

```bash
# Basic usage
bwenv push --project MyApp

# Custom input file
bwenv push --project MyApp --input .env.staging

# Overwrite existing secrets in Bitwarden
bwenv push --project MyApp --overwrite
```

**Options:**
- `-p, --project <PROJECT>` - Project name or ID (required)
- `-i, --input <INPUT>` - Input .env file (default: `.env`)
- `--overwrite` - Overwrite existing secrets (default: creates new)

---

### `status` - Check Drift

See exactly what's different between local and remote:

```bash
# Check sync status
bwenv status --project MyApp

# Example output:
# 🔍 Checking sync status...
# 📦 Project: MyApp (a1b2c3d4-...)
#
# ⚠️  Out of sync detected:
#
# 📥 Only in Bitwarden (2):
#    - NEW_API_KEY
#    - FEATURE_FLAG_X
#    → Run 'bwenv pull' to download these
#
# 📤 Only in local .env (1):
#    - LOCAL_DEBUG_MODE
#    → Run 'bwenv push' to upload these
#
# 🔄 Different values (1):
#    - DATABASE_URL
#    → Run 'bwenv pull --force' to overwrite local
#    → Run 'bwenv push --overwrite' to overwrite remote
```

**Options:**
- `-p, --project <PROJECT>` - Project name or ID (required)
- `-e, --env-file <FILE>` - Custom .env file to compare (default: `.env`)

---

### `list` - Show Projects

List all projects and secrets:

```bash
# List all projects
bwenv list

# Output:
# Projects:
#   MyApp (a1b2c3d4-5678-...)
#   StagingEnv (b2c3d4e5-6789-...)
#
# Use 'bwenv list --project <name>' to see secrets

# List secrets in a specific project
bwenv list --project MyApp

# Output:
# Project: MyApp (a1b2c3d4-...)
#
# Secrets:
#   DATABASE_URL = <hidden>
#   API_KEY = <hidden>
#   REDIS_URL = <hidden>
```

**Options:**
- `-p, --project <PROJECT>` - Show secrets in specific project (optional)

---

### `init` - Initialize Config

Create `.bwenv.toml` configuration file:

```bash
bwenv init

# Output:
# ✓ Created .bwenv.toml configuration file
#
# Next steps:
#   1. Edit .bwenv.toml and set your default project
#   2. Run 'bwenv push' to upload your .env to Bitwarden
#   3. Add .bwenv.toml to git (safe to commit)
#   4. Add .env to .gitignore (contains secrets)
```

**Generated `.bwenv.toml`:**
```toml
# bwenv Configuration
default_project = "MyProject"
env_file = ".env"
auto_sync = false
show_secrets = false
```

---

### `validate` - Check .env Format

Validate `.env` file syntax:

```bash
# Validate default .env
bwenv validate

# Output:
# ✓ .env is valid

# Validate specific file
bwenv validate --input .env.production

# Invalid file output:
# ✗ Validation failed: Line 5: Missing '=' separator
```

**Options:**
- `-i, --input <INPUT>` - File to validate (default: `.env`)

---

## Authentication

bwenv uses **Bitwarden Secrets Manager access tokens**:

### Generate Token

1. Go to [Bitwarden Settings](https://vault.bitwarden.com/#/settings/security/security-keys)
2. Click **"New Access Token"**
3. Name: `bwenv` (or any name)
4. Permissions: ✓ **Secrets Manager**
5. Copy token (shown once)

### Export Token

```bash
# Temporary (current shell session)
export BITWARDEN_ACCESS_TOKEN="your_token_here"

# Permanent (add to shell profile)
echo 'export BITWARDEN_ACCESS_TOKEN="your_token"' >> ~/.bashrc

# Or for zsh
echo 'export BITWARDEN_ACCESS_TOKEN="your_token"' >> ~/.zshrc
```

### Security Notes

- ✅ Access tokens are scoped to Secrets Manager only
- ✅ Tokens can be revoked anytime in Bitwarden settings
- ✅ Zero-knowledge encryption (Bitwarden can't see secrets)
- ⚠️ Never commit tokens to git (add `.env.test` to `.gitignore`)

---

## Why bwenv vs Competitors?

| Feature | Infisical | Doppler | Vault | **bwenv** |
|---------|-----------|---------|-------|-----------|
| **Setup Time** | 30+ min | 15 min | 4 hours | **5 min** ⚡ |
| **Complexity** | High | Medium | Very High | **Very Low** |
| **Free Tier** | Limited | 5 users | ❌ | **Full features** 🆓 |
| **Drift Detection** | ❌ | ✅ | ❌ | **✅** 🔍 |
| **Self-Hosting** | ✅ | ❌ | ✅ | **✅ (via BW)** |
| **Commands to Learn** | 20+ | 15+ | 30+ | **6** 🎯 |
| **Local Development** | Requires server | Cloud-only | Complex | **Works offline** |
| **SDK Language** | Node/Python | Node | Go | **Rust** 🦀 |

### Target Audience

**Perfect for:**
- 👨‍💻 Individual developers
- 👥 Small teams (2-10 people)
- 🚀 Startups & indie hackers
- 🎓 Side projects & learning

**Not ideal for:**
- 🏢 Large enterprises (50+ devs) → Use Vault or Infisical
- ☁️ AWS-only shops → Use AWS Secrets Manager
- 🔐 Teams needing dynamic secrets → Use Vault

See [COMPETITIVE_ANALYSIS.md](COMPETITIVE_ANALYSIS.md) for detailed comparison.

---

## Testing

bwenv has **comprehensive test coverage** across multiple layers:

### Test Categories

- ✅ **Unit Tests** (72 tests) - Core functionality (parser, provider, errors)
- ✅ **Integration Tests** (29 tests) - CLI command parsing
- ✅ **E2E Tests** (20 scenarios) - Real Bitwarden integration
- ✅ **Property Tests** (5 tests) - Edge case discovery with random inputs
- ✅ **Security Tests** (13 tests) - Secrets leakage prevention
- ✅ **Benchmarks** (11 tests) - Performance tracking

### Quick Test Run

```bash
# Unit tests (fast, < 1 second)
cargo test --lib

# All tests
cargo test

# E2E tests (requires Bitwarden token)
./scripts/run-e2e-tests.sh
```

### E2E Testing

**End-to-end tests** verify bwenv works correctly with **real Bitwarden Secrets Manager**:

```bash
# Setup (one-time, 5 minutes)
cp .env.test.example .env.test
# Edit .env.test with your Bitwarden access token

# Run E2E tests
./scripts/run-e2e-tests.sh

# Output:
# === bwenv E2E Test Runner ===
# Mode: docker
# Project: E2E-Test
#
# running 20 tests
# test test_push_basic_secrets ... ok
# test test_pull_basic_secrets ... ok
# test test_roundtrip_push_pull_integrity ... ok
# ...
# ✓ All E2E tests passed!
```

**E2E Test Coverage:**
- Push command (5 scenarios)
- Pull command (4 scenarios)
- Roundtrip integrity (2 tests)
- List command (2 scenarios)
- Validate command (3 scenarios)
- Error handling (4 scenarios)

### Documentation

- [TESTING.md](TESTING.md) - Comprehensive testing guide (500+ lines)
- [E2E_QUICKSTART.md](E2E_QUICKSTART.md) - E2E test setup (5-minute guide)
- `tests/e2e/README.md` - Developer guide for writing E2E tests

---

## Troubleshooting

### "BITWARDEN_ACCESS_TOKEN not set"

**Cause:** Environment variable not exported

**Fix:**
```bash
# Check if token is set
echo $BITWARDEN_ACCESS_TOKEN

# Export token
export BITWARDEN_ACCESS_TOKEN="your_token_here"

# Or add to shell profile for persistence
echo 'export BITWARDEN_ACCESS_TOKEN="your_token"' >> ~/.bashrc
source ~/.bashrc
```

---

### "Project not found"

**Cause:** Project doesn't exist or name is incorrect

**Fix:**
```bash
# List all projects to see correct name
bwenv list

# Projects are case-sensitive
bwenv pull --project MyApp  # ✓ Correct
bwenv pull --project myapp  # ✗ Wrong case
```

---

### "Authentication failed"

**Cause:** Invalid or expired access token

**Fix:**
1. Generate new access token in Bitwarden
2. Go to Settings → Security → Access Tokens
3. Click "New Access Token"
4. Ensure "Secrets Manager" permission is checked
5. Export new token: `export BITWARDEN_ACCESS_TOKEN="new_token"`

---

### Drift detection shows differences incorrectly

**Cause:** .env file might not exist or be in wrong location

**Fix:**
```bash
# Check .env file exists
ls -la .env

# Specify custom env file location
bwenv status --project MyApp --env-file /path/to/.env

# Pull to create .env if missing
bwenv pull --project MyApp
```

---

### "Failed to read .env file"

**Cause:** Malformed .env file or permission issues

**Fix:**
```bash
# Validate .env format
bwenv validate

# Check file permissions
ls -l .env

# Fix permissions if needed
chmod 600 .env
```

---

For more troubleshooting, see:
- [TESTING.md](TESTING.md) - Troubleshooting section
- [E2E_QUICKSTART.md](E2E_QUICKSTART.md) - E2E test troubleshooting
- [GitHub Issues](https://github.com/yourusername/bwenv/issues)

---

## Contributing

We welcome contributions! 🎉

### Development Setup

```bash
# Clone repository
git clone https://github.com/yourusername/bwenv.git
cd bwenv

# Build
cargo build

# Run tests
cargo test

# Run E2E tests (requires Bitwarden token)
./scripts/run-e2e-tests.sh --local
```

### Guidelines

1. **Write tests first** (TDD approach)
2. **Run full test suite** before submitting PR
   ```bash
   cargo test                    # Unit + integration tests
   cargo clippy                  # Linting
   cargo fmt                     # Formatting
   ./scripts/run-e2e-tests.sh    # E2E tests
   ```
3. **Add E2E tests** for new commands (see [E2E_QUICKSTART.md](E2E_QUICKSTART.md))
4. **Update documentation** (README, TESTING.md, command help text)
5. **Follow Rust conventions** (rustfmt, clippy clean)

### Pull Request Process

1. **Fork** the repository
2. **Create** feature branch
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Write tests** for your changes
   ```bash
   cargo test
   ```
4. **Run E2E tests** (if adding commands)
   ```bash
   ./scripts/run-e2e-tests.sh
   ```
5. **Commit** changes
   ```bash
   git commit -m 'Add amazing feature'
   ```
6. **Push** to branch
   ```bash
   git push origin feature/amazing-feature
   ```
7. **Open** Pull Request

### What to Contribute

**Good First Issues:**
- 📝 Improve documentation
- 🐛 Fix bugs
- ✅ Add more tests
- 📊 Add examples

**Medium Issues:**
- 🚀 Performance improvements
- 🔧 New command options
- 📦 Package managers (Homebrew, apt)

**Advanced Issues:**
- 🎯 New commands
- 🔌 CI/CD integrations
- 🏗️ Architectural improvements

See [TESTING.md](TESTING.md) for comprehensive testing documentation.

---

## Roadmap

### Phase 1: Foundation ✅ (Complete)
- ✅ Core CLI commands (pull, push, list, status, validate, init)
- ✅ Real Bitwarden SDK integration
- ✅ Drift detection
- ✅ E2E testing infrastructure (20 tests)
- ✅ Comprehensive documentation

### Phase 2: Developer Experience (Next 3 months)
- [ ] Git hooks integration (auto-sync on pull)
- [ ] CI/CD examples (GitHub Actions, GitLab CI)
- [ ] Config file support (.bwenv.toml reading)
- [ ] Better error messages (actionable suggestions)
- [ ] Homebrew formula (easy macOS install)

### Phase 3: Team Features (6 months)
- [ ] Team templates (shared .env.example generation)
- [ ] Secret rotation helpers (detect stale secrets)
- [ ] Import/export (migrate from Doppler/Infisical)
- [ ] Desktop notifications (drift alerts)

### Phase 4: Enterprise-Lite (12 months)
- [ ] Policy enforcement (required keys validation)
- [ ] Change approvals (via Bitwarden workflows)
- [ ] Cost tracking (secret usage analytics)
- [ ] SSO integration (via Bitwarden Enterprise)

---

## Security Considerations

- 🔒 **Zero-knowledge encryption** - Bitwarden uses end-to-end encryption
- 🔑 **Access tokens** are scoped to Secrets Manager only
- 🚫 **No master password storage** - bwenv never sees your master password
- ✅ **Secrets never leave Bitwarden** except when explicitly pulled
- 🔍 **Open source** - Audit the code yourself
- 🛡️ **Battle-tested SDK** - Uses official Bitwarden Rust SDK

### Best Practices

1. **Never commit `.env` files** - Add to `.gitignore`
2. **Rotate tokens regularly** - Generate new access tokens monthly
3. **Use separate projects** - Different projects for dev/staging/prod
4. **Limit token permissions** - Only grant Secrets Manager access
5. **Review team access** - Check who has access to projects in Bitwarden

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- **Bitwarden** - For the excellent Secrets Manager SDK
- **Rust Community** - For amazing libraries and tooling
- **Contributors** - Thank you for improving bwenv! 🎉

---

## Support

- 📖 **Documentation**: [TESTING.md](TESTING.md), [E2E_QUICKSTART.md](E2E_QUICKSTART.md)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/yourusername/bwenv/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/yourusername/bwenv/discussions)
- 📧 **Email**: your.email@example.com

---

**Built with ❤️ by developers, for developers.**
