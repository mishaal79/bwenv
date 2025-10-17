# Contributing to bwenv

Thank you for your interest in contributing to bwenv! This document provides guidelines for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.85+ ([rustup](https://rustup.rs/))
- [pre-commit](https://pre-commit.com/) for git hooks
- Node.js (for commitlint)

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/bwenv.git
cd bwenv

# Install pre-commit hooks
pre-commit install
pre-commit install --hook-type commit-msg

# Build the project
cargo build

# Run tests
cargo test
```

## Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification. This enables automated changelog generation and semantic versioning.

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- `feat`: New feature (triggers MINOR version bump)
- `fix`: Bug fix (triggers PATCH version bump)
- `docs`: Documentation only
- `perf`: Performance improvement
- `refactor`: Code restructuring (no behavior change)
- `test`: Adding or updating tests
- `style`: Code formatting (no logic change)
- `build`: Build system or dependencies
- `ci`: CI/CD configuration changes
- `chore`: Maintenance tasks (release, deps)
- `revert`: Revert previous commit

### Scopes

Use scopes to indicate which part of the codebase is affected:

- `cli`: Command-line interface
- `sdk`: Bitwarden SDK integration
- `config`: Configuration management (.bwenv.toml)
- `env`: .env file parsing
- `sync`: Synchronization engine
- `logging`: Logging system
- `deps`: Dependencies

### Examples

**Feature:**
```bash
git commit -m "feat(cli): add --verbose flag for detailed output"
```

**Bug Fix:**
```bash
git commit -m "fix(sdk): handle expired access tokens gracefully"
```

**Breaking Change:**
```bash
git commit -m "feat(sync)!: change conflict resolution strategy

BREAKING CHANGE: Conflicts now default to local-wins instead of remote-wins.
Update your workflow if you relied on remote-wins behavior.
"
```

**Documentation:**
```bash
git commit -m "docs: update installation instructions for macOS"
```

**Performance:**
```bash
git commit -m "perf(env): optimize large .env file parsing"
```

**Chore:**
```bash
git commit -m "chore(deps): update bitwarden SDK to 1.2.0"
```

### Commit Message Validation

Your commits will be automatically validated by pre-commit hooks. If a commit message doesn't follow the convention, the commit will be rejected with a helpful error message.

**Common mistakes to avoid:**
- ❌ `fix: Fix bug` (subject should be lowercase, no period)
- ❌ `feat: Added new feature` (use imperative mood: "add" not "added")
- ❌ `update stuff` (missing type prefix)
- ✅ `fix: resolve authentication timeout issue`

## Code Style

### Formatting

We use `rustfmt` for code formatting:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

We use `clippy` for linting:

```bash
# Run clippy
cargo clippy --all-targets -- -D warnings

# Fix auto-fixable issues
cargo clippy --fix
```

### Configuration

Our clippy configuration is in `clippy.toml`. Key rules:
- Cognitive complexity threshold: 15
- Max function parameters: 7
- Type complexity threshold: 250

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Test Guidelines

- Write tests for all new features
- Maintain or improve code coverage
- Use descriptive test names
- Test edge cases and error conditions

## Pull Request Process

1. **Fork the repository** and create a feature branch from `main`

2. **Make your changes** following the code style guidelines

3. **Write tests** for your changes

4. **Update documentation** if needed (README, docs, comments)

5. **Ensure all checks pass:**
   ```bash
   cargo test
   cargo fmt -- --check
   cargo clippy --all-targets -- -D warnings
   ```

6. **Commit with conventional commits** format

7. **Push to your fork** and create a pull request

8. **Respond to review feedback** promptly

### PR Title Format

PR titles should also follow conventional commits:

```
feat(cli): add configuration file support
fix(sync): resolve race condition in token refresh
docs: improve installation guide
```

## Release Process

Releases are fully automated through GitHub Actions:

1. When commits are pushed to `main`, **release-plz** analyzes conventional commits
2. A **Release PR** is automatically created with:
   - Version bump (based on commit types)
   - Updated CHANGELOG.md
3. When the Release PR is merged:
   - Published to crates.io
   - Git tag created
   - GitHub release created with binaries for multiple platforms
   - Homebrew formula updated (if configured)

**You don't need to manually manage versions or changelogs** - the automation handles it!

## Code Review

### What We Look For

- **Correctness**: Does the code work as intended?
- **Tests**: Are there adequate tests?
- **Documentation**: Is the change well-documented?
- **Style**: Does it follow our style guidelines?
- **Performance**: Are there any performance concerns?
- **Security**: Are there any security implications?

### Review Timeline

We aim to review PRs within 48 hours. If you haven't received a review, feel free to ping the maintainers.

## Getting Help

- **Documentation**: Check the [README](README.md) and [Product Roadmap](PRODUCT_ROADMAP.md)
- **Issues**: Search [existing issues](https://github.com/yourusername/bwenv/issues)
- **Discussions**: Use [GitHub Discussions](https://github.com/yourusername/bwenv/discussions)

## License

By contributing to bwenv, you agree that your contributions will be licensed under the project's MIT OR Apache-2.0 dual license.

## Code of Conduct

Be respectful, inclusive, and professional. We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

---

Thank you for contributing to bwenv! 🎉
