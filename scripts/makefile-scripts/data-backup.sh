#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Creating data backup...${NC}"

mkdir -p data/backups

BACKUP_FILE="data/backups/data_backup_$(date +%Y%m%d_%H%M%S).tar.gz"

tar -czf "$BACKUP_FILE" data/ src-tauri/data/

echo -e "${GREEN}Data backup created in $BACKUP_FILE${NC}"
