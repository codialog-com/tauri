pub mod test_bitwarden;
pub mod test_llm;
pub mod test_session;
pub mod test_logging;
pub mod test_database;
pub mod integration_tests;

#[cfg(test)]
mod common {
    use serde_json::json;
    use std::collections::HashMap;
    
    pub fn create_test_user_data() -> serde_json::Value {
        json!({
            "email": "test@example.com",
            "fullname": "Test User",
            "phone": "+1234567890",
            "username": "testuser",
            "password": "testpassword123",
            "cv_path": "/tmp/test_cv.pdf"
        })
    }
    
    pub fn create_test_html_form() -> String {
        r#"
        <html>
        <body>
            <form id="test-form">
                <input type="email" name="email" id="email-input" placeholder="Enter email">
                <input type="text" name="fullname" id="name-input" placeholder="Full name">
                <input type="tel" name="phone" id="phone-input" placeholder="Phone number">
                <input type="password" name="password" id="password-input" placeholder="Password">
                <input type="file" name="cv" id="cv-upload" accept=".pdf,.doc,.docx">
                <input type="checkbox" name="terms" id="terms-checkbox">
                <button type="submit" id="submit-btn">Submit Application</button>
            </form>
            <div class="cookie-banner">
                <button id="accept-cookies">Accept Cookies</button>
            </div>
        </body>
        </html>
        "#.to_string()
    }
    
    pub fn create_complex_html_form() -> String {
        r#"
        <html>
        <body>
            <form class="complex-form" data-step="1">
                <div class="step-1">
                    <input type="email" name="work_email" id="work-email" required>
                    <input type="text" name="first_name" id="first-name" required>
                    <input type="text" name="last_name" id="last-name" required>
                </div>
                <div class="step-2" style="display: none;">
                    <select name="experience" id="experience-select">
                        <option value="0-1">0-1 years</option>
                        <option value="2-5">2-5 years</option>
                        <option value="5+">5+ years</option>
                    </select>
                    <textarea name="motivation" id="motivation-text" rows="5"></textarea>
                </div>
                <button type="button" onclick="nextStep()" class="next-btn">Next Step</button>
                <button type="submit" id="final-submit" style="display: none;">Submit Application</button>
            </form>
            <script>
                function nextStep() { /* complex JS logic */ }
            </script>
        </body>
        </html>
        "#.to_string()
    }
    
    pub async fn setup_test_database() -> sqlx::PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test_user:test_pass@localhost:5433/test_codialog".to_string());
        
        sqlx::PgPool::connect(&database_url).await.expect("Failed to connect to test database")
    }
    
    pub fn create_test_credentials() -> Vec<crate::bitwarden::BitwardenCredential> {
        vec![
            crate::bitwarden::BitwardenCredential {
                id: "test-id-1".to_string(),
                name: "Test Login 1".to_string(),
                username: Some("testuser1@example.com".to_string()),
                password: Some("password123".to_string()),
                uri: Some("https://example.com".to_string()),
                notes: Some("Test credential 1".to_string()),
            },
            crate::bitwarden::BitwardenCredential {
                id: "test-id-2".to_string(),
                name: "Test Login 2".to_string(),
                username: Some("testuser2@example.com".to_string()),
                password: Some("password456".to_string()),
                uri: Some("https://test.com".to_string()),
                notes: Some("Test credential 2".to_string()),
            },
        ]
    }
}
