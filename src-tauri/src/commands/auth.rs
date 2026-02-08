use crate::auth::{
    clear_tokens, get_tokens, handle_oauth_callback, has_valid_tokens, refresh_access_token,
    start_oauth_flow, start_oauth_flow_for_provider, TokenData,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub email: Option<String>,
}

/// Check if user is authenticated
/// If token is expired but refresh token exists, attempt to refresh
#[tauri::command]
pub async fn check_auth_status() -> Result<AuthStatus, String> {
    // First check if we have valid (non-expired) tokens
    if has_valid_tokens() {
        return Ok(AuthStatus {
            authenticated: true,
            email: Some("user@example.com".to_string()),
        });
    }

    // Check if we have any accounts stored
    let has_accounts = {
        let project_dirs = directories::ProjectDirs::from("com", "inboxed", "inboxed");
        if let Some(dirs) = project_dirs {
            let db_path = dirs.data_dir().join("emails.db");
            if let Ok(database) = crate::db::EmailDatabase::new(db_path) {
                database.list_accounts().map(|a| !a.is_empty()).unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
    };

    if has_accounts {
        return Ok(AuthStatus {
            authenticated: true,
            email: None,
        });
    }

    // Try to get tokens - if we have a refresh token, try to refresh
    match get_tokens() {
        Ok(tokens) => {
            if let Some(refresh_token) = tokens.refresh_token {
                match refresh_access_token(&refresh_token).await {
                    Ok(_) => {
                        return Ok(AuthStatus {
                            authenticated: true,
                            email: Some("user@example.com".to_string()),
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to refresh token: {}", e);
                    }
                }
            }
        }
        Err(_) => {}
    }

    Ok(AuthStatus {
        authenticated: false,
        email: None,
    })
}

/// Start OAuth authentication flow
/// Accepts optional provider (gmail, outlook) and account_id
#[tauri::command]
pub async fn start_auth(
    provider: Option<String>,
    account_id: Option<String>,
) -> Result<String, String> {
    let provider_str = provider.as_deref().unwrap_or("gmail");
    start_oauth_flow_for_provider(provider_str, account_id.as_deref()).map_err(|e| e.to_string())
}

/// Complete OAuth flow after user authorization
#[tauri::command]
pub async fn complete_auth() -> Result<TokenData, String> {
    handle_oauth_callback()
        .await
        .map_err(|e| e.to_string())
}

/// Refresh access token
#[tauri::command]
pub async fn refresh_token() -> Result<TokenData, String> {
    let tokens = get_tokens().map_err(|e| e.to_string())?;

    let refresh_token = tokens
        .refresh_token
        .ok_or_else(|| "No refresh token available".to_string())?;

    refresh_access_token(&refresh_token)
        .await
        .map_err(|e| e.to_string())
}

/// Sign out - clear all stored tokens
#[tauri::command]
pub async fn sign_out() -> Result<(), String> {
    clear_tokens().map_err(|e| e.to_string())
}

/// Get current access token (for making API calls)
#[tauri::command]
pub async fn get_access_token() -> Result<String, String> {
    let tokens = get_tokens().map_err(|e| e.to_string())?;
    Ok(tokens.access_token)
}
