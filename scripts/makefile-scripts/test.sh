#!/bin/bash

echo "$(tput setaf 3)🧪 Running all tests...$(tput sgr0)"
cd src-tauri && cargo test --verbose
