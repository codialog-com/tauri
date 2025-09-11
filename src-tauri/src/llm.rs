use serde_json::Value;
use reqwest;
use tracing::{info, error, debug, warn};
use crate::tagui::escape_for_dsl;

pub async fn generate_dsl_script(html: &str, user_data: &Value) -> String {
    info!("Generating DSL script from HTML and user data");
    
    // Sprawdź czy to jest złożony formularz - jeśli tak, użyj LLM
    if is_complex_form(html) {
        if let Ok(llm_script) = generate_dsl_with_llm(html, user_data).await {
            if !llm_script.is_empty() {
                return llm_script;
            }
        }
    }
    
    // Fallback do prostej logiki parsowania HTML
    generate_simple_dsl(html, user_data)
}

fn generate_simple_dsl(html: &str, user_data: &Value) -> String {
    debug!("Using simple DSL generation");
    
    let mut script = String::new();
    
    // Analiza formularza w HTML i mapowanie pól
    
    // Sprawdź czy jest przycisk logowania
    if html.contains("id=\"login-btn\"") || html.contains("class=\"login") {
        script.push_str("click \"#login-btn\"\n");
    }
    
    // Mapowanie danych użytkownika na pola formularza
    let field_mappings = vec![
        ("username", vec!["#username", "#user", "[name=\"username\"]", "[name=\"email\"]"]),
        ("password", vec!["#password", "#pass", "[name=\"password\"]"]),
        ("fullname", vec!["#fullname", "#full-name", "#name", "[name=\"fullname\"]", "[name=\"name\"]"]),
        ("email", vec!["#email", "[name=\"email\"]", "[type=\"email\"]"]),
        ("phone", vec!["#phone", "#telephone", "[name=\"phone\"]", "[type=\"tel\"]"]),
        ("cv_path", vec!["#cv-upload", "#resume", "#cv", "[type=\"file\"]"]),
    ];
    
    for (data_key, selectors) in field_mappings {
        if let Some(value) = user_data.get(data_key) {
            if let Some(value_str) = value.as_str() {
                if !value_str.is_empty() {
                    for selector in selectors {
                        if html.contains(&selector.replace("#", "id=\"").replace("[", "").replace("]", "")) 
                           || html.contains(selector) {
                            
                            let escaped_value = escape_for_dsl(value_str);
                            
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
    }
    
    // Znajdź przycisk submit
    let submit_selectors = vec![
        "#submit", "#apply", "#send", "#login", "#apply-submit", 
        "[type=\"submit\"]", "button[type=\"submit\"]"
    ];
    
    for selector in submit_selectors {
        if html.contains(&selector.replace("#", "id=\"").replace("[", "").replace("]", "")) 
           || html.contains(selector) {
            script.push_str(&format!("click \"{}\"\n", selector));
            break;
        }
    }
    
    debug!("Generated simple DSL script with {} lines", script.lines().count());
    script
}

fn is_complex_form(html: &str) -> bool {
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
