# Monitoring i skrypty pomocnicze dla Codialog

## 1. Docker Compose z monitoringiem

### `docker-compose.monitoring.yml`

```yaml
version: '3.9'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: codialog-prometheus
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    networks:
      - codialog-network

  grafana:
    image: grafana/grafana:latest
    container_name: codialog-grafana
    volumes:
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_INSTALL_PLUGINS=redis-datasource
    ports:
      - "3000:3000"
    depends_on:
      - prometheus
    networks:
      - codialog-network

  loki:
    image: grafana/loki:latest
    container_name: codialog-loki
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki-config.yaml:/etc/loki/local-config.yaml
      - loki-data:/loki
    command: -config.file=/etc/loki/local-config.yaml
    networks:
      - codialog-network

  promtail:
    image: grafana/promtail:latest
    container_name: codialog-promtail
    volumes:
      - ./monitoring/promtail-config.yaml:/etc/promtail/config.yml
      - /var/log:/var/log
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
    command: -config.file=/etc/promtail/config.yml
    networks:
      - codialog-network

  jaeger:
    image: jaegertracing/all-in-one:latest
    container_name: codialog-jaeger
    environment:
      - COLLECTOR_ZIPKIN_HOST_PORT=:9411
    ports:
      - "5775:5775/udp"
      - "6831:6831/udp"
      - "6832:6832/udp"
      - "5778:5778"
      - "16686:16686"
      - "14268:14268"
      - "14250:14250"
      - "9411:9411"
    networks:
      - codialog-network

volumes:
  prometheus-data:
  grafana-data:
  loki-data:

networks:
  codialog-network:
    external: true
```

## 2. Database initialization

### `init.sql`

```sql
-- Create database schema for Codialog
CREATE SCHEMA IF NOT EXISTS codialog;

-- Users table
CREATE TABLE IF NOT EXISTS codialog.users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    fullname VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Automation scripts table
CREATE TABLE IF NOT EXISTS codialog.scripts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES codialog.users(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    dsl_content TEXT NOT NULL,
    form_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Execution history table
CREATE TABLE IF NOT EXISTS codialog.executions (
    id SERIAL PRIMARY KEY,
    script_id INTEGER REFERENCES codialog.scripts(id),
    user_id INTEGER REFERENCES codialog.users(id),
    status VARCHAR(50) NOT NULL, -- 'pending', 'running', 'success', 'failed'
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT,
    execution_log TEXT
);

-- User files table (for CV uploads)
CREATE TABLE IF NOT EXISTS codialog.user_files (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES codialog.users(id),
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_type VARCHAR(50),
    file_size INTEGER,
    uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Templates table
CREATE TABLE IF NOT EXISTS codialog.templates (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    category VARCHAR(100),
    dsl_template TEXT NOT NULL,
    description TEXT,
    variables JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_scripts_user_id ON codialog.scripts(user_id);
CREATE INDEX idx_executions_script_id ON codialog.executions(script_id);
CREATE INDEX idx_executions_status ON codialog.executions(status);
CREATE INDEX idx_user_files_user_id ON codialog.user_files(user_id);

-- Insert sample data
INSERT INTO codialog.templates (name, category, dsl_template, description, variables) VALUES
('LinkedIn Easy Apply', 'job_application', 
'click "#jobs-apply-button"
type "#email" "{email}"
type "#phone" "{phone}"
upload "#resume" "{cv_path}"
click "#submit-application"',
'Template for LinkedIn Easy Apply',
'{"email": "string", "phone": "string", "cv_path": "string"}'::jsonb),

('Generic Job Form', 'job_application',
'type "#first-name" "{first_name}"
type "#last-name" "{last_name}"
type "#email" "{email}"
type "#phone" "{phone}"
upload "#cv-upload" "{cv_path}"
click "#submit"',
'Generic job application form',
'{"first_name": "string", "last_name": "string", "email": "string", "phone": "string", "cv_path": "string"}'::jsonb);

-- Create update trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON codialog.users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_scripts_updated_at BEFORE UPDATE ON codialog.scripts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

## 3. Helper Scripts

### `scripts/wait-for-it.sh`

```bash
#!/usr/bin/env bash
# Wait for a service to be ready

