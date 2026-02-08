//! Embedding engine using Candle with Metal acceleration
//!
//! Generates text embeddings using sentence transformer models for semantic search.

use anyhow::{anyhow, Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{Repo, RepoType};
use std::path::{Path, PathBuf};
use tokenizers::Tokenizer;

/// Default embedding model - small and fast
pub const DEFAULT_EMBEDDING_MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2";
pub const EMBEDDING_DIMENSIONS: usize = 384;

/// Files needed for the embedding model
const MODEL_FILES: [&str; 3] = ["config.json", "tokenizer.json", "model.safetensors"];

/// Embedding engine for generating text embeddings
pub struct EmbeddingEngine {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    model_id: String,
}

/// Get the custom cache directory for embedding model files
fn get_custom_cache_dir(model_id: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let cache_dir = PathBuf::from(home)
        .join(".cache")
        .join("inboxed")
        .join("embedding_models")
        .join(model_id.replace('/', "--"));
    Ok(cache_dir)
}

/// Check if model files exist in custom cache
fn check_custom_cache(model_id: &str) -> Option<(PathBuf, PathBuf, PathBuf)> {
    let cache_dir = get_custom_cache_dir(model_id).ok()?;
    let config = cache_dir.join("config.json");
    let tokenizer = cache_dir.join("tokenizer.json");
    let weights = cache_dir.join("model.safetensors");
    if config.exists() && tokenizer.exists() && weights.exists() {
        Some((config, tokenizer, weights))
    } else {
        None
    }
}

/// Check if model files exist in hf-hub cache
fn check_hf_cache(model_id: &str) -> Option<(PathBuf, PathBuf, PathBuf)> {
    let cache = hf_hub::Cache::default();
    let repo = cache.repo(Repo::new(model_id.to_string(), RepoType::Model));
    if let (Some(c), Some(t), Some(w)) = (
        repo.get("config.json"),
        repo.get("tokenizer.json"),
        repo.get("model.safetensors"),
    ) {
        Some((c, t, w))
    } else {
        None
    }
}

/// Download embedding model files directly via HTTP from HuggingFace CDN.
/// Falls back from hf-hub API to direct HTTP download.
pub async fn download_embedding_model(
    model_id: Option<&str>,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let model_id = model_id.unwrap_or(DEFAULT_EMBEDDING_MODEL);

    // 1. Check hf-hub cache
    if let Some(paths) = check_hf_cache(model_id) {
        eprintln!("Using hf-hub cached embedding model");
        return Ok(paths);
    }

    // 2. Check custom cache
    if let Some(paths) = check_custom_cache(model_id) {
        eprintln!("Using custom cached embedding model");
        return Ok(paths);
    }

    // 3. Try hf-hub API download first
    eprintln!("Attempting hf-hub API download for embedding model...");
    match try_hf_hub_download(model_id) {
        Ok(paths) => return Ok(paths),
        Err(e) => {
            eprintln!(
                "hf-hub download failed ({}), falling back to direct HTTP...",
                e
            );
        }
    }

    // 4. Direct HTTP download from HuggingFace CDN
    eprintln!("Downloading embedding model via direct HTTP...");
    let cache_dir = get_custom_cache_dir(model_id)?;
    std::fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Failed to create cache dir: {}", cache_dir.display()))?;

    let base_url = format!("https://huggingface.co/{}/resolve/main", model_id);
    let client = reqwest::Client::new();

    for filename in &MODEL_FILES {
        let dest = cache_dir.join(filename);
        if dest.exists() {
            eprintln!("  {} already downloaded", filename);
            continue;
        }

        let url = format!("{}/{}", base_url, filename);
        eprintln!("  Downloading {}...", filename);

        let response = client
            .get(&url)
            .header("User-Agent", "inboxed-email-client/0.1")
            .send()
            .await
            .with_context(|| format!("HTTP request failed for {}", filename))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "HTTP {} downloading {} from {}",
                response.status(),
                filename,
                url
            ));
        }

        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("Failed to read response body for {}", filename))?;

        std::fs::write(&dest, &bytes)
            .with_context(|| format!("Failed to write {}", dest.display()))?;

        eprintln!("  Downloaded {} ({:.2} MB)", filename, bytes.len() as f64 / 1_048_576.0);
    }

    eprintln!("Embedding model download complete");
    Ok((
        cache_dir.join("config.json"),
        cache_dir.join("tokenizer.json"),
        cache_dir.join("model.safetensors"),
    ))
}

/// Try downloading via hf-hub crate API (sync)
fn try_hf_hub_download(model_id: &str) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let api = hf_hub::api::sync::Api::new()?;
    let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));

    let c = repo.get("config.json")?;
    let t = repo.get("tokenizer.json")?;
    let w = repo.get("model.safetensors")?;
    Ok((c, t, w))
}

impl EmbeddingEngine {
    /// Create a new embedding engine from pre-downloaded file paths
    pub fn from_paths(
        model_id: &str,
        config_path: &Path,
        tokenizer_path: &Path,
        weights_path: &Path,
    ) -> Result<Self> {
        let config_str = std::fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        // Try Metal first, fall back to CPU if forward pass fails
        let metal_device = Device::new_metal(0).ok();

        if let Some(ref device) = metal_device {
            eprintln!("[RAG] Attempting Metal GPU for embedding model '{}'", model_id);
            let vb = unsafe {
                VarBuilder::from_mmaped_safetensors(&[weights_path.to_path_buf()], DTYPE, device)?
            };
            let model = BertModel::load(vb, &config)?;

            // Test with a short forward pass to verify Metal works for this model
            let test_input = Tensor::zeros((1, 1), candle_core::DType::U32, device)?;
            match model.forward(&test_input, &test_input, None) {
                Ok(_) => {
                    eprintln!("[RAG] Metal GPU works, using Metal for embeddings");
                    return Ok(Self {
                        model,
                        tokenizer,
                        device: device.clone(),
                        model_id: model_id.to_string(),
                    });
                }
                Err(e) => {
                    eprintln!("[RAG] Metal forward pass failed ({}), falling back to CPU", e);
                }
            }
        }

        // CPU fallback
        let device = Device::Cpu;
        eprintln!("[RAG] Loading embedding model '{}' on CPU", model_id);

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path.to_path_buf()], DTYPE, &device)?
        };
        let model = BertModel::load(vb, &config)?;

        eprintln!("[RAG] Embedding model loaded successfully on CPU");

        Ok(Self {
            model,
            tokenizer,
            device,
            model_id: model_id.to_string(),
        })
    }

    /// Create a new embedding engine, downloading the model if necessary (sync, uses cache only)
    pub fn new(model_id: Option<&str>) -> Result<Self> {
        let model_id = model_id.unwrap_or(DEFAULT_EMBEDDING_MODEL);

        // Try hf-hub cache, then custom cache
        let (config_path, tokenizer_path, weights_path) = check_hf_cache(model_id)
            .or_else(|| check_custom_cache(model_id))
            .ok_or_else(|| {
                anyhow!(
                    "Embedding model not found in cache. Use init_rag to download it first."
                )
            })?;

        Self::from_paths(model_id, &config_path, &tokenizer_path, &weights_path)
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

/// Check if the embedding model is downloaded (local cache only, no network)
pub fn is_model_downloaded(model_id: Option<&str>) -> bool {
    let model_id = model_id.unwrap_or(DEFAULT_EMBEDDING_MODEL);
    check_hf_cache(model_id).is_some() || check_custom_cache(model_id).is_some()
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
