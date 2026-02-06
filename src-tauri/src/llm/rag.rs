//! RAG (Retrieval-Augmented Generation) engine
//!
//! Combines embedding-based retrieval with LLM generation for contextual responses.

use anyhow::{anyhow, Result};
use std::sync::Arc;

use super::embeddings::EmbeddingEngine;
use super::summarizer::Summarizer;
use crate::db::vector_db::{EmailEmbedding, SimilarEmail, VectorDatabase};

/// Context retrieved for RAG
#[derive(Debug, Clone)]
pub struct RetrievedContext {
    pub email_id: String,
    pub subject: String,
    pub from: String,
    pub snippet: String,
    pub similarity: f32,
}

/// RAG Engine combining retrieval and generation
pub struct RagEngine {
    embedding_engine: Option<Arc<EmbeddingEngine>>,
    vector_db: Option<Arc<VectorDatabase>>,
}

impl RagEngine {
    /// Create a new RAG engine
    pub fn new() -> Self {
        Self {
            embedding_engine: None,
            vector_db: None,
        }
    }

    /// Initialize with embedding engine and vector database
    pub fn init(&mut self, embedding_engine: Arc<EmbeddingEngine>, vector_db: Arc<VectorDatabase>) {
        self.embedding_engine = Some(embedding_engine);
        self.vector_db = Some(vector_db);
    }

    /// Check if the engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.embedding_engine.is_some() && self.vector_db.is_some()
    }

    /// Generate embedding for text
    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let engine = self
            .embedding_engine
            .as_ref()
            .ok_or_else(|| anyhow!("Embedding engine not initialized"))?;
        engine.embed(text)
    }

    /// Store embedding for an email
    pub fn store_email_embedding(&self, email_id: &str, text: &str, text_hash: &str) -> Result<()> {
        let engine = self
            .embedding_engine
            .as_ref()
            .ok_or_else(|| anyhow!("Embedding engine not initialized"))?;
        let vector_db = self
            .vector_db
            .as_ref()
            .ok_or_else(|| anyhow!("Vector database not initialized"))?;

        // Generate embedding
        let embedding = engine.embed(text)?;

        // Store in database
        let email_embedding = EmailEmbedding {
            email_id: email_id.to_string(),
            embedding,
            embedding_model: engine.model_id().to_string(),
            text_hash: text_hash.to_string(),
            created_at: chrono::Utc::now().timestamp(),
        };

        vector_db.store_embedding(&email_embedding)?;
        Ok(())
    }

    /// Search for similar emails
    pub fn search_similar(
        &self,
        query: &str,
        top_k: usize,
        exclude_email_id: Option<&str>,
    ) -> Result<Vec<SimilarEmail>> {
        let engine = self
            .embedding_engine
            .as_ref()
            .ok_or_else(|| anyhow!("Embedding engine not initialized"))?;
        let vector_db = self
            .vector_db
            .as_ref()
            .ok_or_else(|| anyhow!("Vector database not initialized"))?;

        // Generate query embedding
        let query_embedding = engine.embed(query)?;

        // Search in vector database
        let similar = vector_db.search_similar(&query_embedding, top_k, exclude_email_id)?;

        Ok(similar)
    }

    /// Build context string from similar emails for LLM
    pub fn build_context(&self, contexts: &[RetrievedContext], max_chars: usize) -> String {
        let mut context = String::new();
        let mut current_len = 0;

        for (i, ctx) in contexts.iter().enumerate() {
            let entry = format!(
                "Email {}: From: {} | Subject: {} | {}\n",
                i + 1,
                ctx.from,
                ctx.subject,
                ctx.snippet
            );

            if current_len + entry.len() > max_chars {
                break;
            }

            context.push_str(&entry);
            current_len += entry.len();
        }

        context
    }

    /// Generate a response using RAG
    pub fn generate_with_context(
        &self,
        summarizer: &Summarizer,
        query: &str,
        contexts: &[RetrievedContext],
    ) -> Result<String> {
        if contexts.is_empty() {
            return summarizer.chat(query, None);
        }

        let context_str = self.build_context(contexts, 2000);

        let prompt = format!(
            "Based on the following emails:\n{}\n\nAnswer the question: {}",
            context_str, query
        );

        summarizer.chat(&prompt, Some(&context_str))
    }

    /// Get the embedding engine
    pub fn embedding_engine(&self) -> Option<Arc<EmbeddingEngine>> {
        self.embedding_engine.clone()
    }

    /// Get the vector database
    pub fn vector_db(&self) -> Option<Arc<VectorDatabase>> {
        self.vector_db.clone()
    }
}

impl Default for RagEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Prepare email text for embedding (combine subject + body)
pub fn prepare_email_text(subject: &str, from: &str, body: &str) -> String {
    // Strip HTML and limit length
    let clean_body = strip_html(body);
    let truncated_body = truncate_text(&clean_body, 1000);

    format!(
        "From: {} Subject: {} Content: {}",
        from, subject, truncated_body
    )
}

/// Calculate text hash for change detection
pub fn calculate_text_hash(text: &str) -> String {
    format!("{:x}", md5::compute(text))
}

/// Strip HTML tags from text
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_style = false;
    let mut in_script = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                if html.contains("<style") {
                    in_style = true;
                }
                if html.contains("<script") {
                    in_script = true;
                }
            }
            '>' => {
                in_tag = false;
                if in_style && html.contains("</style>") {
                    in_style = false;
                }
                if in_script && html.contains("</script>") {
                    in_script = false;
                }
            }
            _ if !in_tag && !in_style && !in_script => {
                result.push(c);
            }
            _ => {}
        }
    }

    // Clean up whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Truncate text to max characters
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        text.to_string()
    } else {
        text.chars().take(max_chars).collect::<String>() + "..."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_email_text() {
        let text = prepare_email_text(
            "Meeting Tomorrow",
            "John Doe",
            "<html><body>Let's meet at 3pm</body></html>",
        );
        assert!(text.contains("Meeting Tomorrow"));
        assert!(text.contains("John Doe"));
        assert!(text.contains("meet at 3pm"));
    }

    #[test]
    fn test_strip_html() {
        let html = "<p>Hello <b>World</b></p>";
        let text = strip_html(html);
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_calculate_text_hash() {
        let hash1 = calculate_text_hash("hello");
        let hash2 = calculate_text_hash("hello");
        let hash3 = calculate_text_hash("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
