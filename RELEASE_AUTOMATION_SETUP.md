# Release Automation Setup Guide

This document provides instructions for setting up and using the automated release workflow for bwenv.

## Overview

bwenv uses a modern, fully automated release workflow:

- **release-plz**: Automatic version bumping and changelog generation
- **git-cliff**: Conventional commit-based changelog formatting
- **cargo-dist**: Multi-platform binary distribution
- **commitlint**: Commit message validation

## Prerequisites

### Local Development

Install the following tools:

```bash
# Install pre-commit (choose one)
brew install pre-commit          # macOS
pip install pre-commit            # Python/pipx
apt install pre-commit            # Debian/Ubuntu

# Install Node.js (for commitlint)
brew install node                 # macOS
# Or use nvm, asdf, etc.

# Install Rust tools (optional, for manual releases)
cargo install release-plz cargo-dist git-cliff
```

### GitHub Setup

1. **Create GitHub Personal Access Token**
   - Go to: https://github.com/settings/tokens
   - Click "Generate new token (classic)"
   - Scopes needed:
     - `repo` (full access)
     - `workflow` (update GitHub Actions)
   - Save token securely

2. **Create crates.io Token**
   - Go to: https://crates.io/me
   - Click "New Token"
   - Name: `bwenv-github-actions`
   - Save token securely

3. **Add GitHub Secrets**
   - Go to: `https://github.com/yourusername/bwenv/settings/secrets/actions`
   - Add new secrets:
     - `RELEASE_PLZ_TOKEN`: Your GitHub PAT from step 1
     - `CARGO_REGISTRY_TOKEN`: Your crates.io token from step 2

## Initial Setup

### 1. Install Pre-commit Hooks

```bash
# In the project root
pre-commit install
pre-commit install --hook-type commit-msg

# Test the hooks
pre-commit run --all-files
```

### 2. Verify Configuration

All configuration files should already be in place:

- `.commitlintrc.js` - Commit message validation
- `cliff.toml` - Changelog generation
- `release-plz.toml` - Release automation
- `.pre-commit-config.yaml` - Git hooks
- `.github/workflows/release-plz.yml` - GitHub Actions workflow

### 3. Initialize cargo-dist (Optional)

If you want to regenerate the release workflow:

```bash
cargo dist init

# Select options:
# - CI: GitHub
# - Installers: shell, powershell, homebrew
# - Platforms: All recommended platforms
```

This will create/update `.github/workflows/release.yml`.

## Daily Usage

### Making Commits

Use conventional commit format:

```bash
# Feature
git commit -m "feat(cli): add --config flag support"

# Bug fix
git commit -m "fix(sdk): handle network timeout gracefully"

# Breaking change
git commit -m "feat(api)!: change authentication method

BREAKING CHANGE: API now requires OAuth tokens instead of API keys.
See migration guide in docs/migration.md"
```

**The pre-commit hook will validate your commit message automatically.**

### Commit Types

| Type | Description | SemVer Impact |
|------|-------------|---------------|
| `feat` | New feature | MINOR |
| `fix` | Bug fix | PATCH |
| `perf` | Performance improvement | PATCH |
| `refactor` | Code restructuring | PATCH |
| `docs` | Documentation only | None |
| `test` | Test changes | None |
| `chore` | Maintenance | None |
| `BREAKING CHANGE` | Breaking change (footer) | MAJOR |
| `type!` | Breaking change (exclamation) | MAJOR |

### Scopes (Optional)

Use scopes to indicate the affected module:

- `cli` - Command-line interface
- `sdk` - Bitwarden SDK integration
- `config` - Configuration (.bwenv.toml)
- `env` - .env file parsing
- `sync` - Sync engine
- `logging` - Logging system
- `deps` - Dependencies

Example: `feat(config): add multi-environment support`

## Release Process

### Automated Release (Recommended)

The release process is **fully automated**:

1. **Make commits** with conventional commit format
2. **Push to main** branch
3. **release-plz** analyzes commits and creates a Release PR
4. **Review the PR**: Check version bump and changelog
5. **Merge the PR**: Triggers automatic release:
   - Publishes to crates.io
   - Creates git tag
   - Builds binaries for all platforms
   - Creates GitHub release
   - Updates Homebrew formula (if configured)

### Manual Release (Emergency)

If automation fails, you can release manually:

```bash
# 1. Bump version
sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml

# 2. Generate changelog
git cliff --tag v0.2.0 --output CHANGELOG.md

# 3. Commit and tag
git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): bump version to 0.2.0"
git tag v0.2.0
git push --tags

# 4. Publish to crates.io
cargo publish

# 5. GitHub release will be created automatically by cargo-dist
```

## Troubleshooting

### Pre-commit Hook Fails

```bash
# Update hooks to latest version
pre-commit autoupdate

# Re-run all hooks
pre-commit run --all-files

# Skip hooks if necessary (emergency only)
git commit --no-verify -m "fix: emergency commit"
```

### Commit Message Validation Error

Common mistakes:

- ❌ `Fix bug` → ✅ `fix: resolve authentication timeout`
- ❌ `feat: Added feature` → ✅ `feat: add configuration file support`
- ❌ `update stuff` → ✅ `chore: update dependencies`

### Release PR Not Created

Check:
1. Commits are on `main` branch
2. Commits follow conventional format
3. GitHub secrets are configured correctly
4. GitHub Actions is enabled in repository settings

View logs: `https://github.com/yourusername/bwenv/actions`

### cargo-dist Build Fails

Check:
1. All target platforms are building locally: `cargo build --release`
2. Dependencies support all target platforms
3. Review build logs in GitHub Actions

## Advanced Configuration

### Customize Changelog Format

Edit `cliff.toml` to customize changelog appearance:

```toml
[git]
commit_parsers = [
  { message = "^feat", group = "🚀 Features" },
  { message = "^fix", group = "🐛 Bug Fixes" },
  { message = "^perf", group = "⚡ Performance" },
  # Add more customizations
]
```

### Add More Platforms

Edit `Cargo.toml`:

```toml
[workspace.metadata.dist]
targets = [
    # Add more targets
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
]
```

### Configure Homebrew Tap

Edit `Cargo.toml`:

```toml
[workspace.metadata.dist.homebrew]
tap = "yourusername/homebrew-bwenv"
```

Create the tap repository: `https://github.com/yourusername/homebrew-bwenv`

## Tools Reference

### release-plz

```bash
# Check what version bump would be triggered
release-plz update --dry-run

# Generate release manually
release-plz release
```

### git-cliff

```bash
# Generate changelog for unreleased changes
git cliff --unreleased

# Generate changelog for specific tag
git cliff --tag v1.0.0

# Update CHANGELOG.md
git cliff --output CHANGELOG.md
```

### cargo-dist

```bash
# Plan what would be built
cargo dist plan

# Build releases locally
cargo dist build

# Generate installer scripts
cargo dist generate
```

## Resources

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)
- [release-plz Documentation](https://release-plz.dev/)
- [cargo-dist Guide](https://github.com/axodotdev/cargo-dist)
- [git-cliff Documentation](https://git-cliff.org/)

## Questions?

- Check [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines
- See [PRODUCT_ROADMAP.md](PRODUCT_ROADMAP.md) for project vision
- Open an issue: https://github.com/yourusername/bwenv/issues

---

**Summary:** Commit with conventional format → Push to main → Release PR auto-created → Merge PR → Automatic release! 🎉
