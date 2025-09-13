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
	@echo "$(YELLOW)Creating database backup...$(NC)"
	@mkdir -p data/backups
	@docker exec codialog-postgres pg_dump -U $${POSTGRES_USER:-codialog} $${POSTGRES_DB:-codialog} > data/backups/backup_$$(date +%Y%m%d_%H%M%S).sql
	@echo "$(GREEN)Database backup created in data/backups/$(NC)"

# Bitwarden Management  
bw-status: ## Check Bitwarden CLI status
	@echo "$(YELLOW)Checking Bitwarden status...$(NC)"
	@if docker ps | grep -q codialog-bitwarden-cli; then \
		docker exec codialog-bitwarden-cli bw status; \
	else \
		echo "$(RED)Bitwarden CLI container not running$(NC)"; \
	fi

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
	@echo "$(YELLOW)Checking all service health...$(NC)"
	@make health
	@echo ""
	@make docker-status
	@echo ""
	@curl -s http://localhost:8080/alive >/dev/null && \
		echo "$(GREEN)✓ Vaultwarden (port 8080)$(NC)" || \
		echo "$(RED)✗ Vaultwarden not accessible$(NC)"

# Logs Management
logs-app: ## Show application logs only
	@echo "$(YELLOW)Application logs:$(NC)"
	@find logs -name "*.log" -exec tail -f {} +

logs-docker: ## Show Docker service logs
	make docker-logs

logs-all: ## Show all logs (app + Docker)
	@echo "$(YELLOW)Starting log monitoring (Ctrl+C to stop)...$(NC)"
	@make logs-app & make logs-docker

# Environment management
env-check: ## Validate environment configuration  
	@echo "$(YELLOW)Checking environment...$(NC)"
	@if [ ! -f ".env" ]; then \
		echo "$(RED)No .env file found. Run 'make init' first$(NC)"; \
		exit 1; \
	fi
	@echo "$(GREEN)Environment configuration found$(NC)"

env-template: ## Create .env from template
	@if [ ! -f ".env" ]; then \
		cp .env.example .env; \
		echo "$(GREEN)Created .env from template$(NC)"; \
	else \
		echo "$(YELLOW).env already exists$(NC)"; \
	fi

# Data Management
data-clean: ## Clean application data (uploads, logs, sessions)
	@echo "$(YELLOW)Cleaning application data...$(NC)"
	rm -rf data/uploads/* data/sessions/* data/logs/* 2>/dev/null || true
	rm -rf src-tauri/data/uploads/* src-tauri/data/sessions/* src-tauri/data/logs/* 2>/dev/null || true
	@echo "$(GREEN)Application data cleaned$(NC)"

data-backup: ## Backup application data
	@echo "$(YELLOW)Creating data backup...$(NC)"
	@mkdir -p data/backups
	@tar -czf data/backups/data_backup_$$(date +%Y%m%d_%H%M%S).tar.gz data/ src-tauri/data/
	@echo "$(GREEN)Data backup created in data/backups/$(NC)"

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
	@echo "$(YELLOW)📊 Generating test coverage report...$(NC)"
	cd src-tauri && cargo install cargo-tarpaulin --locked 2>/dev/null || true
	cd src-tauri && cargo tarpaulin --out Html --output-dir ../coverage --timeout 300
	@echo "$(GREEN)Coverage report generated in coverage/tarpaulin-report.html$(NC)"

test-watch: ## Run tests in watch mode
	@echo "$(YELLOW)👀 Running tests in watch mode...$(NC)"
	cd src-tauri && cargo install cargo-watch --locked 2>/dev/null || true
	cd src-tauri && cargo watch -x test

test-bench: ## Run performance benchmarks
	@echo "$(YELLOW)⚡ Running performance benchmarks...$(NC)"
	cd src-tauri && cargo bench

test-clean: ## Clean test artifacts
	@echo "$(YELLOW)🧹 Cleaning test artifacts...$(NC)"
	cd src-tauri && cargo clean
	rm -rf coverage/ 2>/dev/null || true
	@echo "$(GREEN)Test artifacts cleaned$(NC)"

# Database Management
db-migrate: ## Run database migrations
	@echo "$(YELLOW)🗄️  Running database migrations...$(NC)"
	@if docker exec codialog-postgres pg_isready -U codialog_user > /dev/null 2>&1; then \
		echo "$(GREEN)Database is ready, running migrations...$(NC)"; \
		cd src-tauri && cargo run --bin migrate 2>/dev/null || echo "$(YELLOW)Migration binary not found, skipping$(NC)"; \
	else \
		echo "$(RED)Database is not ready, please start it first with 'make docker-up'$(NC)"; \
	fi

db-reset: ## Reset database (drop and recreate)
	@echo "$(YELLOW)🗄️  Resetting database...$(NC)"
	docker exec codialog-postgres psql -U codialog_user -d postgres -c "DROP DATABASE IF EXISTS codialog;"
	docker exec codialog-postgres psql -U codialog_user -d postgres -c "CREATE DATABASE codialog;"
	@echo "$(GREEN)Database reset completed$(NC)"

db-seed: ## Seed database with test data
	@echo "$(YELLOW)🌱 Seeding database with test data...$(NC)"
	docker exec codialog-postgres psql -U codialog_user -d codialog -c "\
		INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) VALUES \
		('test-session-1', '{\"email\":\"test@example.com\",\"name\":\"Test User\"}', NOW(), NOW(), NOW() + INTERVAL '1 day', true), \
		('test-session-2', '{\"email\":\"demo@example.com\",\"name\":\"Demo User\"}', NOW(), NOW(), NOW() + INTERVAL '1 day', true) \
		ON CONFLICT (session_id) DO NOTHING;"
	@echo "$(GREEN)Database seeded with test data$(NC)"

# Performance and Monitoring
perf-test: ## Run performance tests
	@echo "$(YELLOW)⚡ Running performance tests...$(NC)"
	@echo "Testing API endpoints..."
	@for endpoint in "/health" "/api/status"; do \
		echo "Testing $$endpoint..."; \
		curl -w "Time: %{time_total}s\n" -s "http://localhost:3000$$endpoint" -o /dev/null || true; \
	done

monitor: ## Start monitoring dashboard
	@echo "$(YELLOW)📊 Starting monitoring dashboard...$(NC)"
	@echo "Database status:"
	@docker exec codialog-postgres pg_isready -U codialog_user || echo "Database not ready"
	@echo "Redis status:"
	@docker exec codialog-redis redis-cli ping || echo "Redis not ready"
	@echo "Application logs (last 10 lines):"
	@tail -n 10 logs/app.log 2>/dev/null || echo "No app logs found"

# Maintenance
maintenance-mode: ## Enable maintenance mode
	@echo "$(YELLOW)🚧 Enabling maintenance mode...$(NC)"
	@echo "maintenance" > .maintenance
	docker pause codialog-app 2>/dev/null || true
	@echo "$(GREEN)Maintenance mode enabled$(NC)"

maintenance-off: ## Disable maintenance mode
	@echo "$(YELLOW)🔧 Disabling maintenance mode...$(NC)"
	@rm -f .maintenance
	docker unpause codialog-app 2>/dev/null || true
	@echo "$(GREEN)Maintenance mode disabled$(NC)"

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
