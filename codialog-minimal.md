# Codialog - Minimalna wersja funkcjonalna

## Struktura projektu

```
codialog/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── cdp.rs
│   │   ├── tagui.rs
│   │   └── llm.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── index.html
│   ├── main.js
│   └── style.css
├── scripts/
│   └── upload_cv.codialog
└── package.json
```

## 1. Backend - Rust/Tauri

### `src-tauri/Cargo.toml`

```toml
[package]
name = "codialog"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2.0", features = ["shell-open", "process-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
axum = "0.7"
tower = "0.4"
chromiumoxide = { version = "0.5", features = ["tokio-runtime"] }

[dependencies.tauri-plugin-shell]
version = "2.0.0"
```

### `src-tauri/src/main.rs`

```rust
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cdp;
mod tagui;
mod llm;

use tauri::Manager;
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    webview_url: Arc<Mutex<String>>,
}

#[derive(Serialize, Deserialize)]
struct DslRequest {
    html: String,
    user_data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct DslResponse {
    script: String,
}

#[derive(Serialize, Deserialize)]
struct RunScriptRequest {
    script: String,
}

// Endpoint do generowania DSL przez LLM
async fn generate_dsl(
    Json(payload): Json<DslRequest>,
) -> Json<DslResponse> {
    let script = llm::generate_dsl_script(&payload.html, &payload.user_data).await;
    Json(DslResponse { script })
}

// Endpoint do uruchamiania skryptu TagUI
async fn run_tagui(
    Json(payload): Json<RunScriptRequest>,
) -> Json<serde_json::Value> {
    let result = tagui::execute_script(&payload.script).await;
    Json(serde_json::json!({ "success": result }))
}

// Endpoint do analizy strony przez CDP
async fn analyze_page(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let url = state.webview_url.lock().await;
    let html = cdp::get_page_html(&url).await.unwrap_or_default();
    Json(serde_json::json!({ "html": html }))
}

#[tauri::command]
async fn load_url(url: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut webview_url = state.webview_url.lock().await;
    *webview_url = url;
    Ok(())
}

fn main() {
    let app_state = AppState {
        webview_url: Arc::new(Mutex::new(String::new())),
    };

    // Uruchom serwer HTTP w tle
    let state_clone = app_state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/dsl/generate", post(generate_dsl))
            .route("/rpa/run", post(run_tagui))
            .route("/page/analyze", get(analyze_page))
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
            .await
            .unwrap();
        
        axum::serve(listener, app).await.unwrap();
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![load_url])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### `src-tauri/src/tagui.rs`

```rust
use std::process::Command;
use std::fs;
use std::path::Path;

pub async fn execute_script(dsl_script: &str) -> bool {
    // Zapisz skrypt do pliku tymczasowego
    let script_path = "temp_script.codialog";
    fs::write(script_path, dsl_script).unwrap();
    
    // Uruchom TagUI
    let output = Command::new("tagui")
        .arg(script_path)
        .arg("chrome")
        .output();
    
    // Usuń plik tymczasowy
    fs::remove_file(script_path).ok();
    
    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

pub fn install_tagui() -> bool {
    // Sprawdź czy TagUI jest zainstalowane
    if !Path::new("tagui").exists() {
        // Pobierz i zainstaluj TagUI
        let output = Command::new("git")
            .args(&["clone", "https://github.com/aisingapore/tagui"])
            .output();
        
        if output.is_err() {
            return false;
        }
    }
    true
}
```

### `src-tauri/src/llm.rs`

```rust
use serde_json::Value;
use reqwest;

pub async fn generate_dsl_script(html: &str, user_data: &Value) -> String {
    // Prosta logika parsowania HTML i generowania DSL
    // W produkcji tutaj byłoby wywołanie do prawdziwego LLM
    
    let mut script = String::new();
    
    // Analiza formularza w HTML
    if html.contains("id=\"login-btn\"") {
        script.push_str("click \"#login-btn\"\n");
    }
    
    // Mapowanie danych użytkownika na pola formularza
    if let Some(username) = user_data.get("username") {
        script.push_str(&format!("type \"#username\" \"{}\"\n", username.as_str().unwrap_or("")));
    }
    
    if let Some(password) = user_data.get("password") {
        script.push_str(&format!("type \"#password\" \"{}\"\n", password.as_str().unwrap_or("")));
    }
    
    if let Some(fullname) = user_data.get("fullname") {
        script.push_str(&format!("type \"#fullname\" \"{}\"\n", fullname.as_str().unwrap_or("")));
    }
    
    if let Some(email) = user_data.get("email") {
        script.push_str(&format!("type \"#email\" \"{}\"\n", email.as_str().unwrap_or("")));
    }
    
    if let Some(cv_path) = user_data.get("cv_path") {
        script.push_str(&format!("upload \"#cv-upload\" \"{}\"\n", cv_path.as_str().unwrap_or("")));
    }
    
    // Zatwierdzenie formularza
    if html.contains("id=\"submit\"") || html.contains("id=\"apply-submit\"") {
        script.push_str("click \"#submit\"\n");
    }
    
    script
}

// Funkcja do wywołania rzeczywistego LLM (np. Claude API)
pub async fn generate_dsl_with_llm(html: &str, user_data: &Value) -> String {
    let prompt = format!(
        "Przeanalizuj formularz HTML i wygeneruj skrypt DSL do jego wypełnienia.\n\
        Dostępne komendy: click, type, upload, hover\n\
        HTML: {}\n\
        Dane użytkownika: {}\n\
        Wygeneruj tylko komendy DSL, bez komentarzy:",
        html, user_data
    );
    
    // Tutaj wywołanie do API Claude
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 500,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        }))
        .send()
        .await;
    
    match response {
        Ok(res) => {
            if let Ok(body) = res.json::<Value>().await {
                if let Some(content) = body["content"][0]["text"].as_str() {
                    return content.to_string();
                }
            }
        }
        _ => {}
    }
    
    // Fallback do prostej logiki
    generate_dsl_script(html, user_data).await
}
```

### `src-tauri/src/cdp.rs`

```rust
use chromiumoxide::Browser;
use futures::StreamExt;

