use chromiumoxide::Browser;
use futures::StreamExt;
use tracing::{info, error, debug};

pub async fn get_page_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching HTML content from URL: {}", url);
    
    if url.is_empty() {
        return Err("URL cannot be empty".into());
    }
    
    let (mut browser, mut handler) = Browser::launch(
        chromiumoxide::BrowserConfig::builder()
            .build()?
    ).await?;
    
    let handle = tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });
    
    let page = browser.new_page(url).await?;
    
    // Poczekaj na załadowanie strony
    page.wait_for_navigation().await?;
    
    // Pobierz HTML content
    let html = page.content().await?;
    
    debug!("Retrieved HTML content, length: {} characters", html.len());
    
    browser.close().await?;
    handle.abort();
    
    Ok(html)
}

pub async fn extract_form_elements(html: &str) -> Vec<FormElement> {
    debug!("Extracting form elements from HTML");
    
    let mut elements = Vec::new();
    
    // Proste parsowanie HTML - w produkcji użyj scraper lub html5ever
    if html.contains("type=\"text\"") || html.contains("type='text'") {
        elements.push(FormElement {
            tag: "input".to_string(),
            element_type: Some("text".to_string()),
            id: extract_id_from_input(html, "text"),
            name: extract_name_from_input(html, "text"),
            selector: generate_selector(html, "text"),
        });
    }
    
    if html.contains("type=\"email\"") || html.contains("type='email'") {
        elements.push(FormElement {
            tag: "input".to_string(),
            element_type: Some("email".to_string()),
            id: extract_id_from_input(html, "email"),
            name: extract_name_from_input(html, "email"),
            selector: generate_selector(html, "email"),
        });
    }
    
    if html.contains("type=\"password\"") || html.contains("type='password'") {
        elements.push(FormElement {
            tag: "input".to_string(),
            element_type: Some("password".to_string()),
            id: extract_id_from_input(html, "password"),
            name: extract_name_from_input(html, "password"),
            selector: generate_selector(html, "password"),
        });
    }
    
    if html.contains("type=\"file\"") || html.contains("type='file'") {
        elements.push(FormElement {
            tag: "input".to_string(),
            element_type: Some("file".to_string()),
            id: extract_id_from_input(html, "file"),
            name: extract_name_from_input(html, "file"),
            selector: generate_selector(html, "file"),
        });
    }
    
    if html.contains("<button") || html.contains("type=\"submit\"") {
        elements.push(FormElement {
            tag: "button".to_string(),
            element_type: Some("submit".to_string()),
            id: extract_button_id(html),
            name: None,
            selector: generate_button_selector(html),
        });
    }
    
    debug!("Found {} form elements", elements.len());
    elements
}

#[derive(Debug, Clone)]
pub struct FormElement {
    pub tag: String,
    pub element_type: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub selector: String,
}

fn extract_id_from_input(html: &str, input_type: &str) -> Option<String> {
    // Bardzo prosty parser - w produkcji użyj regex lub właściwego parsera HTML
    let pattern = format!("type=\"{}\"", input_type);
    if let Some(pos) = html.find(&pattern) {
        let before = &html[..pos];
        if let Some(id_pos) = before.rfind("id=\"") {
            let id_start = id_pos + 4;
            if let Some(id_end) = html[id_start..].find('"') {
                return Some(html[id_start..id_start + id_end].to_string());
            }
        }
    }
    None
}

fn extract_name_from_input(html: &str, input_type: &str) -> Option<String> {
    let pattern = format!("type=\"{}\"", input_type);
    if let Some(pos) = html.find(&pattern) {
        let before = &html[..pos];
        if let Some(name_pos) = before.rfind("name=\"") {
            let name_start = name_pos + 6;
            if let Some(name_end) = html[name_start..].find('"') {
                return Some(html[name_start..name_start + name_end].to_string());
            }
        }
    }
    None
}

fn generate_selector(html: &str, input_type: &str) -> String {
    if let Some(id) = extract_id_from_input(html, input_type) {
        return format!("#{}", id);
    }
    
    if let Some(name) = extract_name_from_input(html, input_type) {
        return format!("[name=\"{}\"]", name);
    }
    
    format!("input[type=\"{}\"]", input_type)
}

fn extract_button_id(html: &str) -> Option<String> {
    // Znajdź pierwszy button lub input[type=submit]
    if let Some(pos) = html.find("<button") {
        let button_section = &html[pos..];
        if let Some(id_pos) = button_section.find("id=\"") {
            let id_start = id_pos + 4;
            if let Some(id_end) = button_section[id_start..].find('"') {
                return Some(button_section[id_start..id_start + id_end].to_string());
            }
        }
    }
    None
}

fn generate_button_selector(html: &str) -> String {
    if let Some(id) = extract_button_id(html) {
        return format!("#{}", id);
    }
    
    // Fallback selectors
    if html.contains("type=\"submit\"") {
        return "input[type=\"submit\"]".to_string();
    }
    
    "button".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_form_elements() {
        let html = r#"
            <form>
                <input id="username" name="user" type="text">
                <input id="email" type="email">
                <input id="password" type="password">
                <input id="cv-upload" type="file">
                <button id="submit" type="submit">Submit</button>
            </form>
        "#;

        let elements = extract_form_elements(html).await;
        assert_eq!(elements.len(), 5);
        
        // Test text input
        let text_input = &elements[0];
        assert_eq!(text_input.tag, "input");
        assert_eq!(text_input.element_type, Some("text".to_string()));
        assert_eq!(text_input.id, Some("username".to_string()));
    }

    #[test]
    fn test_generate_selector() {
        let html_with_id = r#"<input id="test" type="text">"#;
        assert_eq!(generate_selector(html_with_id, "text"), "#test");
        
        let html_with_name = r#"<input name="test" type="text">"#;
        assert_eq!(generate_selector(html_with_name, "text"), "[name=\"test\"]");
        
        let html_basic = r#"<input type="text">"#;
        assert_eq!(generate_selector(html_basic, "text"), "input[type=\"text\"]");
    }
}
