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
	@chmod +x scripts/makefile-scripts/dev.sh
	@./scripts/makefile-scripts/dev.sh

build: ## Build production application
	@chmod +x scripts/makefile-scripts/build.sh
	@./scripts/makefile-scripts/build.sh

# Legacy Testing (replaced by Rust tests in Test Management section)
test-e2e: ## Run E2E tests only
	@chmod +x scripts/makefile-scripts/test-e2e.sh
	@./scripts/makefile-scripts/test-e2e.sh

coverage: ## Generate test coverage report
	@chmod +x scripts/makefile-scripts/coverage.sh
	@./scripts/makefile-scripts/coverage.sh

# Code Quality
lint: ## Lint code (JavaScript + Rust)
	@chmod +x scripts/makefile-scripts/lint.sh
	@./scripts/makefile-scripts/lint.sh

format: ## Format code
	@chmod +x scripts/makefile-scripts/format.sh
	@./scripts/makefile-scripts/format.sh

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
	@chmod +x scripts/makefile-scripts/debug.sh
	@./scripts/makefile-scripts/debug.sh

# Production deployment helpers
build-release: ## Build optimized release version
	@chmod +x scripts/makefile-scripts/build-release.sh
	@./scripts/makefile-scripts/build-release.sh

# Development utilities
watch-backend: ## Watch backend files for changes
	@chmod +x scripts/makefile-scripts/watch-backend.sh
	@./scripts/makefile-scripts/watch-backend.sh

serve-docs: ## Serve documentation (if available)
	@chmod +x scripts/makefile-scripts/serve-docs.sh
	@./scripts/makefile-scripts/serve-docs.sh

# Docker and Services Management
docker-up: ## Start all Docker services (PostgreSQL, Redis, Vaultwarden, Bitwarden CLI)
	@chmod +x scripts/makefile-scripts/docker-up.sh
	@./scripts/makefile-scripts/docker-up.sh

docker-down: ## Stop all Docker services
	@chmod +x scripts/makefile-scripts/docker-down.sh
	@./scripts/makefile-scripts/docker-down.sh

docker-restart: docker-down docker-up ## Restart all Docker services

docker-logs: ## Show logs from all Docker services
	@chmod +x scripts/makefile-scripts/docker-logs.sh
	@./scripts/makefile-scripts/docker-logs.sh

docker-status: ## Check status of all Docker services
	@chmod +x scripts/makefile-scripts/docker-status.sh
	@./scripts/makefile-scripts/docker-status.sh

docker-clean: ## Clean Docker containers, volumes, and networks
	@chmod +x scripts/makefile-scripts/docker-clean.sh
	@./scripts/makefile-scripts/docker-clean.sh

# Legacy Database Management (replaced by newer targets below)

db-backup: ## Create database backup
	@chmod +x scripts/makefile-scripts/db-backup.sh
	@./scripts/makefile-scripts/db-backup.sh

# Bitwarden Management  
bw-status: ## Check Bitwarden CLI status
	@chmod +x scripts/makefile-scripts/bw-status.sh
	@./scripts/makefile-scripts/bw-status.sh

bw-sync: ## Sync Bitwarden vault
	@chmod +x scripts/makefile-scripts/bw-sync.sh
	@./scripts/makefile-scripts/bw-sync.sh

bw-unlock: ## Unlock Bitwarden vault (requires master password)
	@chmod +x scripts/makefile-scripts/bw-unlock.sh
	@./scripts/makefile-scripts/bw-unlock.sh

# Application Setup and Management
init: ## Initialize application (complete setup)
	@chmod +x scripts/makefile-scripts/init.sh
	@./scripts/makefile-scripts/init.sh

init-docker: ## Initialize Docker environment only
	@chmod +x scripts/makefile-scripts/init-docker.sh
	@./scripts/makefile-scripts/init-docker.sh

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
	@chmod +x scripts/makefile-scripts/logs-all.sh
	@./scripts/makefile-scripts/logs-all.sh

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
	@chmod +x scripts/makefile-scripts/test.sh
	@./scripts/makefile-scripts/test.sh

test-unit: ## Run unit tests only
	@chmod +x scripts/makefile-scripts/test-unit.sh
	@./scripts/makefile-scripts/test-unit.sh

test-integration: ## Run integration tests only
	@chmod +x scripts/makefile-scripts/test-integration.sh
	@./scripts/makefile-scripts/test-integration.sh

test-coverage: ## Generate test coverage report
	@chmod +x scripts/makefile-scripts/test-coverage.sh
	@./scripts/makefile-scripts/test-coverage.sh

test-watch: ## Run tests in watch mode
	@chmod +x scripts/makefile-scripts/test-watch.sh
	@./scripts/makefile-scripts/test-watch.sh

test-bench: ## Run performance benchmarks
	@chmod +x scripts/makefile-scripts/test-bench.sh
	@./scripts/makefile-scripts/test-bench.sh

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
	@chmod +x scripts/makefile-scripts/docs.sh
	@./scripts/makefile-scripts/docs.sh

docs-api: ## Generate API documentation
	@chmod +x scripts/makefile-scripts/docs-api.sh
	@./scripts/makefile-scripts/docs-api.sh

# Quick Commands
full-reset: docker-down clean data-clean test-clean init docker-up ## Complete reset (clean everything and reinitialize)

quick-start: init docker-up dev ## Quick start for new users (complete setup and run)

quick-test: test-unit test-integration ## Quick test suite (unit + integration)

dev-setup: init docker-up db-migrate db-seed ## Complete development setup

.PHONY: test test-unit test-integration test-coverage test-watch test-bench test-clean \
        db-migrate db-reset db-seed perf-test monitor maintenance-mode maintenance-off \
        docs docs-api full-reset quick-start quick-test dev-setup
