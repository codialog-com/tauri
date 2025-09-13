#!/bin/bash

echo "$(tput setaf 3)📚 Generating documentation...$(tput sgr0)"
cd src-tauri && cargo doc --no-deps --open 2>/dev/null || cargo doc --no-deps
echo "$(tput setaf 2)Documentation generated and opened in browser$(tput sgr0)"
