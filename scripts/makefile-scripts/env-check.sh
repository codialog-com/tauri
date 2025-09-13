#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Checking environment...${NC}"

if [ ! -f ".env" ]; then
    echo -e "${RED}No .env file found. Run 'make init' first${NC}"
    exit 1
fi

echo -e "${GREEN}Environment configuration found${NC}"
