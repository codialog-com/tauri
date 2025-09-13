#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🌱 Seeding database with test data...${NC}"

docker exec codialog-postgres psql -U codialog_user -d codialog -c "\
    INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) VALUES \
    ('test-session-1', '{\"email\":\"test@example.com\",\"name\":\"Test User\"}', NOW(), NOW(), NOW() + INTERVAL '1 day', true), \
    ('test-session-2', '{\"email\":\"demo@example.com\",\"name\":\"Demo User\"}', NOW(), NOW(), NOW() + INTERVAL '1 day', true) \
    ON CONFLICT (session_id) DO NOTHING;"

echo -e "${GREEN}Database seeded with test data${NC}"
