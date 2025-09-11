use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::process::Command;
use anyhow::{Result, Context};
use tracing::{info, warn, error, debug};
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitwardenCredential {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub uri: Option<String>,
    pub notes: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginSession {
    pub session_token: String,
    pub user_id: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct BitwardenManager {
    server_url: String,
    cli_server_url: String,
    client: Client,
    session: Option<LoginSession>,
}

impl BitwardenManager {
    pub fn new(server_url: String, cli_server_url: String) -> Self {
        Self {
            server_url,
            cli_server_url,
            client: Client::new(),
            session: None,
        }
    }

    /// Inicjalizuje połączenie z serwerem Bitwarden
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Bitwarden connection to: {}", self.server_url);
        
        // Sprawdź czy serwer Bitwarden jest dostępny
        self.check_server_health().await?;
        
        // Sprawdź czy CLI server jest dostępny
        self.check_cli_server().await?;
        
        info!("Bitwarden manager initialized successfully");
        Ok(())
    }

    /// Sprawdź dostępność serwera Bitwarden
    async fn check_server_health(&self) -> Result<()> {
        let health_url = format!("{}/alive", self.server_url);
        
        let response = timeout(Duration::from_secs(5), 
            self.client.get(&health_url).send()
        ).await
        .context("Timeout while checking Bitwarden server")??;

        if response.status().is_success() {
            info!("Bitwarden server is healthy");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bitwarden server health check failed: {}", response.status()))
        }
    }

    /// Sprawdź dostępność CLI servera
    async fn check_cli_server(&self) -> Result<()> {
        let status_url = format!("{}/status", self.cli_server_url);
        
        match timeout(Duration::from_secs(3), 
            self.client.get(&status_url).send()
        ).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    info!("Bitwarden CLI server is accessible");
                    Ok(())
                } else {
                    warn!("Bitwarden CLI server returned status: {}", response.status());
                    Ok(()) // CLI server może nie być krytyczny
                }
            }
            Ok(Err(e)) => {
                warn!("Failed to connect to Bitwarden CLI server: {}", e);
                Ok(()) // CLI server może nie być krytyczny
            }
            Err(_) => {
                warn!("Timeout connecting to Bitwarden CLI server");
                Ok(()) // CLI server może nie być krytyczny
            }
        }
    }

    /// Zaloguj się do Bitwarden używając master password
    pub async fn login(&mut self, email: &str, master_password: &str) -> Result<()> {
        info!("Attempting login to Bitwarden for user: {}", email);

        // Użyj CLI do zalogowania
        let output = Command::new("bw")
            .args(&["login", email, master_password, "--raw"])
            .output()
            .context("Failed to execute bitwarden CLI login command")?;

        if output.status.success() {
            let session_token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            
            self.session = Some(LoginSession {
                session_token: session_token.clone(),
                user_id: email.to_string(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            });

            info!("Successfully logged into Bitwarden");
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to login to Bitwarden: {}", error_msg);
            Err(anyhow::anyhow!("Bitwarden login failed: {}", error_msg))
        }
    }

    /// Odblokowuje vault używając master password
    pub async fn unlock(&mut self, master_password: &str) -> Result<()> {
        info!("Unlocking Bitwarden vault");

        if let Some(ref session) = self.session {
            let output = Command::new("bw")
                .args(&["unlock", master_password, "--raw"])
                .env("BW_SESSION", &session.session_token)
                .output()
                .context("Failed to execute bitwarden CLI unlock command")?;

            if output.status.success() {
                let session_token = String::from_utf8_lossy(&output.stdout).trim().to_string();
                
                // Aktualizuj token sesji
                if let Some(ref mut session) = self.session {
                    session.session_token = session_token;
                }

                info!("Successfully unlocked Bitwarden vault");
                Ok(())
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                error!("Failed to unlock Bitwarden vault: {}", error_msg);
                Err(anyhow::anyhow!("Bitwarden unlock failed: {}", error_msg))
            }
        } else {
            Err(anyhow::anyhow!("No active Bitwarden session. Please login first."))
        }
    }

    /// Pobierz wszystkie dane logowania z vault
    pub async fn get_all_credentials(&self) -> Result<Vec<BitwardenCredential>> {
        info!("Retrieving all credentials from Bitwarden vault");

        if let Some(ref session) = self.session {
            let output = Command::new("bw")
                .args(&["list", "items", "--session", &session.session_token])
                .output()
                .context("Failed to execute bitwarden CLI list command")?;

            if output.status.success() {
                let json_output = String::from_utf8_lossy(&output.stdout);
                let items: Vec<serde_json::Value> = serde_json::from_str(&json_output)
                    .context("Failed to parse Bitwarden items JSON")?;

                let credentials: Vec<BitwardenCredential> = items
                    .into_iter()
                    .filter_map(|item| {
                        if item["type"] == 1 { // Type 1 = login item
                            Some(BitwardenCredential {
                                id: item["id"].as_str().unwrap_or("").to_string(),
                                name: item["name"].as_str().unwrap_or("").to_string(),
                                username: item["login"]["username"].as_str().map(|s| s.to_string()),
                                password: item["login"]["password"].as_str().map(|s| s.to_string()),
                                uri: item["login"]["uris"][0]["uri"].as_str().map(|s| s.to_string()),
                                notes: item["notes"].as_str().map(|s| s.to_string()),
                                folder_id: item["folderId"].as_str().map(|s| s.to_string()),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                info!("Retrieved {} credentials from Bitwarden", credentials.len());
                Ok(credentials)
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                error!("Failed to retrieve credentials: {}", error_msg);
                Err(anyhow::anyhow!("Failed to retrieve Bitwarden credentials: {}", error_msg))
            }
        } else {
            Err(anyhow::anyhow!("No active Bitwarden session. Please login first."))
        }
    }

    /// Pobierz dane logowania dla konkretnej strony/domeny
    pub async fn get_credentials_for_url(&self, url: &str) -> Result<Vec<BitwardenCredential>> {
        info!("Searching for credentials matching URL: {}", url);

        let all_credentials = self.get_all_credentials().await?;
        
        let matching_credentials: Vec<BitwardenCredential> = all_credentials
            .into_iter()
            .filter(|cred| {
                if let Some(ref uri) = cred.uri {
                    uri.contains(url) || url.contains(uri)
                } else {
                    false
                }
            })
            .collect();

        info!("Found {} matching credentials for URL: {}", matching_credentials.len(), url);
        Ok(matching_credentials)
    }

    /// Dodaj nowe dane logowania do vault
    pub async fn add_credential(&self, credential: &BitwardenCredential) -> Result<String> {
        info!("Adding new credential to Bitwarden vault: {}", credential.name);

        if let Some(ref session) = self.session {
            // Utwórz obiekt JSON dla nowego elementu
            let item = serde_json::json!({
                "type": 1,
                "name": credential.name,
                "login": {
                    "username": credential.username,
                    "password": credential.password,
                    "uris": [{"uri": credential.uri}]
                },
                "notes": credential.notes,
                "folderId": credential.folder_id
            });

            // Zapisz do pliku tymczasowego
            let temp_file = format!("/tmp/bw_item_{}.json", uuid::Uuid::new_v4());
            std::fs::write(&temp_file, item.to_string())
                .context("Failed to write temporary Bitwarden item file")?;

            let output = Command::new("bw")
                .args(&["create", "item", &temp_file, "--session", &session.session_token])
                .output()
                .context("Failed to execute bitwarden CLI create command")?;

            // Usuń plik tymczasowy
            let _ = std::fs::remove_file(&temp_file);

            if output.status.success() {
                let created_item: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
                    .context("Failed to parse created item response")?;
                
                let item_id = created_item["id"].as_str().unwrap_or("").to_string();
                info!("Successfully added credential with ID: {}", item_id);
                Ok(item_id)
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                error!("Failed to add credential: {}", error_msg);
                Err(anyhow::anyhow!("Failed to add Bitwarden credential: {}", error_msg))
            }
        } else {
            Err(anyhow::anyhow!("No active Bitwarden session. Please login first."))
        }
    }

    /// Sprawdź czy sesja jest nadal aktywna
    pub fn is_session_valid(&self) -> bool {
        if let Some(ref session) = self.session {
            chrono::Utc::now() < session.expires_at
        } else {
            false
        }
    }

    /// Pobierz status sesji
    pub fn get_session_info(&self) -> Option<&LoginSession> {
        self.session.as_ref()
    }

    /// Wyloguj się z Bitwarden
    pub async fn logout(&mut self) -> Result<()> {
        info!("Logging out from Bitwarden");

        let _output = Command::new("bw")
            .args(&["logout"])
            .output()
            .context("Failed to execute bitwarden CLI logout command")?;

        self.session = None;
        info!("Successfully logged out from Bitwarden");
        Ok(())
    }
}
