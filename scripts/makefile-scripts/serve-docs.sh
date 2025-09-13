#!/bin/bash

if [ -d "docs" ]; then
    python3 -m http.server 8080 -d docs
else
    echo "$(tput setaf 1)No docs directory found$(tput sgr0)"
fi
