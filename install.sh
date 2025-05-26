#!/bin/bash

# DirAnalyzer Installation Script
# A blazingly fast directory analysis tool written in Rust

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="diranalyzer"
REPO_URL="https://github.com/RustyFellows/diranalyzer"
VERSION="latest"

# Banner
print_banner() {
    echo -e "${CYAN}"
    echo "██████╗ ██╗██████╗  █████╗ ███╗   ██╗ █████╗ ██╗  ██╗   ██╗███████╗███████╗██████╗ "
    echo "██╔══██╗██║██╔══██╗██╔══██╗████╗  ██║██╔══██╗██║  ╚██╗ ██╔╝╚══███╔╝██╔════╝██╔══██╗"
    echo "██║  ██║██║██████╔╝███████║██╔██╗ ██║███████║██║   ╚████╔╝   ███╔╝ █████╗  ██████╔╝"
    echo "██║  ██║██║██╔══██╗██╔══██║██║╚██╗██║██╔══██║██║    ╚██╔╝   ███╔╝  ██╔══╝  ██╔══██╗"
    echo "██████╔╝██║██║  ██║██║  ██║██║ ╚████║██║  ██║███████╗██║   ███████╗███████╗██║  ██║"
    echo "╚═════╝ ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝   ╚══════╝╚══════╝╚═╝  ╚═╝"
    echo -e "${NC}"
    echo -e "${YELLOW}🚀 Lightning-Fast Directory Analysis Tool Installation${NC}"
    echo -e "${PURPLE}========================================${NC}"
    echo
}

# Check if running as root for system-wide installation
check_permissions() {
    if [[ $EUID -eq 0 ]]; then
        echo -e "${GREEN}✓${NC} Running with administrator privileges"
        INSTALL_DIR="/usr/local/bin"
    else
        echo -e "${YELLOW}ℹ${NC} Installing to user directory (no sudo detected)"
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        
        # Add to PATH if not already there
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            echo -e "${YELLOW}📝${NC} Adding $INSTALL_DIR to your PATH..."
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
            echo -e "${CYAN}💡${NC} Please run: ${YELLOW}source ~/.bashrc${NC} after installation"
        fi
    fi
}

# Detect system architecture and OS
detect_system() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case $ARCH in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="aarch64"
            ;;
        *)
            echo -e "${RED}❌ Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac
    
    case $OS in
        linux)
            TARGET="$ARCH-unknown-linux-gnu"
            ;;
        darwin)
            TARGET="$ARCH-apple-darwin"
            ;;
        *)
            echo -e "${RED}❌ Unsupported operating system: $OS${NC}"
            exit 1
            ;;
    esac
    
    echo -e "${GREEN}✓${NC} Detected system: ${CYAN}$OS-$ARCH${NC}"
}

# Check if Rust is installed
check_rust() {
    if command -v cargo &> /dev/null; then
        echo -e "${GREEN}✓${NC} Rust is installed"
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        echo -e "${CYAN}  Version: $RUST_VERSION${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠${NC} Rust is not installed"
        return 1
    fi
}

# Install Rust if needed
install_rust() {
    echo -e "${BLUE}🦀${NC} Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✓${NC} Rust installed successfully"
}

# Download and install from pre-built binary (if available)
install_prebuilt() {
    local download_url="$REPO_URL/releases/latest/download/diranalyzer-$TARGET"
    local temp_file="/tmp/diranalyzer"
    
    echo -e "${BLUE}📦${NC} Attempting to download pre-built binary..."
    
    if curl -L --fail "$download_url" -o "$temp_file" 2>/dev/null; then
        chmod +x "$temp_file"
        mv "$temp_file" "$INSTALL_DIR/$BINARY_NAME"
        echo -e "${GREEN}✓${NC} Successfully installed pre-built binary"
        return 0
    else
        echo -e "${YELLOW}⚠${NC} Pre-built binary not available, will build from source"
        return 1
    fi
}

