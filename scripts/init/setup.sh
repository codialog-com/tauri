#!/bin/bash

# Codialog Application Setup Script
# This script initializes the application environment and dependencies

set -e  # Exit on error

echo "🚀 Inicjalizacja aplikacji Codialog..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}==== $1 ====${NC}"
}

# Check if Docker is installed
check_docker() {
    print_header "Sprawdzanie Docker..."
    if ! command -v docker &> /dev/null; then
        print_error "Docker nie jest zainstalowany. Zainstaluj Docker i spróbuj ponownie."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose nie jest zainstalowany. Zainstaluj Docker Compose i spróbuj ponownie."
        exit 1
    fi
    
    print_status "Docker i Docker Compose są dostępne"
}

# Check if Rust is installed
check_rust() {
    print_header "Sprawdzanie Rust..."
    if ! command -v cargo &> /dev/null; then
        print_error "Rust nie jest zainstalowany. Zainstaluj Rust z https://rustup.rs/"
        exit 1
    fi
    
    print_status "Rust jest dostępny: $(rustc --version)"
}

# Check if Node.js is installed
check_node() {
    print_header "Sprawdzanie Node.js..."
    if ! command -v node &> /dev/null; then
        print_error "Node.js nie jest zainstalowany. Zainstaluj Node.js i spróbuj ponownie."
        exit 1
    fi
    
    if ! command -v npm &> /dev/null; then
        print_error "npm nie jest dostępny. Zainstaluj npm i spróbuj ponownie."
        exit 1
    fi
    
    print_status "Node.js jest dostępny: $(node --version)"
    print_status "npm jest dostępny: $(npm --version)"
}

# Create necessary directories
create_directories() {
    print_header "Tworzenie katalogów..."
    
    # Create main data directories
    mkdir -p data/{uploads,sessions,logs,backups,bitwarden,database,redis}
    mkdir -p src-tauri/data/{uploads,sessions,logs,backups,bitwarden,database,redis}
    mkdir -p logs/{app,debug,error}
    mkdir -p docker/volumes/{postgres,redis,bitwarden}
    
    print_status "Katalogi utworzone pomyślnie"
}

# Set proper permissions
set_permissions() {
    print_header "Ustawianie uprawnień..."
    
    # Set read/write permissions for data directories
    chmod -R 755 data/ src-tauri/data/ logs/ docker/volumes/ 2>/dev/null || true
    
    # Make scripts executable
    find scripts/ -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true
    
    print_status "Uprawnienia ustawione"
}

# Copy environment configuration
setup_env() {
    print_header "Konfiguracja środowiska..."
    
    if [ ! -f ".env" ]; then
        if [ -f ".env.example" ]; then
            cp .env.example .env
            print_status "Skopiowano .env.example do .env"
            print_warning "Edytuj plik .env aby dostosować konfigurację do swojego środowiska"
        else
            print_error "Plik .env.example nie został znaleziony"
            exit 1
        fi
    else
        print_status "Plik .env już istnieje"
    fi
}

# Install Node.js dependencies
install_node_deps() {
    print_header "Instalacja zależności Node.js..."
    
    if [ -f "package.json" ]; then
        npm install
        print_status "Zależności Node.js zainstalowane"
    else
        print_warning "Plik package.json nie został znaleziony, pomijam instalację npm"
    fi
}

# Install Rust dependencies and build
build_rust() {
    print_header "Budowanie aplikacji Rust..."
    
    cd src-tauri
    
    # Check Cargo.toml exists
    if [ ! -f "Cargo.toml" ]; then
        print_error "Plik Cargo.toml nie został znaleziony w src-tauri/"
        exit 1
    fi
    
    # Build in debug mode first
    cargo check
    print_status "Sprawdzenie składni Rust zakończone pomyślnie"
    
    cd ..
}

# Initialize database schema
init_database() {
    print_header "Inicjalizacja schematu bazy danych..."
    
    if [ -f "src-tauri/migrations/001_initial.sql" ]; then
        print_status "Skrypt migracji bazy danych znaleziony"
        print_warning "Uruchom 'make docker-up' aby zainicjalizować bazę danych"
    else
        print_error "Skrypt migracji nie został znaleziony"
        exit 1
    fi
}

# Main setup process
main() {
    print_header "SETUP APLIKACJI CODIALOG"
    
    # Navigate to project root
    cd "$(dirname "$0")/../.."
    
    check_docker
    check_rust
    check_node
    create_directories
    set_permissions
    setup_env
    install_node_deps
    build_rust
    init_database
    
    print_header "SETUP ZAKOŃCZONY"
    print_status "Aplikacja Codialog została pomyślnie zainicjalizowana!"
    echo ""
    print_status "Następne kroki:"
    echo "  1. Edytuj plik .env aby dostosować konfigurację"
    echo "  2. Uruchom 'make docker-up' aby uruchomić usługi Docker"
    echo "  3. Uruchom 'make dev' aby uruchomić aplikację w trybie deweloperskim"
    echo "  4. Otwórz http://localhost:1420 w przeglądarce"
    echo ""
}

# Run main function
main "$@"
