#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}⚡ Running performance tests...${NC}"
echo "Testing API endpoints..."

for endpoint in "/health" "/api/status"; do
    echo "Testing $endpoint..."
    curl -w "Time: %{time_total}s\n" -s "http://localhost:3000$endpoint" -o /dev/null || true
done
