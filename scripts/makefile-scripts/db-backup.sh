#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Creating database backup...${NC}"

mkdir -p data/backups

# Use default values if environment variables are not set
POSTGRES_USER=${POSTGRES_USER:-codialog}
POSTGRES_DB=${POSTGRES_DB:-codialog}

BACKUP_FILE="data/backups/backup_$(date +%Y%m%d_%H%M%S).sql"

docker exec codialog-postgres pg_dump -U "$POSTGRES_USER" "$POSTGRES_DB" > "$BACKUP_FILE"

echo -e "${GREEN}Database backup created in $BACKUP_FILE${NC}"
