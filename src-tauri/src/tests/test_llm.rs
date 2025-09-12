#![cfg(test)]

use super::*;
use pretty_assertions::assert_eq;
use crate::{
    llm::{
        generate_dsl_script,
        validate_dsl_script,
        process_natural_language_query,
        get_llm_response,
        LLMRequest,
        LLMResponse,
        LLMError,
    },
    database::setup_test_database,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    
    // Helper function to create a test LLM request
    fn create_test_llm_request() -> LLMRequest {
        LLMRequest {
            prompt: "Test prompt".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            ..Default::default()
        }
    }

    // Mock FormAnalyzer for testing
    struct MockFormAnalyzer {
        is_login: bool,
        has_file_input: bool,
    }

impl MockFormAnalyzer {
    fn new() -> Self {
        Self {
            is_login: true,
            has_file_input: false,
        }
    }
    
    fn with_login(mut self, is_login: bool) -> Self {
        self.is_login = is_login;
        self
    }
    
    fn with_file_input(mut self, has_file: bool) -> Self {
        self.has_file_input = has_file;
        self
    }
}

impl FormAnalyzerTrait for MockFormAnalyzer {
    fn is_login_form(&self) -> bool {
        self.is_login
    }
    
    fn has_file_input(&self) -> bool {
        self.has_file_input
    }
    
    fn get_elements_by_type(&self, _: &str) -> Vec<String> {
        vec![]
    }
    
    fn find_submit_button(&self) -> Option<String> {
        Some("#submit".to_string())
    }
    
    fn find_cookie_consent(&self) -> Option<String> {
        None
    }
}

    #[tokio::test]
    async fn test_generate_dsl_script_basic() {
        let html = create_test_html_form();
        let user_data = create_test_user_data();
        
        let script = generate_dsl_script(&html, &user_data).await;
        
        assert!(!script.is_empty(), "Generated script should not be empty");
        assert!(script.contains("type"), "Script should contain type commands");
        assert!(script.contains("click"), "Script should contain click commands");
        assert!(script.contains("test@example.com"), "Script should contain user email");
    }

    #[tokio::test]
    async fn test_generate_dsl_script_with_cache() {
        let pool = setup_test_database().await;
        let html = create_test_html_form();
        let user_data = create_test_user_data();
        
        // First call - should generate and cache
        let script1 = generate_dsl_script_with_cache(&html, &user_data, Some(&pool)).await;
        
        // Second call - should retrieve from cache
        let script2 = generate_dsl_script_with_cache(&html, &user_data, Some(&pool)).await;
        
        assert_eq!(script1, script2, "Cached script should match original");
        assert!(!script1.is_empty(), "Generated script should not be empty");
    }

    #[test]
    fn test_create_cache_key() {
        let html = "<form><input type='email' name='email'></form>";
        let user_data = json!({"email": "test@example.com"});
        
        let key1 = create_cache_key(&html, &user_data).unwrap();
        let key2 = create_cache_key(&html, &user_data).unwrap();
        
        assert_eq!(key1, key2, "Cache keys should be deterministic");
        assert!(key1.starts_with("dsl_"), "Cache key should have proper prefix");
    }

    #[test]
    fn test_is_complex_form() {
        let simple_html = create_test_html_form();
        let complex_html = create_complex_html_form();
        
        assert!(!is_complex_form(&simple_html), "Simple form should not be marked as complex");
        assert!(is_complex_form(&complex_html), "Complex form should be marked as complex");
    }

    #[test]
    fn test_form_analyzer_basic() {
        let html = create_test_html_form();
        let analyzer = FormAnalyzer::new(&html);
        
        assert!(analyzer.is_login_form(), "Form with email and password should be detected as login form");
        
        let submit_button = analyzer.find_submit_button();
        assert!(submit_button.is_some(), "Submit button should be found");
        
        let cookie_consent = analyzer.find_cookie_consent();
        assert!(cookie_consent.is_some(), "Cookie consent should be found");
    }

    #[test]
    fn test_form_analyzer_elements() {
        let html = create_test_html_form();
        let analyzer = FormAnalyzer::new(&html);
        
        let email_fields = analyzer.get_elements_by_type("email");
        assert!(!email_fields.is_empty(), "Should find email input fields");
        
        let text_fields = analyzer.get_elements_by_type("text");
        assert!(!text_fields.is_empty(), "Should find text input fields");
        
        let password_fields = analyzer.get_elements_by_type("password");
        assert!(!password_fields.is_empty(), "Should find password input fields");
        
        let file_fields = analyzer.get_elements_by_type("file");
        assert!(!file_fields.is_empty(), "Should find file input fields");
    }

    #[test]
    fn test_generate_login_sequence() {
        let html = create_test_html_form();
        let analyzer = FormAnalyzer::new(&html);
        let user_data = create_test_user_data();
        
        let login_sequence = generate_login_sequence(&analyzer, &user_data);
        assert!(login_sequence.is_some(), "Should generate login sequence for login form");
        
        let actions = login_sequence.unwrap();
        assert!(!actions.is_empty(), "Login sequence should not be empty");
        assert!(actions.iter().any(|action| action.contains("test@example.com")), 
               "Should include email from user data");
        assert!(actions.iter().any(|action| action.contains("testpassword123")), 
               "Should include password from user data");
    }

    #[test]
    fn test_generate_field_filling_sequence() {
        let html = create_test_html_form();
        let analyzer = FormAnalyzer::new(&html);
        let user_data = create_test_user_data();
        
        let field_sequence = generate_field_filling_sequence(&analyzer, &user_data);
        assert!(!field_sequence.is_empty(), "Should generate field filling sequence");
        
        let has_email = field_sequence.iter().any(|action| action.contains("test@example.com"));
        let has_name = field_sequence.iter().any(|action| action.contains("Test User"));
        let has_phone = field_sequence.iter().any(|action| action.contains("+1234567890"));
        
        assert!(has_email || has_name || has_phone, "Should fill at least one field");
    }

    #[test]
    fn test_generate_upload_sequence() {
        let html = create_test_html_form();
        let analyzer = FormAnalyzer::new(&html);
        let user_data = create_test_user_data();
        
        let upload_sequence = generate_upload_sequence(&analyzer, &user_data);
        assert!(upload_sequence.is_some(), "Should generate upload sequence for form with file input");
        
        let actions = upload_sequence.unwrap();
        assert!(!actions.is_empty(), "Upload sequence should not be empty");
        assert!(actions[0].contains("upload"), "Should contain upload command");
        assert!(actions[0].contains("/tmp/test_cv.pdf"), "Should contain CV path");
    }

    #[test]
    fn test_generate_checkbox_sequence() {
        let html = create_test_html_form();
        let analyzer = FormAnalyzer::new(&html);
        
        let checkbox_sequence = generate_checkbox_sequence(&analyzer);
        assert!(!checkbox_sequence.is_empty(), "Should generate checkbox sequence for form with checkboxes");
        
        assert!(checkbox_sequence.iter().any(|action| action.contains("click")), 
               "Should contain click commands for checkboxes");
    }

    #[test]
    fn test_validate_generated_script() {
        let valid_script = r##"
            wait 2
            type "#email" "test@example.com"
            type "#password" "password123"
            click "#submit"
            wait 3
        "##;
        
        let invalid_script_quotes = r##"
            wait 2
            type "#email" "unclosed quote
            click "#submit"
        "##;
        
        let empty_script = "";
        
        assert!(validate_generated_script(valid_script), 
            "Valid script should pass validation");
        assert!(!validate_generated_script(invalid_script_quotes), 
            "Script with unbalanced quotes should fail");
        assert!(!validate_generated_script(empty_script), 
            "Empty script should fail validation");
    }

    #[test]
    fn test_generate_basic_fallback_script() {
        let html = create_test_html_form();
        let user_data = create_test_user_data();
        
        let script = generate_basic_fallback_script(&html, &user_data);
        
        assert!(!script.is_empty(), 
            "Fallback script should not be empty");
        assert!(script.contains("wait"), 
            "Fallback script should contain wait commands");
        assert!(script.contains("// Basic fallback"), 
            "Should contain fallback comment");
    }

    #[test]
    fn test_generate_emergency_fallback_script() {
        let html = "";
        let user_data = json!({});
        
        let script = generate_emergency_fallback_script(html, &user_data);
        
        assert!(!script.is_empty(), 
            "Emergency fallback should not be empty");
        assert!(script.contains("// Emergency fallback"), 
            "Should contain emergency fallback comment");
        assert!(script.contains("wait"), 
            "Should contain wait command");
    }

    #[tokio::test]
    async fn test_error_handling_empty_html() {
        let empty_html = "";
        let user_data = create_test_user_data();
        
        let script = generate_dsl_script_with_cache(empty_html, &user_data, None).await;
        
        assert!(!script.is_empty(), 
            "Should return fallback script for empty HTML");
        assert!(script.contains("// Basic navigation"), 
            "Should use basic navigation fallback");
    }

    #[tokio::test]
    async fn test_error_handling_invalid_user_data() {
        let html = create_test_html_form();
        let invalid_user_data = json!("invalid_data");
        
        let script = generate_dsl_script_with_cache(&html, &invalid_user_data, None).await;
        
        assert!(!script.is_empty(), 
            "Should handle invalid user data gracefully");
    }

    #[test]
    fn test_escape_for_dsl() {
        use crate::tagui::escape_for_dsl;
        
        let input_with_quotes = "Hello \"World\"";
        let escaped = escape_for_dsl(input_with_quotes);
        
        assert!(!escaped.contains("\""), 
            "Should escape quotes in DSL strings");
    }
}
