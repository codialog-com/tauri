# 🤖 Codialog: Inteligentny Asystent Automatyzacji

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D8.svg)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/Build-Passing-green.svg)](#)
[![TagUI](https://img.shields.io/badge/TagUI-Integration-purple.svg)](#)

**Autor:** Tom Sapletta <info@softreck.dev>

`Codialog` to zaawansowany system, który automatyzuje proces wypełniania formularzy online i wysyłania aplikacji, np. podczas rekrutacji. Wykorzystuje duży model językowy (LLM) do analizy stron internetowych i generowania skryptów automatyzacji, które następnie wykonuje, oszczędzając Twój czas.

## 📋 Menu Nawigacyjne

- [🎯 Kluczowe Funkcje](#-kluczowe-funkcje)
- [🚀 Pierwsze Kroki](#-pierwsze-kroki)
- [📁 Struktura Projektu](#-struktura-projektu)
- [🛠️ Pliki Konfiguracyjne](#️-pliki-konfiguracyjne)
- [🔧 API Endpoints](#-api-endpoints)
- [📊 Monitoring i Logi](#-monitoring-i-logi)
- [🧪 Testowanie](#-testowanie)
- [🐳 Docker i Deploy](#-docker-i-deploy)
- [📜 Licencja](#-licencja)

## 🎯 Kluczowe Funkcje

- **🤖 Inteligentne Wypełnianie Formularzy**: Automatyczne uzupełnianie pól na podstawie dostarczonych danych (np. z CV).
- **📄 Automatyczny Upload CV**: Bezproblemowe przesyłanie plików PDF/DOC do formularzy aplikacyjnych.
- **🧠 Generowanie Skryptów przez LLM**: System analizuje kod HTML strony i dynamicznie tworzy skrypty automatyzacji w dedykowanym języku (DSL).
- **⚙️ Wykonanie przez TagUI**: Rzeczywista automatyzacja interakcji w przeglądarce (klikanie, pisanie, przesyłanie plików).
- **🐳 Pełna Konteneryzacja (Docker)**: Gotowe środowiska deweloperskie, testowe i produkcyjne dla łatwego uruchomienia i skalowania.
- **🧪 Kompleksowe Testowanie**: Wysokie pokrycie testami jednostkowymi, integracyjnymi, E2E (Playwright) i wydajnościowymi (k6).
- **📊 Monitoring i Obserwowalność**: Zintegrowany stos monitoringu z Prometheus, Grafana, Loki i Jaeger do śledzenia metryk, logów i wydajności.
- **🚀 Zautomatyzowane CI/CD**: Potok CI/CD na GitHub Actions zapewniający linting, testowanie, budowanie i wdrażanie aplikacji.

## 💡 Przykład działania

Przepływ pracy systemu jest prosty i skuteczny:

```mermaid
flowchart LR
    User[👤 Użytkownik] --> Data[📄 Dane + CV]
    Data --> Frontend[💻 Panel Aplikacji]
    Frontend --> Analyze[🌐 Analiza Strony]
    Analyze --> LLM["🧠 Generuj Skrypt (DSL)"]
    LLM --> DSL["📜 Skrypt: type/upload/click"]
    DSL --> TagUI[🤖 Wykonaj w Przeglądarce]
    TagUI --> Success[✅ Aplikacja Wysłana]
```

### Przykład wygenerowanego skryptu (DSL)

```dsl
// LLM analizuje formularz i generuje poniższy skrypt
click "#login-btn"
type "#username" "jan.kowalski"
type "#password" "SuperTajneHaslo!"
click "#submit"

type "#fullname" "Jan Kowalski"
type "#email" "jan.kowalski@example.com"
upload "#cv-upload" "C:/Users/Jan/Documents/CV.pdf"
click "#apply-submit"
```

## 🚀 Pierwsze Kroki

System jest w pełni skonteneryzowany, co sprawia, że uruchomienie go jest niezwykle proste.

### Wymagania

- [Docker](https://www.docker.com/get-started)
- [Docker Compose](https://docs.docker.com/compose/install/)
- `make` (opcjonalnie, ale zalecane dla łatwiejszego zarządzania)

### Instalacja i Uruchomienie

1. Sklonuj repozytorium:
   ```bash
   git clone https://github.com/codialog/tauri
   cd tauri
   make install
   ```

2. Uruchom wszystkie usługi za pomocą jednego polecenia:
   ```bash
   # Jeśli masz `make`
   make up

   # Alternatywnie, używając Docker Compose
   docker-compose up -d
   ```

3. Aplikacja będzie dostępna pod adresem: [http://localhost:1420](http://localhost:1420)

## 🛠️ Rozszerzone Zarządzanie Projektem

Dzięki zaktualizowanemu `Makefile` zarządzanie środowiskiem jest bardzo proste:

### 🚀 Podstawowe Komendy
```bash
# Szybki start - wszystko w jednym poleceniu
make quick-start       # Setup + build + run + testy

# Zarządzanie usługami
make up               # Uruchomienie Docker services
make down             # Zatrzymanie usług
make restart          # Restart wszystkich usług
make status           # Status usług i healthchecks
```

### 🧪 Nowe Komendy Testowe
```bash
# Kompleksowe testowanie
make test-all         # Wszystkie testy (unit + integration + e2e)
make quick-test       # Szybkie testy (tylko unit + integration)
make test-coverage    # Raport pokrycia testami
make test-watch       # Tryb ciągłego testowania
make test-bench       # Benchmarki wydajnościowe
```

### 🗄️ Zarządzanie Bazą Danych
```bash
make db-init          # Inicjalizacja bazy danych
make db-migrate       # Wykonanie migracji
make db-reset         # Reset bazy do czystego stanu
make db-seed          # Zasilenie przykładowymi danymi
make db-backup        # Backup bazy danych
make db-restore FILE= # Przywrócenie z backupu
```

### 📊 Monitoring i Wydajność
```bash
make monitor          # Panel Grafana (localhost:3000)
make performance-test # Testy wydajnościowe z k6
make logs-stats       # Statystyki logów aplikacji
make health-check     # Sprawdzenie stanu wszystkich usług
```

### 🔧 Tryb Deweloperski
```bash
make dev              # Tryb deweloperski z hot reload
make dev-setup        # Pełne środowisko deweloperskie  
make lint             # Linting kodu (Rust + JS)
make format           # Formatowanie kodu
make clean-all        # Czyszczenie wszystkich artefaktów
```

## 📁 Struktura Projektu

```
codialog/
├── .github/         # Workflows CI/CD (GitHub Actions)
├── tests/           # Wszystkie testy (unit, integration, e2e, performance)  
├── scripts/         # Przykładowe skrypty DSL
├── src-tauri/       # Backend aplikacji Tauri (Rust)
│   ├── src/
│   │   ├── main.rs     # Główny plik aplikacji
│   │   ├── cdp.rs      # Obsługa Chrome DevTools Protocol
│   │   ├── tagui.rs    # Integracja z TagUI
│   │   └── llm.rs      # Generowanie skryptów przez LLM
│   ├── build.rs        # Skrypt budowania
│   ├── Cargo.toml      # Zależności Rust
│   └── tauri.conf.json # Konfiguracja Tauri
├── src/             # Frontend aplikacji (HTML/JS/CSS)
│   ├── index.html
│   ├── main.js
│   └── style.css  
├── docker/          # Pliki Docker
├── Makefile         # Polecenia zarządzania
├── package.json     # Zależności Node.js
├── .env.example     # Przykład konfiguracji środowiska
└── README.md        # Ta dokumentacja
```

## 🛠️ Pliki Konfiguracyjne

### Kluczowe pliki do modyfikacji:

| Plik | Opis | Link |
|------|------|------|
| **[src-tauri/Cargo.toml](src-tauri/Cargo.toml)** | Zależności Rust i konfiguracja projektu | [📁](src-tauri/Cargo.toml) |
| **[src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)** | Konfiguracja aplikacji Tauri | [⚙️](src-tauri/tauri.conf.json) |
| **[package.json](package.json)** | Zależności Node.js i skrypty npm | [📦](package.json) |
| **[Makefile](Makefile)** | Komendy zarządzania projektem | [🔧](Makefile) |
| **[.env.example](.env.example)** | Przykład zmiennych środowiskowych | [🔐](.env.example) |

### Główne pliki źródłowe:

| Plik | Funkcjonalność | Link |
|------|----------------|------|
| **[src-tauri/src/main.rs](src-tauri/src/main.rs)** | Główna logika aplikacji, HTTP server | [🦀](src-tauri/src/main.rs) |
| **[src-tauri/src/llm.rs](src-tauri/src/llm.rs)** | Generowanie skryptów DSL przez LLM | [🧠](src-tauri/src/llm.rs) |
| **[src-tauri/src/tagui.rs](src-tauri/src/tagui.rs)** | Wykonywanie skryptów TagUI | [🤖](src-tauri/src/tagui.rs) |
| **[src-tauri/src/cdp.rs](src-tauri/src/cdp.rs)** | Analiza stron przez Chrome DevTools | [🌐](src-tauri/src/cdp.rs) |
| **[src/index.html](src/index.html)** | Główny interfejs użytkownika | [🎨](src/index.html) |
| **[src/main.js](src/main.js)** | Logika frontend JavaScript | [⚡](src/main.js) |

## 🔧 Rozszerzone API Endpoints

Aplikacja oferuje RESTful API dostępne pod `http://127.0.0.1:4000`:

### 🩺 Health Check
```http
GET /health
```
**Odpowiedź:**
```json
{
  "status": "healthy",
  "services": {
    "database": "connected", 
    "redis": "connected",
    "bitwarden": "available",
    "tagui": true
  },
  "version": "2.0.0",
  "uptime": "2h 15m 30s"
}
```

### 🔐 Nowe Bitwarden API Endpoints
```http
# Login do Bitwarden
POST /bitwarden/login
Content-Type: application/json
{
  "email": "user@example.com",
  "password": "master_password"
}

# Unlock vault z master password
POST /bitwarden/unlock  
Content-Type: application/json
{
  "master_password": "your_master_password"
}

# Pobranie credentials dla domeny
GET /bitwarden/credentials?domain=linkedin.com

# Status sesji Bitwarden
GET /bitwarden/status
```

### 🧠 Generowanie Skryptów DSL
```http  
POST /dsl/generate
Content-Type: application/json

{
  "action_type": "job_application",
  "user_data": {
    "first_name": "Jan",
    "last_name": "Kowalski",
    "email": "jan.kowalski@example.com", 
    "phone": "+48123456789",
    "cv_path": "/path/to/cv.pdf"
  },
  "target_url": "https://example.com/careers",
  "html": "<form>...</form>"
}
```

**Odpowiedź:**
```json
{
  "script": "type \"#email\" \"jan.kowalski@example.com\"\ntype \"#phone\" \"+48123456789\"\nupload \"#resume\" \"/path/to/cv.pdf\"\nclick \"#submit\""
}
```

### 🤖 Wykonywanie Skryptów RPA
```http
POST /rpa/run
Content-Type: application/json

{
  "script": "click \"#login\"\ntype \"#username\" \"user\"\nclick \"#submit\"",
  "url": "https://example.com"
}
```

### 🌐 Analiza Strony Web  
```http
GET /page/analyze?url=https://example.com
```

**Odpowiedź:**
```json
{
  "html": "<html>...</html>"
}
```

## 📊 Monitoring i Logi

### Zarządzanie Danymi Logowania
Aplikacja integruje się z systemem zarządzania hasłami **Bitwarden** dla bezpiecznego przechowywania danych logowania:

- **🔐 Integracja z Bitwarden**: Automatyczne pobieranie danych logowania z vault'a
- **🐳 Docker Persistence**: Dane zachowane nawet po usunięciu kontenerów
- **🔄 Auto-fill**: Automatyczne wypełnianie pól logowania w formularzach
- **🛡️ Security**: Bezpieczne przechowywanie credentials

### Zaawansowany System Logowania
Aplikacja oferuje kompleksowy system logowania z instrumentacją `tracing`:

```bash
# 📊 PODSTAWOWE LOGI
make logs              # Sprawdzenie logów aplikacji
make logs-follow       # Logi w czasie rzeczywistym  
make logs-stats        # Statystyki logów i wydajności

# 🔍 SZCZEGÓŁOWE DEBUGOWANIE  
RUST_LOG=debug make dev           # Szczegółowe logi debugowe
RUST_LOG=trace make dev           # Maksymalny poziom logów
make logs-filter LEVEL=error      # Filtrowanie po poziomie
make logs-search QUERY="bitwarden" # Wyszukiwanie w logach
```

**Lokalizacje i typy plików logów:**
- `src-tauri/logs/app.log` - Główne logi aplikacji ze strukturą JSON
- `src-tauri/logs/debug.log` - Szczegółowe informacje debugowe z czasem wykonania
- `src-tauri/logs/error.log` - Błędy, ostrzeżenia i stack traces
- `src-tauri/logs/performance.log` - Metryki wydajności i benchmarki
- `data/logs/` - Archiwum starych logów z rotacją

**Nowe funkcje logowania:**
- ✅ **Strukturalne logowanie JSON** z polami: timestamp, level, target, message, spans
- ✅ **Instrumentacja async funkcji** z mierzeniem czasu wykonania
- ✅ **Śledzenie request/response** z unikalnym ID dla każdego żądania
- ✅ **Logowanie do bazy danych** dla analityki i monitoringu
- ✅ **Automatyczna rotacja** plików logów co 10MB/1 tydzień
- ✅ **Filtrowanie i wyszukiwanie** w czasie rzeczywistym

### Trwałość Danych
System zapewnia zachowanie danych między restartami:

- **📁 Persistent Volumes**: Konfiguracja Docker z trwałymi wolumenami
- **💾 Database Backup**: Automatyczne kopie zapasowe bazy danych
- **🔄 Session Management**: Zachowanie sesji użytkownika
- **📝 Script Cache**: Cache wygenerowanych skryptów DSL

## 🧪 Kompleksowe Testowanie

### Nowa Struktura Testów (Rozszerzona)
```
src-tauri/src/tests/
├── mod.rs              # Moduł główny testów z wspólnymi funkcjami
├── test_llm.rs         # Testy generacji DSL (25+ testów)
├── test_bitwarden.rs   # Testy integracji Bitwarden (20+ testów)
├── test_session.rs     # Testy zarządzania sesjami (15+ testów)
├── test_logging.rs     # Testy systemu logowania (15+ testów)
├── test_database.rs    # Testy operacji bazodanowych (10+ testów)
└── integration_tests.rs # Testy integracyjne end-to-end (10+ testów)

tests/                  # Testy zewnętrzne (poprzednia struktura)
├── e2e/               # Testy end-to-end (Playwright)
├── unit/              # Testy jednostkowe JavaScript
├── integration/       # Testy integracyjne API
└── fixtures/          # Dane testowe
```

### Nowe Komendy Testowe
```bash
# 🧪 PODSTAWOWE TESTOWANIE
make test              # Wszystkie testy Rust (unit + integration)
make test-unit         # Tylko testy jednostkowe
make test-integration  # Tylko testy integracyjne
make quick-test        # Szybkie testy (unit + integration)

# 📊 ZAAWANSOWANE TESTOWANIE
make test-coverage     # Raport pokrycia testami (HTML)
make test-watch        # Tryb ciągłego testowania
make test-bench        # Benchmarki wydajnościowe
make test-clean        # Czyszczenie artefaktów testowych

# 🎯 TESTOWANIE KOMPONENTÓW
# Testy DSL generation z cache'owaniem
# Testy Bitwarden CLI integration
# Testy session persistence i recovery
# Testy database operations i migrations
# Testy logging system z filtrowaniem
# Testy error handling i fallbacks
```

### Pokrycie Testami
**Aktualnie: 85%+ pokrycie kodu**
- ✅ **DSL Generation**: 25 testów (cache, LLM, form analysis)
- ✅ **Bitwarden Integration**: 20 testów (login, credentials, parsing)  
- ✅ **Session Management**: 15 testów (create, update, expire, cleanup)
- ✅ **Logging System**: 15 testów (levels, filtering, rotation, performance)
- ✅ **Database Operations**: 10 testów (CRUD, transactions, migrations)
- ✅ **Integration Workflows**: 10 testów (end-to-end scenarios)

### Dodatkowe Testy E2E
Dodano rozszerzone testy pokrywające:
- ✅ Upload CV i automatyczne wypełnianie
- ✅ Integracja z Bitwarden
- ✅ Generowanie i wykonywanie skryptów DSL  
- ✅ Obsługa błędów i recovery
- ✅ Multi-browser testing

## 🐳 Docker i Deploy

### Szybki Start z Dockerem
```bash
# Uruchomienie wszystkich serwisów
docker-compose up -d

# Tylko aplikacja główna
docker-compose up app

# Z Bitwarden integration
docker-compose -f docker-compose.yml -f docker-compose.bitwarden.yml up -d

# Monitoring stack  
docker-compose -f docker-compose.monitoring.yml up -d
```

### Konfiguracja Środowiska
Skopiuj i dostosuj zmienne środowiskowe:
```bash
cp .env.example .env
# Edytuj .env z własnymi kluczami API i konfiguracją
```

**Wymagane zmienne:**
```env
# LLM API Configuration
CLAUDE_API_KEY=your_claude_key
OPENAI_API_KEY=your_openai_key

# Bitwarden Integration  
BITWARDEN_SERVER_URL=https://vault.bitwarden.com
BITWARDEN_CLIENT_ID=your_client_id
BITWARDEN_CLIENT_SECRET=your_client_secret

# Application Settings
APP_ENV=production
LOG_LEVEL=info
DATABASE_URL=postgresql://user:pass@localhost/codialog
```

### Deployment
```bash
# Build produkcyjny
make build-prod

# Deploy na staging
make deploy-staging

# Deploy na production
make deploy-prod
```

## 🚀 Przykłady Użycia Nowych Funkcji

### Pełny Przepływ z Bitwarden
```bash
# 1. Uruchomienie aplikacji z wszystkimi usługami
make quick-start

# 2. Login do Bitwarden przez API
curl -X POST http://localhost:4000/bitwarden/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"master_pass"}'

# 3. Unlock vault
curl -X POST http://localhost:4000/bitwarden/unlock \
  -H "Content-Type: application/json" \
  -d '{"master_password":"master_pass"}'

# 4. Pobranie credentials dla LinkedIn
curl "http://localhost:4000/bitwarden/credentials?domain=linkedin.com"

# 5. Automatyczne wypełnienie formularza z cache DSL
curl -X POST http://localhost:4000/dsl/generate \
  -H "Content-Type: application/json" \
  -d '{"action_type":"job_application","user_data":{...},"target_url":"https://linkedin.com/jobs/apply/123"}'
```

### Monitoring w Czasie Rzeczywistym  
```bash
# Panel monitoringu z wszystkimi metrykami
make monitor

# Śledzenie logów z filtrowaniem
make logs-follow | grep "ERROR\|WARN"

# Statystyki wydajności
make logs-stats

# Testy wydajnościowe
make performance-test
```

### Maintenance Mode
```bash
# Włączenie trybu maintenance (blokada nowych sesji)
make maintenance-on

# Sprawdzenie statusu maintenance
make status

# Wyłączenie trybu maintenance
make maintenance-off

# Backup przed maintenance
make db-backup
```

## 🔧 Rozwój i Kontrybucje

### Setup rozszerzonego środowiska deweloperskiego:
```bash  
# Pełne środowisko deweloperskie
make dev-setup

# Uruchomienie w trybie dev z hot reload
make dev

# Testowanie w trybie watch (ciągłe)
make test-watch

# Linting i formatowanie
make lint
make format

# Generowanie dokumentacji
make docs

# Czyszczenie wszystkich artefaktów
make clean-all
```

### Struktura Pull Request:
1. Fork repozytorium
2. Stwórz branch: `git checkout -b feature/nazwa-funkcji`
3. Commit: `git commit -m 'Add: opis funkcji'`
4. Push: `git push origin feature/nazwa-funkcji`  
5. Otwórz Pull Request

## 📞 Wsparcie

- **📧 Email**: info@softreck.dev
- **🐛 Issues**: [GitHub Issues](https://github.com/codialog-com/tauri/issues)
- **📚 Wiki**: [Documentation Wiki](https://github.com/codialog-com/tauri/wiki)

## 📜 Licencja

Projekt jest udostępniony na licencji [Apache License 2.0](LICENSE).

**Copyright 2024 Tom Sapletta**

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
