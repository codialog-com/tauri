#!/bin/bash

echo "$(tput setaf 3)Unlocking Bitwarden vault...$(tput sgr0)"
docker exec -it codialog-bitwarden-cli bw unlock
