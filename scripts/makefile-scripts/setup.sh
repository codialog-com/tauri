#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Setting up environment...${NC}"

if [ ! -f ".env" ]; then
    cp .env.example .env && \
    echo -e "${GREEN}Created .env file from template${NC}"
fi

mkdir -p uploads logs

echo -e "${GREEN}Setup completed! Run 'make dev' to start development${NC}"