set -e

host="$1"
port="$2"
shift 2
cmd="$@"

until nc -z "$host" "$port"; do
  >&2 echo "Service $host:$port is unavailable - sleeping"
  sleep 1
done

>&2 echo "Service $host:$port is up - executing command"
exec $cmd
```

### `scripts/backup.sh`

```bash
#!/bin/bash
# Backup script for Codialog

BACKUP_DIR="/backups/codialog"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="codialog_backup_${TIMESTAMP}"

echo "🔄 Starting backup: ${BACKUP_NAME}"

# Create backup directory
mkdir -p "${BACKUP_DIR}"

# Backup database
echo "📊 Backing up database..."
docker exec codialog-db pg_dump -U codialog -d codialog | gzip > "${BACKUP_DIR}/${BACKUP_NAME}_db.sql.gz"

# Backup uploaded files
echo "📁 Backing up files..."
docker cp codialog-app:/app/uploads "${BACKUP_DIR}/${BACKUP_NAME}_uploads"

# Backup scripts
echo "📜 Backing up scripts..."
docker cp codialog-app:/app/scripts "${BACKUP_DIR}/${BACKUP_NAME}_scripts"

# Create tar archive
echo "📦 Creating archive..."
cd "${BACKUP_DIR}"
tar -czf "${BACKUP_NAME}.tar.gz" \
    "${BACKUP_NAME}_db.sql.gz" \
    "${BACKUP_NAME}_uploads" \
    "${BACKUP_NAME}_scripts"

# Clean up individual files
rm -rf "${BACKUP_NAME}_db.sql.gz" "${BACKUP_NAME}_uploads" "${BACKUP_NAME}_scripts"

# Keep only last 7 backups
echo "🧹 Cleaning old backups..."
ls -t *.tar.gz | tail -n +8 | xargs -r rm

echo "✅ Backup completed: ${BACKUP_DIR}/${BACKUP_NAME}.tar.gz"
```

### `scripts/restore.sh`

```bash
#!/bin/bash
# Restore script for Codialog

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file.tar.gz>"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "❌ Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "🔄 Starting restore from: $BACKUP_FILE"

# Extract backup
TEMP_DIR="/tmp/codialog_restore_$$"
mkdir -p "$TEMP_DIR"
tar -xzf "$BACKUP_FILE" -C "$TEMP_DIR"

# Find extracted files
DB_BACKUP=$(find "$TEMP_DIR" -name "*_db.sql.gz" | head -1)
UPLOADS_DIR=$(find "$TEMP_DIR" -name "*_uploads" -type d | head -1)
SCRIPTS_DIR=$(find "$TEMP_DIR" -name "*_scripts" -type d | head -1)

# Restore database
echo "📊 Restoring database..."
gunzip -c "$DB_BACKUP" | docker exec -i codialog-db psql -U codialog -d codialog

# Restore uploads
echo "📁 Restoring uploads..."
docker cp "$UPLOADS_DIR" codialog-app:/app/uploads_restore
docker exec codialog-app sh -c "rm -rf /app/uploads && mv /app/uploads_restore /app/uploads"

# Restore scripts
echo "📜 Restoring scripts..."
docker cp "$SCRIPTS_DIR" codialog-app:/app/scripts_restore
docker exec codialog-app sh -c "rm -rf /app/scripts && mv /app/scripts_restore /app/scripts"

# Clean up
rm -rf "$TEMP_DIR"

echo "✅ Restore completed successfully!"
```

### `scripts/dev-setup.sh`

```bash
#!/bin/bash
# Development environment setup

echo "🚀 Setting up Codialog development environment..."

# Check prerequisites
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    fi
}

check_command docker
check_command docker-compose
check_command npm
check_command cargo

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p uploads scripts logs test-results coverage

# Install dependencies
echo "📦 Installing Node.js dependencies..."
npm ci

