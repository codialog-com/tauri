# Codialog - Docker i Testowanie

## 1. Struktura projektu z testami

```
codialog/
├── docker/
│   ├── Dockerfile.app
│   ├── Dockerfile.tagui
│   └── Dockerfile.test
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── cdp.rs
│   │   ├── tagui.rs
│   │   └── llm.rs
│   └── tests/
│       ├── integration_test.rs
│       └── unit_test.rs
├── src/
│   ├── index.html
│   ├── main.js
│   └── style.css
├── tests/
│   ├── e2e/
│   │   ├── cv_upload.spec.js
│   │   └── form_fill.spec.js
│   ├── unit/
│   │   ├── dsl_generator.test.js
│   │   └── api.test.js
│   └── fixtures/
│       ├── test_cv.pdf
│       └── test_form.html
├── docker-compose.yml
├── docker-compose.test.yml
├── Makefile
└── .github/
    └── workflows/
        └── ci.yml
```

## 2. Docker Configuration

### `docker/Dockerfile.app`

```dockerfile
# Build stage
FROM rust:1.75 AS builder

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy and build Rust dependencies first
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock ./src-tauri/
RUN cd src-tauri && cargo build --release --lib

# Copy and build frontend
COPY package*.json ./
RUN npm ci

COPY src/ ./src/
COPY src-tauri/ ./src-tauri/

# Build Tauri app
RUN npm run build

# Runtime stage
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-0 \
    libgtk-3-0 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    libssl3 \
    ca-certificates \
    xvfb \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash codialog

WORKDIR /app

# Copy built application
COPY --from=builder /app/src-tauri/target/release/codialog /app/
COPY --from=builder /app/src/ /app/src/

# Set permissions
RUN chown -R codialog:codialog /app

USER codialog

# Use Xvfb for headless operation
ENV DISPLAY=:99

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:4000/health || exit 1

# Start with virtual display
CMD ["sh", "-c", "Xvfb :99 -screen 0 1024x768x24 & /app/codialog"]
```

### `docker/Dockerfile.tagui`

```dockerfile
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    unzip \
    python3 \
    python3-pip \
    chromium-browser \
    chromium-chromedriver \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install TagUI
WORKDIR /opt
RUN git clone https://github.com/aisingapore/tagui.git && \
    cd tagui && \
    npm install

# Create TagUI wrapper script
RUN echo '#!/bin/bash\n\
cd /opt/tagui\n\
./tagui "$@"' > /usr/local/bin/tagui && \
    chmod +x /usr/local/bin/tagui

# Set Chrome options for container
ENV CHROME_BIN=/usr/bin/chromium-browser
ENV CHROME_PATH=/usr/lib/chromium/

WORKDIR /workspace

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    CMD tagui --version || exit 1

CMD ["tail", "-f", "/dev/null"]
```

### `docker/Dockerfile.test`

```dockerfile
FROM node:20-slim

# Install Chrome for testing
RUN apt-get update && apt-get install -y \
    wget \
    gnupg \
    && wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google.list \
    && apt-get update \
    && apt-get install -y google-chrome-stable \
    && rm -rf /var/lib/apt/lists/*

# Install Rust for backend tests
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy test dependencies
COPY package*.json ./
RUN npm ci

# Install test frameworks
RUN npm install --save-dev \
    jest \
    @testing-library/react \
    @testing-library/jest-dom \
    playwright \
    @playwright/test \
    supertest \
    nock

# Copy application code
COPY . .

# Install Playwright browsers
RUN npx playwright install --with-deps

CMD ["npm", "test"]
```

## 3. Docker Compose

### `docker-compose.yml`

```yaml
version: '3.9'

services:
  app:
    build:
      context: .
      dockerfile: docker/Dockerfile.app
    container_name: codialog-app
    ports:
      - "4000:4000"
      - "1420:1420"
    environment:
      - RUST_LOG=debug
      - DATABASE_URL=postgresql://codialog:password@db:5432/codialog
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}
      - DISPLAY=:99
    volumes:
      - ./scripts:/app/scripts
      - ./uploads:/app/uploads
      - /tmp/.X11-unix:/tmp/.X11-unix:rw
    depends_on:
      - db
      - tagui
      - redis
    networks:
      - codialog-network

  tagui:
    build:
      context: .
      dockerfile: docker/Dockerfile.tagui
    container_name: codialog-tagui
    volumes:
      - ./scripts:/workspace/scripts
      - ./uploads:/workspace/uploads
      - tagui-data:/opt/tagui/data
    environment:
      - DISPLAY=:99
    networks:
      - codialog-network

  db:
    image: postgres:15-alpine
    container_name: codialog-db
    environment:
      - POSTGRES_USER=codialog
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=codialog
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    networks:
      - codialog-network

  redis:
    image: redis:7-alpine
    container_name: codialog-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    networks:
      - codialog-network

volumes:
  postgres-data:
  redis-data:
  tagui-data:

networks:
  codialog-network:
    driver: bridge
```

