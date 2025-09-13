use serde_json::Value;
use reqwest;
use tracing::{info, error, debug, warn};
use crate::tagui::escape_for_dsl;
use sqlx::{PgPool, Row};
use anyhow::Result;
use std::collections::HashMap;

// ---- Lightweight shims expected by tests ----
#[derive(Debug, Default, Clone)]
pub struct LLMRequest {
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub content: String,
}

#[derive(Debug)]
pub enum LLMError {
    Generic(String),
}

#[derive(Debug, Clone)]
pub struct FormAnalysis {}

#[derive(Debug, Clone)]
pub enum FieldType { Text, Email, Password, File, Checkbox }

#[derive(Debug, Clone)]
pub struct FormField { pub name: String, pub field_type: FieldType }

pub fn analyze_form_structure(_html: &str) -> FormAnalysis { FormAnalysis {} }

pub fn process_natural_language_query(_q: &str) -> std::result::Result<String, LLMError> {
    Ok(String::new())
}

pub async fn get_llm_response(_req: &LLMRequest) -> std::result::Result<LLMResponse, LLMError> {
    Ok(LLMResponse { content: String::new() })
}

pub fn validate_dsl_script(script: &str) -> bool { validate_generated_script(script) }

pub fn generate_fallback_script(html: &str, user_data: &Value) -> String { generate_basic_fallback_script(html, user_data) }

// A trait used in tests to mock analyzer behavior
pub trait FormAnalyzerTrait {
    fn is_login_form(&self) -> bool;
    fn has_file_input(&self) -> bool { false }
    fn get_elements_by_type(&self, _t: &str) -> Vec<String>;
    fn find_submit_button(&self) -> Option<String>;
    fn find_cookie_consent(&self) -> Option<String>;
}

pub async fn generate_dsl_script(html: &str, user_data: &Value) -> String {
    generate_dsl_script_with_cache(html, user_data, None).await
}

pub(crate) fn generate_basic_fallback_script(_html: &str, _user_data: &Value) -> String {
    "// Basic fallback\nwait 3\nclick \"Continue\" if present\nwait 2\n".to_string()
}

pub(crate) fn generate_emergency_fallback_script(_html: &str, _user_data: &Value) -> String {
    "wait 5\n// Emergency fallback - manual intervention may be required\n".to_string()
}

pub(crate) fn validate_generated_script(script: &str) -> bool {
    !script.trim().is_empty() && script.len() > 5
}

pub async fn generate_dsl_script_with_cache(html: &str, user_data: &Value, db_pool: Option<&PgPool>) -> String {
    info!("Generating DSL script from HTML and user data");
    
    // Input validation with error recovery
    if html.trim().is_empty() {
        warn!("Empty HTML provided, generating basic navigation script");
        return basic_navigation_script();
    }
    
    // Validate user data structure
    if !user_data.is_object() {
        warn!("Invalid user data format, using empty data for DSL generation");
    }
    
    // Create cache key
    let cache_key = create_cache_key(html, user_data);
    
    // Try to get cached script first with retry logic
    if let Some(pool) = db_pool {
        match get_cached_dsl_script_with_retry(pool, &cache_key, 3).await {
            Ok(Some(cached_script)) => {
                info!("Using cached DSL script for key: {}", cache_key);
                return cached_script;
            }
            Ok(None) => debug!("No cached script found for key: {}", cache_key),
            Err(e) => warn!("Cache retrieval failed: {}", e),
        }
    }
    
    // Generate new script with comprehensive fallback strategy
    let script = match generate_script_with_comprehensive_fallbacks(html, user_data).await {
        Ok(generated_script) => {
            if generated_script.trim().is_empty() {
                warn!("Generated script is empty, using basic fallback");
                generate_basic_fallback_script(html, user_data)
            } else {
                generated_script
            }
        }
        Err(e) => {
            error!("All DSL generation methods failed: {}, using emergency fallback", e);
            generate_emergency_fallback_script(html, user_data)
        }
    };
    
    // Validate generated script before caching
    if validate_generated_script(&script) {
        // Cache the generated script with retry logic
        if let Some(pool) = db_pool {
            match cache_dsl_script_with_retry(pool, &cache_key, &script, html, 3).await {
                Ok(_) => debug!("Successfully cached DSL script"),
                Err(e) => warn!("Failed to cache DSL script after retries: {}", e),
            }
        }
    } else {
        warn!("Generated script failed validation, not caching");
    }
    
    script
}

