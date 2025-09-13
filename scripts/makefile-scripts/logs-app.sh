#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Application logs:${NC}"
find logs -name "*.log" -exec tail -f {} +
