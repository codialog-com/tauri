#!/bin/bash

echo "$(tput setaf 3)Starting Docker services...$(tput sgr0)"

# Create necessary directories for volume mounts
mkdir -p /home/tom/github/codialog-com/tauri/data/postgres
mkdir -p /home/tom/github/codialog-com/tauri/data/redis
mkdir -p /home/tom/github/codialog-com/tauri/data/bitwarden

echo "$(tput setaf 3)Directories for Docker volumes created.$(tput sgr0)"

./scripts/init/docker-setup.sh
