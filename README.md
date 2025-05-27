# bwenv

A Rust-based CLI tool that helps you securely manage your `.env` file secrets using Bitwarden.

## Overview

This tool leverages your existing Bitwarden desktop app session to store and retrieve environment variables from `.env` files. It's designed for developers who want to securely share project configuration between team members or across different environments.

## Features

- Uses your existing Bitwarden desktop app authentication
- Stores `.env` file contents as secure notes in your Bitwarden vault
- Organizes secrets by project or environment using Bitwarden folders
- Retrieves secrets and generates `.env` files
- Supports merging with existing `.env` files
- Comprehensive logging with configurable verbosity levels

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (for building from source)
- [Bitwarden Desktop App](https://bitwarden.com/download/) (installed and logged in)
- [Bitwarden CLI](https://bitwarden.com/help/cli/) (installed and available in your PATH)

## Installation

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/bwenv.git
   cd bwenv
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install the binary to your PATH:
   ```bash
   # On macOS/Linux
   cp target/release/bwenv /usr/local/bin/
   
   # On Windows (using PowerShell as Administrator)
   Copy-Item .\target\release\bwenv.exe -Destination "$env:USERPROFILE\.cargo\bin\"
   ```

### Using Cargo

```
cargo install --git https://github.com/yourusername/bwenv.git
```

## Authentication

This tool uses your existing Bitwarden desktop application session. Before using it:

1. Ensure Bitwarden desktop app is installed and running
2. Make sure you're logged into your Bitwarden account
3. The app should be unlocked (you've entered your master password)

The tool will automatically detect and use your existing session for authentication.

## Usage Examples

### Verbosity Control

Control the verbosity of command output:

```bash
# Normal output (warnings and errors)
bwenv store --file .env

# Verbose output (info, warnings, and errors)
bwenv -v store --file .env

# Debug output (debug info, info, warnings, and errors)
bwenv -vv store --file .env

# Trace output (maximum verbosity)
bwenv -vvv store --file .env

# Quiet mode (only errors)
bwenv -q store --file .env
```

### Storing Secrets

Store environment variables from a `.env` file to Bitwarden:

```bash
# Basic usage
bwenv store --file .env

# Store in a specific folder
bwenv store --file .env --folder "Development/MyProject"

# Store with a custom name
bwenv store --file .env --name "MyProject-Development"

# Overwrite existing secrets
bwenv store --file .env --folder "Development/MyProject" --overwrite
```

### Retrieving Secrets

Retrieve secrets from Bitwarden and generate a `.env` file:

```bash
# Basic usage
bwenv retrieve --name "MyProject-Development"

# Retrieve to a specific file
bwenv retrieve --name "MyProject-Development" --output .env.production

# Retrieve from a specific folder
bwenv retrieve --folder "Development/MyProject" --output .env

# Merge with existing .env file
bwenv retrieve --folder "Development/MyProject" --merge
```

### Listing Environment Sets

List all stored environment sets:

```bash
# Text format (default)
bwenv list

# JSON format
bwenv list --format json
```

## Command Reference

### Global Options

```
bwenv [OPTIONS] <SUBCOMMAND>
```

Options:
- `-v, --verbose`: Increase logging verbosity (can be used multiple times)
- `-q, --quiet`: Suppress all output except errors
- `-h, --help`: Print help information
- `-V, --version`: Print version information

### `store`

```
bwenv store [OPTIONS] --file <FILE>
```

Options:
- `--file <FILE>`: Path to the .env file to store (required)
- `--folder <FOLDER>`: Bitwarden folder path to store secrets in
- `--name <n>`: Name for this environment set (defaults to folder name)
- `--overwrite`: Overwrite existing secrets with the same name

### `retrieve`

```
bwenv retrieve [OPTIONS]
```

Options:
- `--folder <FOLDER>`: Bitwarden folder path to retrieve secrets from
- `--name <n>`: Name of the environment set to retrieve
- `--output <FILE>`: Output file path (defaults to .env)
- `--merge`: Merge with existing .env file instead of overwriting

### `list`

```
bwenv list [OPTIONS]
```

Options:
- `--format <FORMAT>`: Output format (text, json)

## Logging

This tool uses a comprehensive logging system that follows GNU/Linux conventions:

### Verbosity Levels

- **Quiet** (`-q`): Only errors are logged
- **Normal** (default): Warnings and errors
- **Verbose** (`-v`): Info, warnings, and errors
- **Debug** (`-vv`): Debug info, info, warnings, and errors
- **Trace** (`-vvv`): Maximum verbosity with all log messages

### Log Files

Log files are stored in the following locations:

- **Linux**: `~/.local/share/bwenv/logs/`
- **macOS**: `~/.local/share/bwenv/logs/`
- **Windows**: `%USERPROFILE%\.bwenv\logs\`

Logs are rotated automatically, with the 10 most recent log files being retained. Each log file is named with the date (`bwenv-YYYY-MM-DD.log`).

## Security Considerations

- This tool requires an active Bitwarden session from your desktop app
- It does not store your master password
- All secrets are encrypted using Bitwarden's security model
- The tool never transmits your secrets to any third-party services
- Consider who has access to your Bitwarden vault when sharing environment variables

## Troubleshooting

### Common Issues

1. **"Session not found" error**
   - Ensure Bitwarden desktop app is running and unlocked
   - Try locking and unlocking your vault in the desktop app

2. **Permission issues on macOS**
   3. You may need to grant additional permissions:
        ```
        chmod +x /usr/local/bin/bwenv
        ```

3. **Bitwarden CLI not found**
   - Make sure the Bitwarden CLI is installed and in your PATH
   - Run `bw --version` to confirm it's installed correctly

4. **Items not showing up in list**
   - Check that your items are stored as secure notes
   - Verify the format of your stored notes

5. **Log file access issues**
   - Ensure you have write permissions to the log directory
   - For permission errors, try running the command with sudo

## Testing

This project includes a comprehensive test suite with multiple testing strategies:

### Test Categories

- **Unit Tests**: Test individual functions and modules in isolation
- **Integration Tests**: Test CLI commands and end-to-end functionality  
- **Property-Based Tests**: Test with randomly generated inputs using `proptest`
- **Mock Tests**: Test Bitwarden integration logic without requiring actual CLI

### Running Tests

```bash
# Run all tests
cargo test

# Run comprehensive test suite with linting and formatting
./test.sh

# Run specific test categories
cargo test --lib                    # Unit tests only
cargo test --test integration_tests # Integration tests
cargo test --test file_operations_tests # File operation tests
cargo test --test property_tests    # Property-based tests
cargo test --test bitwarden_mock_tests # Mock tests

# Run with verbose output
cargo test -- --nocapture
```

### Test Coverage

Generate coverage reports using `cargo-tarpaulin`:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/tarpaulin-report.html
```

### Benchmarks

Run performance benchmarks:

```bash
cargo bench
```

For detailed testing documentation, see [TESTING.md](TESTING.md).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for your changes (see [TESTING.md](TESTING.md))
4. Run the test suite (`./test.sh`)
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Guidelines

- Write tests for all new functionality
- Ensure all tests pass before submitting PR
- Follow existing code style and patterns
- Update documentation as needed
- Add appropriate error handling and logging
</edits>

## License

This project is licensed under the MIT License - see the LICENSE file for details.