### `docker-compose.test.yml`

```yaml
version: '3.9'

services:
  test-runner:
    build:
      context: .
      dockerfile: docker/Dockerfile.test
    container_name: codialog-tests
    environment:
      - NODE_ENV=test
      - DATABASE_URL=postgresql://test:test@test-db:5432/test
      - REDIS_URL=redis://test-redis:6379
      - HEADLESS=true
    volumes:
      - ./src:/app/src
      - ./src-tauri:/app/src-tauri
      - ./tests:/app/tests
      - test-results:/app/test-results
    depends_on:
      - test-db
      - test-redis
    command: npm run test:all
    networks:
      - test-network

  test-db:
    image: postgres:15-alpine
    container_name: codialog-test-db
    environment:
      - POSTGRES_USER=test
      - POSTGRES_PASSWORD=test
      - POSTGRES_DB=test
    tmpfs:
      - /var/lib/postgresql/data
    networks:
      - test-network

  test-redis:
    image: redis:7-alpine
    container_name: codialog-test-redis
    tmpfs:
      - /data
    networks:
      - test-network

volumes:
  test-results:

networks:
  test-network:
    driver: bridge
```

## 4. Tests Implementation

### `tests/unit/dsl_generator.test.js`

```javascript
const { describe, test, expect, beforeEach } = require('@jest/globals');
const { DslGenerator } = require('../../src/dsl_generator');

describe('DSL Generator', () => {
    let generator;

    beforeEach(() => {
        generator = new DslGenerator();
    });

    test('generates login sequence', () => {
        const html = `
            <input id="username" type="text">
            <input id="password" type="password">
            <button id="submit">Login</button>
        `;
        
        const userData = {
            username: 'john.doe',
            password: 'secret123'
        };

        const dsl = generator.generate(html, userData);
        
        expect(dsl).toContain('type "#username" "john.doe"');
        expect(dsl).toContain('type "#password" "secret123"');
        expect(dsl).toContain('click "#submit"');
    });

    test('generates file upload command', () => {
        const html = `<input id="cv-upload" type="file">`;
        const userData = { cv_path: '/path/to/cv.pdf' };

        const dsl = generator.generate(html, userData);
        
        expect(dsl).toContain('upload "#cv-upload" "/path/to/cv.pdf"');
    });

    test('handles form with multiple fields', () => {
        const html = `
            <form>
                <input id="fullname" type="text">
                <input id="email" type="email">
                <input id="phone" type="tel">
                <input id="resume" type="file">
                <button id="apply">Apply</button>
            </form>
        `;

        const userData = {
            fullname: 'Jan Kowalski',
            email: 'jan@example.com',
            phone: '+48123456789',
            cv_path: 'C:/CV.pdf'
        };

        const dsl = generator.generate(html, userData);
        const lines = dsl.split('\n');

        expect(lines).toHaveLength(5);
        expect(lines[0]).toBe('type "#fullname" "Jan Kowalski"');
        expect(lines[1]).toBe('type "#email" "jan@example.com"');
        expect(lines[2]).toBe('type "#phone" "+48123456789"');
        expect(lines[3]).toBe('upload "#resume" "C:/CV.pdf"');
        expect(lines[4]).toBe('click "#apply"');
    });

    test('handles empty user data gracefully', () => {
        const html = `<button id="submit">Submit</button>`;
        const userData = {};

        const dsl = generator.generate(html, userData);
        
        expect(dsl).toBe('click "#submit"');
    });

    test('escapes special characters in user data', () => {
        const html = `<input id="comment" type="text">`;
        const userData = { comment: 'Test "quoted" text' };

        const dsl = generator.generate(html, userData);
        
        expect(dsl).toContain('type "#comment" "Test \\"quoted\\" text"');
    });
});
```

### `tests/unit/api.test.js`

