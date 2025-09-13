#!/bin/bash

echo "$(tput setaf 3)Stopping Docker services...$(tput sgr0)"
docker-compose -f docker-compose.bitwarden.yml down
