//! Embedding engine using Candle with Metal acceleration
//!
//! Generates text embeddings using sentence transformer models for semantic search.

use anyhow::{anyhow, Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::path::PathBuf;
use tokenizers::Tokenizer;

/// Default embedding model - small and fast
pub const DEFAULT_EMBEDDING_MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2";
pub const EMBEDDING_DIMENSIONS: usize = 384;

/// Embedding engine for generating text embeddings
pub struct EmbeddingEngine {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    model_id: String,
}

impl EmbeddingEngine {
    /// Create a new embedding engine, downloading the model if necessary
    pub fn new(model_id: Option<&str>) -> Result<Self> {
        let model_id = model_id.unwrap_or(DEFAULT_EMBEDDING_MODEL);

        // Use Metal acceleration on macOS, fall back to CPU
        let device = Device::new_metal(0).unwrap_or(Device::Cpu);

        eprintln!("Loading embedding model '{}' on {:?}", model_id, device);

        // Download model files from HuggingFace
        let api = Api::new()?;
        let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));

        let config_path = repo
            .get("config.json")
            .with_context(|| format!("Failed to download config.json from {}", model_id))?;
        let tokenizer_path = repo
            .get("tokenizer.json")
            .with_context(|| format!("Failed to download tokenizer.json from {}", model_id))?;
        let weights_path = repo
            .get("model.safetensors")
            .with_context(|| format!("Failed to download model.safetensors from {}", model_id))?;

        // Load config
        let config_str = std::fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        // Load model weights
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)? };

        let model = BertModel::load(vb, &config)?;

        eprintln!("Embedding model loaded successfully");

        Ok(Self {
            model,
            tokenizer,
            device,
            model_id: model_id.to_string(),
        })
    }

    /// Generate embedding for a single text
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text])?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No embedding generated"))
    }

    /// Generate embeddings for multiple texts
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Tokenize all texts
        let encodings = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        // Find max length for padding
        let max_len = encodings
            .iter()
            .map(|e| e.get_ids().len())
            .max()
            .unwrap_or(0);

        // Build input tensors with padding
        let mut input_ids_vec = Vec::new();
        let mut attention_mask_vec = Vec::new();
        let mut token_type_ids_vec = Vec::new();

        for encoding in &encodings {
            let ids = encoding.get_ids();
            let attention = encoding.get_attention_mask();
            let type_ids = encoding.get_type_ids();

            // Pad to max length
            let mut ids_padded: Vec<u32> = ids.to_vec();
            let mut attention_padded: Vec<u32> = attention.to_vec();
            let mut type_ids_padded: Vec<u32> = type_ids.to_vec();

            while ids_padded.len() < max_len {
                ids_padded.push(0);
                attention_padded.push(0);
                type_ids_padded.push(0);
            }

            input_ids_vec.extend(ids_padded);
            attention_mask_vec.extend(attention_padded);
            token_type_ids_vec.extend(type_ids_padded);
        }

        let batch_size = texts.len();

        // Create tensors
        let input_ids = Tensor::from_vec(input_ids_vec, (batch_size, max_len), &self.device)?;
        let attention_mask = Tensor::from_vec(
            attention_mask_vec.clone(),
            (batch_size, max_len),
            &self.device,
        )?;
        let token_type_ids =
            Tensor::from_vec(token_type_ids_vec, (batch_size, max_len), &self.device)?;

        // Forward pass
        let embeddings = self
            .model
            .forward(&input_ids, &token_type_ids, Some(&attention_mask))?;

        // Mean pooling over sequence length (with attention mask)
        let attention_mask_expanded = attention_mask
            .unsqueeze(2)?
            .to_dtype(candle_core::DType::F32)?
            .broadcast_as(embeddings.shape())?;

        let masked_embeddings = embeddings.mul(&attention_mask_expanded)?;
        let sum_embeddings = masked_embeddings.sum(1)?;

        // Create attention sum for division
        let attention_sum = Tensor::from_vec(
            attention_mask_vec
                .iter()
                .map(|&x| x as f32)
                .collect::<Vec<_>>(),
            (batch_size, max_len),
            &self.device,
        )?
        .sum(1)?
        .unsqueeze(1)?
        .broadcast_as(sum_embeddings.shape())?;

        let mean_embeddings = sum_embeddings.div(&attention_sum)?;

        // Normalize embeddings (L2 normalization)
        let norms = mean_embeddings
            .sqr()?
            .sum(1)?
            .sqrt()?
            .unsqueeze(1)?
            .broadcast_as(mean_embeddings.shape())?;
        let normalized = mean_embeddings.div(&norms)?;

        // Convert to Vec<Vec<f32>>
        let normalized_cpu = normalized.to_device(&Device::Cpu)?;
        let flat: Vec<f32> = normalized_cpu
            .to_vec2::<f32>()?
            .into_iter()
            .flatten()
            .collect();

        let embedding_dim = flat.len() / batch_size;
        let result: Vec<Vec<f32>> = flat
            .chunks(embedding_dim)
            .map(|chunk| chunk.to_vec())
            .collect();

        Ok(result)
    }

    /// Get the model ID
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Get the device being used
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get embedding dimensions
    pub fn dimensions(&self) -> usize {
        EMBEDDING_DIMENSIONS
    }
}

/// Check if the embedding model is downloaded
pub fn is_model_downloaded(model_id: Option<&str>) -> bool {
    let model_id = model_id.unwrap_or(DEFAULT_EMBEDDING_MODEL);

    if let Ok(api) = Api::new() {
        let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));
        // Check if model files exist in cache
        repo.get("config.json").is_ok()
            && repo.get("tokenizer.json").is_ok()
            && repo.get("model.safetensors").is_ok()
    } else {
        false
    }
}

/// Get the cache path for embedding models
pub fn get_embedding_cache_path() -> Result<PathBuf> {
    let cache = hf_hub::Cache::default();
    Ok(cache.path().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dimensions() {
        assert_eq!(EMBEDDING_DIMENSIONS, 384);
    }

    // Integration test - requires model download
    #[test]
    #[ignore]
    fn test_embedding_generation() {
        let engine = EmbeddingEngine::new(None).unwrap();
        let embedding = engine.embed("Hello, world!").unwrap();
        assert_eq!(embedding.len(), EMBEDDING_DIMENSIONS);

        // Check normalization (should be unit vector)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[test]
    #[ignore]
    fn test_batch_embedding() {
        let engine = EmbeddingEngine::new(None).unwrap();
        let texts = ["Hello, world!", "How are you?", "Rust is great!"];
        let embeddings = engine.embed_batch(&texts).unwrap();

        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            assert_eq!(emb.len(), EMBEDDING_DIMENSIONS);
        }
    }
}