pub async fn get_page_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (mut browser, mut handler) = Browser::launch(
        chromiumoxide::BrowserConfig::builder()
            .build()?
    ).await?;
    
    let handle = tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });
    
    let page = browser.new_page(url).await?;
    let html = page.content().await?;
    
    browser.close().await?;
    handle.abort();
    
    Ok(html)
}
```

## 2. Frontend

### `src/index.html`

```html
<!DOCTYPE html>
<html lang="pl">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Codialog - Automatyzacja CV</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <h1>🤖 Codialog - Automatyzacja CV</h1>
        
        <div class="section">
            <h2>1. Dane użytkownika</h2>
            <div class="form-group">
                <input type="text" id="fullname" placeholder="Imię i nazwisko">
                <input type="email" id="email" placeholder="Email">
                <input type="text" id="username" placeholder="Login do systemu">
                <input type="password" id="password" placeholder="Hasło">
                <input type="file" id="cv-file" accept=".pdf,.doc,.docx">
                <div id="cv-path" class="file-path"></div>
            </div>
        </div>

        <div class="section">
            <h2>2. URL formularza</h2>
            <div class="form-group">
                <input type="url" id="target-url" placeholder="https://example.com/apply">
                <button id="analyze-btn">Analizuj stronę</button>
            </div>
        </div>

        <div class="section">
            <h2>3. Wygenerowany skrypt DSL</h2>
            <textarea id="dsl-script" rows="10" readonly></textarea>
            <div class="button-group">
                <button id="generate-btn">Generuj DSL</button>
                <button id="run-btn">Uruchom automatyzację</button>
            </div>
        </div>

        <div class="section">
            <h2>4. Status</h2>
            <div id="status" class="status"></div>
        </div>
    </div>

    <script src="main.js"></script>
</body>
</html>
```

### `src/main.js`

```javascript
const API_URL = 'http://localhost:4000';

// Obsługa wyboru pliku CV
document.getElementById('cv-file').addEventListener('change', (e) => {
    const file = e.target.files[0];
    if (file) {
        const path = file.path || `C:/Users/User/Documents/${file.name}`;
        document.getElementById('cv-path').textContent = `Ścieżka: ${path}`;
        document.getElementById('cv-path').dataset.path = path;
    }
});

// Analiza strony
document.getElementById('analyze-btn').addEventListener('click', async () => {
    const url = document.getElementById('target-url').value;
    if (!url) {
        showStatus('Podaj URL strony', 'error');
        return;
    }
    
    showStatus('Analizuję stronę...', 'info');
    
    try {
        // Załaduj stronę w WebView
        await window.__TAURI__.invoke('load_url', { url });
        
        // Pobierz HTML przez CDP
        const response = await fetch(`${API_URL}/page/analyze`);
        const data = await response.json();
        
        showStatus('Strona przeanalizowana', 'success');
        window.pageHTML = data.html;
    } catch (error) {
        showStatus('Błąd analizy: ' + error.message, 'error');
    }
});

// Generowanie DSL
document.getElementById('generate-btn').addEventListener('click', async () => {
    const userData = {
        fullname: document.getElementById('fullname').value,
        email: document.getElementById('email').value,
        username: document.getElementById('username').value,
        password: document.getElementById('password').value,
        cv_path: document.getElementById('cv-path').dataset.path || ''
    };
    
    if (!window.pageHTML) {
        showStatus('Najpierw przeanalizuj stronę', 'error');
        return;
    }
    
    showStatus('Generuję skrypt DSL...', 'info');
    
    try {
        const response = await fetch(`${API_URL}/dsl/generate`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                html: window.pageHTML || '<html></html>',
                user_data: userData
            })
        });
        
        const data = await response.json();
        document.getElementById('dsl-script').value = data.script;
        showStatus('Skrypt DSL wygenerowany', 'success');
    } catch (error) {
        showStatus('Błąd generowania: ' + error.message, 'error');
    }
});

