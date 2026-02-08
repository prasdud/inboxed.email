use crate::auth::account::Account;
use crate::db::EmailDatabase;
use crate::email::imap_client::{ImapClient, ImapCredentials};
use crate::email::server_presets::{get_server_preset, AuthType, ProviderType, ServerConfig};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Holds active IMAP clients for all connected accounts
pub struct AccountManager {
    pub clients: Mutex<HashMap<String, Arc<tokio::sync::Mutex<ImapClient>>>>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_client(
        &self,
        account_id: &str,
    ) -> Option<Arc<tokio::sync::Mutex<ImapClient>>> {
        let clients = self.clients.lock().unwrap();
        clients.get(account_id).cloned()
    }

    pub fn add_client(&self, account_id: String, client: ImapClient) {
        let mut clients = self.clients.lock().unwrap();
        clients.insert(account_id, Arc::new(tokio::sync::Mutex::new(client)));
    }

    pub fn remove_client(&self, account_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        clients.remove(account_id);
    }
}

type DbState = Arc<Mutex<Option<EmailDatabase>>>;

/// Add a new email account (OAuth — tokens already obtained)
#[tauri::command]
pub async fn add_account(
    db: State<'_, DbState>,
    _account_manager: State<'_, AccountManager>,
    email: String,
    display_name: String,
    provider: String,
    imap_host: Option<String>,
    imap_port: Option<u16>,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    auth_type: String,
) -> Result<Account, String> {
    let provider_type = ProviderType::from_str(&provider);
    let auth = if auth_type == "oauth2" {
        AuthType::OAuth2
    } else {
        AuthType::Password
    };

    // Use presets for known providers, or custom config
    let server_config = if let Some(preset) = get_server_preset(&provider_type) {
        ServerConfig {
            imap_host: imap_host.unwrap_or(preset.imap_host),
            imap_port: imap_port.unwrap_or(preset.imap_port),
            smtp_host: smtp_host.unwrap_or(preset.smtp_host),
            smtp_port: smtp_port.unwrap_or(preset.smtp_port),
            use_tls: preset.use_tls,
        }
    } else {
        ServerConfig {
            imap_host: imap_host.ok_or("IMAP host required for custom provider")?,
            imap_port: imap_port.unwrap_or(993),
            smtp_host: smtp_host.ok_or("SMTP host required for custom provider")?,
            smtp_port: smtp_port.unwrap_or(465),
            use_tls: true,
        }
    };

    let account = Account::new(
        email,
        display_name,
        provider_type,
        server_config.imap_host,
        server_config.imap_port,
        server_config.smtp_host,
        server_config.smtp_port,
        auth,
    );

    // Store in database
    {
        let db_lock = db.lock().unwrap();
        let database = db_lock.as_ref().ok_or("Database not initialized")?;
        database
            .store_account(&account)
            .map_err(|e| e.to_string())?;
    }

    Ok(account)
}

/// Remove an account and all its data
#[tauri::command]
pub async fn remove_account(
    db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    account_id: String,
) -> Result<(), String> {
    // Remove IMAP client
    account_manager.remove_client(&account_id);

    // Remove from database
    {
        let db_lock = db.lock().unwrap();
        let database = db_lock.as_ref().ok_or("Database not initialized")?;
        database
            .remove_account(&account_id)
            .map_err(|e| e.to_string())?;
    }

    // Clear stored tokens for this account
    crate::auth::storage::clear_account_tokens(&account_id).map_err(|e| e.to_string())?;

    Ok(())
}

/// List all accounts
#[tauri::command]
pub async fn list_accounts(db: State<'_, DbState>) -> Result<Vec<Account>, String> {
    let db_lock = db.lock().unwrap();
    let database = db_lock.as_ref().ok_or("Database not initialized")?;

    database
        .list_accounts()
        .map_err(|e| e.to_string())
}

/// Set active account
#[tauri::command]
pub async fn set_active_account(
    db: State<'_, DbState>,
    account_id: String,
) -> Result<(), String> {
    let db_lock = db.lock().unwrap();
    let database = db_lock.as_ref().ok_or("Database not initialized")?;

    database
        .set_active_account(&account_id)
        .map_err(|e| e.to_string())
}

/// Connect an account's IMAP client using stored credentials
#[tauri::command]
pub async fn connect_account(
    db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    account_id: String,
) -> Result<(), String> {
    // Get account info
    let account = {
        let db_lock = db.lock().unwrap();
        let database = db_lock.as_ref().ok_or("Database not initialized")?;
        database
            .get_account(&account_id)
            .map_err(|e| e.to_string())?
            .ok_or("Account not found")?
    };

    // Get credentials from storage
    let credentials = if account.auth_type == "oauth2" {
        let tokens = crate::auth::storage::get_account_tokens(&account_id)
            .map_err(|e| format!("No tokens for account: {}", e))?;
        ImapCredentials::OAuth2 {
            user: account.email.clone(),
            access_token: tokens.access_token,
        }
    } else {
        let password = crate::auth::storage::get_app_password(&account_id)
            .map_err(|e| format!("No password for account: {}", e))?;
        ImapCredentials::Password {
            user: account.email.clone(),
            password,
        }
    };

    let server_config = ServerConfig {
        imap_host: account.imap_host.clone(),
        imap_port: account.imap_port,
        smtp_host: account.smtp_host.clone(),
        smtp_port: account.smtp_port,
        use_tls: true,
    };

    let client = ImapClient::new(
        account.id.clone(),
        account.email.clone(),
        account.provider_type(),
        server_config,
        credentials,
    );

    // Test connection
    client.reconnect().await.map_err(|e| format!("Connection failed: {}", e))?;

    account_manager.add_client(account.id, client);

    Ok(())
}
