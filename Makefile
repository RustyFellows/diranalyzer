# DirAnalyzer Makefile
# Lightning-fast directory analysis tool

.PHONY: help build install uninstall clean test fmt clippy docs release dev all

# Default target
all: build

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m

# Configuration
BINARY_NAME := diranalyzer
INSTALL_DIR := /usr/local/bin
BUILD_DIR := target/release

help: ## Show this help message
	@echo "$(CYAN)🚀 DirAnalyzer - Lightning-Fast Directory Analysis Tool$(NC)"
	@echo "$(YELLOW)=====================================================$(NC)"
	@echo
	@echo "$(GREEN)Available targets:$(NC)"
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ { printf "  $(CYAN)%-15s$(NC) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
	@echo
	@echo "$(YELLOW)Quick start:$(NC)"
	@echo "  $(CYAN)make build$(NC)     # Build the project"
	@echo "  $(CYAN)make install$(NC)   # Install system-wide"
	@echo "  $(CYAN)make test$(NC)      # Run all tests"

build: ## Build the release binary
	@echo "$(CYAN)🔨 Building DirAnalyzer...$(NC)"
	cargo build --release
	@echo "$(GREEN)✓ Build complete!$(NC)"

build-debug: ## Build debug version
	@echo "$(CYAN)🔨 Building debug version...$(NC)"
	cargo build
	@echo "$(GREEN)✓ Debug build complete!$(NC)"

install: build ## Install DirAnalyzer system-wide
	@echo "$(CYAN)📦 Installing DirAnalyzer...$(NC)"
	@if [ ! -f "$(BUILD_DIR)/$(BINARY_NAME)" ]; then \
		echo "$(RED)❌ Binary not found. Run 'make build' first.$(NC)"; \
		exit 1; \
	fi
	@if [ "$$(id -u)" -eq 0 ]; then \
		cp $(BUILD_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/; \
		chmod +x $(INSTALL_DIR)/$(BINARY_NAME); \
		echo "$(GREEN)✓ Installed to $(INSTALL_DIR)/$(BINARY_NAME)$(NC)"; \
	else \
		echo "$(YELLOW)⚡ Using installation script for proper setup...$(NC)"; \
		./install.sh; \
	fi

install-user: build ## Install DirAnalyzer for current user
	@echo "$(CYAN)📦 Installing DirAnalyzer for current user...$(NC)"
	@mkdir -p $$HOME/.local/bin
	@cp $(BUILD_DIR)/$(BINARY_NAME) $$HOME/.local/bin/
	@chmod +x $$HOME/.local/bin/$(BINARY_NAME)
	@echo "$(GREEN)✓ Installed to $$HOME/.local/bin/$(BINARY_NAME)$(NC)"
	@echo "$(YELLOW)💡 Make sure $$HOME/.local/bin is in your PATH$(NC)"

uninstall: ## Uninstall DirAnalyzer
	@echo "$(CYAN)🗑️  Uninstalling DirAnalyzer...$(NC)"
	./uninstall.sh

clean: ## Clean build artifacts
	@echo "$(CYAN)🧹 Cleaning build artifacts...$(NC)"
	cargo clean
	@rm -rf target/
	@echo "$(GREEN)✓ Clean complete!$(NC)"

test: ## Run all tests
	@echo "$(CYAN)🧪 Running tests...$(NC)"
	cargo test --all-features --workspace
	@echo "$(GREEN)✓ Tests complete!$(NC)"

test-coverage: ## Run tests with coverage
	@echo "$(CYAN)📊 Running tests with coverage...$(NC)"
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info; \
		echo "$(GREEN)✓ Coverage report generated: lcov.info$(NC)"; \
	else \
		echo "$(YELLOW)⚠ cargo-llvm-cov not found. Install with: cargo install cargo-llvm-cov$(NC)"; \
		cargo test --all-features --workspace; \
	fi

fmt: ## Format code
	@echo "$(CYAN)✨ Formatting code...$(NC)"
	cargo fmt --all
	@echo "$(GREEN)✓ Code formatted!$(NC)"

clippy: ## Run clippy lints
	@echo "$(CYAN)🔍 Running clippy...$(NC)"
	cargo clippy --all-targets --all-features --workspace -- -D warnings
	@echo "$(GREEN)✓ Clippy checks passed!$(NC)"

docs: ## Generate documentation
	@echo "$(CYAN)📚 Generating documentation...$(NC)"
	cargo doc --all-features --no-deps --open
	@echo "$(GREEN)✓ Documentation generated!$(NC)"

docs-book: ## Build mdBook documentation (if available)
	@echo "$(CYAN)📖 Building documentation book...$(NC)"
	@if command -v mdbook >/dev/null 2>&1; then \
		mdbook build docs; \
		echo "$(GREEN)✓ Documentation book built!$(NC)"; \
	else \
		echo "$(YELLOW)⚠ mdbook not found. Install with: cargo install mdbook$(NC)"; \
	fi

bench: ## Run benchmarks
	@echo "$(CYAN)⚡ Running benchmarks...$(NC)"
	cargo bench
	@echo "$(GREEN)✓ Benchmarks complete!$(NC)"

audit: ## Security audit
	@echo "$(CYAN)🔒 Running security audit...$(NC)"
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
		echo "$(GREEN)✓ Security audit complete!$(NC)"; \
	else \
		echo "$(YELLOW)⚠ cargo-audit not found. Install with: cargo install cargo-audit$(NC)"; \
	fi

dev: fmt clippy test ## Full development check (format, lint, test)
	@echo "$(GREEN)🎉 Development checks complete!$(NC)"

ci: fmt clippy test audit ## CI pipeline (format, lint, test, audit)
	@echo "$(GREEN)🎉 CI pipeline complete!$(NC)"

release: clean ci build ## Full release build
	@echo "$(CYAN)🚀 Creating release build...$(NC)"
	@strip $(BUILD_DIR)/$(BINARY_NAME) 2>/dev/null || true
	@echo "$(GREEN)✓ Release build complete!$(NC)"
	@echo "$(YELLOW)📦 Binary location: $(BUILD_DIR)/$(BINARY_NAME)$(NC)"
	@echo "$(YELLOW)📏 Binary size: $$(du -h $(BUILD_DIR)/$(BINARY_NAME) | cut -f1)$(NC)"

demo: build ## Run demo analysis on current directory
	@echo "$(CYAN)🎬 Running demo analysis...$(NC)"
	@$(BUILD_DIR)/$(BINARY_NAME) . --top 5 || echo "$(YELLOW)⚠ Build the project first with 'make build'$(NC)"

demo-full: build ## Run full demo with duplicate detection
	@echo "$(CYAN)🎬 Running full demo with duplicate detection...$(NC)"
	@$(BUILD_DIR)/$(BINARY_NAME) . --duplicates --top 10 --verbose || echo "$(YELLOW)⚠ Build the project first with 'make build'$(NC)"

package: release ## Create distribution package
	@echo "$(CYAN)📦 Creating distribution package...$(NC)"
	@mkdir -p dist
	@cp $(BUILD_DIR)/$(BINARY_NAME) dist/
	@cp README.md LICENSE install.sh uninstall.sh dist/
	@tar -czf dist/diranalyzer-$$(date +%Y%m%d).tar.gz -C dist .
	@echo "$(GREEN)✓ Package created in dist/ directory$(NC)"

docker-build: ## Build Docker image
	@echo "$(CYAN)🐳 Building Docker image...$(NC)"
	@if command -v docker >/dev/null 2>&1; then \
		docker build -t diranalyzer:latest .; \
		echo "$(GREEN)✓ Docker image built!$(NC)"; \
	else \
		echo "$(RED)❌ Docker not found$(NC)"; \
	fi

check-deps: ## Check for outdated dependencies
	@echo "$(CYAN)📋 Checking dependencies...$(NC)"
	@if command -v cargo-outdated >/dev/null 2>&1; then \
		cargo outdated; \
	else \
		echo "$(YELLOW)⚠ cargo-outdated not found. Install with: cargo install cargo-outdated$(NC)"; \
	fi

size: build ## Show binary size information
	@echo "$(CYAN)📏 Binary size analysis:$(NC)"
	@ls -lh $(BUILD_DIR)/$(BINARY_NAME)
	@echo "$(YELLOW)💡 Stripped size:$(NC)"
	@cp $(BUILD_DIR)/$(BINARY_NAME) /tmp/$(BINARY_NAME)-stripped
	@strip /tmp/$(BINARY_NAME)-stripped 2>/dev/null || true
	@ls -lh /tmp/$(BINARY_NAME)-stripped
	@rm -f /tmp/$(BINARY_NAME)-stripped

watch: ## Watch for changes and rebuild
	@echo "$(CYAN)👀 Watching for changes...$(NC)"
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x 'build'; \
	else \
		echo "$(YELLOW)⚠ cargo-watch not found. Install with: cargo install cargo-watch$(NC)"; \
		echo "$(YELLOW)💡 Falling back to manual rebuild...$(NC)"; \
		$(MAKE) build; \
	fi

setup-dev: ## Setup development environment
	@echo "$(CYAN)🛠️  Setting up development environment...$(NC)"
	@rustup component add rustfmt clippy
	@cargo install cargo-watch cargo-audit cargo-llvm-cov cargo-outdated
	@chmod +x install.sh uninstall.sh
	@echo "$(GREEN)✓ Development environment ready!$(NC)"

.PHONY: show-info
show-info: ## Show project information
	@echo "$(CYAN)📊 DirAnalyzer Project Information$(NC)"
	@echo "$(YELLOW)=================================$(NC)"
	@echo "Binary name:     $(BINARY_NAME)"
	@echo "Install dir:     $(INSTALL_DIR)"
	@echo "Build dir:       $(BUILD_DIR)"
	@echo "Rust version:    $$(rustc --version 2>/dev/null || echo 'Not installed')"
	@echo "Cargo version:   $$(cargo --version 2>/dev/null || echo 'Not installed')"
	@if [ -f "$(BUILD_DIR)/$(BINARY_NAME)" ]; then \
		echo "Binary size:     $$(du -h $(BUILD_DIR)/$(BINARY_NAME) | cut -f1)"; \
		echo "Binary status:   $(GREEN)✓ Built$(NC)"; \
	else \
		echo "Binary status:   $(YELLOW)⚠ Not built$(NC)"; \
	fi
	@if command -v $(BINARY_NAME) >/dev/null 2>&1; then \
		echo "Install status:  $(GREEN)✓ Installed$(NC)"; \
		echo "Install path:    $$(which $(BINARY_NAME))"; \
	else \
		echo "Install status:  $(YELLOW)⚠ Not installed$(NC)"; \
	fi
