use tauri::{State, Emitter};
use std::sync::{Arc, Mutex};
use directories::ProjectDirs;
use anyhow::{Context, Result};
use tokio::task;
use chrono::Utc;

use crate::db::{EmailDatabase, email_db::{EmailWithInsight, IndexingStatus, EmailInsight}};
use crate::email::types::Email;
use crate::auth::storage;
use crate::commands::ai::SUMMARIZER;

type DbState = Arc<Mutex<Option<EmailDatabase>>>;

#[tauri::command]
pub async fn init_database() -> Result<(), String> {
    let project_dirs = ProjectDirs::from("com", "inboxed", "inboxed")
        .ok_or("Failed to get project directory")?;
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;

    let db_path = data_dir.join("emails.db");
    let _db = EmailDatabase::new(db_path).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_smart_inbox(
    db: State<'_, DbState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<EmailWithInsight>, String> {
    let db_lock = db.lock().unwrap();
    let database = db_lock.as_ref().ok_or("Database not initialized")?;

    let emails = database
        .get_emails_by_priority(limit.unwrap_or(50), offset.unwrap_or(0))
        .map_err(|e: anyhow::Error| e.to_string())?;

    Ok(emails)
}

#[tauri::command]
pub async fn get_emails_by_category(
    db: State<'_, DbState>,
    category: String,
    limit: Option<i64>,
) -> Result<Vec<EmailWithInsight>, String> {
    let db_lock = db.lock().unwrap();
    let database = db_lock.as_ref().ok_or("Database not initialized")?;

    let emails = database
        .get_emails_by_category(&category, limit.unwrap_or(50))
        .map_err(|e: anyhow::Error| e.to_string())?;

    Ok(emails)
}

#[tauri::command]
pub async fn search_smart_emails(
    db: State<'_, DbState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<EmailWithInsight>, String> {
    let db_lock = db.lock().unwrap();
    let database = db_lock.as_ref().ok_or("Database not initialized")?;

    let emails = database
        .search_emails(&query, limit.unwrap_or(50))
        .map_err(|e: anyhow::Error| e.to_string())?;

    Ok(emails)
}

#[tauri::command]
pub async fn get_indexing_status(db: State<'_, DbState>) -> Result<IndexingStatus, String> {
    let db_lock = db.lock().unwrap();
    let database = db_lock.as_ref().ok_or("Database not initialized")?;

    let status = database
        .get_indexing_status()
        .map_err(|e: anyhow::Error| e.to_string())?;
    Ok(status)
}

#[tauri::command]
pub async fn start_email_indexing<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    _db: State<'_, DbState>,
    max_emails: Option<usize>,
) -> Result<(), String> {
    // Get database instance
    let project_dirs = ProjectDirs::from("com", "inboxed", "inboxed")
        .ok_or("Failed to get project directory")?;
    let data_dir = project_dirs.data_dir();
    let db_path = data_dir.join("emails.db");
    let database = EmailDatabase::new(db_path).map_err(|e| e.to_string())?;

    // Check if already indexing
    let status = database
        .get_indexing_status()
        .map_err(|e: anyhow::Error| e.to_string())?;
    if status.is_indexing {
        return Err("Indexing already in progress".to_string());
    }

    // Get access token
    let token_data = storage::get_tokens()
        .map_err(|e| format!("Not authenticated: {}", e))?;

    // Spawn background task
    task::spawn(async move {
        if let Err(e) = index_emails_background(
            app,
            database,
            token_data.access_token,
            max_emails.unwrap_or(100),
        ).await {
            eprintln!("Indexing error: {}", e);
        }
    });

    Ok(())
}

async fn index_emails_background<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    database: EmailDatabase,
    access_token: String,
    max_emails: usize,
) -> Result<()> {
    // Mark as indexing
    database.update_indexing_status(true, None, Some(0), None)?;
    let _ = app.emit("indexing:started", ());

    // Fetch emails from Gmail
    let gmail_client = crate::email::gmail::GmailClient::new(access_token);
    let response = gmail_client
        .list_messages(Some(max_emails as u32), None, None)
        .await
        .context("Failed to fetch emails from Gmail")?;

    let message_ids = response.messages.unwrap_or_default();
    let total = message_ids.len() as i64;
    database.update_indexing_status(true, Some(total), Some(0), None)?;

    // Process each email
    for (idx, email_item) in message_ids.iter().enumerate() {
        // Fetch full email (GmailMessage)
        let gmail_message = match gmail_client.get_message(&email_item.id).await {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to fetch email {}: {}", email_item.id, e);
                continue;
            }
        };

        // Convert to Email
        let email = gmail_client.parse_email(gmail_message);

        // Store email in database
        if let Err(e) = database.store_email(&email) {
            eprintln!("Failed to store email {}: {}", email.id, e);
            continue;
        }

        // Generate AI insights
        let insight = generate_email_insights(&email).await;

        // Store insights
        if let Err(e) = database.store_insights(&insight) {
            eprintln!("Failed to store insights for {}: {}", email.id, e);
        }

        // Update progress
        let processed = (idx + 1) as i64;
        if let Err(e) = database.update_indexing_status(true, None, Some(processed), None) {
            eprintln!("Failed to update progress: {}", e);
        }

        // Emit progress event
        let progress = (processed as f64 / total as f64 * 100.0) as i32;
        let _ = app.emit("indexing:progress", progress);
    }

    // Mark as complete
    database.update_indexing_status(false, None, None, None)?;
    let _ = app.emit("indexing:complete", ());

    Ok(())
}

