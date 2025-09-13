#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Checking service health...${NC}"

curl -s http://localhost:4000/health >/dev/null && \
    echo -e "${GREEN}✓ Backend API (port 4000)${NC}" || \
    echo -e "${RED}✗ Backend API not running${NC}"

curl -s http://localhost:1420 >/dev/null && \
    echo -e "${GREEN}✓ Frontend (port 1420)${NC}" || \
    echo -e "${RED}✗ Frontend not running${NC}"