# Build from source
build_from_source() {
    echo -e "${BLUE}🔨${NC} Building DirAnalyzer from source..."
    
    local temp_dir="/tmp/diranalyzer-build"
    rm -rf "$temp_dir"
    
    echo -e "${CYAN}📥${NC} Cloning repository..."
    git clone "$REPO_URL" "$temp_dir" || {
        echo -e "${RED}❌ Failed to clone repository${NC}"
        exit 1
    }
    
    cd "$temp_dir"
    
    echo -e "${CYAN}🔧${NC} Building release binary..."
    cargo build --release || {
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    }
    
    echo -e "${CYAN}📋${NC} Installing binary..."
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/" || {
        echo -e "${RED}❌ Failed to install binary${NC}"
        exit 1
    }
    
    cd - > /dev/null
    rm -rf "$temp_dir"
    
    echo -e "${GREEN}✓${NC} Successfully built and installed from source"
}

# Verify installation
verify_installation() {
    if command -v "$BINARY_NAME" &> /dev/null; then
        echo -e "${GREEN}✓${NC} Installation verified successfully"
        echo -e "${CYAN}📍${NC} Installed to: ${YELLOW}$(which $BINARY_NAME)${NC}"
        
        echo -e "${BLUE}🧪${NC} Testing installation..."
        "$BINARY_NAME" --help | head -5
        echo
        return 0
    else
        echo -e "${RED}❌ Installation verification failed${NC}"
        return 1
    fi
}

# Create desktop entry (Linux only)
create_desktop_entry() {
    if [[ "$OS" == "linux" ]] && command -v xdg-desktop-menu &> /dev/null; then
        local desktop_file="$HOME/.local/share/applications/diranalyzer.desktop"
        mkdir -p "$(dirname "$desktop_file")"
        
        cat > "$desktop_file" << EOF
[Desktop Entry]
Name=DirAnalyzer
Comment=Lightning-fast directory analysis tool
Exec=gnome-terminal -- diranalyzer
Icon=folder-documents
Type=Application
Categories=System;FileTools;
Terminal=true
EOF
        
        echo -e "${GREEN}✓${NC} Created desktop entry"
    fi
}

# Print success message and usage
print_success() {
    echo -e "${GREEN}🎉 DirAnalyzer installed successfully!${NC}"
    echo
    echo -e "${YELLOW}📚 Quick Start:${NC}"
    echo -e "  ${CYAN}$BINARY_NAME .${NC}                    # Analyze current directory"
    echo -e "  ${CYAN}$BINARY_NAME /path --duplicates${NC}   # Find duplicates"
    echo -e "  ${CYAN}$BINARY_NAME /home --export json${NC}  # Export results"
    echo
    echo -e "${YELLOW}📖 Full documentation:${NC} $REPO_URL"
    echo -e "${YELLOW}🐛 Report issues:${NC} $REPO_URL/issues"
    echo
    echo -e "${PURPLE}Happy analyzing! 🔍${NC}"
}

# Main installation function
main() {
    print_banner
    
    echo -e "${BLUE}🔍${NC} Checking system requirements..."
    check_permissions
    detect_system
    
    # Try to install pre-built binary first, fallback to source build
    if ! install_prebuilt; then
        if ! check_rust; then
            echo -e "${YELLOW}🦀${NC} Rust is required to build from source"
            read -p "Install Rust automatically? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                install_rust
            else
                echo -e "${RED}❌ Installation cancelled${NC}"
                exit 1
            fi
        fi
        
        build_from_source
    fi
    
    # Post-installation steps
    verify_installation || exit 1
    create_desktop_entry
    
    print_success
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "DirAnalyzer Installation Script"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --version, -v  Show version information"
        echo "  --uninstall    Uninstall DirAnalyzer"
        echo
        echo "Environment Variables:"
        echo "  INSTALL_DIR    Custom installation directory"
        echo
        exit 0
        ;;
    --version|-v)
        echo "DirAnalyzer Installation Script v1.0.0"
        exit 0
        ;;
    --uninstall)
        exec "$(dirname "$0")/uninstall.sh"
        ;;
    *)
        main "$@"
        ;;
esac
