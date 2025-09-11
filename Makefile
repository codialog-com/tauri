.PHONY: help install dev build test clean setup

# Colors for output
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Default target
help: ## Show this help message
	@echo '$(YELLOW)Codialog - Intelligent CV Automation$(NC)'
	@echo ''
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Setup and Installation
install: ## Install all dependencies (Rust + Node.js + TagUI)
	@echo "$(YELLOW)Installing Codialog dependencies...$(NC)"
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "$(RED)Error: Rust/Cargo not found. Install from https://rustup.rs/$(NC)"; \
		exit 1; \
	fi
	@if ! command -v npm >/dev/null 2>&1; then \
		echo "$(RED)Error: Node.js/npm not found. Install Node.js first$(NC)"; \
		exit 1; \
	fi
	npm install
	cargo install tauri-cli
	@echo "$(GREEN)Dependencies installed successfully!$(NC)"

install-tagui: ## Install TagUI for automation
	@echo "$(YELLOW)Installing TagUI...$(NC)"
	@if [ ! -d "tagui" ]; then \
		git clone https://github.com/aisingapore/tagui && \
		cd tagui && npm install && \
		echo "$(GREEN)TagUI installed successfully!$(NC)"; \
	else \
		echo "$(YELLOW)TagUI already exists$(NC)"; \
	fi

setup: install install-tagui ## Complete setup (install all dependencies)
	@echo "$(YELLOW)Setting up environment...$(NC)"
	@if [ ! -f ".env" ]; then \
		cp .env.example .env && \
		echo "$(GREEN)Created .env file from template$(NC)"; \
	fi
	mkdir -p uploads logs
	@echo "$(GREEN)Setup completed! Run 'make dev' to start development$(NC)"

# Development
dev: ## Start development server
	@echo "$(YELLOW)Starting Codialog in development mode...$(NC)"
	npm run dev

build: ## Build production application
	@echo "$(YELLOW)Building Codialog for production...$(NC)"
	npm run build

# Testing
test: ## Run all tests
	@echo "$(YELLOW)Running all tests...$(NC)"
	npm test

test-unit: ## Run unit tests only
	npm run test:unit

test-e2e: ## Run E2E tests only
	npm run test:e2e

test-watch: ## Run tests in watch mode
	npm run test:watch

coverage: ## Generate test coverage report
	npm run test:coverage
	@echo "$(GREEN)Coverage report generated in coverage/index.html$(NC)"

# Code Quality
lint: ## Lint code (JavaScript + Rust)
	@echo "$(YELLOW)Linting code...$(NC)"
	npm run lint

format: ## Format code
	npm run format

check: lint test ## Run linting and tests

# Project Management
clean: ## Clean build artifacts and dependencies
	@echo "$(YELLOW)Cleaning project...$(NC)"
	rm -rf node_modules
	rm -rf src-tauri/target
	rm -rf coverage
	rm -rf dist
	rm -rf uploads/*
	rm -rf logs/*
	@echo "$(GREEN)Project cleaned!$(NC)"

reset: clean install ## Reset project (clean + reinstall)
	@echo "$(GREEN)Project reset completed!$(NC)"

# File Operations
create-script: ## Create new DSL script (usage: make create-script NAME=script_name)
	@if [ -z "$(NAME)" ]; then \
		echo "$(RED)Error: Please provide script name. Usage: make create-script NAME=my_script$(NC)"; \
		exit 1; \
	fi
	@echo "// New DSL Script: $(NAME)" > scripts/$(NAME).codialog
	@echo "// Created: $$(date)" >> scripts/$(NAME).codialog
	@echo "" >> scripts/$(NAME).codialog
	@echo "// Add your DSL commands here" >> scripts/$(NAME).codialog
	@echo "click \"#example\"" >> scripts/$(NAME).codialog
	@echo "type \"#field\" \"value\"" >> scripts/$(NAME).codialog
	@echo "$(GREEN)Created new script: scripts/$(NAME).codialog$(NC)"

list-scripts: ## List all available DSL scripts
	@echo "$(YELLOW)Available DSL scripts:$(NC)"
	@find scripts -name "*.codialog" -exec basename {} .codialog \; | sort | sed 's/^/  - /'

# Information
info: ## Show project information
	@echo "$(YELLOW)Codialog Project Information$(NC)"
	@echo "Version: $$(grep '"version":' package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')"
	@echo "Node.js: $$(node --version 2>/dev/null || echo 'Not installed')"
	@echo "npm: $$(npm --version 2>/dev/null || echo 'Not installed')"
	@echo "Rust: $$(rustc --version 2>/dev/null || echo 'Not installed')"
	@echo "Cargo: $$(cargo --version 2>/dev/null || echo 'Not installed')"
	@echo "Tauri CLI: $$(tauri --version 2>/dev/null || echo 'Not installed')"
	@echo "TagUI: $$(if [ -d 'tagui' ]; then echo 'Installed'; else echo 'Not installed'; fi)"

status: ## Show project status
	@echo "$(YELLOW)Project Status:$(NC)"
	@echo "Backend files: $$(find src-tauri/src -name "*.rs" | wc -l) Rust files"
	@echo "Frontend files: $$(find src -name "*.html" -o -name "*.js" -o -name "*.css" | wc -l) files"
	@echo "DSL scripts: $$(find scripts -name "*.codialog" 2>/dev/null | wc -l) scripts"
	@echo "Dependencies: $$(if [ -d 'node_modules' ]; then echo 'Installed'; else echo 'Not installed'; fi)"
	@echo "TagUI: $$(if [ -d 'tagui' ]; then echo 'Ready'; else echo 'Not installed'; fi)"

# Health Checks
health: ## Check if services are running
	@echo "$(YELLOW)Checking service health...$(NC)"
	@curl -s http://localhost:4000/health >/dev/null && \
		echo "$(GREEN)✓ Backend API (port 4000)$(NC)" || \
		echo "$(RED)✗ Backend API not running$(NC)"
	@curl -s http://localhost:1420 >/dev/null && \
		echo "$(GREEN)✓ Frontend (port 1420)$(NC)" || \
		echo "$(RED)✗ Frontend not running$(NC)"

logs: ## Show application logs
	@echo "$(YELLOW)Application logs:$(NC)"
	@if [ -f "logs/codialog.log" ]; then \
		tail -f logs/codialog.log; \
	else \
		echo "$(RED)No log file found$(NC)"; \
	fi

# Quick Start
start: setup dev ## Complete setup and start development (first time users)

# Debug mode
debug: ## Start in debug mode with verbose logging
	RUST_LOG=debug npm run dev

# Production deployment helpers
build-release: ## Build optimized release version
	npm run build
	@echo "$(GREEN)Release build completed!$(NC)"

# Development utilities
watch-backend: ## Watch backend files for changes
	@echo "$(YELLOW)Watching Rust backend files...$(NC)"
	cargo watch -x "check --manifest-path src-tauri/Cargo.toml"

serve-docs: ## Serve documentation (if available)
	@if [ -d "docs" ]; then \
		python3 -m http.server 8080 -d docs; \
	else \
		echo "$(RED)No docs directory found$(NC)"; \
	fi

# Environment management
env-check: ## Validate environment configuration  
	@echo "$(YELLOW)Checking environment...$(NC)"
	@if [ ! -f ".env" ]; then \
		echo "$(RED)No .env file found. Run 'make setup' first$(NC)"; \
		exit 1; \
	fi
	@echo "$(GREEN)Environment configuration found$(NC)"
