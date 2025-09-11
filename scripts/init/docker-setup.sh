#!/bin/bash

# Docker Environment Setup Script for Codialog
# Initializes Docker services and volumes for Bitwarden integration

set -e

echo "🐳 Konfiguracja środowiska Docker dla Codialog..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Load environment variables
load_env() {
    print_header "Ładowanie konfiguracji środowiska..."
    
    if [ -f ".env" ]; then
        export $(cat .env | grep -v '^#' | xargs)
        print_status "Zmienne środowiskowe załadowane z .env"
    else
        print_error "Plik .env nie został znaleziony. Uruchom najpierw setup.sh"
        exit 1
    fi
}

# Create Docker volumes and networks
setup_docker_infrastructure() {
    print_header "Tworzenie infrastruktury Docker..."
    
    # Create custom network
    docker network create codialog-network 2>/dev/null || print_status "Sieć codialog-network już istnieje"
    
    # Create named volumes
    docker volume create codialog_postgres_data 2>/dev/null || print_status "Volume codialog_postgres_data już istnieje"
    docker volume create codialog_redis_data 2>/dev/null || print_status "Volume codialog_redis_data już istnieje"
    docker volume create codialog_bitwarden_data 2>/dev/null || print_status "Volume codialog_bitwarden_data już istnieje"
    
    print_status "Infrastruktura Docker przygotowana"
}

# Initialize database
init_database() {
    print_header "Inicjalizacja bazy danych PostgreSQL..."
    
    # Start only PostgreSQL first
    docker-compose -f docker-compose.bitwarden.yml up -d postgres
    
    # Wait for PostgreSQL to be ready
    print_status "Oczekiwanie na gotowość PostgreSQL..."
    for i in {1..30}; do
        if docker exec codialog-postgres pg_isready -U ${POSTGRES_USER:-codialog} >/dev/null 2>&1; then
            break
        fi
        sleep 2
    done
    
    # Run database migrations
    if [ -f "src-tauri/migrations/001_initial.sql" ]; then
        print_status "Wykonywanie migracji bazy danych..."
        docker exec -i codialog-postgres psql -U ${POSTGRES_USER:-codialog} -d ${POSTGRES_DB:-codialog} < src-tauri/migrations/001_initial.sql
        print_status "Migracje bazy danych zakończone pomyślnie"
    else
        print_warning "Plik migracji nie został znaleziony, pomijam inicjalizację schematu"
    fi
}

# Start Bitwarden/Vaultwarden
start_bitwarden() {
    print_header "Uruchamianie Vaultwarden..."
    
    # Start Vaultwarden
    docker-compose -f docker-compose.bitwarden.yml up -d vaultwarden
    
    # Wait for Vaultwarden to be ready
    print_status "Oczekiwanie na gotowość Vaultwarden..."
    for i in {1..30}; do
        if curl -s http://localhost:${VAULTWARDEN_PORT:-8080}/alive >/dev/null 2>&1; then
            break
        fi
        sleep 2
    done
    
    print_status "Vaultwarden jest gotowy na porcie ${VAULTWARDEN_PORT:-8080}"
}

# Start Redis
start_redis() {
    print_header "Uruchamianie Redis..."
    
    docker-compose -f docker-compose.bitwarden.yml up -d redis
    
    # Test Redis connection
    print_status "Testowanie połączenia z Redis..."
    for i in {1..15}; do
        if docker exec codialog-redis redis-cli ping >/dev/null 2>&1; then
            break
        fi
        sleep 1
    done
    
    print_status "Redis jest gotowy"
}

# Start Bitwarden CLI service
start_bitwarden_cli() {
    print_header "Uruchamianie usługi Bitwarden CLI..."
    
    docker-compose -f docker-compose.bitwarden.yml up -d bitwarden-cli
    
    print_status "Usługa Bitwarden CLI uruchomiona"
}

# Health check all services
health_check() {
    print_header "Sprawdzanie stanu usług..."
    
    services=("postgres" "redis" "vaultwarden" "bitwarden-cli")
    
    for service in "${services[@]}"; do
        if docker-compose -f docker-compose.bitwarden.yml ps | grep -q "$service.*Up"; then
            print_status "✅ $service: Działa"
        else
            print_error "❌ $service: Nie działa"
        fi
    done
}

# Display connection information
show_info() {
    print_header "INFORMACJE O POŁĄCZENIU"
    
    echo ""
    print_status "🔗 Dostęp do usług:"
    echo "  • Vaultwarden Web UI:    http://localhost:${VAULTWARDEN_PORT:-8080}"
    echo "  • PostgreSQL:           localhost:${POSTGRES_PORT:-5432}"
    echo "  • Redis:                localhost:${REDIS_PORT:-6379}"
    echo "  • Codialog App:         http://localhost:1420 (po uruchomieniu)"
    echo ""
    
    print_status "🔑 Dane logowania do PostgreSQL:"
    echo "  • Użytkownik: ${POSTGRES_USER:-codialog}"
    echo "  • Baza danych: ${POSTGRES_DB:-codialog}"
    echo "  • Hasło: [zobacz plik .env]"
    echo ""
    
    print_status "📋 Następne kroki:"
    echo "  1. Otwórz http://localhost:${VAULTWARDEN_PORT:-8080} i utwórz konto Bitwarden"
    echo "  2. Uruchom aplikację Codialog: make dev"
    echo "  3. Zaloguj się do Bitwarden przez aplikację"
    echo ""
}

# Main function
main() {
    print_header "SETUP DOCKER DLA CODIALOG"
    
    # Navigate to project root
    cd "$(dirname "$0")/../.."
    
    load_env
    setup_docker_infrastructure
    init_database
    start_redis
    start_bitwarden
    start_bitwarden_cli
    
    sleep 5  # Allow services to fully start
    
    health_check
    show_info
    
    print_header "DOCKER SETUP ZAKOŃCZONY"
    print_status "Wszystkie usługi Docker są gotowe!"
}

# Cleanup function for graceful shutdown
cleanup() {
    print_header "ZATRZYMYWANIE USŁUG"
    docker-compose -f docker-compose.bitwarden.yml down
    print_status "Usługi zatrzymane"
}

# Handle Ctrl+C
trap cleanup SIGINT SIGTERM

# Run main function
main "$@"
