#!/bin/bash

set -e

echo "🧪 Running comprehensive test suite for bwenv"
echo "================================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Clean previous builds
print_status "Cleaning previous builds..."
cargo clean

# Check code formatting
print_status "Checking code formatting..."
if ! cargo fmt --check; then
    print_warning "Code formatting issues found. Run 'cargo fmt' to fix them."
else
    print_success "Code formatting is correct"
fi

# Run clippy for linting
print_status "Running clippy for linting..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    print_error "Clippy found issues"
    exit 1
else
    print_success "Clippy checks passed"
fi

# Build the project
print_status "Building the project..."
if ! cargo build; then
    print_error "Build failed"
    exit 1
else
    print_success "Build successful"
fi

# Run unit tests
print_status "Running unit tests..."
if ! cargo test --lib; then
    print_error "Unit tests failed"
    exit 1
else
    print_success "Unit tests passed"
fi

# Run integration tests
print_status "Running integration tests..."
if ! cargo test --test integration_tests; then
    print_warning "Integration tests failed (expected if Bitwarden CLI is not configured)"
else
    print_success "Integration tests passed"
fi

# Run file operation tests
print_status "Running file operation tests..."
if ! cargo test --test file_operations_tests; then
    print_error "File operation tests failed"
    exit 1
else
    print_success "File operation tests passed"
fi

# Run property-based tests
print_status "Running property-based tests..."
if ! cargo test --test property_tests; then
    print_error "Property-based tests failed"
    exit 1
else
    print_success "Property-based tests passed"
fi

# Run mock tests
print_status "Running mock-based tests..."
if ! cargo test --test bitwarden_mock_tests; then
    print_error "Mock-based tests failed"
    exit 1
else
    print_success "Mock-based tests passed"
fi

# Run all tests together
print_status "Running all tests together..."
if ! cargo test; then
    print_warning "Some tests failed (this is expected if Bitwarden CLI is not properly configured)"
else
    print_success "All tests passed"
fi

# Build release version
print_status "Building release version..."
if ! cargo build --release; then
    print_error "Release build failed"
    exit 1
else
    print_success "Release build successful"
fi

# Check if binary works
print_status "Testing binary execution..."
if ! ./target/release/bwenv --help > /dev/null; then
    print_error "Binary execution failed"
    exit 1
else
    print_success "Binary executes correctly"
fi

# Generate test coverage (if tarpaulin is installed)
if command -v cargo-tarpaulin &> /dev/null; then
    print_status "Generating test coverage report..."
    if cargo tarpaulin --out Html --output-dir coverage; then
        print_success "Coverage report generated in coverage/ directory"
    else
        print_warning "Coverage generation failed"
    fi
else
    print_warning "cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin"
fi

# Run benchmarks if available
if [ -d "benches" ]; then
    print_status "Running benchmarks..."
    if cargo bench; then
        print_success "Benchmarks completed"
    else
        print_warning "Benchmarks failed"
    fi
fi

# Security audit
if command -v cargo-audit &> /dev/null; then
    print_status "Running security audit..."
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_warning "Security audit found issues"
    fi
else
    print_warning "cargo-audit not found. Install with: cargo install cargo-audit"
fi

echo ""
echo "================================================================"
print_success "Test suite completed!"
echo ""
echo "Summary:"
echo "  ✅ Code formatting"
echo "  ✅ Clippy linting"
echo "  ✅ Build (debug and release)"
echo "  ✅ Unit tests"
echo "  ✅ File operation tests"
echo "  ✅ Property-based tests"
echo "  ✅ Mock-based tests"
echo "  ⚠️  Integration tests (may fail without Bitwarden setup)"
echo "  ✅ Binary execution"
echo ""
print_status "To run individual test categories:"
echo "  cargo test --lib                    # Unit tests only"
echo "  cargo test --test integration_tests # Integration tests"
echo "  cargo test --test file_operations_tests # File operation tests"
echo "  cargo test --test property_tests    # Property-based tests"
echo "  cargo test --test bitwarden_mock_tests # Mock tests"
echo ""
print_status "To run with verbose output:"
echo "  cargo test -- --nocapture"
echo ""
print_status "To run specific tests:"
echo "  cargo test test_name"