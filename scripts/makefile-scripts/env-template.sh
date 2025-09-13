#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

if [ ! -f ".env" ]; then
    cp .env.example .env
    echo -e "${GREEN}Created .env from template${NC}"
else
    echo -e "${YELLOW}.env already exists${NC}"
fi
