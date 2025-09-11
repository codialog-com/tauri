use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use redis::AsyncCommands;
use anyhow::{Result, Context};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: String,
    pub user_id: String,
    pub bitwarden_session: Option<String>,
    pub user_data: UserData,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub cv_path: Option<String>,
    pub cover_letter_path: Option<String>,
    pub preferences: HashMap<String, serde_json::Value>,
    pub form_data: HashMap<String, serde_json::Value>,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            first_name: None,
            last_name: None,
            email: None,
            phone: None,
            address: None,
            cv_path: None,
            cover_letter_path: None,
            preferences: HashMap::new(),
            form_data: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    db_pool: PgPool,
    redis_client: redis::Client,
}

impl SessionManager {
    pub fn new(db_pool: PgPool, redis_client: redis::Client) -> Self {
        Self {
            db_pool,
            redis_client,
        }
    }

    /// Inicjalizuje strukturę bazy danych dla sesji
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing session management database tables");

        // Tabela dla sesji użytkowników
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_sessions (
                session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id VARCHAR(255) NOT NULL,
                bitwarden_session TEXT,
                user_data JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ NOT NULL,
                last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(user_id)
            );

            CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);
            "#,
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to create user_sessions table")?;

        // Tabela dla przesłanych plików
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_files (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                session_id UUID NOT NULL REFERENCES user_sessions(session_id) ON DELETE CASCADE,
                file_type VARCHAR(50) NOT NULL, -- 'cv', 'cover_letter', 'attachment'
                original_filename VARCHAR(500) NOT NULL,
                stored_filename VARCHAR(500) NOT NULL,
                file_path VARCHAR(1000) NOT NULL,
                file_size BIGINT NOT NULL,
                mime_type VARCHAR(100),
                uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                is_active BOOLEAN NOT NULL DEFAULT TRUE
            );

            CREATE INDEX IF NOT EXISTS idx_user_files_session_id ON user_files(session_id);
            CREATE INDEX IF NOT EXISTS idx_user_files_type ON user_files(file_type);
            "#,
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to create user_files table")?;

        // Tabela dla form data cache
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS form_data_cache (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                session_id UUID NOT NULL REFERENCES user_sessions(session_id) ON DELETE CASCADE,
                url_pattern VARCHAR(500) NOT NULL,
                form_data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_form_data_session_id ON form_data_cache(session_id);
            CREATE INDEX IF NOT EXISTS idx_form_data_url ON form_data_cache(url_pattern);
            "#,
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to create form_data_cache table")?;

        info!("Session management tables initialized successfully");
        Ok(())
    }

    /// Tworzy nową sesję użytkownika
    pub async fn create_session(&self, user_id: &str, user_data: UserData) -> Result<UserSession> {
        info!("Creating new session for user: {}", user_id);

        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::hours(24); // Sesja wygasa po 24 godzinach

        let session = UserSession {
            session_id: session_id.clone(),
            user_id: user_id.to_string(),
            bitwarden_session: None,
            user_data,
            created_at: now,
            expires_at,
            last_activity: now,
        };

        // Zapisz sesję w bazie danych
        sqlx::query(
            r#"
            INSERT INTO user_sessions (session_id, user_id, user_data, expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE SET
                session_id = EXCLUDED.session_id,
                user_data = EXCLUDED.user_data,
                expires_at = EXCLUDED.expires_at,
                last_activity = NOW()
            "#,
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(serde_json::to_value(&session.user_data)?)
        .bind(&expires_at)
        .execute(&self.db_pool)
        .await
        .context("Failed to create session in database")?;

        // Cache w Redis dla szybkiego dostępu
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        let session_json = serde_json::to_string(&session)?;
        redis::cmd("SETEX")
            .arg(&format!("session:{}", session_id))
            .arg(86400)
            .arg(session_json)
            .query_async(&mut redis_conn)
            .await?;

        info!("Session created successfully: {}", session_id);
        Ok(session)
    }

    /// Pobiera sesję po ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<UserSession>> {
        debug!("Retrieving session: {}", session_id);

        // Najpierw sprawdź Redis cache
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        
        if let Ok(cached_session) = redis_conn
            .get::<&str, String>(&format!("session:{}", session_id))
            .await
        {
            if let Ok(session) = serde_json::from_str::<UserSession>(&cached_session) {
                if session.expires_at > Utc::now() {
                    debug!("Session found in Redis cache: {}", session_id);
                    return Ok(Some(session));
                }
            }
        }

        // Jeśli nie ma w cache, sprawdź bazę danych
        let row = sqlx::query(
            r#"
            SELECT session_id, user_id, bitwarden_session, user_data, 
                   created_at, expires_at, last_activity
            FROM user_sessions 
            WHERE session_id = $1 AND expires_at > NOW()
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to fetch session from database")?;

        if let Some(row) = row {
            let user_data: UserData = serde_json::from_value(row.get("user_data"))?;
            
            let session = UserSession {
                session_id: row.get("session_id"),
                user_id: row.get("user_id"),
                bitwarden_session: row.get("bitwarden_session"),
                user_data,
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
                last_activity: row.get("last_activity"),
            };

            // Odśwież cache w Redis
            let session_json = serde_json::to_string(&session)?;
            redis::cmd("SETEX")
                .arg(&format!("session:{}", session_id))
                .arg(86400)
                .arg(session_json)
                .query_async(&mut redis_conn)
                .await?;

            debug!("Session found in database and cached: {}", session_id);
            Ok(Some(session))
        } else {
            debug!("Session not found: {}", session_id);
            Ok(None)
        }
    }

    /// Aktualizuje dane sesji
    pub async fn update_session(&self, session: &UserSession) -> Result<()> {
        debug!("Updating session: {}", session.session_id);

        // Aktualizuj w bazie danych
        sqlx::query(
            r#"
            UPDATE user_sessions 
            SET bitwarden_session = $1, user_data = $2, last_activity = NOW()
            WHERE session_id = $3
            "#,
        )
        .bind(&session.bitwarden_session)
        .bind(serde_json::to_value(&session.user_data)?)
        .bind(&session.session_id)
        .execute(&self.db_pool)
        .await
        .context("Failed to update session in database")?;

        // Aktualizuj cache w Redis
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        let session_json = serde_json::to_string(session)?;
        redis::cmd("SETEX")
            .arg(&format!("session:{}", session.session_id))
            .arg(86400)
            .arg(session_json)
            .query_async(&mut redis_conn)
            .await?;

        debug!("Session updated successfully: {}", session.session_id);
        Ok(())
    }

    /// Usuwa wygasłe sesje
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        info!("Cleaning up expired sessions");

        // Usuń z bazy danych
        let result = sqlx::query("DELETE FROM user_sessions WHERE expires_at < NOW()")
            .execute(&self.db_pool)
            .await
            .context("Failed to delete expired sessions")?;

        let deleted_count = result.rows_affected();
        
        if deleted_count > 0 {
            info!("Cleaned up {} expired sessions", deleted_count);
        }

        Ok(deleted_count)
    }

    /// Zapisuje plik dla sesji
    pub async fn save_file(
        &self,
        session_id: &str,
        file_type: &str,
        original_filename: &str,
        stored_filename: &str,
        file_path: &str,
        file_size: i64,
        mime_type: Option<&str>,
    ) -> Result<String> {
        debug!("Saving file for session {}: {}", session_id, original_filename);

        let file_id = Uuid::new_v4().to_string();

        // Zapisz informacje o pliku w bazie
        sqlx::query(
            r#"
            INSERT INTO user_files 
            (id, session_id, file_type, original_filename, stored_filename, 
             file_path, file_size, mime_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&file_id)
        .bind(session_id)
        .bind(file_type)
        .bind(original_filename)
        .bind(stored_filename)
        .bind(file_path)
        .bind(file_size)
        .bind(mime_type)
        .execute(&self.db_pool)
        .await
        .context("Failed to save file information")?;

        info!("File saved successfully: {} ({})", original_filename, file_id);
        Ok(file_id)
    }

    /// Pobiera pliki dla sesji
    pub async fn get_session_files(&self, session_id: &str) -> Result<Vec<serde_json::Value>> {
        debug!("Retrieving files for session: {}", session_id);

        let rows = sqlx::query(
            r#"
            SELECT id, file_type, original_filename, stored_filename, 
                   file_path, file_size, mime_type, uploaded_at
            FROM user_files 
            WHERE session_id = $1 AND is_active = true
            ORDER BY uploaded_at DESC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to fetch session files")?;

        let files: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<String, _>("id"),
                    "file_type": row.get::<String, _>("file_type"),
                    "original_filename": row.get::<String, _>("original_filename"),
                    "stored_filename": row.get::<String, _>("stored_filename"),
                    "file_path": row.get::<String, _>("file_path"),
                    "file_size": row.get::<i64, _>("file_size"),
                    "mime_type": row.get::<Option<String>, _>("mime_type"),
                    "uploaded_at": row.get::<DateTime<Utc>, _>("uploaded_at")
                })
            })
            .collect();

        debug!("Retrieved {} files for session: {}", files.len(), session_id);
        Ok(files)
    }

    /// Zapisuje dane formularza dla konkretnej strony
    pub async fn save_form_data(
        &self,
        session_id: &str,
        url_pattern: &str,
        form_data: &serde_json::Value,
    ) -> Result<()> {
        debug!("Saving form data for session {} at URL: {}", session_id, url_pattern);

        sqlx::query(
            r#"
            INSERT INTO form_data_cache (session_id, url_pattern, form_data)
            VALUES ($1, $2, $3)
            ON CONFLICT (session_id, url_pattern) DO UPDATE SET
                form_data = EXCLUDED.form_data,
                updated_at = NOW()
            "#,
        )
        .bind(session_id)
        .bind(url_pattern)
        .bind(form_data)
        .execute(&self.db_pool)
        .await
        .context("Failed to save form data")?;

        debug!("Form data saved successfully for session: {}", session_id);
        Ok(())
    }

    /// Pobiera zapisane dane formularza
    pub async fn get_form_data(
        &self,
        session_id: &str,
        url_pattern: &str,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Retrieving form data for session {} at URL: {}", session_id, url_pattern);

        let row = sqlx::query(
            r#"
            SELECT form_data FROM form_data_cache 
            WHERE session_id = $1 AND url_pattern = $2
            "#,
        )
        .bind(session_id)
        .bind(url_pattern)
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to fetch form data")?;

        if let Some(row) = row {
            let form_data: serde_json::Value = row.get("form_data");
            debug!("Found cached form data for session: {}", session_id);
            Ok(Some(form_data))
        } else {
            debug!("No cached form data found for session: {}", session_id);
            Ok(None)
        }
    }
}
