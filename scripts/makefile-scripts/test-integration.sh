#!/bin/bash

echo "$(tput setaf 3)🧪 Running integration tests...$(tput sgr0)"
cd src-tauri && cargo test --features integration_tests --verbose
