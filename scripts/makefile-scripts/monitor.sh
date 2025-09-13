#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}📊 Starting monitoring dashboard...${NC}"
echo "Database status:"
docker exec codialog-postgres pg_isready -U codialog_user || echo "Database not ready"
echo "Redis status:"
docker exec codialog-redis redis-cli ping || echo "Redis not ready"
echo "Application logs (last 10 lines):"
tail -n 10 logs/app.log 2>/dev/null || echo "No app logs found"
