#!/bin/bash

echo "$(tput setaf 3)Watching Rust backend files...$(tput sgr0)"
cargo watch -x "check --manifest-path src-tauri/Cargo.toml"
