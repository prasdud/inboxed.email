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
    // Check if summarizer is available and model is loaded
    {
        let summarizer_guard = SUMMARIZER.lock().unwrap();
        if let Some(summarizer) = summarizer_guard.as_ref() {
            if summarizer.is_model_loaded() {
                println!("[Indexing] Starting with LLM model loaded - summaries will use AI");
            } else {
                println!("[Indexing] Starting with fallback mode - summaries will use keyword extraction");
            }
        } else {
            println!("[Indexing] WARNING: No summarizer available - summaries will be skipped");
        }
    }

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
            if summarizer.is_model_loaded() {
                match summarizer.summarize_email(&email.subject, &email.from, body) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        eprintln!("[Indexing] LLM summarization failed for {}: {}", email.id, e);
                        None
                    }
                }
            } else {
                // Model exists but not loaded - use fallback
                println!("[Indexing] Model not loaded, using fallback for: {}", email.id);
                summarizer
                    .summarize_email(&email.subject, &email.from, body)
                    .ok()
            }
        } else {
            println!("[Indexing] No summarizer available, skipping summary for: {}", email.id);
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

/// Query intent categories for chat
#[derive(Debug)]
enum QueryIntent {
    TodayEmails,
    ImportantEmails,
    SearchEmails(String),
    GeneralEmailQuestion,
    GeneralChat,
}

/// Check if query is asking about the AI's identity
fn is_identity_query(query: &str) -> bool {
    let q = query.to_lowercase();
    q.contains("which model")
        || q.contains("what model")
        || q.contains("who are you")
        || q.contains("what are you")
        || q.contains("what can you do")
        || q.contains("your name")
        || q.contains("introduce yourself")
}

/// Get a response about AI identity without using LLM
fn get_identity_response() -> String {
    "I'm your intelligent email assistant for Inboxed! I use a local AI model (LFM2.5) running on your device to help you:\n\n\
    • Summarize and understand your emails\n\
    • Find important or urgent messages\n\
    • Search through your inbox\n\
    • Answer questions about your emails\n\n\
    All processing happens locally on your device for privacy. Try asking me about today's emails or any important messages!".to_string()
}

/// Detect the intent of a chat query
fn detect_intent(query: &str) -> QueryIntent {
    let q = query.to_lowercase();

    // Check for today's emails
    if q.contains("today") || q.contains("today's") {
        return QueryIntent::TodayEmails;
    }

    // Check for important/priority emails
    if q.contains("important") || q.contains("priority") || q.contains("urgent") {
        return QueryIntent::ImportantEmails;
    }

    // Check for explicit search queries
    if q.starts_with("search ") || q.starts_with("find ") || q.starts_with("from ") {
        let search_term = q
            .trim_start_matches("search ")
            .trim_start_matches("find ")
            .trim_start_matches("from ")
            .to_string();
        return QueryIntent::SearchEmails(search_term);
    }

    // Check for email-related keywords
    let email_keywords = ["email", "emails", "message", "messages", "inbox", "mail", "sent", "received", "unread"];
    if email_keywords.iter().any(|kw| q.contains(kw)) {
        return QueryIntent::GeneralEmailQuestion;
    }

    // Default to general chat
    QueryIntent::GeneralChat
}

/// Format email context for LLM consumption (compact, ~500 tokens max)
fn format_email_context(emails: &[EmailWithInsight], max_emails: usize) -> String {
    emails
        .iter()
        .take(max_emails)
        .map(|e| {
            let summary = e.summary.clone().unwrap_or_else(|| {
                // Truncate snippet if no summary
                let snippet = &e.snippet;
                if snippet.len() > 100 {
                    format!("{}...", &snippet[..100])
                } else {
                    snippet.clone()
                }
            });
            format!(
                "- From: {} | Subject: {} | Priority: {} | Summary: {}",
                e.from_name, e.subject, e.priority, summary
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[tauri::command]
pub async fn chat_query(
    db: State<'_, DbState>,
    query: String,
) -> Result<String, String> {
    // Handle identity queries without LLM
    if is_identity_query(&query) {
        return Ok(get_identity_response());
    }

    let intent = detect_intent(&query);

    // Get relevant emails based on intent
    let (emails, context_description) = {
        let db_lock = db.lock().unwrap();
        let database = db_lock.as_ref().ok_or("Database not initialized")?;

        match &intent {
            QueryIntent::TodayEmails => {
                let emails = database
                    .get_emails_from_today()
                    .map_err(|e: anyhow::Error| e.to_string())?;
                (emails, "today's emails")
            }
            QueryIntent::ImportantEmails => {
                let emails = database
                    .get_emails_by_priority(20, 0)
                    .map_err(|e: anyhow::Error| e.to_string())?;
                let high_priority: Vec<_> = emails
                    .into_iter()
                    .filter(|e| e.priority == "HIGH")
                    .collect();
                (high_priority, "high priority emails")
            }
            QueryIntent::SearchEmails(term) => {
                let emails = database
                    .search_emails(term, 10)
                    .map_err(|e: anyhow::Error| e.to_string())?;
                (emails, "search results")
            }
            QueryIntent::GeneralEmailQuestion => {
                let emails = database
                    .get_emails_by_priority(10, 0)
                    .map_err(|e: anyhow::Error| e.to_string())?;
                (emails, "recent emails")
            }
            QueryIntent::GeneralChat => {
                // No email context needed for general chat
                (vec![], "")
            }
        }
    };

    // Prepare email context if we have emails
    let email_context = if !emails.is_empty() {
        Some(format!(
            "Found {} {}:\n{}",
            emails.len(),
            context_description,
            format_email_context(&emails, 8)
        ))
    } else if !matches!(intent, QueryIntent::GeneralChat) {
        // Email query but no results
        return Ok(match intent {
            QueryIntent::TodayEmails => "You haven't received any emails today yet.".to_string(),
            QueryIntent::ImportantEmails => "You don't have any high priority emails right now.".to_string(),
            QueryIntent::SearchEmails(term) => format!("I couldn't find any emails matching '{}'.", term),
            _ => "I couldn't find any relevant emails.".to_string(),
        });
    } else {
        None
    };

    // Try to use LLM for response
    let summarizer_guard = SUMMARIZER.lock().unwrap();
    if let Some(summarizer) = summarizer_guard.as_ref() {
        if summarizer.is_model_loaded() {
            // Use LLM for intelligent response
            match summarizer.chat(&query, email_context.as_deref()) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    eprintln!("[Chat] LLM error: {}", e);
                    // Fall through to fallback
                }
            }
        }
    }
    drop(summarizer_guard);

    // Fallback: return formatted email list or helpful message
    if let Some(ctx) = email_context {
        Ok(format!(
            "Here's what I found:\n\n{}\n\n(Note: AI model not loaded for detailed analysis)",
            ctx
        ))
    } else {
        Ok("I'm your email assistant! I can help you find and understand your emails. Try asking about today's emails, important messages, or search for specific topics.".to_string())
    }
}