echo "📦 Installing Rust dependencies..."
cd src-tauri && cargo build --release && cd ..

# Set up git hooks
echo "🔗 Setting up git hooks..."
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
npm run lint
npm run test:unit
EOF
chmod +x .git/hooks/pre-commit

# Build Docker images
echo "🐳 Building Docker images..."
docker-compose build

# Initialize database
echo "📊 Initializing database..."
docker-compose up -d db
sleep 5
docker exec -i codialog-db psql -U codialog -d codialog < init.sql

# Install TagUI
if [ ! -d "tagui" ]; then
    echo "🤖 Installing TagUI..."
    git clone https://github.com/aisingapore/tagui
    cd tagui && npm install && cd ..
fi

# Create .env file
if [ ! -f ".env" ]; then
    echo "📝 Creating .env file..."
    cat > .env << EOF
NODE_ENV=development
DATABASE_URL=postgresql://codialog:password@localhost:5432/codialog
REDIS_URL=redis://localhost:6379
CLAUDE_API_KEY=your_api_key_here
PORT=4000
EOF
fi

echo "✅ Development environment setup complete!"
echo ""
echo "Next steps:"
echo "1. Update .env with your Claude API key"
echo "2. Run 'make dev' to start development server"
echo "3. Run 'make test-all' to run tests"
echo "4. Open http://localhost:1420 in your browser"
```

## 4. Monitoring Configuration

### `monitoring/prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'codialog-app'
    static_configs:
      - targets: ['codialog-app:4000']
    metrics_path: '/metrics'

  - job_name: 'postgres'
    static_configs:
      - targets: ['codialog-db:5432']

  - job_name: 'redis'
    static_configs:
      - targets: ['codialog-redis:6379']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
```

### `monitoring/loki-config.yaml`

```yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    address: 127.0.0.1
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
    final_sleep: 0s

schema_config:
  configs:
    - from: 2023-01-01
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/boltdb-shipper-active
    cache_location: /loki/boltdb-shipper-cache
    shared_store: filesystem
  filesystem:
    directory: /loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h
```

### `monitoring/grafana/dashboards/codialog.json`

```json
{
  "dashboard": {
    "id": null,
    "title": "Codialog Monitoring",
    "tags": ["codialog"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "type": "graph",
        "title": "API Request Rate",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{path}}"
          }
        ]
      },
      {
        "id": 2,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "type": "graph",
        "title": "DSL Generation Time",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(dsl_generation_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "id": 3,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
        "type": "graph",
        "title": "TagUI Execution Success Rate",
        "targets": [
          {
            "expr": "rate(tagui_executions_total{status=\"success\"}[5m]) / rate(tagui_executions_total[5m])",
            "legendFormat": "Success Rate"
          }
        ]
      },
      {
        "id": 4,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
        "type": "stat",
        "title": "Total CV Uploads",
        "targets": [
          {
            "expr": "sum(cv_uploads_total)",
            "legendFormat": "Total Uploads"
          }
        ]
      }
    ]
  }
}
```

## 5. Performance Testing

### `tests/performance/load_test.js`

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const dslGenRate = new Rate('dsl_generation_success');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 10 },  // Ramp up to 10 users
    { duration: '1m', target: 10 },   // Stay at 10 users
    { duration: '30s', target: 50 },  // Ramp up to 50 users
    { duration: '2m', target: 50 },   // Stay at 50 users
    { duration: '30s', target: 0 },   // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms
    errors: ['rate<0.1'],              // Error rate must be below 10%
    dsl_generation_success: ['rate>0.9'] // DSL generation success rate > 90%
  },
};

const BASE_URL = 'http://localhost:4000';

// Test data
const testHtml = `
  <form>
    <input id="username" type="text">
    <input id="password" type="password">
    <input id="email" type="email">
    <input id="cv" type="file">
    <button id="submit">Submit</button>
  </form>
