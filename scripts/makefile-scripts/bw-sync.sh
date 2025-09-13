#!/bin/bash

echo "$(tput setaf 3)Syncing Bitwarden vault...$(tput sgr0)"
docker exec codialog-bitwarden-cli bw sync
