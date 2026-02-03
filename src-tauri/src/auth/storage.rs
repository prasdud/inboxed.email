use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SERVICE_NAME: &str = "com.inboxed.app";
const ACCESS_TOKEN_KEY: &str = "gmail_access_token";
const REFRESH_TOKEN_KEY: &str = "gmail_refresh_token";
const EXPIRY_KEY: &str = "gmail_token_expiry";

// Dev mode: use file storage to avoid keychain prompts
const USE_FILE_STORAGE: bool = cfg!(debug_assertions);

fn get_token_file_path() -> PathBuf {
    // Use a stable location in the user's home directory instead of temp
    if let Ok(home) = std::env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".inboxed");
        // Create directory if it doesn't exist
        let _ = std::fs::create_dir_all(&path);
        path.push("tokens.json");
        path
    } else {
        // Fallback to temp dir
        let mut path = std::env::temp_dir();
        path.push("inboxed_tokens.json");
        path
    }
}

#[derive(Serialize, Deserialize, Default)]
struct FileTokenStorage {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// Store access token in system keychain
pub fn store_access_token(token: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, ACCESS_TOKEN_KEY)
        .context("Failed to create keychain entry for access token")?;
    entry
        .set_password(token)
        .context("Failed to store access token in keychain")?;
    Ok(())
}

/// Store refresh token in system keychain
pub fn store_refresh_token(token: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
        .context("Failed to create keychain entry for refresh token")?;
    entry
        .set_password(token)
        .context("Failed to store refresh token in keychain")?;
    Ok(())
}

/// Store token expiry time
pub fn store_token_expiry(expires_at: DateTime<Utc>) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, EXPIRY_KEY)
        .context("Failed to create keychain entry for token expiry")?;
    let expiry_str = expires_at.to_rfc3339();
    entry
        .set_password(&expiry_str)
        .context("Failed to store token expiry in keychain")?;
    Ok(())
}

/// Store complete token data
pub fn store_tokens(token_data: &TokenData) -> Result<()> {
    if USE_FILE_STORAGE {
        // Dev mode: use file storage
        let storage = FileTokenStorage {
            access_token: Some(token_data.access_token.clone()),
            refresh_token: token_data.refresh_token.clone(),
            expires_at: Some(token_data.expires_at.to_rfc3339()),
        };
        let json = serde_json::to_string(&storage)?;
        fs::write(get_token_file_path(), json)?;
        Ok(())
    } else {
        // Production: use keychain
        store_access_token(&token_data.access_token)?;
        if let Some(ref refresh_token) = token_data.refresh_token {
            store_refresh_token(refresh_token)?;
        }
        store_token_expiry(token_data.expires_at)?;
        Ok(())
    }
}

/// Retrieve access token from keychain
pub fn get_access_token() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, ACCESS_TOKEN_KEY)
        .context("Failed to create keychain entry for access token")?;
    entry
        .get_password()
        .context("Failed to retrieve access token from keychain")
}

/// Retrieve refresh token from keychain
pub fn get_refresh_token() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
        .context("Failed to create keychain entry for refresh token")?;
    entry
        .get_password()
        .context("Failed to retrieve refresh token from keychain")
}

/// Retrieve token expiry time
pub fn get_token_expiry() -> Result<DateTime<Utc>> {
    let entry = Entry::new(SERVICE_NAME, EXPIRY_KEY)
        .context("Failed to create keychain entry for token expiry")?;
    let expiry_str = entry
        .get_password()
        .context("Failed to retrieve token expiry from keychain")?;
    DateTime::parse_from_rfc3339(&expiry_str)
        .context("Failed to parse token expiry")?
        .with_timezone(&Utc)
        .pipe(Ok)
}

/// Retrieve complete token data
pub fn get_tokens() -> Result<TokenData> {
    if USE_FILE_STORAGE {
        // Dev mode: read from file
        let json = fs::read_to_string(get_token_file_path())
            .context("Failed to read token file")?;
        let storage: FileTokenStorage = serde_json::from_str(&json)
            .context("Failed to parse token file")?;

        let access_token = storage.access_token
            .context("No access token in file")?;
        let expires_at = storage.expires_at
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .context("Invalid expiry time")?;

        Ok(TokenData {
            access_token,
            refresh_token: storage.refresh_token,
            expires_at,
        })
    } else {
        // Production: use keychain
        let access_token = get_access_token()?;
        let refresh_token = get_refresh_token().ok();
        let expires_at = get_token_expiry()?;

        Ok(TokenData {
            access_token,
            refresh_token,
            expires_at,
        })
    }
}

/// Check if we have valid tokens stored
pub fn has_valid_tokens() -> bool {
    match get_tokens() {
        Ok(token_data) => token_data.expires_at > Utc::now(),
        Err(_) => false,
    }
}

/// Clear all stored tokens
pub fn clear_tokens() -> Result<()> {
    if USE_FILE_STORAGE {
        // Dev mode: delete file
        let _ = fs::remove_file(get_token_file_path());
    } else {
        // Production: clear keychain
        let _ = Entry::new(SERVICE_NAME, ACCESS_TOKEN_KEY)
            .and_then(|e| e.delete_credential());
        let _ = Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
            .and_then(|e| e.delete_credential());
        let _ = Entry::new(SERVICE_NAME, EXPIRY_KEY)
            .and_then(|e| e.delete_credential());
    }
    Ok(())
}

// Helper trait for pipe operation
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}
