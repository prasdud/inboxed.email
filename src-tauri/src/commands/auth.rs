use crate::auth::{
    clear_tokens, get_tokens, handle_oauth_callback, has_valid_tokens, refresh_access_token,
    start_oauth_flow, TokenData,
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

    // Try to get tokens - if we have a refresh token, try to refresh
    match get_tokens() {
        Ok(tokens) => {
            if let Some(refresh_token) = tokens.refresh_token {
                // Attempt to refresh the access token
                match refresh_access_token(&refresh_token).await {
                    Ok(_) => {
                        return Ok(AuthStatus {
                            authenticated: true,
                            email: Some("user@example.com".to_string()),
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to refresh token: {}", e);
                        // Fall through to unauthenticated
                    }
                }
            }
        }
        Err(_) => {
            // No tokens stored at all
        }
    }

    Ok(AuthStatus {
        authenticated: false,
        email: None,
    })
}

/// Start OAuth authentication flow
#[tauri::command]
pub async fn start_auth() -> Result<String, String> {
    start_oauth_flow().map_err(|e| e.to_string())
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
