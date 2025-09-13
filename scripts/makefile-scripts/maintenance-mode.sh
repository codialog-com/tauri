#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚧 Enabling maintenance mode...${NC}"

echo "maintenance" > .maintenance
docker pause codialog-app 2>/dev/null || true

echo -e "${GREEN}Maintenance mode enabled${NC}"
