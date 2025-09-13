#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Project Status:${NC}"
echo "Backend files: $(find src-tauri/src -name "*.rs" | wc -l) Rust files"
echo "Frontend files: $(find src -name "*.html" -o -name "*.js" -o -name "*.css" | wc -l) files"
echo "DSL scripts: $(find scripts -name "*.codialog" 2>/dev/null | wc -l) scripts"
echo "Dependencies: $(if [ -d 'node_modules' ]; then echo 'Installed'; else echo 'Not installed'; fi)"
echo "TagUI: $(if [ -d 'tagui' ]; then echo 'Ready'; else echo 'Not installed'; fi)"
