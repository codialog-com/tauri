#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Codialog Project Information${NC}"
echo "Version: $(grep '"version":' package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')"
echo "Node.js: $(node --version 2>/dev/null || echo 'Not installed')"
echo "npm: $(npm --version 2>/dev/null || echo 'Not installed')"
echo "Rust: $(rustc --version 2>/dev/null || echo 'Not installed')"
echo "Cargo: $(cargo --version 2>/dev/null || echo 'Not installed')"
echo "Tauri CLI: $(tauri --version 2>/dev/null || echo 'Not installed')"
echo "TagUI: $(if [ -d 'tagui' ]; then echo 'Installed'; else echo 'Not installed'; fi)"
