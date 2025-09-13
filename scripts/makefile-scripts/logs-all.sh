#!/bin/bash

echo "$(tput setaf 3)Starting log monitoring (Ctrl+C to stop)...$(tput sgr0)"
make logs-app & make logs-docker
