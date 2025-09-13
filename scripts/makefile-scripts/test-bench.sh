#!/bin/bash

echo "$(tput setaf 3)⚡ Running performance benchmarks...$(tput sgr0)"
cd src-tauri && cargo bench
