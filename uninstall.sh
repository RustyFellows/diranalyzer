#!/bin/bash

# DirAnalyzer Uninstallation Script
# Clean removal of the directory analysis tool

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
BINARY_NAME="diranalyzer"
POSSIBLE_LOCATIONS=(
    "/usr/local/bin/$BINARY_NAME"
    "/usr/bin/$BINARY_NAME"
    "$HOME/.local/bin/$BINARY_NAME"
    "$HOME/.cargo/bin/$BINARY_NAME"
)

# Banner
print_banner() {
    echo -e "${RED}"
    echo "██╗   ██╗███╗   ██╗██╗███╗   ██╗███████╗████████╗ █████╗ ██╗     ██╗     "
    echo "██║   ██║████╗  ██║██║████╗  ██║██╔════╝╚══██╔══╝██╔══██╗██║     ██║     "
    echo "██║   ██║██╔██╗ ██║██║██╔██╗ ██║███████╗   ██║   ███████║██║     ██║     "
    echo "██║   ██║██║╚██╗██║██║██║╚██╗██║╚════██║   ██║   ██╔══██║██║     ██║     "
    echo "╚██████╔╝██║ ╚████║██║██║ ╚████║███████║   ██║   ██║  ██║███████╗███████╗"
    echo " ╚═════╝ ╚═╝  ╚═══╝╚═╝╚═╝  ╚═══╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝"
    echo -e "${NC}"
    echo -e "${YELLOW}🗑️  DirAnalyzer Uninstallation Script${NC}"
    echo -e "${PURPLE}=====================================${NC}"
    echo
}

# Find installed locations
find_installations() {
    local found_locations=()
    
    echo -e "${BLUE}🔍${NC} Searching for DirAnalyzer installations..."
    
    for location in "${POSSIBLE_LOCATIONS[@]}"; do
        if [[ -f "$location" ]]; then
            found_locations+=("$location")
            echo -e "${YELLOW}📍${NC} Found: ${CYAN}$location${NC}"
        fi
    done
    
    # Also check PATH
    local path_location
    path_location=$(command -v "$BINARY_NAME" 2>/dev/null || true)
    if [[ -n "$path_location" ]]; then
        # Check if this location is already in our list
        local already_found=false
        for loc in "${found_locations[@]}"; do
            if [[ "$loc" == "$path_location" ]]; then
                already_found=true
                break
            fi
        done
        
        if [[ "$already_found" == false ]]; then
            found_locations+=("$path_location")
            echo -e "${YELLOW}📍${NC} Found in PATH: ${CYAN}$path_location${NC}"
        fi
    fi
    
    echo "${found_locations[@]}"
}

# Remove binary files
remove_binaries() {
    local locations=($1)
    local removed_count=0
    
    if [[ ${#locations[@]} -eq 0 ]]; then
        echo -e "${GREEN}✓${NC} No DirAnalyzer installations found"
        return 0
    fi
    
    echo -e "${BLUE}🗑️${NC} Removing DirAnalyzer binaries..."
    
    for location in "${locations[@]}"; do
        if [[ -f "$location" ]]; then
            if rm "$location" 2>/dev/null; then
                echo -e "${GREEN}✓${NC} Removed: ${CYAN}$location${NC}"
                ((removed_count++))
            else
                echo -e "${RED}❌${NC} Failed to remove: ${CYAN}$location${NC} (try with sudo)"
            fi
        fi
    done
    
    echo -e "${GREEN}✓${NC} Removed $removed_count binary file(s)"
}

# Remove configuration files
remove_config() {
    local config_locations=(
        "$HOME/.config/diranalyzer"
        "$HOME/.diranalyzer"
        "/etc/diranalyzer"
    )
    
    echo -e "${BLUE}🗑️${NC} Removing configuration files..."
    local removed_configs=0
    
    for config_dir in "${config_locations[@]}"; do
        if [[ -d "$config_dir" ]]; then
            if rm -rf "$config_dir" 2>/dev/null; then
                echo -e "${GREEN}✓${NC} Removed config: ${CYAN}$config_dir${NC}"
                ((removed_configs++))
            else
                echo -e "${RED}❌${NC} Failed to remove config: ${CYAN}$config_dir${NC}"
            fi
        fi
    done
    
    if [[ $removed_configs -eq 0 ]]; then
        echo -e "${GREEN}✓${NC} No configuration files found"
    fi
}

# Remove desktop entry
remove_desktop_entry() {
    local desktop_file="$HOME/.local/share/applications/diranalyzer.desktop"
    
    if [[ -f "$desktop_file" ]]; then
        echo -e "${BLUE}🗑️${NC} Removing desktop entry..."
        if rm "$desktop_file" 2>/dev/null; then
            echo -e "${GREEN}✓${NC} Removed desktop entry"
        else
            echo -e "${RED}❌${NC} Failed to remove desktop entry"
        fi
    fi
}

# Remove from PATH (if added by installer)
remove_from_path() {
    local bashrc_backup="$HOME/.bashrc.diranalyzer.bak"
    
    echo -e "${BLUE}🔧${NC} Checking PATH modifications..."
    
    if [[ -f "$HOME/.bashrc" ]]; then
        # Check if our PATH addition exists
        if grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.bashrc" 2>/dev/null; then
            echo -e "${YELLOW}⚠${NC} Found PATH modification in ~/.bashrc"
            read -p "Remove DirAnalyzer PATH addition from ~/.bashrc? (y/N): " -n 1 -r
            echo
            
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                # Create backup
                cp "$HOME/.bashrc" "$bashrc_backup"
                echo -e "${CYAN}💾${NC} Created backup: ${YELLOW}$bashrc_backup${NC}"
                
                # Remove the PATH addition
                sed -i '/export PATH="\$HOME\/\.local\/bin:\$PATH"/d' "$HOME/.bashrc"
                echo -e "${GREEN}✓${NC} Removed PATH modification"
                echo -e "${CYAN}💡${NC} Restart your terminal or run: ${YELLOW}source ~/.bashrc${NC}"
            fi
        else
            echo -e "${GREEN}✓${NC} No PATH modifications found"
        fi
    fi
}

# Remove cached data
remove_cache() {
    local cache_locations=(
        "$HOME/.cache/diranalyzer"
        "/tmp/diranalyzer-*"
        "/var/cache/diranalyzer"
    )
    
    echo -e "${BLUE}🗑️${NC} Removing cache and temporary files..."
    local removed_cache=0
    
    for cache_pattern in "${cache_locations[@]}"; do
        if ls $cache_pattern 1> /dev/null 2>&1; then
            if rm -rf $cache_pattern 2>/dev/null; then
                echo -e "${GREEN}✓${NC} Removed cache: ${CYAN}$cache_pattern${NC}"
                ((removed_cache++))
            fi
        fi
    done
    
    if [[ $removed_cache -eq 0 ]]; then
        echo -e "${GREEN}✓${NC} No cache files found"
    fi
}

# Verify complete removal
verify_removal() {
    echo -e "${BLUE}🔍${NC} Verifying complete removal..."
    
    if command -v "$BINARY_NAME" &> /dev/null; then
        local remaining_location
        remaining_location=$(command -v "$BINARY_NAME")
        echo -e "${YELLOW}⚠${NC} DirAnalyzer still found at: ${CYAN}$remaining_location${NC}"
        echo -e "${RED}❌${NC} Uninstallation incomplete"
        return 1
    else
        echo -e "${GREEN}✓${NC} DirAnalyzer completely removed from system"
        return 0
    fi
}

# Interactive mode
interactive_uninstall() {
    echo -e "${YELLOW}🤔${NC} What would you like to remove?"
    echo
    echo "1) Complete removal (recommended)"
    echo "2) Binary files only"
    echo "3) Configuration files only"
    echo "4) Cache and temporary files only"
    echo "5) Cancel"
    echo
    read -p "Choose an option (1-5): " -n 1 -r
    echo
    
    case $REPLY in
        1)
            echo -e "${BLUE}🗑️${NC} Performing complete removal..."
            return 0
            ;;
        2)
            local installations
            installations=$(find_installations)
            remove_binaries "$installations"
            echo -e "${GREEN}🎉${NC} Binary removal complete!"
            exit 0
            ;;
        3)
            remove_config
            echo -e "${GREEN}🎉${NC} Configuration removal complete!"
            exit 0
            ;;
        4)
            remove_cache
            echo -e "${GREEN}🎉${NC} Cache cleanup complete!"
            exit 0
            ;;
        5)
            echo -e "${YELLOW}❌${NC} Uninstallation cancelled"
            exit 0
            ;;
        *)
            echo -e "${RED}❌${NC} Invalid option"
            exit 1
            ;;
    esac
}

