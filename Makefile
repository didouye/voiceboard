# Voiceboard Makefile
# Run `make help` to see available commands

.PHONY: help test test-rust test-angular lint lint-rust lint-angular format format-rust format-angular check build dev clean coverage

# Colors for output
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

help: ## Show this help
	@echo "$(CYAN)Voiceboard Development Commands$(RESET)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(RESET) %s\n", $$1, $$2}'

# =============================================================================
# Testing
# =============================================================================

test: test-rust test-angular ## Run all tests

test-rust: ## Run Rust tests
	@echo "$(CYAN)Running Rust tests...$(RESET)"
	cargo test --manifest-path src-tauri/Cargo.toml

test-angular: ## Run Angular tests (headless)
	@echo "$(CYAN)Running Angular tests...$(RESET)"
	npm test -- --no-watch --browsers=ChromeHeadless

test-watch: ## Run Angular tests in watch mode
	npm test

# =============================================================================
# Linting
# =============================================================================

lint: lint-rust lint-angular ## Run all linters

lint-rust: ## Run Rust linter (clippy)
	@echo "$(CYAN)Running Clippy...$(RESET)"
	cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

lint-angular: ## Run Angular linter (ESLint)
	@echo "$(CYAN)Running ESLint...$(RESET)"
	npm run lint --if-present || echo "$(YELLOW)No lint script configured$(RESET)"

# =============================================================================
# Formatting
# =============================================================================

format: format-rust format-angular ## Format all code

format-rust: ## Format Rust code
	@echo "$(CYAN)Formatting Rust code...$(RESET)"
	cargo fmt --manifest-path src-tauri/Cargo.toml

format-angular: ## Format Angular code (Prettier)
	@echo "$(CYAN)Formatting Angular code...$(RESET)"
	npx prettier --write "src/**/*.{ts,html,scss,css}" --ignore-unknown 2>/dev/null || echo "$(YELLOW)Prettier not configured$(RESET)"

# =============================================================================
# Pre-commit checks
# =============================================================================

check: format lint test ## Run format, lint, and tests (pre-commit)
	@echo "$(GREEN)All checks passed!$(RESET)"

check-rust: format-rust lint-rust test-rust ## Run all Rust checks
	@echo "$(GREEN)Rust checks passed!$(RESET)"

check-angular: format-angular lint-angular test-angular ## Run all Angular checks
	@echo "$(GREEN)Angular checks passed!$(RESET)"

# =============================================================================
# Building
# =============================================================================

build: ## Build the application (release)
	@echo "$(CYAN)Building application...$(RESET)"
	npm run tauri build

build-debug: ## Build the application (debug)
	@echo "$(CYAN)Building application (debug)...$(RESET)"
	npm run build
	cargo build --manifest-path src-tauri/Cargo.toml

dev: ## Start development server
	npm run tauri dev

# =============================================================================
# Coverage
# =============================================================================

coverage: coverage-rust ## Run code coverage

coverage-rust: ## Run Rust code coverage (requires cargo-tarpaulin)
	@echo "$(CYAN)Running Rust coverage...$(RESET)"
	@command -v cargo-tarpaulin >/dev/null 2>&1 || { echo "$(YELLOW)Installing cargo-tarpaulin...$(RESET)"; cargo install cargo-tarpaulin; }
	cd src-tauri && cargo tarpaulin --out Stdout

# =============================================================================
# Maintenance
# =============================================================================

clean: ## Clean build artifacts
	@echo "$(CYAN)Cleaning build artifacts...$(RESET)"
	rm -rf dist/
	rm -rf src-tauri/target/
	rm -rf node_modules/.cache/
	@echo "$(GREEN)Clean complete$(RESET)"

clean-all: clean ## Clean everything including node_modules
	rm -rf node_modules/
	@echo "$(YELLOW)Run 'npm install' to restore dependencies$(RESET)"

deps: ## Install/update dependencies
	@echo "$(CYAN)Installing dependencies...$(RESET)"
	npm install
	@echo "$(GREEN)Dependencies installed$(RESET)"

update: ## Update dependencies
	@echo "$(CYAN)Updating dependencies...$(RESET)"
	npm update
	cargo update --manifest-path src-tauri/Cargo.toml
	@echo "$(GREEN)Dependencies updated$(RESET)"

# =============================================================================
# Git helpers
# =============================================================================

status: ## Show git status and recent commits
	@echo "$(CYAN)Git Status:$(RESET)"
	@git status --short
	@echo ""
	@echo "$(CYAN)Recent commits:$(RESET)"
	@git log --oneline -5

diff: ## Show staged and unstaged changes
	@echo "$(CYAN)Staged changes:$(RESET)"
	@git diff --cached --stat
	@echo ""
	@echo "$(CYAN)Unstaged changes:$(RESET)"
	@git diff --stat