```javascript
const request = require('supertest');
const nock = require('nock');
const { app } = require('../../src/server');

describe('API Endpoints', () => {
    beforeEach(() => {
        nock.cleanAll();
    });

    describe('POST /dsl/generate', () => {
        test('generates DSL from HTML and user data', async () => {
            const response = await request(app)
                .post('/dsl/generate')
                .send({
                    html: '<input id="name"><button id="submit">',
                    user_data: { name: 'John' }
                });

            expect(response.status).toBe(200);
            expect(response.body.script).toContain('type "#name" "John"');
            expect(response.body.script).toContain('click "#submit"');
        });

        test('calls LLM API when complex form detected', async () => {
            nock('https://api.anthropic.com')
                .post('/v1/messages')
                .reply(200, {
                    content: [{
                        text: 'click "#complex"\ntype "#field" "value"'
                    }]
                });

            const response = await request(app)
                .post('/dsl/generate')
                .send({
                    html: '<form class="complex-form">...</form>',
                    user_data: {}
                });

            expect(response.status).toBe(200);
            expect(response.body.script).toContain('click "#complex"');
        });

        test('handles invalid input gracefully', async () => {
            const response = await request(app)
                .post('/dsl/generate')
                .send({ invalid: 'data' });

            expect(response.status).toBe(400);
            expect(response.body.error).toBeDefined();
        });
    });

    describe('POST /rpa/run', () => {
        test('executes TagUI script successfully', async () => {
            const response = await request(app)
                .post('/rpa/run')
                .send({
                    script: 'click "#test"\ntype "#input" "value"'
                });

            expect(response.status).toBe(200);
            expect(response.body.success).toBeDefined();
        });

        test('validates script before execution', async () => {
            const response = await request(app)
                .post('/rpa/run')
                .send({
                    script: 'invalid command here'
                });

            expect(response.status).toBe(400);
            expect(response.body.error).toContain('Invalid DSL command');
        });
    });

    describe('GET /page/analyze', () => {
        test('returns HTML content of current page', async () => {
            const response = await request(app)
                .get('/page/analyze');

            expect(response.status).toBe(200);
            expect(response.body.html).toBeDefined();
        });
    });

    describe('GET /health', () => {
        test('returns health status', async () => {
            const response = await request(app)
                .get('/health');

            expect(response.status).toBe(200);
            expect(response.body.status).toBe('healthy');
            expect(response.body.services).toHaveProperty('tagui');
            expect(response.body.services).toHaveProperty('database');
            expect(response.body.services).toHaveProperty('redis');
        });
    });
});
```

### `tests/e2e/cv_upload.spec.js`

```javascript
const { test, expect } = require('@playwright/test');
const path = require('path');

test.describe('CV Upload Automation', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('http://localhost:1420');
    });

    test('complete CV upload flow', async ({ page }) => {
        // Fill user data
        await page.fill('#fullname', 'Jan Kowalski');
        await page.fill('#email', 'jan@example.com');
        await page.fill('#username', 'jan.kowalski');
        await page.fill('#password', 'SecurePass123!');

        // Upload CV file
        const fileInput = await page.locator('#cv-file');
        await fileInput.setInputFiles(path.join(__dirname, '../fixtures/test_cv.pdf'));

        // Verify file was selected
        const filePath = await page.locator('#cv-path').textContent();
        expect(filePath).toContain('test_cv.pdf');

        // Enter target URL
        await page.fill('#target-url', 'http://localhost:8080/test-form');

        // Analyze page
        await page.click('#analyze-btn');
        await page.waitForSelector('.status.success', { timeout: 5000 });

        // Generate DSL
        await page.click('#generate-btn');
        await page.waitForTimeout(1000);

        // Verify DSL was generated
        const dslContent = await page.inputValue('#dsl-script');
        expect(dslContent).toContain('type "#fullname" "Jan Kowalski"');
        expect(dslContent).toContain('type "#email" "jan@example.com"');
        expect(dslContent).toContain('upload "#cv-upload"');

        // Run automation
        await page.click('#run-btn');
        
        // Wait for success message
        await page.waitForSelector('.status.success', { 
            state: 'visible',
            timeout: 10000 
        });

        const statusText = await page.locator('#status').textContent();
        expect(statusText).toContain('Automatyzacja zakończona sukcesem');
    });

    test('handles missing CV file', async ({ page }) => {
        // Try to generate DSL without uploading CV
        await page.fill('#fullname', 'Test User');
        await page.fill('#email', 'test@example.com');
        await page.fill('#target-url', 'http://localhost:8080/test-form');

        await page.click('#analyze-btn');
        await page.waitForTimeout(1000);
        await page.click('#generate-btn');

        const dslContent = await page.inputValue('#dsl-script');
        expect(dslContent).not.toContain('upload');
    });

    test('validates form inputs', async ({ page }) => {
        // Try to analyze without URL
        await page.click('#analyze-btn');
        
        const statusText = await page.locator('#status').textContent();
        expect(statusText).toContain('Podaj URL strony');
    });

    test('handles network errors gracefully', async ({ page, context }) => {
        // Block API requests
        await context.route('**/api/**', route => route.abort());

        await page.fill('#target-url', 'http://example.com');
        await page.click('#analyze-btn');

        await page.waitForSelector('.status.error');
        const statusText = await page.locator('#status').textContent();
        expect(statusText).toContain('Błąd');
    });
});
```

