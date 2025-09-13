#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Available DSL scripts:${NC}"
find scripts -name "*.codialog" -exec basename {} .codialog \; | sort | sed 's/^/  - /'
