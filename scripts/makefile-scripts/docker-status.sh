#!/bin/bash

echo "$(tput setaf 3)Docker Services Status:$(tput sgr0)"
docker-compose -f docker-compose.bitwarden.yml ps
