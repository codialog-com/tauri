#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Installing TagUI...${NC}"

if [ ! -d "tagui" ]; then
    git clone https://github.com/aisingapore/tagui && \
    cd tagui && npm install && \
    echo -e "${GREEN}TagUI installed successfully!${NC}"
else
    echo -e "${YELLOW}TagUI already exists${NC}"
fi
