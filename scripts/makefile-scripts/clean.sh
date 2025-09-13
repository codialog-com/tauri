#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Cleaning project...${NC}"

rm -rf node_modules
rm -rf src-tauri/target
rm -rf coverage
rm -rf dist
rm -rf uploads/*
rm -rf logs/*

echo -e "${GREEN}Project cleaned!${NC}"
