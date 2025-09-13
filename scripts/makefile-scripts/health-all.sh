#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Checking all service health...${NC}"

./scripts/makefile-scripts/health.sh

echo ""

docker-compose -f docker-compose.bitwarden.yml ps

echo ""

curl -s http://localhost:8080/alive >/dev/null && \
    echo -e "${GREEN}✓ Vaultwarden (port 8080)${NC}" || \
    echo -e "${RED}✗ Vaultwarden not accessible${NC}"
