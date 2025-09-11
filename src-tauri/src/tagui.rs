use std::process::Command;
use std::fs;
use std::path::Path;
use tracing::{info, error, debug};

pub async fn execute_script(dsl_script: &str) -> bool {
    info!("Executing TagUI script");
    
    // Validate script first
    if let Err(e) = validate_dsl_script(dsl_script) {
        error!("Invalid DSL script: {}", e);
        return false;
    }
    
    // Zapisz skrypt do pliku tymczasowego
    let script_path = "temp_script.codialog";
    match fs::write(script_path, dsl_script) {
        Ok(_) => debug!("Script written to {}", script_path),
        Err(e) => {
            error!("Failed to write script file: {}", e);
            return false;
        }
    }
    
    // Uruchom TagUI
    let output = Command::new("tagui")
        .arg(script_path)
        .arg("chrome")
        .output();
    
    // Usuń plik tymczasowy
    fs::remove_file(script_path).ok();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                info!("TagUI script executed successfully");
                true
            } else {
                error!("TagUI execution failed: {}", String::from_utf8_lossy(&result.stderr));
                false
            }
        }
        Err(e) => {
            error!("Failed to execute TagUI: {}", e);
            false
        }
    }
}

pub fn install_tagui() -> bool {
    info!("Installing TagUI...");
    
    // Sprawdź czy TagUI jest zainstalowane
    if Path::new("tagui").exists() {
        info!("TagUI directory already exists");
        return true;
    }
    
    // Pobierz i zainstaluj TagUI
    let output = Command::new("git")
        .args(&["clone", "https://github.com/aisingapore/tagui"])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                info!("TagUI cloned successfully");
                
                // Zainstaluj zależności npm w folderze tagui
                let npm_install = Command::new("npm")
                    .arg("install")
                    .current_dir("tagui")
                    .output();
                
                match npm_install {
                    Ok(npm_result) => {
                        if npm_result.status.success() {
                            info!("TagUI dependencies installed");
                            true
                        } else {
                            error!("Failed to install TagUI npm dependencies: {}", 
                                   String::from_utf8_lossy(&npm_result.stderr));
                            false
                        }
                    }
                    Err(e) => {
                        error!("Failed to run npm install: {}", e);
                        false
                    }
                }
            } else {
                error!("Failed to clone TagUI: {}", String::from_utf8_lossy(&result.stderr));
                false
            }
        }
        Err(e) => {
            error!("Git command failed: {}", e);
            false
        }
    }
}

pub async fn check_tagui_installed() -> bool {
    // Sprawdź czy TagUI jest dostępne w PATH
    if let Ok(output) = Command::new("tagui").arg("--version").output() {
        return output.status.success();
    }
    
    // Sprawdź czy istnieje lokalna instalacja
    Path::new("tagui/tagui").exists() || Path::new("tagui/tagui.cmd").exists()
}

pub fn validate_dsl_script(script: &str) -> Result<(), String> {
    let valid_commands = ["click", "type", "upload", "hover", "wait"];
    
    for line in script.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        let command = parts[0];
        if !valid_commands.contains(&command) {
            return Err(format!("Invalid DSL command: {}", command));
        }
        
        // Sprawdź poprawność składni dla każdej komendy
        match command {
            "click" | "hover" => {
                if parts.len() != 2 {
                    return Err(format!("Command '{}' requires exactly one argument", command));
                }
            }
            "type" | "upload" => {
                if parts.len() < 3 {
                    return Err(format!("Command '{}' requires at least two arguments", command));
                }
            }
            "wait" => {
                if parts.len() != 2 {
                    return Err(format!("Command 'wait' requires exactly one argument"));
                }
                // Sprawdź czy argument jest liczbą
                if parts[1].parse::<f64>().is_err() {
                    return Err(format!("Wait time must be a number"));
                }
            }
            _ => {}
        }
    }
    
    Ok(())
}

pub fn escape_for_dsl(input: &str) -> String {
    input.replace('\\', "\\\\").replace('\"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_dsl_script() {
        let valid_script = r#"click "#button"
type "#input" "text"
upload "#file" "path/to/file.pdf""#;
        
        assert!(validate_dsl_script(valid_script).is_ok());
        
        let invalid_script = "invalid_command \"#test\"";
        assert!(validate_dsl_script(invalid_script).is_err());
    }
    
    #[test]
    fn test_escape_for_dsl() {
        assert_eq!(escape_for_dsl("test \"quoted\" text"), "test \\\"quoted\\\" text");
        assert_eq!(escape_for_dsl("normal text"), "normal text");
    }
}
