#!/bin/bash

npm run test:coverage
echo "$(tput setaf 2)Coverage report generated in coverage/index.html$(tput sgr0)"