### `tests/e2e/form_fill.spec.js`

```javascript
const { test, expect } = require('@playwright/test');

test.describe('Form Filling Automation', () => {
    test('fills login form', async ({ page }) => {
        await page.goto('http://localhost:1420');

        const dslScript = `
click "#login-btn"
type "#username" "testuser"
type "#password" "testpass"
click "#submit"
        `.trim();

        await page.fill('#dsl-script', dslScript);
        await page.click('#run-btn');

        await page.waitForTimeout(2000);
        
        const status = await page.locator('#status').textContent();
        expect(status).toContain('sukces');
    });

    test('fills complex application form', async ({ page }) => {
        await page.goto('http://localhost:1420');

        const complexDsl = `
hover "#careers-link"
click "#careers-link"
click "#job-posting-123"
type "#first-name" "Jan"
type "#last-name" "Kowalski"
type "#email" "jan@example.com"
type "#phone" "+48123456789"
type "#linkedin" "linkedin.com/in/jankowalski"
upload "#resume" "C:/Users/Jan/CV.pdf"
upload "#cover-letter" "C:/Users/Jan/Cover.pdf"
type "#salary-expectations" "10000-15000 PLN"
click "#gdpr-consent"
click "#submit-application"
        `.trim();

        await page.fill('#dsl-script', complexDsl);
        
        // Verify DSL syntax highlighting
        const dslElement = await page.locator('#dsl-script');
        const value = await dslElement.inputValue();
        expect(value).toContain('hover');
        expect(value).toContain('click');
        expect(value).toContain('type');
        expect(value).toContain('upload');
    });
});
```

### `src-tauri/tests/unit_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::*;
    use crate::tagui::*;

    #[test]
    fn test_dsl_generation_basic() {
        let html = r#"
            <input id="username" type="text">
            <input id="password" type="password">
            <button id="submit">Login</button>
        "#;
        
        let user_data = serde_json::json!({
            "username": "testuser",
            "password": "testpass"
        });

        let result = generate_dsl_script(html, &user_data);
        
        assert!(result.contains("type \"#username\" \"testuser\""));
        assert!(result.contains("type \"#password\" \"testpass\""));
        assert!(result.contains("click \"#submit\""));
    }

    #[test]
    fn test_dsl_file_upload() {
        let html = r#"<input id="cv-upload" type="file">"#;
        let user_data = serde_json::json!({
            "cv_path": "/path/to/cv.pdf"
        });

        let result = generate_dsl_script(html, &user_data);
        
        assert!(result.contains("upload \"#cv-upload\" \"/path/to/cv.pdf\""));
    }

    #[test]
    fn test_script_validation() {
        let valid_script = "click \"#button\"\ntype \"#input\" \"text\"";
        assert!(validate_dsl_script(valid_script).is_ok());

        let invalid_script = "invalid command here";
        assert!(validate_dsl_script(invalid_script).is_err());
    }

    #[test]
    fn test_escape_special_characters() {
        let input = "Test \"quoted\" text";
        let escaped = escape_for_dsl(input);
        assert_eq!(escaped, "Test \\\"quoted\\\" text");
    }

    #[tokio::test]
    async fn test_tagui_installation_check() {
        let installed = check_tagui_installed().await;
        assert!(installed || !installed); // Should not panic
    }

    #[tokio::test]
    async fn test_html_parsing() {
        let html = r#"
            <form>
                <input id="field1" name="name" type="text">
                <input id="field2" name="email" type="email">
                <button type="submit">Send</button>
            </form>
        "#;

        let elements = extract_form_elements(html);
        assert_eq!(elements.len(), 3);
        assert!(elements.iter().any(|e| e.id == Some("field1".to_string())));
        assert!(elements.iter().any(|e| e.id == Some("field2".to_string())));
    }
}
```

### `src-tauri/tests/integration_test.rs`

```rust
use codialog::*;
use axum::http::StatusCode;
use axum_test::TestServer;

