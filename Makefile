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
	@chmod +x scripts/makefile-scripts/install.sh
	@./scripts/makefile-scripts/install.sh

install-tagui: ## Install TagUI for automation
	@chmod +x scripts/makefile-scripts/install-tagui.sh
	@./scripts/makefile-scripts/install-tagui.sh

setup: install install-tagui ## Complete setup (install all dependencies)
	@chmod +x scripts/makefile-scripts/setup.sh
	@./scripts/makefile-scripts/setup.sh

# Development
dev: ## Start development server
	@echo "$(YELLOW)Starting Codialog in development mode...$(NC)"
	npm run dev

build: ## Build production application
	@echo "$(YELLOW)Building Codialog for production...$(NC)"
	npm run build

# Legacy Testing (replaced by Rust tests in Test Management section)
test-e2e: ## Run E2E tests only
	npm run test:e2e

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
	@chmod +x scripts/makefile-scripts/clean.sh
	@./scripts/makefile-scripts/clean.sh

reset: clean install ## Reset project (clean + reinstall)

# File Operations
create-script: ## Create new DSL script (usage: make create-script NAME=script_name)
	@if [ -z "$(NAME)" ]; then \
		echo "$(RED)Error: Please provide script name. Usage: make create-script NAME=my_script$(NC)"; \
		exit 1; \
	fi
	@chmod +x scripts/makefile-scripts/create-script.sh
	@./scripts/makefile-scripts/create-script.sh $(NAME)

list-scripts: ## List all available DSL scripts
	@chmod +x scripts/makefile-scripts/list-scripts.sh
	@./scripts/makefile-scripts/list-scripts.sh

# Information
info: ## Show project information
	@chmod +x scripts/makefile-scripts/info.sh
	@./scripts/makefile-scripts/info.sh

status: ## Show project status
	@chmod +x scripts/makefile-scripts/status.sh
	@./scripts/makefile-scripts/status.sh

# Health Checks
health: ## Check if services are running
	@chmod +x scripts/makefile-scripts/health.sh
	@./scripts/makefile-scripts/health.sh

logs: ## Show application logs
	@chmod +x scripts/makefile-scripts/logs.sh
	@./scripts/makefile-scripts/logs.sh

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

# Docker and Services Management
docker-up: ## Start all Docker services (PostgreSQL, Redis, Vaultwarden, Bitwarden CLI)
	@echo "$(YELLOW)Starting Docker services...$(NC)"
	./scripts/init/docker-setup.sh

docker-down: ## Stop all Docker services
	@echo "$(YELLOW)Stopping Docker services...$(NC)"
	docker-compose -f docker-compose.bitwarden.yml down

docker-restart: docker-down docker-up ## Restart all Docker services

docker-logs: ## Show logs from all Docker services
	docker-compose -f docker-compose.bitwarden.yml logs -f

docker-status: ## Check status of all Docker services
	@echo "$(YELLOW)Docker Services Status:$(NC)"
	@docker-compose -f docker-compose.bitwarden.yml ps

docker-clean: ## Clean Docker containers, volumes, and networks
	@echo "$(YELLOW)Cleaning Docker resources...$(NC)"
	docker-compose -f docker-compose.bitwarden.yml down -v
	docker volume prune -f
	docker network prune -f
	@echo "$(GREEN)Docker resources cleaned$(NC)"

# Legacy Database Management (replaced by newer targets below)

db-backup: ## Create database backup
	@chmod +x scripts/makefile-scripts/db-backup.sh
	@./scripts/makefile-scripts/db-backup.sh

# Bitwarden Management  
bw-status: ## Check Bitwarden CLI status
	@chmod +x scripts/makefile-scripts/bw-status.sh
	@./scripts/makefile-scripts/bw-status.sh

bw-sync: ## Sync Bitwarden vault
	@echo "$(YELLOW)Syncing Bitwarden vault...$(NC)"
	@docker exec codialog-bitwarden-cli bw sync

bw-unlock: ## Unlock Bitwarden vault (requires master password)
	@echo "$(YELLOW)Unlocking Bitwarden vault...$(NC)"
	@docker exec -it codialog-bitwarden-cli bw unlock

# Application Setup and Management
init: ## Initialize application (complete setup)
	@echo "$(YELLOW)Initializing Codialog application...$(NC)"
	./scripts/init/setup.sh

init-docker: ## Initialize Docker environment only
	./scripts/init/docker-setup.sh

# Development with Services
dev-full: docker-up dev ## Start all services and development server

# Health Checks Enhanced
health-all: ## Check health of all services (app + Docker)
	@chmod +x scripts/makefile-scripts/health-all.sh
	@./scripts/makefile-scripts/health-all.sh

# Logs Management
logs-app: ## Show application logs only
	@chmod +x scripts/makefile-scripts/logs-app.sh
	@./scripts/makefile-scripts/logs-app.sh

