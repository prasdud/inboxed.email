//! RAG (Retrieval-Augmented Generation) commands
//!
//! Tauri commands for embedding generation, semantic search, and contextual AI chat.

use crate::db::vector_db::{EmbeddingStatus, VectorDatabase};
use crate::llm::embeddings::{self, EmbeddingEngine, DEFAULT_EMBEDDING_MODEL};
use crate::llm::rag::{calculate_text_hash, prepare_email_text, RagEngine};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

lazy_static! {
    pub static ref RAG_ENGINE: Mutex<Option<RagEngine>> = Mutex::new(None);
    static ref EMBEDDING_ENGINE: Mutex<Option<Arc<EmbeddingEngine>>> = Mutex::new(None);
    static ref VECTOR_DB: Mutex<Option<Arc<VectorDatabase>>> = Mutex::new(None);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub email_id: String,
    pub similarity: f32,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingProgress {
    pub total: i64,
    pub embedded: i64,
    pub current_email_id: Option<String>,
}

/// Initialize the RAG system (embedding engine + vector database)
#[tauri::command]
pub async fn init_rag(app: AppHandle) -> Result<bool, String> {
    eprintln!("[RAG] Initializing RAG system...");

    // Skip if already initialized
    {
        let guard = RAG_ENGINE.lock().unwrap();
        if guard.as_ref().map(|r| r.is_initialized()).unwrap_or(false) {
            eprintln!("[RAG] RAG system already initialized, skipping");
            return Ok(true);
        }
    }

    // Get app data directory for vector database
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    let db_path = app_data_dir.join("email_vectors.db");

    // Initialize vector database
    let vector_db = Arc::new(
        VectorDatabase::new(db_path)
            .map_err(|e| format!("Failed to create vector database: {}", e))?,
    );

    // Store vector db
    {
        let mut db_guard = VECTOR_DB.lock().unwrap();
        *db_guard = Some(vector_db.clone());
    }

    // Download embedding model (async, with direct HTTP fallback)
    let (config_path, tokenizer_path, weights_path) =
        embeddings::download_embedding_model(None)
            .await
            .map_err(|e| format!("Failed to download embedding model: {}", e))?;

    eprintln!("[RAG] Embedding model files ready");

    // Load embedding engine from downloaded paths
    match EmbeddingEngine::from_paths(
        DEFAULT_EMBEDDING_MODEL,
        &config_path,
        &tokenizer_path,
        &weights_path,
    ) {
        Ok(engine) => {
            let engine = Arc::new(engine);
            {
                let mut engine_guard = EMBEDDING_ENGINE.lock().unwrap();
                *engine_guard = Some(engine.clone());
            }

            // Initialize RAG engine
            let mut rag = RagEngine::new();
            rag.init(engine, vector_db);

            {
                let mut rag_guard = RAG_ENGINE.lock().unwrap();
                *rag_guard = Some(rag);
            }

            eprintln!("[RAG] RAG system initialized successfully");
            Ok(true)
        }
        Err(e) => {
            Err(format!("Failed to initialize embedding engine: {}", e))
        }
    }
}

/// Check if RAG is initialized
#[tauri::command]
pub fn is_rag_ready() -> bool {
    let rag_guard = RAG_ENGINE.lock().unwrap();
    rag_guard
        .as_ref()
        .map(|r| r.is_initialized())
        .unwrap_or(false)
}

/// Check if the embedding model is downloaded
#[tauri::command]
pub fn is_embedding_model_downloaded() -> bool {
    crate::llm::embeddings::is_model_downloaded(None)
}

/// Get embedding status
#[tauri::command]
pub fn get_embedding_status() -> Result<EmbeddingStatus, String> {
    let db_guard = VECTOR_DB.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Vector database not initialized")?;

    db.get_embedding_status()
        .map_err(|e| format!("Failed to get embedding status: {}", e))
}

/// Embed a single email
#[tauri::command]
pub fn embed_email(
    email_id: String,
    subject: String,
    from: String,
    body: String,
) -> Result<(), String> {
    let rag_guard = RAG_ENGINE.lock().unwrap();
    let rag = rag_guard.as_ref().ok_or("RAG engine not initialized")?;

    let text = prepare_email_text(&subject, &from, &body);
    let text_hash = calculate_text_hash(&text);

    // Check if already embedded with same hash
    if let Some(vector_db) = rag.vector_db() {
        if vector_db
            .has_embedding(&email_id, &text_hash)
            .unwrap_or(false)
        {
            return Ok(()); // Already embedded
        }
    }

    rag.store_email_embedding(&email_id, &text, &text_hash)
        .map_err(|e| format!("Failed to embed email: {}", e))
}

/// Embed all unembedded emails (batch operation)
#[tauri::command]
pub async fn embed_all_emails(app: AppHandle) -> Result<i64, String> {
    // Get email database to fetch emails
    let email_db = crate::db::EmailDatabase::new(
        app.path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?
            .join("emails.db"),
    )
    .map_err(|e| format!("Failed to open email database: {}", e))?;

    let vector_db = {
        let db_guard = VECTOR_DB.lock().unwrap();
        db_guard.clone().ok_or("Vector database not initialized")?
    };

    let embedding_engine = {
        let engine_guard = EMBEDDING_ENGINE.lock().unwrap();
        engine_guard
            .clone()
            .ok_or("Embedding engine not initialized")?
    };

    // Get all email IDs from the email database, then filter out already-embedded ones
    let all_email_ids = email_db
        .get_all_email_ids(1000)
        .map_err(|e| format!("Failed to get email IDs: {}", e))?;

    eprintln!("[RAG] Found {} email IDs in email DB", all_email_ids.len());

    let embedded_ids = vector_db
        .get_embedded_email_ids()
        .map_err(|e| format!("Failed to get embedded email IDs: {}", e))?;

    eprintln!("[RAG] Already embedded: {}", embedded_ids.len());

    let unembedded_ids: Vec<String> = all_email_ids
        .into_iter()
        .filter(|id| !embedded_ids.contains(id))
        .collect();

    eprintln!("[RAG] Unembedded emails to process: {}", unembedded_ids.len());

    if unembedded_ids.is_empty() {
        eprintln!("[RAG] All emails already embedded, nothing to do");
        return Ok(0);
    }

    let total = unembedded_ids.len() as i64;

    // Update status
    vector_db
        .update_embedding_status(
            true,
            Some(total),
            Some(0),
            Some(embedding_engine.model_id()),
            None,
        )
        .map_err(|e| format!("Failed to update status: {}", e))?;

    let mut embedded_count = 0i64;

    for email_id in unembedded_ids {
        // Get email content
        match email_db.get_email_by_id(&email_id) {
            Ok(Some(email)) => {
                let body = email.body_plain.as_deref().unwrap_or("");
                let text = prepare_email_text(&email.subject, &email.from_email, body);
                let text_hash = calculate_text_hash(&text);

                // Generate embedding
                match embedding_engine.embed(&text) {
                    Ok(embedding) => {
                        let email_embedding = crate::db::vector_db::EmailEmbedding {
                            email_id: email_id.clone(),
                            embedding,
                            embedding_model: embedding_engine.model_id().to_string(),
                            text_hash,
                            created_at: chrono::Utc::now().timestamp(),
                        };

                        if vector_db.store_embedding(&email_embedding).is_ok() {
                            embedded_count += 1;

                            // Emit progress event
                            let _ = app.emit(
                                "embedding:progress",
                                EmbeddingProgress {
                                    total,
                                    embedded: embedded_count,
                                    current_email_id: Some(email_id),
                                },
                            );

                            // Update status periodically
                            if embedded_count % 10 == 0 {
                                let _ = vector_db.update_embedding_status(
                                    true,
                                    Some(total),
                                    Some(embedded_count),
                                    None,
                                    None,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[RAG] Failed to embed email {}: {}", email_id, e);
                    }
                }
            }
            Ok(None) => {
                eprintln!("[RAG] Email {} not found in DB, skipping", email_id);
            }
            Err(e) => {
                eprintln!("[RAG] Failed to fetch email {}: {}", email_id, e);
            }
        }
    }

    // Update final status
    vector_db
        .update_embedding_status(false, Some(total), Some(embedded_count), None, None)
        .map_err(|e| format!("Failed to update status: {}", e))?;

    eprintln!("[RAG] Embedding complete: {}/{} emails embedded", embedded_count, total);

    // Emit completion event
    let _ = app.emit("embedding:complete", embedded_count);

    Ok(embedded_count)
}

/// Semantic search for emails
#[tauri::command]
pub fn search_emails_semantic(
    app: AppHandle,
    query: String,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    // Step 1: Lock RAG_ENGINE, perform search, drop lock
    let similar = {
        let rag_guard = RAG_ENGINE.lock().unwrap();
        let rag = rag_guard.as_ref().ok_or("RAG engine not initialized")?;
        rag.search_similar(&query, limit, None)
            .map_err(|e| format!("Failed to search: {}", e))?
    };

    // Step 2: Open EmailDatabase to enrich results with metadata
    let email_db = crate::db::EmailDatabase::new(
        app.path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?
            .join("emails.db"),
    )
    .map_err(|e| format!("Failed to open email database: {}", e))?;

    let results: Vec<SearchResult> = similar
        .into_iter()
        .map(|s| {
            let (subject, from, snippet) =
                if let Ok(Some(email)) = email_db.get_email_by_id(&s.email_id) {
                    (
                        Some(email.subject),
                        Some(email.from),
                        Some(email.snippet),
                    )
                } else {
                    (None, None, None)
                };
            SearchResult {
                email_id: s.email_id,
                similarity: s.similarity,
                subject,
                from,
                snippet,
            }
        })
        .collect();

    Ok(results)
}

/// Find emails similar to a given email
#[tauri::command]
pub fn find_similar_emails(email_id: String, limit: usize) -> Result<Vec<SearchResult>, String> {
    let rag_guard = RAG_ENGINE.lock().unwrap();
    let rag = rag_guard.as_ref().ok_or("RAG engine not initialized")?;

    let vector_db = rag.vector_db().ok_or("Vector database not initialized")?;

    // Get embedding for the source email
    let embedding = vector_db
        .get_embedding(&email_id)
        .map_err(|e| format!("Failed to get embedding: {}", e))?
        .ok_or("Email not embedded")?;

    // Search for similar (excluding the source email)
    let similar = vector_db
        .search_similar(&embedding.embedding, limit, Some(&email_id))
        .map_err(|e| format!("Failed to search: {}", e))?;

    let results: Vec<SearchResult> = similar
        .into_iter()
        .map(|s| SearchResult {
            email_id: s.email_id,
            similarity: s.similarity,
            subject: None,
            from: None,
            snippet: None,
        })
        .collect();

    Ok(results)
}

/// Get count of embedded emails
#[tauri::command]
pub fn get_embedded_count() -> Result<i64, String> {
    let db_guard = VECTOR_DB.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Vector database not initialized")?;

    db.get_embedded_count()
        .map_err(|e| format!("Failed to get count: {}", e))
}

/// Clear all embeddings
#[tauri::command]
pub fn clear_embeddings() -> Result<(), String> {
    let db_guard = VECTOR_DB.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Vector database not initialized")?;

    db.clear_all_embeddings()
        .map_err(|e| format!("Failed to clear embeddings: {}", e))
}

/// Chat with RAG context
#[tauri::command]
pub fn chat_with_context(
    app: AppHandle,
    query: String,
    limit: usize,
) -> Result<String, String> {
    use crate::llm::rag::RetrievedContext;

    // Step 1: Lock RAG_ENGINE → semantic search → drop lock
    let similar = {
        let rag_guard = RAG_ENGINE.lock().unwrap();
        let rag = rag_guard.as_ref().ok_or("RAG engine not initialized")?;
        rag.search_similar(&query, limit, None)
            .map_err(|e| format!("Failed to search: {}", e))?
    };

    if similar.is_empty() {
        return Ok(format!("No relevant emails found for: {}", query));
    }

    // Step 2: Open EmailDatabase → fetch metadata → build RetrievedContext list
    let email_db = crate::db::EmailDatabase::new(
        app.path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?
            .join("emails.db"),
    )
    .map_err(|e| format!("Failed to open email database: {}", e))?;

    let contexts: Vec<RetrievedContext> = similar
        .into_iter()
        .filter_map(|s| {
            if let Ok(Some(email)) = email_db.get_email_by_id(&s.email_id) {
                let snippet = email
                    .body_plain
                    .as_deref()
                    .unwrap_or(&email.snippet)
                    .chars()
                    .take(200)
                    .collect::<String>();
                Some(RetrievedContext {
                    email_id: s.email_id,
                    subject: email.subject,
                    from: email.from,
                    snippet,
                    similarity: s.similarity,
                })
            } else {
                None
            }
        })
        .collect();

    if contexts.is_empty() {
        return Ok(format!("No relevant emails found for: {}", query));
    }

    // Build context string for the LLM
    let context_str = contexts
        .iter()
        .enumerate()
        .map(|(i, ctx)| {
            format!(
                "Email {}: From: {} | Subject: {} | {}",
                i + 1,
                ctx.from,
                ctx.subject,
                ctx.snippet
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Step 3: Lock SUMMARIZER → generate response → drop lock
    let summarizer_guard = crate::commands::ai::SUMMARIZER.lock().unwrap();
    if let Some(summarizer) = summarizer_guard.as_ref() {
        if summarizer.is_model_loaded() {
            match summarizer.chat(&query, Some(&context_str)) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    let err_msg = e.to_string();
                    eprintln!("[RAG Chat] LLM error: {}", err_msg);
                    drop(summarizer_guard);
                    return Ok(format!(
                        "Found {} relevant emails:\n\n{}\n\n(AI generation error: {})",
                        contexts.len(), context_str, err_msg
                    ));
                }
            }
        }
    }
    drop(summarizer_guard);

    // Fallback: model genuinely not loaded
    Ok(format!(
        "Found {} relevant emails:\n\n{}\n\n(AI model not loaded for detailed analysis)",
        contexts.len(),
        context_str
    ))
}