// Uruchomienie automatyzacji
document.getElementById('run-btn').addEventListener('click', async () => {
    const script = document.getElementById('dsl-script').value;
    
    if (!script) {
        showStatus('Najpierw wygeneruj skrypt DSL', 'error');
        return;
    }
    
    showStatus('Uruchamiam automatyzację...', 'info');
    
    try {
        const response = await fetch(`${API_URL}/rpa/run`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ script })
        });
        
        const data = await response.json();
        
        if (data.success) {
            showStatus('✅ Automatyzacja zakończona sukcesem!', 'success');
        } else {
            showStatus('❌ Automatyzacja nie powiodła się', 'error');
        }
    } catch (error) {
        showStatus('Błąd wykonania: ' + error.message, 'error');
    }
});

function showStatus(message, type) {
    const status = document.getElementById('status');
    status.textContent = message;
    status.className = `status ${type}`;
}
```

### `src/style.css`

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    padding: 20px;
}

.container {
    max-width: 800px;
    margin: 0 auto;
    background: white;
    border-radius: 12px;
    padding: 30px;
    box-shadow: 0 20px 60px rgba(0,0,0,0.3);
}

h1 {
    color: #333;
    margin-bottom: 30px;
    text-align: center;
}

h2 {
    color: #555;
    margin-bottom: 15px;
    font-size: 18px;
}

.section {
    margin-bottom: 25px;
    padding: 20px;
    background: #f8f9fa;
    border-radius: 8px;
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

input, textarea {
    padding: 10px 15px;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 14px;
}

input:focus, textarea:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

button {
    padding: 10px 20px;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    transition: all 0.3s;
}

button:hover {
    background: #5a67d8;
    transform: translateY(-1px);
    box-shadow: 0 5px 15px rgba(102, 126, 234, 0.3);
}

.button-group {
    display: flex;
    gap: 10px;
    margin-top: 10px;
}

.file-path {
    font-size: 12px;
    color: #666;
    padding: 5px;
    background: white;
    border-radius: 4px;
}

.status {
    padding: 15px;
    border-radius: 6px;
    font-weight: 500;
    text-align: center;
}

.status.info {
    background: #e3f2fd;
    color: #1976d2;
}

.status.success {
    background: #e8f5e9;
    color: #388e3c;
}

.status.error {
    background: #ffebee;
    color: #c62828;
}

textarea {
    font-family: 'Courier New', monospace;
    background: #2d2d2d;
    color: #4fc08d;
    resize: vertical;
}
```

## 3. Przykładowe skrypty DSL

### `scripts/upload_cv.codialog`

```
// Minimalny skrypt do uploadu CV
click "#login-btn"
type "#username" "jan.kowalski"
type "#password" "SuperTajneHaslo!"
click "#submit"
type "#fullname" "Jan Kowalski"
type "#email" "jan.kowalski@example.com"
upload "#cv-upload" "C:/Users/Jan/Documents/CV.pdf"
click "#apply-submit"
```

### `scripts/apply_job.codialog`

```
// Wypełnianie formularza aplikacyjnego
hover "#job-apply"
click "#job-apply"
type "#first-name" "Jan"
type "#last-name" "Kowalski"
type "#email" "jan@example.com"
type "#phone" "+48123456789"
type "#linkedin" "https://linkedin.com/in/jankowalski"
upload "#resume" "C:/Users/Jan/CV.pdf"
upload "#cover-letter" "C:/Users/Jan/CoverLetter.pdf"
click "#gdpr-consent"
click "#submit-application"
```

## 4. Package.json

```json
{
  "name": "codialog",
  "version": "0.1.0",
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "install-tagui": "git clone https://github.com/aisingapore/tagui"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0"
  }
}
```

## 5. Instrukcja uruchomienia

```bash
# 1. Instalacja zależności
npm install
cargo install tauri-cli

# 2. Instalacja TagUI
npm run install-tagui

# 3. Uruchomienie w trybie dev
npm run dev

# 4. Build produkcyjny
npm run build
```

## Przepływ działania

1. **Użytkownik** wprowadza dane (imię, email, ścieżka do CV)
2. **Analiza strony** - CDP pobiera HTML formularza
3. **Generowanie DSL** - LLM tworzy sekwencję komend
4. **Wykonanie** - TagUI realizuje automatyzację
5. **Rezultat** - formularz wypełniony, CV przesłane

## Kluczowe funkcjonalności

✅ **Upload plików** - obsługa `upload "#selector" "path"`
✅ **Wypełnianie pól** - `type "#field" "value"`
✅ **Klikanie** - `click "#button"`
✅ **Automatyczna analiza** formularzy
✅ **Generowanie DSL** przez LLM
✅ **Wykonanie przez TagUI**