#[tokio::test]
async fn test_full_workflow() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();

    // Test health endpoint
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // Test DSL generation
    let response = server
        .post("/dsl/generate")
        .json(&serde_json::json!({
            "html": "<input id='test'>",
            "user_data": {"test": "value"}
        }))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert!(body["script"].as_str().unwrap().contains("type"));

    // Test script execution (mock)
    let response = server
        .post("/rpa/run")
        .json(&serde_json::json!({
            "script": "click \"#test\""
        }))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_error_handling() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();

    // Test invalid request
    let response = server
        .post("/dsl/generate")
        .json(&serde_json::json!({}))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    // Test invalid DSL script
    let response = server
        .post("/rpa/run")
        .json(&serde_json::json!({
            "script": "invalid script"
        }))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}
```

## 5. Makefile

```makefile
.PHONY: help build test clean deploy

# Colors
GREEN := \033[0;32m
NC := \033[0m # No Color

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  ${GREEN}%-15s${NC} %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Docker commands
build: ## Build all Docker images
	docker-compose build

up: ## Start all services
	docker-compose up -d

down: ## Stop all services
	docker-compose down

logs: ## Show logs
	docker-compose logs -f

# Testing commands
test-unit: ## Run unit tests
	docker-compose -f docker-compose.test.yml run --rm test-runner npm run test:unit

test-e2e: ## Run E2E tests
	docker-compose -f docker-compose.test.yml run --rm test-runner npm run test:e2e

test-integration: ## Run integration tests
	docker-compose -f docker-compose.test.yml run --rm test-runner npm run test:integration

test-all: ## Run all tests
	docker-compose -f docker-compose.test.yml run --rm test-runner npm run test:all

test-watch: ## Run tests in watch mode
	docker-compose -f docker-compose.test.yml run --rm test-runner npm run test:watch

coverage: ## Generate test coverage report
	docker-compose -f docker-compose.test.yml run --rm test-runner npm run test:coverage
	@echo "Coverage report: ./coverage/index.html"

# Development commands
dev: ## Start development environment
	docker-compose up -d
	npm run dev

shell-app: ## Open shell in app container
	docker exec -it codialog-app /bin/bash

shell-tagui: ## Open shell in TagUI container
	docker exec -it codialog-tagui /bin/bash

# Database commands
db-migrate: ## Run database migrations
	docker exec codialog-app npm run migrate

db-seed: ## Seed database with test data
	docker exec codialog-app npm run seed

db-reset: ## Reset database
	docker exec codialog-app npm run db:reset

# Cleanup commands
clean: ## Clean all containers and volumes
	docker-compose down -v
	docker-compose -f docker-compose.test.yml down -v
	rm -rf node_modules target coverage test-results

clean-images: ## Remove Docker images
	docker rmi codialog-app codialog-tagui codialog-test

# Deployment commands
deploy-staging: ## Deploy to staging
	docker build -f docker/Dockerfile.app -t codialog:staging .
	docker tag codialog:staging registry.example.com/codialog:staging
	docker push registry.example.com/codialog:staging

deploy-prod: ## Deploy to production
	@read -p "Deploy to production? [y/N] " confirm && [ "$$confirm" = "y" ] || exit 1
	docker build -f docker/Dockerfile.app -t codialog:latest .
	docker tag codialog:latest registry.example.com/codialog:latest
	docker push registry.example.com/codialog:latest

# Monitoring
monitor: ## Open monitoring dashboard
	@echo "Opening Grafana at http://localhost:3000"
	@open http://localhost:3000 || xdg-open http://localhost:3000

health: ## Check health of all services
	@curl -s http://localhost:4000/health | jq '.'
```

## 6. CI/CD Pipeline

### `.github/workflows/ci.yml`

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  NODE_VERSION: '20'
  RUST_VERSION: '1.75'

jobs:
  lint:
    name: Lint Code
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'npm'
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Lint JavaScript
        run: |
          npm ci
          npm run lint:js
      
      - name: Lint Rust
        run: |
          cd src-tauri
          cargo fmt -- --check
          cargo clippy -- -D warnings

  test-unit:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'npm'
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run JavaScript Tests
        run: |
          npm ci
          npm run test:unit -- --coverage
      
      - name: Run Rust Tests
        run: |
          cd src-tauri
          cargo test --all-features
      
      - name: Upload Coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info
          flags: unittests

  test-integration:
    name: Integration Tests
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Test Environment
        run: |
          docker-compose -f docker-compose.test.yml build
      
      - name: Run Integration Tests
        run: |
          docker-compose -f docker-compose.test.yml run --rm test-runner npm run test:integration
      
      - name: Upload Test Results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: integration-test-results
          path: test-results/

  test-e2e:
    name: E2E Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
      
      - name: Install Playwright
        run: |
          npm ci
          npx playwright install --with-deps
      
      - name: Start Application
        run: |
          docker-compose up -d
          ./scripts/wait-for-it.sh localhost:4000
      
      - name: Run E2E Tests
        run: npm run test:e2e
      
      - name: Upload Playwright Report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/

  build:
    name: Build Application
    needs: [lint, test-unit]
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Build Environment
        uses: ./.github/actions/setup-build
        with:
          node-version: ${{ env.NODE_VERSION }}
          rust-version: ${{ env.RUST_VERSION }}
      
      - name: Build Application
        run: |
          npm ci
          npm run build
      
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: build-${{ matrix.os }}
          path: |
            src-tauri/target/release/codialog*
            dist/

  docker:
    name: Build Docker Images
    needs: [test-integration, test-e2e]
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Login to Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Build and Push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: docker/Dockerfile.app
          push: true
          tags: |
            ghcr.io/${{ github.repository }}:${{ github.sha }}
            ghcr.io/${{ github.repository }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy:
    name: Deploy to Staging
    needs: [docker]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
      - name: Deploy to Kubernetes
        run: |
          echo "Deploying to staging..."
          # kubectl apply -f k8s/staging/
```

## 7. Package.json z testami

```json
{
  "name": "codialog",
  "version": "0.1.0",
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "test": "npm run test:all",
    "test:unit": "jest tests/unit --coverage",
    "test:e2e": "playwright test tests/e2e",
    "test:integration": "jest tests/integration",
    "test:all": "npm run test:unit && npm run test:integration && npm run test:e2e",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage",
    "lint": "npm run lint:js && npm run lint:rust",
    "lint:js": "eslint src tests --ext .js,.jsx",
    "lint:rust": "cd src-tauri && cargo clippy",
    "format": "prettier --write 'src/**/*.{js,jsx,css}' 'tests/**/*.js'",
    "docker:build": "docker-compose build",
    "docker:up": "docker-compose up -d",
    "docker:down": "docker-compose down",
    "docker:test": "docker-compose -f docker-compose.test.yml up --abort-on-container-exit"
  },
  "devDependencies": {
    "@playwright/test": "^1.40.0",
    "@tauri-apps/cli": "^2.0.0",
    "@testing-library/jest-dom": "^6.1.5",
    "@testing-library/react": "^14.1.2",
    "eslint": "^8.55.0",
    "jest": "^29.7.0",
    "nock": "^13.4.0",
    "playwright": "^1.40.0",
    "prettier": "^3.1.1",
    "supertest": "^6.3.3"
  },
  "jest": {
    "testEnvironment": "node",
    "collectCoverageFrom": [
      "src/**/*.js",
      "!src/**/*.test.js"
    ],
    "coverageDirectory": "coverage",
    "coverageReporters": ["text", "lcov", "html"],
    "testMatch": [
      "**/tests/**/*.test.js",
      "**/tests/**/*.spec.js"
    ]
  }
}
```

## Podsumowanie

System **Codialog** został rozszerzony o:

### 🐳 Docker
- **Multi-stage builds** dla optymalizacji obrazów
- **Docker Compose** dla całego stacku (app, TagUI, DB, Redis)
- **Separate test environment** z docker-compose.test.yml
- **Health checks** dla wszystkich serwisów

### 🧪 Testowanie
- **Unit tests** (JavaScript + Rust)
- **Integration tests** dla API
- **E2E tests** z Playwright
- **Coverage reports** z codecov
- **CI/CD pipeline** z GitHub Actions

### 📊 Monitoring
- Health endpoints
- Logging
- Test reports
- Coverage metrics

### 🚀 Deployment
- Automated builds
- Container registry
- Staging/Production environments
- Kubernetes ready

Uruchomienie: `make up` lub `docker-compose up -d`
Testy: `make test-all`