# Print final message
print_farewell() {
    echo
    echo -e "${GREEN}🎉 DirAnalyzer has been successfully uninstalled!${NC}"
    echo
    echo -e "${YELLOW}📊 Thanks for using DirAnalyzer!${NC}"
    echo -e "${CYAN}💡${NC} If you change your mind, you can always reinstall from:"
    echo -e "${BLUE}   https://github.com/yourusername/diranalyzer${NC}"
    echo
    echo -e "${PURPLE}🙏 We'd love your feedback on why you uninstalled:${NC}"
    echo -e "${BLUE}   https://github.com/yourusername/diranalyzer/discussions${NC}"
    echo
}

# Main uninstallation function
main() {
    print_banner
    
    # Check if DirAnalyzer is installed
    if ! command -v "$BINARY_NAME" &> /dev/null; then
        echo -e "${YELLOW}🤷${NC} DirAnalyzer doesn't appear to be installed"
        echo -e "${CYAN}💡${NC} Running cleanup anyway to remove any leftover files..."
        echo
    fi
    
    # Confirm uninstallation
    echo -e "${YELLOW}⚠${NC} This will remove DirAnalyzer from your system"
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}❌${NC} Uninstallation cancelled"
        exit 0
    fi
    
    # Find and remove installations
    local installations
    installations=$(find_installations)
    
    remove_binaries "$installations"
    remove_config
    remove_desktop_entry
    remove_from_path
    remove_cache
    
    # Verify and finish
    if verify_removal; then
        print_farewell
    else
        echo -e "${YELLOW}⚠${NC} Some files may still remain. Manual cleanup may be required."
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "DirAnalyzer Uninstallation Script"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h        Show this help message"
        echo "  --force, -f       Force removal without confirmation"
        echo "  --interactive, -i Interactive mode with options"
        echo "  --dry-run        Show what would be removed without doing it"
        echo
        exit 0
        ;;
    --force|-f)
        print_banner
        echo -e "${RED}⚡${NC} Force uninstalling DirAnalyzer..."
        installations=$(find_installations)
        remove_binaries "$installations"
        remove_config
        remove_desktop_entry
        remove_cache
        verify_removal && print_farewell
        ;;
    --interactive|-i)
        print_banner
        interactive_uninstall
        main
        ;;
    --dry-run)
        print_banner
        echo -e "${CYAN}🔍${NC} Dry run - showing what would be removed:"
        find_installations
        echo -e "${YELLOW}📝${NC} Would also check for config files, cache, and desktop entries"
        echo -e "${CYAN}💡${NC} Run without --dry-run to actually remove files"
        ;;
    *)
        main "$@"
        ;;
esac
