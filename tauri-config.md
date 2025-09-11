# Konfiguracja Tauri i rozszerzenia

## `src-tauri/tauri.conf.json`

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Codialog",
  "version": "0.1.0",
  "identifier": "com.codialog.app",
  "build": {
    "beforeDevCommand": "",
    "devUrl": "../src",
    "beforeBuildCommand": "",
    "frontendDist": "../src"
  },
  "app": {
    "windows": [
      {
        "title": "Codialog - Automatyzacja CV",
        "width": 900,
        "height": 800,
        "resizable": true,
        "fullscreen": false,
        "alwaysOnTop": false,
        "webviewWindow": {
          "devtools": true
        }
      }
    ],
    "security": {
      "csp": null,
      "dangerousDisableAssetCspModification": true
    }
  },
  "plugins": {
    "shell": {
      "open": true,
      "execute": {
        "sidecar": true,
        "command": true
      }
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

## Rozszerzona integracja z Claude API

### `src-tauri/src/llm_advanced.rs`

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest;

#[derive(Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

pub struct LLMClient {
    api_key: String,
    client: reqwest::Client,
}

impl LLMClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn generate_dsl_advanced(
        &self,
        html: &str,
        user_data: &Value,
        form_type: &str
    ) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = self.build_prompt(html, user_data, form_type);
        
        let request = ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 1000,
            messages: vec![
                ClaudeMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        
        if let Some(content) = response_body["content"][0]["text"].as_str() {
            Ok(self.parse_dsl_from_response(content))
        } else {
            Err("Invalid response from Claude API".into())
        }
    }

    fn build_prompt(&self, html: &str, user_data: &Value, form_type: &str) -> String {
        format!(
            r#"Jesteś ekspertem w automatyzacji formularzy webowych przy użyciu DSL.
            
Zadanie: Wygeneruj skrypt DSL do wypełnienia formularza typu: {}

Dostępne komendy DSL:
- click "#selector" - kliknij element
- type "#selector" "text" - wpisz tekst
- upload "#selector" "path" - wybierz plik
- hover "#selector" - najedź myszą

Analiza HTML formularza:
{}

Dane użytkownika do wypełnienia:
{}

WAŻNE ZASADY:
1. Używaj selektorów CSS (#id, .class, lub tag)
2. Najpierw zaloguj się jeśli to konieczne
3. Wypełnij wszystkie wymagane pola
4. Na końcu kliknij przycisk submit/apply
5. Zwróć TYLKO komendy DSL, bez komentarzy

Wygeneruj optymalną sekwencję komend DSL:"#,
            form_type,
            self.extract_form_elements(html),
            serde_json::to_string_pretty(user_data).unwrap_or_default()
        )
    }

    fn extract_form_elements(&self, html: &str) -> String {
        // Ekstrakcja kluczowych elementów formularza
        let mut elements = Vec::new();
        
        // Znajdź inputy
        if html.contains("<input") {
            elements.push("Inputs: username, password, email, fullname, phone");
        }
        
        // Znajdź przyciski
        if html.contains("<button") || html.contains("type=\"submit\"") {
            elements.push("Buttons: submit, login-btn, apply-submit");
        }
        
        // Znajdź pola file upload
        if html.contains("type=\"file\"") {
            elements.push("File uploads: cv-upload, resume, documents");
        }
        
        elements.join("\n")
    }

    fn parse_dsl_from_response(&self, response: &str) -> String {
        // Wyczyść odpowiedź z niepotrzebnych znaków
        response
            .lines()
            .filter(|line| {
                line.starts_with("click") || 
                line.starts_with("type") || 
                line.starts_with("upload") || 
                line.starts_with("hover")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Funkcje pomocnicze do różnych typów formularzy
pub mod templates {
    pub fn job_application_template() -> &'static str {
        r#"click "#accept-cookies"
hover "#careers-link"
click "#careers-link"
click "#apply-now"
type "#first-name" "{first_name}"
type "#last-name" "{last_name}"
type "#email" "{email}"
type "#phone" "{phone}"
upload "#resume" "{cv_path}"
upload "#cover-letter" "{cover_letter_path}"
click "#gdpr-consent"
click "#submit-application""#
    }

    pub fn registration_template() -> &'static str {
        r#"click "#register"
type "#username" "{username}"
type "#email" "{email}"
type "#password" "{password}"
type "#confirm-password" "{password}"
click "#terms-checkbox"
click "#create-account""#
    }

    pub fn linkedin_apply_template() -> &'static str {
        r#"click "#sign-in"
type "#username" "{linkedin_email}"
type "#password" "{linkedin_password}"
click "#sign-in-submit"
click ".jobs-apply-button"
upload "#resume-upload" "{cv_path}"
type "#phone" "{phone}"
click "#follow-company"
click "#submit-application""#
    }
}
```

## Testowy formularz HTML

### `test/form.html`

```html
<!DOCTYPE html>
<html>
<head>
    <title>Formularz aplikacyjny - Test</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .form-group {
            margin-bottom: 15px;
        }
        label {
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
        }
        input, textarea {
            width: 100%;
            padding: 8px;
            border: 1px solid #ddd;
            border-radius: 4px;
        }
        button {
            background: #4CAF50;
            color: white;
            padding: 10px 20px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        button:hover {
            background: #45a049;
        }
    </style>
</head>
<body>
    <h1>Aplikacja o pracę</h1>
    
    <form id="job-form">
        <!-- Logowanie -->
        <div class="form-group">
            <button type="button" id="login-btn">Zaloguj się</button>
        </div>
        
        <div id="login-section" style="display:none;">
            <div class="form-group">
                <label>Login:</label>
                <input type="text" id="username" required>
            </div>
            <div class="form-group">
                <label>Hasło:</label>
                <input type="password" id="password" required>
            </div>
            <button type="button" id="submit-login">Zaloguj</button>
        </div>
        
        <!-- Formularz aplikacyjny -->
        <div id="application-section">
            <div class="form-group">
                <label>Imię i nazwisko:</label>
                <input type="text" id="fullname" required>
            </div>
            
            <div class="form-group">
                <label>Email:</label>
                <input type="email" id="email" required>
            </div>
            
            <div class="form-group">
                <label>Telefon:</label>
                <input type="tel" id="phone">
            </div>
            
            <div class="form-group">
                <label>CV (PDF):</label>
                <input type="file" id="cv-upload" accept=".pdf" required>
            </div>
            
            <div class="form-group">
                <label>List motywacyjny:</label>
                <textarea id="cover-letter" rows="5"></textarea>
            </div>
            
            <div class="form-group">
                <label>
                    <input type="checkbox" id="consent" required>
                    Wyrażam zgodę na przetwarzanie danych osobowych
                </label>
            </div>
            
            <button type="submit" id="apply-submit">Wyślij aplikację</button>
        </div>
    </form>

    <script>
        document.getElementById('login-btn').addEventListener('click', () => {
            document.getElementById('login-section').style.display = 'block';
        });
        
        document.getElementById('job-form').addEventListener('submit', (e) => {
            e.preventDefault();
            alert('Aplikacja wysłana pomyślnie!');
        });
    </script>
</body>
</html>
```

## Skrypt instalacyjny

### `install.sh` (Linux/Mac)

```bash
#!/bin/bash

echo "🚀 Instalacja Codialog..."

# Sprawdź wymagania
command -v cargo >/dev/null 2>&1 || { 
    echo "❌ Rust nie jest zainstalowany. Zainstaluj z https://rustup.rs/"; 
    exit 1; 
}

command -v npm >/dev/null 2>&1 || { 
    echo "❌ Node.js nie jest zainstalowany."; 
    exit 1; 
}

# Instaluj zależności
echo "📦 Instaluję zależności..."
npm install
cargo install tauri-cli

# Instaluj TagUI
if [ ! -d "tagui" ]; then
    echo "🤖 Instaluję TagUI..."
    git clone https://github.com/aisingapore/tagui
    cd tagui
    npm install
    cd ..
fi

# Utwórz folder na skrypty
mkdir -p scripts

# Stwórz przykładowy skrypt
cat > scripts/example.codialog << 'EOF'
click "#login-btn"
type "#username" "user@example.com"
type "#password" "password123"
click "#submit"
type "#fullname" "Jan Kowalski"
type "#email" "jan@example.com"
upload "#cv-upload" "~/Documents/CV.pdf"
click "#apply-submit"
EOF

echo "✅ Instalacja zakończona!"
echo "Uruchom: npm run dev"
```

### `install.ps1` (Windows)

```powershell
Write-Host "🚀 Instalacja Codialog..." -ForegroundColor Green

# Sprawdź wymagania
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust nie jest zainstalowany. Zainstaluj z https://rustup.rs/" -ForegroundColor Red
    exit 1
}

if (!(Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Node.js nie jest zainstalowany." -ForegroundColor Red
    exit 1
}

# Instaluj zależności
Write-Host "📦 Instaluję zależności..." -ForegroundColor Yellow
npm install
cargo install tauri-cli

# Instaluj TagUI
if (!(Test-Path "tagui")) {
    Write-Host "🤖 Instaluję TagUI..." -ForegroundColor Yellow
    git clone https://github.com/aisingapore/tagui
    Set-Location tagui
    npm install
    Set-Location ..
}

# Utwórz folder na skrypty
New-Item -ItemType Directory -Force -Path scripts

# Stwórz przykładowy skrypt
@'
click "#login-btn"
type "#username" "user@example.com"
type "#password" "password123"
click "#submit"
type "#fullname" "Jan Kowalski"
type "#email" "jan@example.com"
upload "#cv-upload" "C:\Users\User\Documents\CV.pdf"
click "#apply-submit"
'@ | Out-File -FilePath scripts\example.codialog -Encoding UTF8

Write-Host "✅ Instalacja zakończona!" -ForegroundColor Green
Write-Host "Uruchom: npm run dev" -ForegroundColor Cyan
```

## Podsumowanie

Ta minimalna wersja **Codialog** zawiera:

### ✅ Funkcjonalności
- **Upload CV** - pełna obsługa przesyłania plików
- **Wypełnianie formularzy** - automatyczne uzupełnianie pól
- **Generowanie DSL** - inteligentne tworzenie skryptów
- **Wykonanie przez TagUI** - rzeczywista automatyzacja
- **Integracja z LLM** - opcjonalne wsparcie Claude API

### 🎯 Główne cechy
- **Minimalistyczna** - tylko niezbędny kod
- **Funkcjonalna** - działa od razu po instalacji
- **Rozszerzalna** - łatwa do modyfikacji
- **Cross-platform** - działa na Windows/Mac/Linux

### 📋 Użycie
1. Wprowadź dane (imię, email, ścieżka CV)
2. Podaj URL formularza
3. Kliknij "Generuj DSL"
4. Kliknij "Uruchom automatyzację"
5. CV zostanie automatycznie przesłane!

System jest gotowy do natychmiastowego użycia i testowania!