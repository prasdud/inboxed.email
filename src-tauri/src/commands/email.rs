use crate::auth::storage::{get_account_tokens, get_tokens};
use crate::commands::account::AccountManager;
use crate::db::EmailDatabase;
use crate::email::imap_client::{ImapClient, ImapCredentials};
use crate::email::provider::{EmailProvider, ImapFlag};
use crate::email::server_presets::ServerConfig;
use crate::email::types::{Email, EmailListItem};
use std::sync::{Arc, Mutex};
use tauri::State;

type DbState = Arc<Mutex<Option<EmailDatabase>>>;

/// Parse a unified email ID "{account_id}:{folder}:{uid}" into parts
fn parse_email_id(email_id: &str) -> Option<(String, String, u32)> {
    let parts: Vec<&str> = email_id.splitn(3, ':').collect();
    if parts.len() == 3 {
        let uid = parts[2].parse::<u32>().ok()?;
        Some((parts[0].to_string(), parts[1].to_string(), uid))
    } else {
        None
    }
}

/// Get or create an ImapClient for the active account
async fn get_active_client(
    db: &DbState,
    account_manager: &AccountManager,
) -> Result<Arc<tokio::sync::Mutex<ImapClient>>, String> {
    // Get active account from DB
    let account = {
        let db_lock = db.lock().unwrap();
        let database = db_lock.as_ref().ok_or("Database not initialized")?;
        database
            .get_active_account()
            .map_err(|e| e.to_string())?
            .ok_or("No active account. Please add an account first.")?
    };

    // Check if client exists
    if let Some(client) = account_manager.get_client(&account.id) {
        return Ok(client);
    }

    // Create a new client
    let credentials = if account.auth_type == "oauth2" {
        let tokens = get_account_tokens(&account.id)
            .or_else(|_| get_tokens()) // Fallback to legacy tokens
            .map_err(|e| format!("Not authenticated: {}", e))?;
        ImapCredentials::OAuth2 {
            user: account.email.clone(),
            access_token: tokens.access_token,
        }
    } else {
        let password = crate::auth::storage::get_app_password(&account.id)
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

    account_manager.add_client(account.id.clone(), client);

    account_manager
        .get_client(&account.id)
        .ok_or_else(|| "Failed to store client".to_string())
}

#[tauri::command]
pub async fn fetch_emails(
    db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    max_results: Option<u32>,
    query: Option<String>,
    force_refresh: Option<bool>,
) -> Result<Vec<EmailListItem>, String> {
    let should_refresh = force_refresh.unwrap_or(false);

    // Try cache first if not forcing refresh
    if !should_refresh {
        let db_lock = db.lock().unwrap();
        if let Some(database) = db_lock.as_ref() {
            if let Ok(cached_emails) = database.get_cached_emails(max_results.unwrap_or(50) as i64)
            {
                if !cached_emails.is_empty() {
                    return Ok(cached_emails);
                }
            }
        }
    }

    // Try IMAP client
    match get_active_client(&db, &account_manager).await {
        Ok(client_arc) => {
            let client = client_arc.lock().await;
            let items = client
                .list_messages("INBOX", max_results.unwrap_or(50), 0)
                .await
                .map_err(|e| e.to_string())?;

            // Cache the emails we fetched (fetch full for caching)
            for item in &items {
                if let Some((_, folder, uid)) = parse_email_id(&item.id) {
                    match client.get_message(&folder, uid).await {
                        Ok(email) => {
                            let db_lock = db.lock().unwrap();
                            if let Some(database) = db_lock.as_ref() {
                                let _ = database.store_email(&email);
                            }
                        }
                        Err(e) => eprintln!("Failed to fetch message uid={}: {}", uid, e),
                    }
                }
            }

            Ok(items)
        }
        Err(_) => {
            // Fallback to legacy Gmail API path
            let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
            let gmail_client = crate::email::GmailClient::new(tokens.access_token);

            let list_response = gmail_client
                .list_messages(max_results, query.as_deref(), None)
                .await
                .map_err(|e| e.to_string())?;

            let messages = list_response.messages.unwrap_or_default();
            let mut emails = Vec::new();

            for msg_id in messages.iter().take(max_results.unwrap_or(50) as usize) {
                match gmail_client.get_message(&msg_id.id).await {
                    Ok(gmail_msg) => {
                        let email = gmail_client.parse_email(gmail_msg);
                        {
                            let db_lock = db.lock().unwrap();
                            if let Some(database) = db_lock.as_ref() {
                                let _ = database.store_email(&email);
                            }
                        }
                        emails.push(crate::email::GmailClient::to_list_item(&email));
                    }
                    Err(e) => eprintln!("Failed to fetch message {}: {}", msg_id.id, e),
                }
            }

            Ok(emails)
        }
    }
}

#[tauri::command]
pub async fn get_email(
    db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    email_id: String,
) -> Result<Email, String> {
    // Try IMAP path: parse the composite ID
    if let Some((account_id, folder, uid)) = parse_email_id(&email_id) {
        if let Some(client_arc) = account_manager.get_client(&account_id) {
            let client = client_arc.lock().await;
            return client
                .get_message(&folder, uid)
                .await
                .map_err(|e| e.to_string());
        }
    }

    // Fallback: try database cache
    {
        let db_lock = db.lock().unwrap();
        if let Some(database) = db_lock.as_ref() {
            if let Ok(Some(email)) = database.get_email_by_id(&email_id) {
                return Ok(email);
            }
        }
    }

    // Fallback: legacy Gmail API
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let gmail_client = crate::email::GmailClient::new(tokens.access_token);
    let gmail_msg = gmail_client
        .get_message(&email_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(gmail_client.parse_email(gmail_msg))
}

#[tauri::command]
pub async fn send_email(
    db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    to: Vec<String>,
    subject: String,
    body: String,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
) -> Result<String, String> {
    // Try IMAP/SMTP path
    match get_active_client(&db, &account_manager).await {
        Ok(client_arc) => {
            let client = client_arc.lock().await;
            client
                .send_email(
                    &client.email,
                    to,
                    cc.unwrap_or_default(),
                    bcc.unwrap_or_default(),
                    &subject,
                    &body,
                    "", // plain text version
                )
                .await
                .map_err(|e| e.to_string())?;
            Ok("sent".to_string())
        }
        Err(_) => {
            // Fallback to legacy Gmail API
            let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
            let gmail_client = crate::email::GmailClient::new(tokens.access_token);
            gmail_client
                .send_email(to, subject, body, cc, bcc)
                .await
                .map_err(|e| e.to_string())
        }
    }
}

#[tauri::command]
pub async fn mark_email_read(
    _db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    email_id: String,
    read: bool,
) -> Result<(), String> {
    if let Some((account_id, folder, uid)) = parse_email_id(&email_id) {
        if let Some(client_arc) = account_manager.get_client(&account_id) {
            let client = client_arc.lock().await;
            return client
                .set_flags(&folder, uid, &[ImapFlag::Seen], read)
                .await
                .map_err(|e| e.to_string());
        }
    }

    // Fallback to legacy Gmail API
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let gmail_client = crate::email::GmailClient::new(tokens.access_token);
    let (add, remove) = if read {
        (vec![], vec!["UNREAD".to_string()])
    } else {
        (vec!["UNREAD".to_string()], vec![])
    };
    gmail_client
        .modify_labels(&email_id, add, remove)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn star_email(
    _db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    email_id: String,
    starred: bool,
) -> Result<(), String> {
    if let Some((account_id, folder, uid)) = parse_email_id(&email_id) {
        if let Some(client_arc) = account_manager.get_client(&account_id) {
            let client = client_arc.lock().await;
            return client
                .set_flags(&folder, uid, &[ImapFlag::Flagged], starred)
                .await
                .map_err(|e| e.to_string());
        }
    }

    // Fallback to legacy Gmail API
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let gmail_client = crate::email::GmailClient::new(tokens.access_token);
    let (add, remove) = if starred {
        (vec!["STARRED".to_string()], vec![])
    } else {
        (vec![], vec!["STARRED".to_string()])
    };
    gmail_client
        .modify_labels(&email_id, add, remove)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trash_email(
    _db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    email_id: String,
) -> Result<(), String> {
    if let Some((account_id, folder, uid)) = parse_email_id(&email_id) {
        if let Some(client_arc) = account_manager.get_client(&account_id) {
            let client = client_arc.lock().await;
            // Move to Trash folder
            return client
                .move_message(&folder, uid, "Trash")
                .await
                .map_err(|e| e.to_string());
        }
    }

    // Fallback to legacy Gmail API
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let gmail_client = crate::email::GmailClient::new(tokens.access_token);
    gmail_client
        .trash_email(&email_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_email(
    _db: State<'_, DbState>,
    account_manager: State<'_, AccountManager>,
    email_id: String,
) -> Result<(), String> {
    if let Some((account_id, folder, uid)) = parse_email_id(&email_id) {
        if let Some(client_arc) = account_manager.get_client(&account_id) {
            let client = client_arc.lock().await;
            // Move to Archive folder
            return client
                .move_message(&folder, uid, "Archive")
                .await
                .map_err(|e| e.to_string());
        }
    }

    // Fallback to legacy Gmail API
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let gmail_client = crate::email::GmailClient::new(tokens.access_token);
    gmail_client
        .modify_labels(&email_id, vec![], vec!["INBOX".to_string()])
        .await
        .map_err(|e| e.to_string())
}
