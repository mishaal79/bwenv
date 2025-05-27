#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Print banner
echo -e "${BLUE}"
echo "╭───────────────────────────────────────────────╮"
echo "│ Qrius Bitwarden CLI Environment Loader        │"
echo "│ Installation Script                           │"
echo "╰───────────────────────────────────────────────╯"
echo -e "${NC}"

# Detect OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macOS"
    INSTALL_DIR="/usr/local/bin"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="Linux"
    INSTALL_DIR="/usr/local/bin"
else
    OS="Other"
    INSTALL_DIR="$HOME/.local/bin"
fi

echo -e "${BLUE}Detected OS: ${YELLOW}$OS${NC}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required dependencies
echo -e "\n${BLUE}Checking dependencies...${NC}"

if ! command_exists cargo; then
    echo -e "${RED}Error: Rust and Cargo are required but not installed.${NC}"
    echo -e "Please install Rust from https://rustup.rs/ and try again."
    exit 1
else
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    echo -e "${GREEN}✓${NC} Rust ${YELLOW}$RUST_VERSION${NC} is installed"
fi

if ! command_exists bw; then
    echo -e "${YELLOW}Warning: Bitwarden CLI is not installed.${NC}"
    echo -e "The tool requires Bitwarden CLI to function."
    echo -e "Please install it from https://bitwarden.com/help/cli/"
else
    BW_VERSION=$(bw --version)
    echo -e "${GREEN}✓${NC} Bitwarden CLI ${YELLOW}$BW_VERSION${NC} is installed"
fi

# Build the project
echo -e "\n${BLUE}Building project...${NC}"
cargo build --release

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Build failed.${NC}"
    exit 1
fi

BINARY_PATH="target/release/bwenv"

if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: Binary not found after build.${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Build successful"

# Create installation directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo -e "\n${BLUE}Creating installation directory ${YELLOW}$INSTALL_DIR${NC}..."
    mkdir -p "$INSTALL_DIR"
    
    # Add directory to PATH if it's not already there and it's not a standard location
    if [[ "$INSTALL_DIR" != "/usr/local/bin" && ! "$PATH" =~ (^|:)"$INSTALL_DIR"(:|$) ]]; then
        echo -e "${YELLOW}Adding $INSTALL_DIR to your PATH...${NC}"
        
        # Determine which shell config file to use
        SHELL_CONFIG=""
        if [[ -f "$HOME/.zshrc" && "$SHELL" == *"zsh"* ]]; then
            SHELL_CONFIG="$HOME/.zshrc"
        elif [[ -f "$HOME/.bashrc" ]]; then
            SHELL_CONFIG="$HOME/.bashrc"
        elif [[ -f "$HOME/.bash_profile" ]]; then
            SHELL_CONFIG="$HOME/.bash_profile"
        fi
        
        if [[ -n "$SHELL_CONFIG" ]]; then
            echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
            echo -e "${GREEN}✓${NC} Added $INSTALL_DIR to $SHELL_CONFIG"
            echo -e "${YELLOW}Please run 'source $SHELL_CONFIG' or start a new terminal to update your PATH.${NC}"
        else
            echo -e "${YELLOW}Could not determine shell config file. Please add $INSTALL_DIR to your PATH manually.${NC}"
        fi
    fi
fi

# Install the binary
echo -e "\n${BLUE}Installing to ${YELLOW}$INSTALL_DIR${NC}..."
if [ "$OS" == "Linux" ] || [ "$OS" == "macOS" ]; then
    # Use sudo if needed
    if [ -w "$INSTALL_DIR" ]; then
        cp "$BINARY_PATH" "$INSTALL_DIR/"
    else
        echo -e "${YELLOW}Need elevated permissions to install to $INSTALL_DIR${NC}"
        sudo cp "$BINARY_PATH" "$INSTALL_DIR/"
    fi
    
    # Make executable
    if [ -w "$INSTALL_DIR/bwenv" ]; then
        chmod +x "$INSTALL_DIR/bwenv"
    else
        sudo chmod +x "$INSTALL_DIR/bwenv"
    fi
else
    cp "$BINARY_PATH" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/bwenv"
fi

echo -e "${GREEN}✓${NC} Installation complete"

# Create log directory
LOG_DIR=""
if [[ "$OS" == "macOS" ]]; then
    LOG_DIR="$HOME/Library/Application Support/bwenv/logs"
elif [[ "$OS" == "Linux" ]]; then
    LOG_DIR="$HOME/.local/share/bwenv/logs"
else
    LOG_DIR="$HOME/.bwenv/logs"
fi

mkdir -p "$LOG_DIR"
echo -e "${GREEN}✓${NC} Created log directory: ${YELLOW}$LOG_DIR${NC}"

# Test the installation
echo -e "\n${BLUE}Testing installation...${NC}"
if command_exists bwenv; then
    VERSION=$(bwenv --version)
    echo -e "${GREEN}✓${NC} Successfully installed ${YELLOW}$VERSION${NC}"
else
    echo -e "${YELLOW}Warning: Installation successful, but the command is not in your PATH.${NC}"
    echo -e "You can run it directly from ${YELLOW}$INSTALL_DIR/bwenv${NC}"
    echo -e "Or add ${YELLOW}$INSTALL_DIR${NC} to your PATH."
fi

echo -e "\n${GREEN}Installation completed successfully!${NC}"
echo -e "Run ${YELLOW}bwenv --help${NC} to get started."