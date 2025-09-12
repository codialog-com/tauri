use serde_json::json;

pub fn create_test_html_form() -> String {
    r##"
    <form>
        <input type="email" name="email" id="email">
        <input type="password" name="password" id="password">
        <input type="text" name="name" id="name">
        <input type="file" name="resume" id="resume">
        <input type="checkbox" name="terms" id="terms">
        <button type="submit">Submit</button>
    </form>
    "##.to_string()
}

pub fn create_complex_html_form() -> String {
    r##"
    <form>
        <div class="form-section">
            <h2>Personal Information</h2>
            <input type="text" name="first_name" placeholder="First Name">
            <input type="text" name="last_name" placeholder="Last Name">
            <input type="email" name="email" placeholder="Email">
            <input type="tel" name="phone" placeholder="Phone">
        </div>
        
        <div class="form-section">
            <h2>Address</h2>
            <input type="text" name="street" placeholder="Street Address">
            <input type="text" name="city" placeholder="City">
            <select name="country">
                <option value="">Select Country</option>
                <option value="us">United States</option>
                <option value="uk">United Kingdom</option>
            </select>
            <input type="text" name="zip" placeholder="ZIP/Postal Code">
        </div>
        
        <div class="form-section">
            <h2>Preferences</h2>
            <div class="checkbox-group">
                <label><input type="checkbox" name="newsletter"> Subscribe to newsletter</label>
                <label><input type="checkbox" name="notifications"> Enable notifications</label>
            </div>
            
            <div class="radio-group">
                <label><input type="radio" name="contact_method" value="email" checked> Email</label>
                <label><input type="radio" name="contact_method" value="phone"> Phone</label>
            </div>
        </div>
        
        <div class="form-actions">
            <button type="button" class="btn-cancel">Cancel</button>
            <button type="submit" class="btn-submit">Submit Application</button>
        </div>
    </form>
    "##.to_string()
}

pub fn create_test_user_data() -> serde_json::Value {
    json!({
        "email": "test@example.com",
        "password": "testpassword123",
        "first_name": "Test",
        "last_name": "User",
        "phone": "+1234567890",
        "address": "123 Test St",
        "city": "Testville",
        "country": "US",
        "zip": "12345",
        "newsletter": true,
        "notifications": false,
        "contact_method": "email"
    })
}

pub async fn setup_test_database() -> sqlx::PgPool {
    use std::env;
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/codialog_test".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create test database pool");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_test_html_form() {
        let form = create_test_html_form();
        assert!(form.contains("<form>"));
        assert!(form.contains("type=\"email\""));
        assert!(form.contains("type=\"password\""));
    }
    
    #[test]
    fn test_create_complex_html_form() {
        let form = create_complex_html_form();
        assert!(form.contains("<form>"));
        assert!(form.contains("Personal Information"));
        assert!(form.contains("Address"));
        assert!(form.contains("Preferences"));
    }
    
    #[test]
    fn test_create_test_user_data() {
        let user_data = create_test_user_data();
        assert_eq!(user_data["email"], "test@example.com");
        assert_eq!(user_data["first_name"], "Test");
        assert_eq!(user_data["last_name"], "User");
    }
}