logs-docker: ## Show Docker service logs
	make docker-logs

logs-all: ## Show all logs (app + Docker)
	@echo "$(YELLOW)Starting log monitoring (Ctrl+C to stop)...$(NC)"
	@make logs-app & make logs-docker

# Environment management
env-check: ## Validate environment configuration  
	@chmod +x scripts/makefile-scripts/env-check.sh
	@./scripts/makefile-scripts/env-check.sh

env-template: ## Create .env from template
	@chmod +x scripts/makefile-scripts/env-template.sh
	@./scripts/makefile-scripts/env-template.sh

# Data Management
data-clean: ## Clean application data (uploads, logs, sessions)
	@chmod +x scripts/makefile-scripts/data-clean.sh
	@./scripts/makefile-scripts/data-clean.sh

data-backup: ## Backup application data
	@chmod +x scripts/makefile-scripts/data-backup.sh
	@./scripts/makefile-scripts/data-backup.sh

# Test Management
test: ## Run all tests
	@echo "$(YELLOW)🧪 Running all tests...$(NC)"
	cd src-tauri && cargo test --verbose

test-unit: ## Run unit tests only
	@echo "$(YELLOW)🧪 Running unit tests...$(NC)"
	cd src-tauri && cargo test --lib --verbose

test-integration: ## Run integration tests only
	@echo "$(YELLOW)🧪 Running integration tests...$(NC)"
	cd src-tauri && cargo test --features integration_tests --verbose

test-coverage: ## Generate test coverage report
	@chmod +x scripts/makefile-scripts/test-coverage.sh
	@./scripts/makefile-scripts/test-coverage.sh

test-watch: ## Run tests in watch mode
	@chmod +x scripts/makefile-scripts/test-watch.sh
	@./scripts/makefile-scripts/test-watch.sh

test-bench: ## Run performance benchmarks
	@echo "$(YELLOW)⚡ Running performance benchmarks...$(NC)"
	cd src-tauri && cargo bench

test-clean: ## Clean test artifacts
	@chmod +x scripts/makefile-scripts/test-clean.sh
	@./scripts/makefile-scripts/test-clean.sh

# Database Management
db-migrate: ## Run database migrations
	@chmod +x scripts/makefile-scripts/db-migrate.sh
	@./scripts/makefile-scripts/db-migrate.sh

db-reset: ## Reset database (drop and recreate)
	@chmod +x scripts/makefile-scripts/db-reset.sh
	@./scripts/makefile-scripts/db-reset.sh

db-seed: ## Seed database with test data
	@chmod +x scripts/makefile-scripts/db-seed.sh
	@./scripts/makefile-scripts/db-seed.sh

# Performance and Monitoring
perf-test: ## Run performance tests
	@chmod +x scripts/makefile-scripts/perf-test.sh
	@./scripts/makefile-scripts/perf-test.sh

monitor: ## Start monitoring dashboard
	@chmod +x scripts/makefile-scripts/monitor.sh
	@./scripts/makefile-scripts/monitor.sh

# Maintenance
maintenance-mode: ## Enable maintenance mode
	@chmod +x scripts/makefile-scripts/maintenance-mode.sh
	@./scripts/makefile-scripts/maintenance-mode.sh

maintenance-off: ## Disable maintenance mode
	@chmod +x scripts/makefile-scripts/maintenance-off.sh
	@./scripts/makefile-scripts/maintenance-off.sh

# Documentation
docs: ## Generate documentation
	@echo "$(YELLOW)📚 Generating documentation...$(NC)"
	cd src-tauri && cargo doc --no-deps --open 2>/dev/null || cargo doc --no-deps
	@echo "$(GREEN)Documentation generated and opened in browser$(NC)"

docs-api: ## Generate API documentation
	@echo "$(YELLOW)📋 API Documentation available at:$(NC)"
	@echo "  - Health Check: http://localhost:3000/health"
	@echo "  - DSL Generation: POST http://localhost:3000/dsl/generate"
	@echo "  - Bitwarden Login: POST http://localhost:3000/bitwarden/login"
	@echo "  - Session Management: GET/POST http://localhost:3000/session"

# Quick Commands
full-reset: docker-down clean data-clean test-clean init docker-up ## Complete reset (clean everything and reinitialize)

quick-start: init docker-up dev ## Quick start for new users (complete setup and run)

quick-test: test-unit test-integration ## Quick test suite (unit + integration)

dev-setup: init docker-up db-migrate db-seed ## Complete development setup

.PHONY: test test-unit test-integration test-coverage test-watch test-bench test-clean \
        db-migrate db-reset db-seed perf-test monitor maintenance-mode maintenance-off \
        docs docs-api full-reset quick-start quick-test dev-setup
