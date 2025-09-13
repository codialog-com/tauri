#!/bin/bash

echo "$(tput setaf 3)📋 API Documentation available at:$(tput sgr0)"
echo "  - Health Check: http://localhost:3000/health"
echo "  - DSL Generation: POST http://localhost:3000/dsl/generate"
echo "  - Bitwarden Login: POST http://localhost:3000/bitwarden/login"
echo "  - Session Management: GET/POST http://localhost:3000/session"
