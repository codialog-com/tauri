#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Application logs:${NC}"

if [ -f "logs/codialog.log" ]; then
    tail -f logs/codialog.log
else
    echo -e "${RED}No log file found${NC}"
fi