async fn generate_email_insights(email: &Email) -> EmailInsight {
    let body = email.body_plain.as_deref()
        .or(email.body_html.as_deref())
        .unwrap_or("");

    // Generate summary using the global SUMMARIZER
    let summary = {
        let summarizer_guard = SUMMARIZER.lock().unwrap();
        if let Some(summarizer) = summarizer_guard.as_ref() {
            summarizer
                .summarize_email(&email.subject, &email.from, body)
                .ok()
        } else {
            None
        }
    };

    // Classify priority
    let (priority, priority_score) = classify_priority_internal(email, body);

    // Detect insights
    let has_deadline = body.to_lowercase().contains("deadline")
        || body.to_lowercase().contains("due date")
        || body.to_lowercase().contains("by end of");

    let has_meeting = body.to_lowercase().contains("meeting")
        || body.to_lowercase().contains("call")
        || body.to_lowercase().contains("zoom")
        || body.to_lowercase().contains("teams");

    let has_financial = body.to_lowercase().contains("invoice")
        || body.to_lowercase().contains("payment")
        || body.to_lowercase().contains("$")
        || body.to_lowercase().contains("price");

    // Categorize
    let category = categorize_email(email, body);

    EmailInsight {
        email_id: email.id.clone(),
        summary,
        priority,
        priority_score,
        category: Some(category),
        insights: None,
        action_items: None,
        has_deadline,
        has_meeting,
        has_financial,
        sentiment: None,
        indexed_at: Utc::now().timestamp(),
    }
}

fn classify_priority_internal(email: &Email, body: &str) -> (String, f64) {
    let mut score: f64 = 0.5;

    // Check for urgency keywords
    let urgent_keywords = ["urgent", "asap", "immediately", "critical", "emergency"];
    if urgent_keywords.iter().any(|&kw| body.to_lowercase().contains(kw)) {
        score += 0.3;
    }

    // Check for action keywords
    let action_keywords = ["please review", "need your", "waiting for", "action required"];
    if action_keywords.iter().any(|&kw| body.to_lowercase().contains(kw)) {
        score += 0.2;
    }

    // Check if starred
    if email.is_starred {
        score += 0.2;
    }

    // Clamp score
    score = score.min(1.0).max(0.0);

    let priority = if score >= 0.7 {
        "HIGH"
    } else if score >= 0.4 {
        "MEDIUM"
    } else {
        "LOW"
    };

    (priority.to_string(), score)
}

fn categorize_email(email: &Email, body: &str) -> String {
    let subject_lower = email.subject.to_lowercase();
    let body_lower = body.to_lowercase();

    if subject_lower.contains("re:") || subject_lower.contains("fwd:") {
        return "conversation".to_string();
    }

    if body_lower.contains("meeting") || body_lower.contains("calendar") {
        return "meetings".to_string();
    }

    if body_lower.contains("invoice") || body_lower.contains("payment") {
        return "financial".to_string();
    }

    if body_lower.contains("newsletter") || body_lower.contains("unsubscribe") {
        return "newsletters".to_string();
    }

    if body_lower.contains("notification") || subject_lower.contains("[") {
        return "notifications".to_string();
    }

    "general".to_string()
}

#[tauri::command]
pub async fn chat_query(
    db: State<'_, DbState>,
    query: String,
) -> Result<String, String> {
    let query_lower = query.to_lowercase();

    // Parse query intent
    if query_lower.contains("today") || query_lower.contains("today's") {
        // Get emails from today
        let db_lock = db.lock().unwrap();
        let database = db_lock.as_ref().ok_or("Database not initialized")?;
        let emails = database
            .get_emails_from_today()
            .map_err(|e: anyhow::Error| e.to_string())?;

        if emails.is_empty() {
            return Ok("You haven't received any emails today yet.".to_string());
        }

        // Generate summary
        let summaries: Vec<String> = emails.iter()
            .take(10)
            .map(|e| {
                let summary = e.summary.clone().unwrap_or_else(|| e.snippet.clone());
                format!("• {} from {} - {}", e.subject, e.from_name, summary)
            })
            .collect();

        Ok(format!(
            "You received {} emails today. Here are the highlights:\n\n{}",
            emails.len(),
            summaries.join("\n")
        ))
    } else if query_lower.contains("important") || query_lower.contains("priority") {
        // Get high priority emails
        let db_lock = db.lock().unwrap();
        let database = db_lock.as_ref().ok_or("Database not initialized")?;
        let emails = database
            .get_emails_by_priority(10, 0)
            .map_err(|e: anyhow::Error| e.to_string())?;

        let high_priority: Vec<_> = emails.iter()
            .filter(|e| e.priority == "HIGH")
            .collect();

        if high_priority.is_empty() {
            return Ok("You don't have any high priority emails right now.".to_string());
        }

        let summaries: Vec<String> = high_priority.iter()
            .map(|e| {
                let summary = e.summary.clone().unwrap_or_else(|| e.snippet.clone());
                format!("• {} from {} - {}", e.subject, e.from_name, summary)
            })
            .collect();

        Ok(format!(
            "You have {} high priority emails:\n\n{}",
            high_priority.len(),
            summaries.join("\n")
        ))
    } else {
        // General search
        let db_lock = db.lock().unwrap();
        let database = db_lock.as_ref().ok_or("Database not initialized")?;
        let emails = database
            .search_emails(&query, 5)
            .map_err(|e: anyhow::Error| e.to_string())?;

        if emails.is_empty() {
            return Ok(format!("I couldn't find any emails matching '{}'.", query));
        }

        let summaries: Vec<String> = emails.iter()
            .map(|e| {
                let summary = e.summary.clone().unwrap_or_else(|| e.snippet.clone());
                format!("• {} from {} - {}", e.subject, e.from_name, summary)
            })
            .collect();

        Ok(format!(
            "Found {} emails matching '{}':\n\n{}",
            emails.len(),
            query,
            summaries.join("\n")
        ))
    }
}
