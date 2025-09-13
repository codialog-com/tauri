#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔧 Disabling maintenance mode...${NC}"

rm -f .maintenance
docker unpause codialog-app 2>/dev/null || true

echo -e "${GREEN}Maintenance mode disabled${NC}"
