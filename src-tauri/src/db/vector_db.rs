//! Vector database operations for email embeddings
//!
//! Provides storage and retrieval of email embeddings for RAG functionality.

use anyhow::{Context, Result as AnyhowResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::schema::create_tables;

/// Embedding dimensions (all-MiniLM-L6-v2 produces 384-dim vectors)
pub const EMBEDDING_DIMENSIONS: usize = 384;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailEmbedding {
    pub email_id: String,
    pub embedding: Vec<f32>,
    pub embedding_model: String,
    pub text_hash: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStatus {
    pub is_embedding: bool,
    pub total_emails: i64,
    pub embedded_emails: i64,
    pub current_model: Option<String>,
    pub last_embedded_at: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarEmail {
    pub email_id: String,
    pub similarity: f32,
}

pub struct VectorDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl VectorDatabase {
    pub fn new(db_path: PathBuf) -> AnyhowResult<Self> {
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;

        create_tables(&conn).context("Failed to create tables")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Store an embedding for an email
    pub fn store_embedding(&self, embedding: &EmailEmbedding) -> AnyhowResult<()> {
        let conn = self.conn.lock().unwrap();

        // Serialize embedding to bytes
        let embedding_bytes = embedding_to_bytes(&embedding.embedding)?;

        conn.execute(
            "INSERT OR REPLACE INTO email_embeddings (email_id, embedding, embedding_model, text_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                embedding.email_id,
                embedding_bytes,
                embedding.embedding_model,
                embedding.text_hash,
                embedding.created_at,
            ],
        )?;

        Ok(())
    }

    /// Get embedding for a specific email
    pub fn get_embedding(&self, email_id: &str) -> AnyhowResult<Option<EmailEmbedding>> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT email_id, embedding, embedding_model, text_hash, created_at FROM email_embeddings WHERE email_id = ?1",
            params![email_id],
            |row| {
                let embedding_bytes: Vec<u8> = row.get(1)?;
                Ok(EmailEmbedding {
                    email_id: row.get(0)?,
                    embedding: bytes_to_embedding(&embedding_bytes).unwrap_or_default(),
                    embedding_model: row.get(2)?,
                    text_hash: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        );

        match result {
            Ok(embedding) => Ok(Some(embedding)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all embeddings (for similarity search)
    pub fn get_all_embeddings(&self) -> AnyhowResult<Vec<EmailEmbedding>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT email_id, embedding, embedding_model, text_hash, created_at FROM email_embeddings",
        )?;

        let embeddings = stmt
            .query_map([], |row| {
                let embedding_bytes: Vec<u8> = row.get(1)?;
                Ok(EmailEmbedding {
                    email_id: row.get(0)?,
                    embedding: bytes_to_embedding(&embedding_bytes).unwrap_or_default(),
                    embedding_model: row.get(2)?,
                    text_hash: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(embeddings)
    }

    /// Find similar emails using cosine similarity
    pub fn search_similar(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        exclude_email_id: Option<&str>,
    ) -> AnyhowResult<Vec<SimilarEmail>> {
        let embeddings = self.get_all_embeddings()?;

        let mut similarities: Vec<SimilarEmail> = embeddings
            .iter()
            .filter(|e| {
                if let Some(exclude_id) = exclude_email_id {
                    e.email_id != exclude_id
                } else {
                    true
                }
            })
            .map(|e| SimilarEmail {
                email_id: e.email_id.clone(),
                similarity: cosine_similarity(query_embedding, &e.embedding),
            })
            .collect();

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // Take top K
        similarities.truncate(top_k);

        Ok(similarities)
    }

    /// Check if an email has an embedding with the given text hash
    pub fn has_embedding(&self, email_id: &str, text_hash: &str) -> AnyhowResult<bool> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM email_embeddings WHERE email_id = ?1 AND text_hash = ?2",
            params![email_id, text_hash],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Get count of embedded emails
    pub fn get_embedded_count(&self) -> AnyhowResult<i64> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM email_embeddings", [], |row| {
            row.get(0)
        })?;

        Ok(count)
    }

    /// Get email IDs that don't have embeddings
    pub fn get_unembedded_email_ids(&self, limit: i64) -> AnyhowResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT e.id FROM emails e 
             LEFT JOIN email_embeddings ee ON e.id = ee.email_id 
             WHERE ee.email_id IS NULL 
             LIMIT ?1",
        )?;

        let ids = stmt
            .query_map(params![limit], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Update embedding status
    pub fn update_embedding_status(
        &self,
        is_embedding: bool,
        total: Option<i64>,
        embedded: Option<i64>,
        model: Option<&str>,
        error: Option<&str>,
    ) -> AnyhowResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE embedding_status SET 
             is_embedding = ?1, 
             total_emails = COALESCE(?2, total_emails),
             embedded_emails = COALESCE(?3, embedded_emails),
             current_model = COALESCE(?4, current_model),
             error_message = ?5,
             last_embedded_at = CASE WHEN ?1 = 0 AND ?3 IS NOT NULL THEN strftime('%s', 'now') ELSE last_embedded_at END
             WHERE id = 1",
            params![
                is_embedding as i32,
                total,
                embedded,
                model,
                error,
            ],
        )?;

        Ok(())
    }

    /// Get embedding status
    pub fn get_embedding_status(&self) -> AnyhowResult<EmbeddingStatus> {
        let conn = self.conn.lock().unwrap();

        let status = conn.query_row(
            "SELECT is_embedding, total_emails, embedded_emails, current_model, last_embedded_at, error_message 
             FROM embedding_status WHERE id = 1",
            [],
            |row| {
                Ok(EmbeddingStatus {
                    is_embedding: row.get::<_, i32>(0)? != 0,
                    total_emails: row.get(1)?,
                    embedded_emails: row.get(2)?,
                    current_model: row.get(3)?,
                    last_embedded_at: row.get(4)?,
                    error_message: row.get(5)?,
                })
            },
        )?;

        Ok(status)
    }

    /// Delete embedding for an email
    pub fn delete_embedding(&self, email_id: &str) -> AnyhowResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM email_embeddings WHERE email_id = ?1",
            params![email_id],
        )?;
        Ok(())
    }

    /// Clear all embeddings
    pub fn clear_all_embeddings(&self) -> AnyhowResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM email_embeddings", [])?;
        conn.execute(
            "UPDATE embedding_status SET embedded_emails = 0, is_embedding = 0 WHERE id = 1",
            [],
        )?;
        Ok(())
    }
}

/// Convert f32 vector to bytes for storage
fn embedding_to_bytes(embedding: &[f32]) -> AnyhowResult<Vec<u8>> {
    let mut bytes = Vec::with_capacity(embedding.len() * 4);
    for &val in embedding {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    Ok(bytes)
}

/// Convert bytes back to f32 vector
fn bytes_to_embedding(bytes: &[u8]) -> AnyhowResult<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        anyhow::bail!("Invalid embedding byte length");
    }

    let mut embedding = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        embedding.push(val);
    }
    Ok(embedding)
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_serialization() {
        let embedding = vec![0.1, 0.2, 0.3, -0.5, 1.0];
        let bytes = embedding_to_bytes(&embedding).unwrap();
        let restored = bytes_to_embedding(&bytes).unwrap();

        assert_eq!(embedding.len(), restored.len());
        for (a, b) in embedding.iter().zip(restored.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 1e-6);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) + 1.0).abs() < 1e-6);
    }
}
