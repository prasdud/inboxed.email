//! RAG (Retrieval-Augmented Generation) commands
//!
//! Tauri commands for embedding generation, semantic search, and contextual AI chat.

use crate::db::vector_db::{EmbeddingStatus, VectorDatabase};
use crate::llm::embeddings::EmbeddingEngine;
use crate::llm::rag::{calculate_text_hash, prepare_email_text, RagEngine};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

lazy_static! {
    static ref RAG_ENGINE: Mutex<Option<RagEngine>> = Mutex::new(None);
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

    // Try to initialize embedding engine (may take time for first model download)
    match EmbeddingEngine::new(None) {
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

            Ok(true)
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize embedding engine: {}", e);
            Ok(false)
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

    // Get unembedded email IDs
    let unembedded_ids = vector_db
        .get_unembedded_email_ids(1000)
        .map_err(|e| format!("Failed to get unembedded emails: {}", e))?;

    if unembedded_ids.is_empty() {
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
        if let Ok(Some(email)) = email_db.get_email_by_id(&email_id) {
            let body = email.body_plain.as_deref().unwrap_or("");
            let text = prepare_email_text(&email.subject, &email.from_email, body);
            let text_hash = calculate_text_hash(&text);

            // Generate embedding
            if let Ok(embedding) = embedding_engine.embed(&text) {
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
        }
    }

    // Update final status
    vector_db
        .update_embedding_status(false, Some(total), Some(embedded_count), None, None)
        .map_err(|e| format!("Failed to update status: {}", e))?;

    // Emit completion event
    let _ = app.emit("embedding:complete", embedded_count);

    Ok(embedded_count)
}

/// Semantic search for emails
#[tauri::command]
pub fn search_emails_semantic(query: String, limit: usize) -> Result<Vec<SearchResult>, String> {
    let rag_guard = RAG_ENGINE.lock().unwrap();
    let rag = rag_guard.as_ref().ok_or("RAG engine not initialized")?;

    let similar = rag
        .search_similar(&query, limit, None)
        .map_err(|e| format!("Failed to search: {}", e))?;

    // Convert to SearchResult (email metadata can be fetched separately)
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
pub fn chat_with_context(query: String, limit: usize) -> Result<String, String> {
    // This would integrate with the existing summarizer for contextual responses
    // For now, we return searched email IDs as context
    let results = search_emails_semantic(query.clone(), limit)?;

    if results.is_empty() {
        return Ok(format!("No relevant emails found for: {}", query));
    }

    let email_ids: Vec<String> = results.iter().map(|r| r.email_id.clone()).collect();
    Ok(format!(
        "Found {} relevant emails: {}",
        results.len(),
        email_ids.join(", ")
    ))
}
