use crate::auth::storage::get_tokens;
use crate::email::{Email, EmailListItem, GmailClient};

#[tauri::command]
pub async fn fetch_emails(
    max_results: Option<u32>,
    query: Option<String>,
) -> Result<Vec<EmailListItem>, String> {
    // Get access token
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;

    let client = GmailClient::new(tokens.access_token);

    // List messages
    let list_response = client
        .list_messages(max_results, query.as_deref(), None)
        .await
        .map_err(|e| e.to_string())?;

    let messages = list_response.messages.unwrap_or_default();

    // Fetch full details for each message
    let mut emails = Vec::new();
    for msg_id in messages.iter().take(max_results.unwrap_or(50) as usize) {
        match client.get_message(&msg_id.id).await {
            Ok(gmail_msg) => {
                let email = client.parse_email(gmail_msg);
                emails.push(GmailClient::to_list_item(&email));
            }
            Err(e) => {
                eprintln!("Failed to fetch message {}: {}", msg_id.id, e);
            }
        }
    }

    Ok(emails)
}

#[tauri::command]
pub async fn get_email(email_id: String) -> Result<Email, String> {
    // Get access token
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;

    let client = GmailClient::new(tokens.access_token);

    // Fetch the email
    let gmail_msg = client
        .get_message(&email_id)
        .await
        .map_err(|e| e.to_string())?;

    let email = client.parse_email(gmail_msg);

    Ok(email)
}

#[tauri::command]
pub async fn send_email(
    to: Vec<String>,
    subject: String,
    body: String,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
) -> Result<String, String> {
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let client = GmailClient::new(tokens.access_token);

    client
        .send_email(to, subject, body, cc, bcc)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_email_read(email_id: String, read: bool) -> Result<(), String> {
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let client = GmailClient::new(tokens.access_token);

    let (add, remove) = if read {
        (vec![], vec!["UNREAD".to_string()])
    } else {
        (vec!["UNREAD".to_string()], vec![])
    };

    client
        .modify_labels(&email_id, add, remove)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn star_email(email_id: String, starred: bool) -> Result<(), String> {
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let client = GmailClient::new(tokens.access_token);

    let (add, remove) = if starred {
        (vec!["STARRED".to_string()], vec![])
    } else {
        (vec![], vec!["STARRED".to_string()])
    };

    client
        .modify_labels(&email_id, add, remove)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trash_email(email_id: String) -> Result<(), String> {
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let client = GmailClient::new(tokens.access_token);

    client
        .trash_email(&email_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_email(email_id: String) -> Result<(), String> {
    let tokens = get_tokens().map_err(|e| format!("Not authenticated: {}", e))?;
    let client = GmailClient::new(tokens.access_token);

    client
        .modify_labels(&email_id, vec![], vec!["INBOX".to_string()])
        .await
        .map_err(|e| e.to_string())
}
