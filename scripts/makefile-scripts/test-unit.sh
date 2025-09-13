#!/bin/bash

echo "$(tput setaf 3)🧪 Running unit tests...$(tput sgr0)"
cd src-tauri && cargo test --lib --verbose