`;

const userData = {
  username: 'testuser',
  password: 'testpass123',
  email: 'test@example.com',
  cv_path: '/path/to/cv.pdf'
};

export default function() {
  // Test 1: Health check
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'health check status is 200': (r) => r.status === 200,
  });
  errorRate.add(healthRes.status !== 200);

  sleep(1);

  // Test 2: DSL Generation
  const dslRes = http.post(
    `${BASE_URL}/dsl/generate`,
    JSON.stringify({
      html: testHtml,
      user_data: userData
    }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const dslSuccess = check(dslRes, {
    'DSL generation status is 200': (r) => r.status === 200,
    'DSL contains expected commands': (r) => {
      const body = JSON.parse(r.body);
      return body.script && 
             body.script.includes('type') && 
             body.script.includes('upload');
    }
  });

  dslGenRate.add(dslSuccess);
  errorRate.add(!dslSuccess);

  sleep(1);

  // Test 3: Script validation
  if (dslRes.status === 200) {
    const script = JSON.parse(dslRes.body).script;
    const runRes = http.post(
      `${BASE_URL}/rpa/validate`,
      JSON.stringify({ script }),
      { headers: { 'Content-Type': 'application/json' } }
    );

    check(runRes, {
      'Script validation status is 200': (r) => r.status === 200,
    });
    errorRate.add(runRes.status !== 200);
  }

  sleep(1);
}

// Teardown function
export function teardown(data) {
  console.log('Test completed!');
}
```

## 6. Quick Start Guide

### `README.md`

```markdown
# 🤖 Codialog - Automated Form Filling with AI

[![CI/CD](https://github.com/your-org/codialog/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/codialog/actions)
[![Coverage](https://codecov.io/gh/your-org/codialog/branch/main/graph/badge.svg)](https://codecov.io/gh/your-org/codialog)
[![Docker](https://img.shields.io/docker/v/your-org/codialog)](https://hub.docker.com/r/your-org/codialog)

## 🚀 Quick Start

### Using Docker (Recommended)

```bash
# Clone repository
git clone https://github.com/your-org/codialog.git
cd codialog

# Set up environment
cp .env.example .env
# Edit .env and add your Claude API key

# Start all services
docker-compose up -d

# Open application
open http://localhost:1420
```

### Manual Installation

```bash
# Install dependencies
npm install
cd src-tauri && cargo build --release

# Install TagUI
git clone https://github.com/aisingapore/tagui
cd tagui && npm install && cd ..

# Start development server
npm run dev
```

## 🧪 Testing

```bash
# Run all tests
make test-all

# Run specific test suites
make test-unit       # Unit tests
make test-e2e        # End-to-end tests
make test-integration # Integration tests

# Run with coverage
make coverage

# Performance testing
npm run test:performance
```

## 📊 Monitoring

```bash
# Start monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d

# Access dashboards
open http://localhost:3000  # Grafana (admin/admin)
open http://localhost:9090  # Prometheus
open http://localhost:16686 # Jaeger
```

## 🔧 Development

```bash
# Set up development environment
./scripts/dev-setup.sh

# Run in development mode
make dev

# Open shell in container
make shell-app   # Application container
make shell-tagui # TagUI container

# Database operations
make db-migrate  # Run migrations
make db-seed     # Seed test data
make db-reset    # Reset database
```

## 📦 Deployment

```bash
# Build for production
make build

# Deploy to staging
make deploy-staging

# Deploy to production (requires confirmation)
make deploy-prod
```

## 🔄 Backup & Restore

```bash
# Create backup
./scripts/backup.sh

# Restore from backup
./scripts/restore.sh backup_file.tar.gz
```

## 📝 DSL Commands

| Command | Description | Example |
|---------|-------------|---------|
| `click` | Click element | `click "#submit"` |
| `type` | Type text | `type "#email" "user@example.com"` |
| `upload` | Upload file | `upload "#cv" "/path/to/cv.pdf"` |
| `hover` | Hover over element | `hover "#menu"` |

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open Pull Request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🆘 Support

- Documentation: [docs.codialog.io](https://docs.codialog.io)
- Issues: [GitHub Issues](https://github.com/your-org/codialog/issues)
- Discord: [Join our community](https://discord.gg/codialog)
```
