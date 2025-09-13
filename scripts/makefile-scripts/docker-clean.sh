#!/bin/bash

echo "$(tput setaf 3)Cleaning Docker resources...$(tput sgr0)"
docker-compose -f docker-compose.bitwarden.yml down -v
docker volume prune -f
docker network prune -f
echo "$(tput setaf 2)Docker resources cleaned$(tput sgr0)"
