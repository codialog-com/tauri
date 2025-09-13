#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Checking Bitwarden status...${NC}"

if docker ps | grep -q codialog-bitwarden-cli; then
    docker exec codialog-bitwarden-cli bw status
else
    echo -e "${RED}Bitwarden CLI container not running${NC}"
fi
