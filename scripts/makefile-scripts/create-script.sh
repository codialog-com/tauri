#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

if [ -z "$1" ]; then
    echo -e "${RED}Error: Please provide script name. Usage: ./create-script.sh my_script${NC}"
    exit 1
fi

NAME=$1

echo "// New DSL Script: $NAME" > scripts/$NAME.codialog
echo "// Created: $(date)" >> scripts/$NAME.codialog
echo "" >> scripts/$NAME.codialog
echo "// Add your DSL commands here" >> scripts/$NAME.codialog
echo "click \"#example\"" >> scripts/$NAME.codialog
echo "type \"#field\" \"value\"" >> scripts/$NAME.codialog

echo -e "${GREEN}Created new script: scripts/$NAME.codialog${NC}"
