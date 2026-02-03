use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use hf_hub::api::sync::Api;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Available model options for users to choose from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOption {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub filename: String,
    pub size_mb: u32,
    pub description: String,
    pub min_ram_gb: u32,
    pub tokens_per_sec: String,
}

/// Get available models based on system specs
pub fn get_available_models() -> Vec<ModelOption> {
    vec![
        ModelOption {
            id: "lfm2.5-1.2b-q4".to_string(),
            name: "LFM2.5 1.2B (Recommended)".to_string(),
            repo: "LiquidAI/LFM2.5-1.2B-Instruct-GGUF".to_string(),
            filename: "LFM2.5-1.2B-Instruct-Q4_K_M.gguf".to_string(),
            size_mb: 731,
            description: "Fastest, most efficient. Great for email tasks.".to_string(),
            min_ram_gb: 2,
            tokens_per_sec: "200+ tok/s".to_string(),
        },
        ModelOption {
            id: "lfm2.5-1.2b-q8".to_string(),
            name: "LFM2.5 1.2B High Quality".to_string(),
            repo: "LiquidAI/LFM2.5-1.2B-Instruct-GGUF".to_string(),
            filename: "LFM2.5-1.2B-Instruct-Q8_0.gguf".to_string(),
            size_mb: 1250,
            description: "Higher quality, still very fast.".to_string(),
            min_ram_gb: 4,
            tokens_per_sec: "150+ tok/s".to_string(),
        },
        ModelOption {
            id: "qwen2.5-3b-q4".to_string(),
            name: "Qwen 2.5 3B".to_string(),
            repo: "Qwen/Qwen2.5-3B-Instruct-GGUF".to_string(),
            filename: "qwen2.5-3b-instruct-q4_k_m.gguf".to_string(),
            size_mb: 2000,
            description: "Larger model, better reasoning.".to_string(),
            min_ram_gb: 8,
            tokens_per_sec: "70-90 tok/s".to_string(),
        },
    ]
}

/// Default model - LFM2.5 1.2B is the recommended choice
pub const DEFAULT_MODEL_REPO: &str = "LiquidAI/LFM2.5-1.2B-Instruct-GGUF";
pub const DEFAULT_MODEL_FILE: &str = "LFM2.5-1.2B-Instruct-Q4_K_M.gguf";

/// Model download status
#[derive(Debug, Clone, PartialEq)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading { progress: f32 },
    Downloaded,
    Loading,
    Ready,
    Error(String),
}

impl Default for ModelStatus {
    fn default() -> Self {
        Self::NotDownloaded
    }
}

/// Manages model downloading and caching
pub struct ModelManager {
    models_dir: PathBuf,
    status: Arc<RwLock<ModelStatus>>,
}

impl ModelManager {
    /// Create a new ModelManager
    pub fn new() -> Result<Self> {
        let models_dir = Self::get_models_dir()?;

        // Ensure models directory exists
        std::fs::create_dir_all(&models_dir)?;

        Ok(Self {
            models_dir,
            status: Arc::new(RwLock::new(ModelStatus::default())),
        })
    }

    /// Get the models directory path
    fn get_models_dir() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "inboxed", "inboxed") {
            let data_dir = proj_dirs.data_dir();
            Ok(data_dir.join("models"))
        } else {
            // Fallback to home directory
            let home = std::env::var("HOME").map_err(|_| anyhow!("HOME not set"))?;
            Ok(PathBuf::from(home)
                .join(".inboxed")
                .join("models"))
        }
    }

    /// Get the full path to a model file
    pub fn get_model_path(&self, model_file: &str) -> PathBuf {
        self.models_dir.join(model_file)
    }

    /// Check if a model is downloaded
    pub fn is_model_downloaded(&self, model_file: &str) -> bool {
        let path = self.get_model_path(model_file);
        path.exists() && path.is_file()
    }

    /// Get current model status
    pub async fn get_status(&self) -> ModelStatus {
        self.status.read().await.clone()
    }

    /// Set model status
    pub async fn set_status(&self, status: ModelStatus) {
        *self.status.write().await = status;
    }

    /// Download a model from HuggingFace
    /// Returns the path to the downloaded model file
    pub fn download_model<F>(
        &self,
        repo_id: &str,
        filename: &str,
        on_progress: F,
    ) -> Result<PathBuf>
    where
        F: Fn(f32) + Send + 'static,
    {
        let target_path = self.get_model_path(filename);

        // Check if already downloaded
        if target_path.exists() {
            on_progress(100.0);
            return Ok(target_path);
        }

        // Download from HuggingFace
        let api = Api::new()?;
        let repo = api.model(repo_id.to_string());

        // Note: hf-hub doesn't provide progress callbacks directly,
        // so we report progress at key stages
        on_progress(0.0);

        // Download the file (this blocks until complete)
        let downloaded_path = repo.get(filename)?;

        on_progress(90.0);

        // Copy to our models directory if not already there
        if downloaded_path != target_path {
            std::fs::copy(&downloaded_path, &target_path)?;
        }

        on_progress(100.0);

        Ok(target_path)
    }

    /// Download the default model
    pub fn download_default_model<F>(&self, on_progress: F) -> Result<PathBuf>
    where
        F: Fn(f32) + Send + 'static,
    {
        self.download_model(DEFAULT_MODEL_REPO, DEFAULT_MODEL_FILE, on_progress)
    }

    /// Get path to default model (if it exists)
    pub fn get_default_model_path(&self) -> Option<PathBuf> {
        let path = self.get_model_path(DEFAULT_MODEL_FILE);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Check if default model is downloaded
    pub fn is_default_model_downloaded(&self) -> bool {
        self.is_model_downloaded(DEFAULT_MODEL_FILE)
    }

    /// Get the models directory
    pub fn models_dir(&self) -> &PathBuf {
        &self.models_dir
    }

    /// Find any downloaded model (checks all known models)
    pub fn find_any_downloaded_model(&self) -> Option<(ModelOption, PathBuf)> {
        for model in get_available_models() {
            let path = self.get_model_path(&model.filename);
            if path.exists() && path.is_file() {
                return Some((model, path));
            }
        }
        None
    }

    /// Get model by ID
    pub fn get_model_by_id(&self, model_id: &str) -> Option<ModelOption> {
        get_available_models()
            .into_iter()
            .find(|m| m.id == model_id)
    }

    /// Download a specific model by ID
    pub fn download_model_by_id<F>(&self, model_id: &str, on_progress: F) -> Result<PathBuf>
    where
        F: Fn(f32) + Send + 'static,
    {
        let model = self
            .get_model_by_id(model_id)
            .ok_or_else(|| anyhow!("Unknown model: {}", model_id))?;

        self.download_model(&model.repo, &model.filename, on_progress)
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ModelManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_models_dir_creation() {
        let manager = ModelManager::new().unwrap();
        assert!(manager.models_dir().exists());
    }

    #[test]
    fn test_model_path() {
        let manager = ModelManager::new().unwrap();
        let path = manager.get_model_path("test.gguf");
        assert!(path.ends_with("test.gguf"));
    }
}
