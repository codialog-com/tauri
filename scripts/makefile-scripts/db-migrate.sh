#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🗄️  Running database migrations...${NC}"

if docker exec codialog-postgres pg_isready -U codialog_user > /dev/null 2>&1; then
    echo -e "${GREEN}Database is ready, running migrations...${NC}"
    cd src-tauri && cargo run --bin migrate 2>/dev/null || echo -e "${YELLOW}Migration binary not found, skipping${NC}"
else
    echo -e "${RED}Database is not ready, please start it first with 'make docker-up'${NC}"
fi