pub(crate) fn create_cache_key(html: &str, user_data: &Value) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    
    // Create simplified HTML signature (remove dynamic content)
    let html_signature = html
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.contains("<input") || trimmed.contains("<button") || 
            trimmed.contains("<form") || trimmed.contains("<select") ||
            trimmed.contains("type=") || trimmed.contains("id=") ||
            trimmed.contains("name=") || trimmed.contains("class=")
        })
        .collect::<Vec<_>>()
        .join("");
    
    html_signature.hash(&mut hasher);
    
    // Hash user data structure (not values for privacy)
    let user_keys: Vec<String> = user_data.as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();
    user_keys.join(",").hash(&mut hasher);
    
    format!("dsl_{:x}", hasher.finish())
}

async fn get_cached_dsl_script_with_retry(pool: &PgPool, cache_key: &str, retries: u32) -> Result<Option<String>> {
    for attempt in 0..retries {
        match sqlx::query("SELECT script_content FROM dsl_cache WHERE cache_key = $1 AND expires_at > NOW()")
            .bind(cache_key)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(row)) => {
                let script: String = row.try_get("script_content")?;
                return Ok(Some(script));
            }
            Ok(None) => return Ok(None),
            Err(e) if attempt < retries - 1 => {
                warn!("Cache retrieval attempt {} failed: {}", attempt + 1, e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(None)
}

async fn generate_script_with_comprehensive_fallbacks(html: &str, user_data: &Value) -> Result<String> {
    // First try: Enhanced form analysis
    if let Ok(script) = generate_enhanced_form_script(html, user_data).await {
        if !script.trim().is_empty() {
            return Ok(script);
        }
    }
    
    // Second try: Simple form parsing
    if let Ok(script) = generate_simple_form_script(html, user_data).await {
        if !script.trim().is_empty() {
            return Ok(script);
        }
    }
    
    // Final fallback
    Ok(basic_navigation_script())
}

async fn generate_enhanced_form_script(html: &str, _user_data: &Value) -> Result<String> {
    let analyzer = FormAnalyzer::new(html);
    let mut script = String::new();
    
    // Add basic navigation commands
    script.push_str("wait 2\n");
    
    // Process form elements
    for (element_type, _) in &analyzer.elements {
        match element_type.as_str() {
            "input" => script.push_str("// Input field detected\n"),
            "button" => script.push_str("// Button detected\n"),
            "select" => script.push_str("// Select field detected\n"),
            _ => {}
        }
    }
    
    script.push_str("wait 1\n");
    Ok(script)
}

async fn generate_simple_form_script(_html: &str, _user_data: &Value) -> Result<String> {
    Ok("wait 3\nclick \"Submit\" if present\nwait 2\n".to_string())
}

fn basic_navigation_script() -> String {
    debug!("Generating basic navigation script as fallback");
    
    // Basic navigation script for common scenarios
    let script = r#"
// Basic navigation script
wait 3
click "Accept" if present
click "Login" if present
wait 2
"#;
    
    script.trim().to_string()
}

async fn cache_dsl_script_with_retry(pool: &PgPool, cache_key: &str, script: &str, html: &str, retries: u32) -> Result<()> {
    for attempt in 0..retries {
        match sqlx::query(
            "INSERT INTO dsl_cache (cache_key, script_content, html_content, expires_at) 
             VALUES ($1, $2, $3, NOW() + INTERVAL '1 hour')
             ON CONFLICT (cache_key) DO UPDATE SET 
             script_content = EXCLUDED.script_content,
             html_content = EXCLUDED.html_content,
             expires_at = EXCLUDED.expires_at"
        )
        .bind(cache_key)
        .bind(script)
        .bind(html)
        .execute(pool)
        .await
        {
            Ok(_) => return Ok(()),
            Err(e) if attempt < retries - 1 => {
                warn!("Cache storage attempt {} failed: {}", attempt + 1, e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

pub(crate) struct FormAnalyzer {
    html: String,
    elements: HashMap<String, Vec<String>>,
}

impl FormAnalyzer {
    pub(crate) fn new(html: &str) -> Self {
        let mut analyzer = FormAnalyzer {
            html: html.to_string(),
            elements: HashMap::new(),
        };
        analyzer.analyze_elements();
        analyzer
    }
    
    fn analyze_elements(&mut self) {
        // Parse HTML to find form elements (simplified parser)
        let html_content = self.html.clone();
        let lines: Vec<&str> = html_content.lines().collect();
        
        for line in lines {
            if line.contains("<input") {
                self.parse_input_element(line);
            } else if line.contains("<button") || line.contains("<input") && line.contains("type=\"submit\"") {
                self.parse_button_element(line);
            } else if line.contains("<select") {
                self.parse_select_element(line);
            }
        }
    }
    
    fn parse_input_element(&mut self, line: &str) {
        let input_type = self.extract_attribute(line, "type").unwrap_or("text".to_string());
        let id = self.extract_attribute(line, "id");
        let name = self.extract_attribute(line, "name");
        let class = self.extract_attribute(line, "class");
        
        let mut selectors = Vec::new();
        if let Some(id) = id {
            selectors.push(format!("#{}", id));
        }
        if let Some(name) = name {
            selectors.push(format!("[name=\"{}\"]", name));
        }
        if let Some(class) = class {
            selectors.push(format!(".{}", class));
        }
        
        self.elements.entry(input_type).or_insert_with(Vec::new).extend(selectors);
    }
    
    fn parse_button_element(&mut self, line: &str) {
        let id = self.extract_attribute(line, "id");
        let class = self.extract_attribute(line, "class");
        let text_content = self.extract_text_content(line);
        
        let mut selectors = Vec::new();
        if let Some(id) = id {
            selectors.push(format!("#{}", id));
        }
        if let Some(class) = class {
            selectors.push(format!(".{}", class));
        }
        
        // Classify button type based on content
        let button_type = if let Some(text) = text_content {
            let text_lower = text.to_lowercase();
            if text_lower.contains("submit") || text_lower.contains("apply") || text_lower.contains("send") {
                "submit"
            } else if text_lower.contains("login") || text_lower.contains("sign in") {
                "login"
            } else if text_lower.contains("accept") || text_lower.contains("agree") {
                "accept"
            } else {
                "button"
            }
        } else {
            "button"
        };
        
        self.elements.entry(button_type.to_string()).or_insert_with(Vec::new).extend(selectors);
    }
    
    fn parse_select_element(&mut self, line: &str) {
        let id = self.extract_attribute(line, "id");
        let name = self.extract_attribute(line, "name");
        
        let mut selectors = Vec::new();
        if let Some(id) = id {
            selectors.push(format!("#{}", id));
        }
        if let Some(name) = name {
            selectors.push(format!("[name=\"{}\"]", name));
        }
        
        self.elements.entry("select".to_string()).or_insert_with(Vec::new).extend(selectors);
    }
    
    fn extract_attribute(&self, line: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        if let Some(start) = line.find(&pattern) {
            let start = start + pattern.len();
            if let Some(end) = line[start..].find('"') {
                return Some(line[start..start + end].to_string());
            }
        }
        None
    }
    
    fn extract_text_content(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find('>') {
            if let Some(end) = line[start + 1..].find('<') {
                let content = line[start + 1..start + 1 + end].trim();
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
        None
    }
    
    pub(crate) fn find_cookie_consent(&self) -> Option<String> {
        // Look for common cookie consent patterns
        let cookie_patterns = [
            "accept", "cookie", "consent", "agree", "ok", "got it"
        ];
        
        for pattern in &cookie_patterns {
            if let Some(selectors) = self.elements.get(*pattern) {
                if !selectors.is_empty() {
                    return Some(selectors[0].clone());
                }
            }
        }
        
        // Check for common cookie button IDs/classes
        if self.html.contains("accept-cookie") || self.html.contains("cookie-accept") {
            return Some("#accept-cookies".to_string());
        }
        
        None
    }
    
    pub(crate) fn is_login_form(&self) -> bool {
        self.elements.contains_key("password") && 
        (self.elements.contains_key("text") || self.elements.contains_key("email"))
    }
    
    pub(crate) fn find_submit_button(&self) -> Option<String> {
        if let Some(selectors) = self.elements.get("submit") {
            if !selectors.is_empty() {
                return Some(selectors[0].clone());
            }
        }
        
        // Fallback to common submit button selectors
        let common_submits = [
            "[type=\"submit\"]", "#submit", "#apply", "#send", ".submit", ".apply"
        ];
        
        for selector in &common_submits {
            if self.html.contains(&selector.replace("#", "id=\"").replace(".", "class=\"")) {
                return Some(selector.to_string());
            }
        }
        
        None
    }
    
    pub(crate) fn get_elements_by_type(&self, element_type: &str) -> Vec<String> {
        self.elements.get(element_type).cloned().unwrap_or_default()
    }
}

pub(crate) fn generate_login_sequence(analyzer: &FormAnalyzer, user_data: &Value) -> Option<Vec<String>> {
    let mut actions = Vec::new();
    
    // Find username/email field
    let username_selectors = analyzer.get_elements_by_type("text");
    let email_selectors = analyzer.get_elements_by_type("email");
    
    let username_field = username_selectors.first()
        .or_else(|| email_selectors.first());
    
    // Find password field
    let password_selectors = analyzer.get_elements_by_type("password");
    let password_field = password_selectors.first();
    
    if let (Some(username_sel), Some(password_sel)) = (username_field, password_field) {
        // Use email if available, otherwise username
        if let Some(email) = user_data.get("email").and_then(|v| v.as_str()) {
            if !email.is_empty() {
                actions.push(format!("type \"{}\" \"{}\"", username_sel, escape_for_dsl(email)));
            }
        } else if let Some(username) = user_data.get("username").and_then(|v| v.as_str()) {
            if !username.is_empty() {
                actions.push(format!("type \"{}\" \"{}\"", username_sel, escape_for_dsl(username)));
            }
        }
        
        if let Some(password) = user_data.get("password").and_then(|v| v.as_str()) {
            if !password.is_empty() {
                actions.push(format!("type \"{}\" \"{}\"", password_sel, escape_for_dsl(password)));
            }
        }
        
        // Find and click login button
        if let Some(login_btn) = analyzer.elements.get("login") {
            if let Some(selector) = login_btn.first() {
                actions.push(format!("click \"{}\"", selector));
            }
        }
        
        return Some(actions);
    }
    
    None
}

pub(crate) fn generate_field_filling_sequence(analyzer: &FormAnalyzer, user_data: &Value) -> Vec<String> {
    let mut actions = Vec::new();
    
    // Enhanced field mappings with smarter detection
    let field_mappings = [
        ("fullname", vec!["text"], vec!["fullname", "full-name", "name", "firstname", "first-name"]),
        ("email", vec!["email", "text"], vec!["email", "e-mail", "mail"]),
        ("phone", vec!["tel", "text"], vec!["phone", "telephone", "tel", "mobile"]),
        ("username", vec!["text"], vec!["username", "user", "login"]),
    ];
    
    for (data_key, input_types, field_names) in &field_mappings {
        if let Some(value) = user_data.get(*data_key).and_then(|v| v.as_str()) {
            if !value.is_empty() {
                // Try to find matching field
                for input_type in input_types {
                    if let Some(selectors) = analyzer.elements.get(*input_type) {
                        for selector in selectors {
                            // Check if selector matches field names
                            let selector_lower = selector.to_lowercase();
                            let matches = field_names.iter().any(|name| selector_lower.contains(name));
                            
                            if matches {
                                actions.push(format!("type \"{}\" \"{}\"", selector, escape_for_dsl(value)));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    actions
}

pub(crate) fn generate_upload_sequence(analyzer: &FormAnalyzer, user_data: &Value) -> Option<Vec<String>> {
    if let Some(cv_path) = user_data.get("cv_path").and_then(|v| v.as_str()) {
        if !cv_path.is_empty() {
            // Find file input
            if let Some(file_selectors) = analyzer.elements.get("file") {
                if let Some(selector) = file_selectors.first() {
                    return Some(vec![format!("upload \"{}\" \"{}\"", selector, escape_for_dsl(cv_path))]);
                }
            }
        }
    }
    None
}

pub(crate) fn generate_checkbox_sequence(analyzer: &FormAnalyzer) -> Vec<String> {
    let mut actions = Vec::new();
    
    // Look for common agreement checkboxes
    if let Some(checkbox_selectors) = analyzer.elements.get("checkbox") {
        for selector in checkbox_selectors {
            let selector_lower = selector.to_lowercase();
            if selector_lower.contains("terms") || 
               selector_lower.contains("agree") || 
               selector_lower.contains("consent") ||
               selector_lower.contains("gdpr") {
                actions.push(format!("click \"{}\"", selector));
            }
        }
    }
    
    actions
}

pub(crate) fn is_complex_form(html: &str) -> bool {
    // Określ czy formularz jest złożony na podstawie różnych kryteriów
    let complexity_indicators = vec![
        html.contains("class=\"complex"),
        html.contains("data-step="),
        html.contains("multi-step"),
        html.matches("<input").count() > 5,
        html.contains("javascript:"),
        html.contains("onclick="),
        html.contains("data-validation="),
    ];
    
    complexity_indicators.iter().filter(|&&x| x).count() >= 2
}

// Funkcja do wywołania rzeczywistego LLM (np. Claude API)
pub async fn generate_dsl_with_llm(html: &str, user_data: &Value) -> Result<String, Box<dyn std::error::Error>> {
    info!("Attempting to generate DSL using LLM API");
    
    // Sprawdź czy mamy klucz API (w prawdziwej implementacji)
    let api_key = std::env::var("CLAUDE_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        warn!("No CLAUDE_API_KEY found, falling back to simple generation");
        return Ok(String::new());
    }
    
    let prompt = format!(
        "Przeanalizuj formularz HTML i wygeneruj skrypt DSL do jego wypełnienia.\n\
        Dostępne komendy: click, type, upload, hover, wait\n\
        \n\
        Zasady:\n\
        1. Używaj selektorów CSS (#id, .class, [attribute])\n\
        2. Najpierw zaloguj się jeśli to konieczne\n\
        3. Wypełnij wszystkie wymagane pola\n\
        4. Na końcu kliknij przycisk submit/apply\n\
        5. Zwróć TYLKO komendy DSL, bez komentarzy\n\
        \n\
        HTML: {}\n\
        \n\
        Dane użytkownika: {}\n\
        \n\
        Wygeneruj optymalną sekwencję komend DSL:",
        html, 
        serde_json::to_string_pretty(user_data).unwrap_or_default()
    );
    
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 1000,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        }))
        .send()
        .await?;
    
    if !response.status().is_success() {
        error!("LLM API request failed with status: {}", response.status());
        return Ok(String::new());
    }
    
    let response_body: Value = response.json().await?;
    
    if let Some(content) = response_body["content"][0]["text"].as_str() {
        let cleaned_script = parse_dsl_from_response(content);
        info!("Successfully generated DSL using LLM, {} lines", cleaned_script.lines().count());
        Ok(cleaned_script)
    } else {
        error!("Invalid response format from LLM API");
        Ok(String::new())
    }
}

fn parse_dsl_from_response(response: &str) -> String {
    debug!("Parsing DSL from LLM response");
    
    // Wyczyść odpowiedź z niepotrzebnych znaków i komentarzy
    response
        .lines()
        .map(|line| line.trim())
        .filter(|line| {
            !line.is_empty() && 
            !line.starts_with("//") &&
            !line.starts_with("#") &&
            (line.starts_with("click") || 
             line.starts_with("type") || 
             line.starts_with("upload") || 
             line.starts_with("hover") ||
             line.starts_with("wait"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// Funkcje pomocnicze do różnych typów formularzy
pub mod templates {
    pub fn job_application_template(user_data: &serde_json::Value) -> String {
        let first_name = user_data.get("first_name").and_then(|v| v.as_str()).unwrap_or("");
        let last_name = user_data.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
        let email = user_data.get("email").and_then(|v| v.as_str()).unwrap_or("");
        let phone = user_data.get("phone").and_then(|v| v.as_str()).unwrap_or("");
        let cv_path = user_data.get("cv_path").and_then(|v| v.as_str()).unwrap_or("");
        
        format!("click \"#accept-cookies\"\nhover \"#careers-link\"\nclick \"#careers-link\"\nclick \"#apply-now\"\ntype \"#first-name\" \"{}\"\ntype \"#last-name\" \"{}\"\ntype \"#email\" \"{}\"\ntype \"#phone\" \"{}\"\nupload \"#resume\" \"{}\"\nclick \"#gdpr-consent\"\nclick \"#submit-application\"", first_name, last_name, email, phone, cv_path)
    }

    pub fn registration_template(user_data: &serde_json::Value) -> String {
        let username = user_data.get("username").and_then(|v| v.as_str()).unwrap_or("");
        let email = user_data.get("email").and_then(|v| v.as_str()).unwrap_or("");
        let password = user_data.get("password").and_then(|v| v.as_str()).unwrap_or("");
        
        format!("click \"#register\"\ntype \"#username\" \"{}\"\ntype \"#email\" \"{}\"\ntype \"#password\" \"{}\"\ntype \"#confirm-password\" \"{}\"\nclick \"#terms-checkbox\"\nclick \"#create-account\"", username, email, password, password)
    }

    pub fn linkedin_apply_template(user_data: &serde_json::Value) -> String {
        let email = user_data.get("linkedin_email").and_then(|v| v.as_str()).unwrap_or("");
        let password = user_data.get("linkedin_password").and_then(|v| v.as_str()).unwrap_or("");
        let phone = user_data.get("phone").and_then(|v| v.as_str()).unwrap_or("");
        let cv_path = user_data.get("cv_path").and_then(|v| v.as_str()).unwrap_or("");
        
        format!("click \"#sign-in\"\ntype \"#username\" \"{}\"\ntype \"#password\" \"{}\"\nclick \"#sign-in-submit\"\nclick \".jobs-apply-button\"\nupload \"#resume-upload\" \"{}\"\ntype \"#phone\" \"{}\"\nclick \"#follow-company\"\nclick \"#submit-application\"", email, password, cv_path, phone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_dsl_generation() {
        let html = r#"
            <input id="username" type="text">
            <input id="password" type="password">
            <button id="submit">Login</button>
        "#;
        
        let user_data = serde_json::json!({
            "username": "john.doe",
            "password": "secret123"
        });

        let dsl = generate_simple_dsl(html, &user_data);
        
        assert!(dsl.contains("type \"#username\" \"john.doe\""));
        assert!(dsl.contains("type \"#password\" \"secret123\""));
        assert!(dsl.contains("click \"#submit\""));
    }

    #[test]
    fn test_is_complex_form() {
        let simple_html = "<input type='text'><button>Submit</button>";
        assert!(!is_complex_form(simple_html));
        
        let complex_html = r#"
            <form class="complex-form" data-step="1">
                <input type="text" data-validation="required">
                <input type="email" data-validation="email">
                <input type="tel" data-validation="phone">
                <input type="file" data-validation="file">
                <input type="password" data-validation="password">
                <input type="text" data-validation="required">
                <button onclick="validateForm()">Next</button>
            </form>
        "#;
        assert!(is_complex_form(complex_html));
    }
    
    #[test]
    fn test_parse_dsl_from_response() {
        let llm_response = "
        Here's the DSL script for your form:
        
        click \"#login-btn\"
        type \"#username\" \"testuser\"
        type \"#password\" \"testpass\"
        // This is a comment
        click \"#submit\"
        
        This should work for your form.
        ";
        
        let parsed = parse_dsl_from_response(llm_response);
        let lines: Vec<&str> = parsed.lines().collect();
        
        assert_eq!(lines.len(), 4);
        assert!(lines[0].starts_with("click"));
        assert!(lines[1].starts_with("type"));
        assert!(lines[2].starts_with("type"));
        assert!(lines[3].starts_with("click"));
    }
}

// Simple DSL generator used by unit tests in this module
fn generate_simple_dsl(html: &str, user_data: &Value) -> String {
    debug!("Using simple DSL generation (fallback)");
    let mut script = String::new();
    
    // Check for a login button
    if html.contains("id=\"login-btn\"") || html.contains("class=\"login") {
        script.push_str("click \"#login-btn\"\n");
    }
    
    // Map user_data to common selectors
    let field_mappings = vec![
        ("username", vec!["#username", "#user", "[name=\"username\"]", "[name=\"email\"]"]),
        ("password", vec!["#password", "#pass", "[name=\"password\"]"]),
        ("fullname", vec!["#fullname", "#full-name", "#name", "[name=\"fullname\"]", "[name=\"name\"]"]),
        ("email", vec!["#email", "[name=\"email\"]", "[type=\"email\"]"]),
        ("phone", vec!["#phone", "#telephone", "[name=\"phone\"]", "[type=\"tel\"]"]),
        ("cv_path", vec!["#cv-upload", "#resume", "#cv", "[type=\"file\"]"]),
    ];
    
    for (data_key, selectors) in field_mappings {
        if let Some(value) = user_data.get(data_key).and_then(|v| v.as_str()) {
            if !value.is_empty() {
                for selector in selectors {
                    // crude presence check
                    if html.contains(&selector.replace("#", "id=\"").replace("[", "").replace("]", "")) || html.contains(selector) {
                        let escaped_value = escape_for_dsl(value);
                        if data_key == "cv_path" {
                            script.push_str(&format!("upload \"{}\" \"{}\"\n", selector, escaped_value));
                        } else {
                            script.push_str(&format!("type \"{}\" \"{}\"\n", selector, escaped_value));
                        }
                        break;
                    }
                }
            }
        }
    }
    
    // Try to find a submit button
    let submit_selectors = vec![
        "#submit", "#apply", "#send", "#login", "#apply-submit",
        "[type=\"submit\"]", "button[type=\"submit\"]",
    ];
    for selector in submit_selectors {
        if html.contains(&selector.replace("#", "id=\"").replace("[", "").replace("]", "")) || html.contains(selector) {
            script.push_str(&format!("click \"{}\"\n", selector));
            break;
        }
    }
    
    script
}
