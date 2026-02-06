use crate::llm::{
    get_available_models, ModelManager, ModelOption, ModelStatus, Summarizer, DEFAULT_MODEL_FILE,
    DEFAULT_MODEL_REPO,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

lazy_static::lazy_static! {
    pub static ref SUMMARIZER: Mutex<Option<Summarizer>> = Mutex::new(None);
    static ref MODEL_MANAGER: Mutex<Option<ModelManager>> = Mutex::new(None);
    static ref CURRENT_MODEL_ID: Mutex<Option<String>> = Mutex::new(None);
    static ref MODEL_LOADING: Mutex<bool> = Mutex::new(false);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSummary {
    pub summary: String,
    pub insights: Vec<String>,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ModelStatusResponse {
    #[serde(rename = "not_downloaded")]
    NotDownloaded,
    #[serde(rename = "downloading")]
    Downloading { progress: f32 },
    #[serde(rename = "downloaded")]
    Downloaded,
    #[serde(rename = "loading")]
    Loading,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "error")]
    Error { message: String },
}

impl From<ModelStatus> for ModelStatusResponse {
    fn from(status: ModelStatus) -> Self {
        match status {
            ModelStatus::NotDownloaded => ModelStatusResponse::NotDownloaded,
            ModelStatus::Downloading { progress } => ModelStatusResponse::Downloading { progress },
            ModelStatus::Downloaded => ModelStatusResponse::Downloaded,
            ModelStatus::Loading => ModelStatusResponse::Loading,
            ModelStatus::Ready => ModelStatusResponse::Ready,
            ModelStatus::Error(message) => ModelStatusResponse::Error { message },
        }
    }
}

/// Initialize the model manager
fn ensure_model_manager() -> Result<(), String> {
    let mut guard = MODEL_MANAGER.lock().unwrap();
    if guard.is_none() {
        let manager = ModelManager::new().map_err(|e| e.to_string())?;
        *guard = Some(manager);
    }
    Ok(())
}

/// Get list of available models
#[tauri::command]
pub async fn get_available_ai_models() -> Result<Vec<ModelOption>, String> {
    Ok(get_available_models())
}

/// Check if the AI model is downloaded and ready
#[tauri::command]
pub async fn check_model_status() -> Result<ModelStatusResponse, String> {
    ensure_model_manager()?;

    // Check if loading is in progress
    {
        let loading_guard = MODEL_LOADING.lock().unwrap();
        if *loading_guard {
            println!("[AI] check_model_status: Loading in progress");
            return Ok(ModelStatusResponse::Loading);
        }
    }

    let guard = MODEL_MANAGER.lock().unwrap();
    let manager = guard.as_ref().ok_or("Model manager not initialized")?;

    // Check if any model is downloaded
    if manager.find_any_downloaded_model().is_some() {
        // Check if model is loaded
        let summarizer_guard = SUMMARIZER.lock().unwrap();
        if let Some(summarizer) = summarizer_guard.as_ref() {
            if summarizer.is_model_loaded() {
                println!("[AI] check_model_status: Ready");
                return Ok(ModelStatusResponse::Ready);
            }
        }
        println!("[AI] check_model_status: Downloaded but not loaded");
        Ok(ModelStatusResponse::Downloaded)
    } else {
        println!("[AI] check_model_status: Not downloaded");
        Ok(ModelStatusResponse::NotDownloaded)
    }
}

/// Check if the model is currently loading
#[tauri::command]
pub async fn is_model_loading() -> Result<bool, String> {
    let loading_guard = MODEL_LOADING.lock().unwrap();
    Ok(*loading_guard)
}

/// Download the default AI model from HuggingFace
#[tauri::command]
pub async fn download_model(app: AppHandle) -> Result<(), String> {
    ensure_model_manager()?;

    // Emit starting event
    app.emit("model:progress", 0.0f32)
        .map_err(|e| e.to_string())?;

    // Clone app handle for the closure
    let app_clone = app.clone();

    // Run download in blocking task
    let result = tokio::task::spawn_blocking(move || {
        let guard = MODEL_MANAGER.lock().unwrap();
        let manager = guard
            .as_ref()
            .ok_or_else(|| "Model manager not initialized".to_string())?;

        manager
            .download_default_model(move |progress| {
                let _ = app_clone.emit("model:progress", progress);
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;

    match result {
        Ok(_) => {
            // Store the model ID
            let mut model_id_guard = CURRENT_MODEL_ID.lock().unwrap();
            *model_id_guard = Some("lfm2.5-1.2b-q4".to_string());

            app.emit("model:complete", ()).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            app.emit("model:error", e.clone())
                .map_err(|e| e.to_string())?;
            Err(e)
        }
    }
}

/// Download a specific model by ID
#[tauri::command]
pub async fn download_model_by_id(app: AppHandle, model_id: String) -> Result<(), String> {
    ensure_model_manager()?;

    // Emit starting event
    app.emit("model:progress", 0.0f32)
        .map_err(|e| e.to_string())?;

    let app_clone = app.clone();
    let model_id_clone = model_id.clone();

    // Run download in blocking task
    let result = tokio::task::spawn_blocking(move || {
        let guard = MODEL_MANAGER.lock().unwrap();
        let manager = guard
            .as_ref()
            .ok_or_else(|| "Model manager not initialized".to_string())?;

        manager
            .download_model_by_id(&model_id_clone, move |progress| {
                let _ = app_clone.emit("model:progress", progress);
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;

    match result {
        Ok(_) => {
            // Store the model ID
            let mut model_id_guard = CURRENT_MODEL_ID.lock().unwrap();
            *model_id_guard = Some(model_id);

            app.emit("model:complete", ()).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            app.emit("model:error", e.clone())
                .map_err(|e| e.to_string())?;
            Err(e)
        }
    }
}

/// Initialize the AI system (load model into memory)
#[tauri::command]
pub async fn init_ai() -> Result<(), String> {
    // Check if model is already loaded - skip reloading
    {
        let guard = SUMMARIZER.lock().unwrap();
        if let Some(summarizer) = guard.as_ref() {
            if summarizer.is_model_loaded() {
                println!("[AI] Model already loaded, skipping init");
                return Ok(());
            }
        }
    }

    // Check if loading is already in progress
    let is_loading = {
        let loading_guard = MODEL_LOADING.lock().unwrap();
        *loading_guard
    };

    if is_loading {
        println!("[AI] Model loading already in progress, waiting...");
        // Wait for loading to complete
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let loading = {
                let guard = MODEL_LOADING.lock().unwrap();
                *guard
            };
            if !loading {
                break;
            }
        }
        // Check if model is now loaded
        let guard = SUMMARIZER.lock().unwrap();
        if let Some(summarizer) = guard.as_ref() {
            if summarizer.is_model_loaded() {
                println!("[AI] Model loaded by another call");
                return Ok(());
            }
        }
        return Err("Model loading failed in another call".to_string());
    }

    // Set loading flag
    {
        let mut loading_guard = MODEL_LOADING.lock().unwrap();
        *loading_guard = true;
    }

    ensure_model_manager()?;

    // Get model path (try any downloaded model)
    let model_path = {
        let guard = MODEL_MANAGER.lock().unwrap();
        let manager = guard.as_ref().ok_or("Model manager not initialized")?;

        match manager.find_any_downloaded_model() {
            Some((model, path)) => {
                println!("[AI] Found downloaded model: {}", model.id);
                path
            }
            None => {
                let mut loading_guard = MODEL_LOADING.lock().unwrap();
                *loading_guard = false;
                return Err("No model downloaded. Please download a model first.".to_string());
            }
        }
    };

    println!("[AI] Loading model from: {:?}", model_path);

    // Load model in blocking task
    let result = tokio::task::spawn_blocking(move || {
        let mut summarizer = Summarizer::new().map_err(|e| e.to_string())?;
        summarizer
            .load_model(&model_path)
            .map_err(|e| e.to_string())?;

        let mut guard = SUMMARIZER.lock().unwrap();
        *guard = Some(summarizer);
        println!("[AI] Model loaded successfully");
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())?;

    // Clear loading flag
    {
        let mut loading_guard = MODEL_LOADING.lock().unwrap();
        *loading_guard = false;
    }

    result
}

/// Initialize AI with fallback (works even without model downloaded)
#[tauri::command]
pub async fn init_ai_fallback() -> Result<bool, String> {
    // Check if model is already loaded - skip reloading
    {
        let guard = SUMMARIZER.lock().unwrap();
        if let Some(summarizer) = guard.as_ref() {
            if summarizer.is_model_loaded() {
                println!("[AI] Model already loaded (fallback check)");
                return Ok(true); // Model already loaded
            }
        }
    }

    // Check if loading is already in progress
    let is_loading = {
        let loading_guard = MODEL_LOADING.lock().unwrap();
        *loading_guard
    };

    if is_loading {
        println!("[AI] Model loading already in progress (fallback), waiting...");
        // Wait for loading to complete
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let loading = {
                let guard = MODEL_LOADING.lock().unwrap();
                *guard
            };
            if !loading {
                break;
            }
        }
        // Check if model is now loaded
        let guard = SUMMARIZER.lock().unwrap();
        if let Some(summarizer) = guard.as_ref() {
            if summarizer.is_model_loaded() {
                println!("[AI] Model loaded by another call (fallback)");
                return Ok(true);
            }
        }
        // Return false to indicate fallback mode
        return Ok(false);
    }

    // Set loading flag
    {
        let mut loading_guard = MODEL_LOADING.lock().unwrap();
        *loading_guard = true;
    }

    ensure_model_manager()?;

    // Try to find any downloaded model
    let model_path = {
        let guard = MODEL_MANAGER.lock().unwrap();
        let manager = guard.as_ref().ok_or("Model manager not initialized")?;
        manager.find_any_downloaded_model().map(|(model, path)| {
            println!("[AI] Found downloaded model for fallback init: {}", model.id);
            path
        })
    };

    if let Some(path) = model_path {
        println!("[AI] Loading model in fallback mode from: {:?}", path);
        // Load model in blocking task
        let result = tokio::task::spawn_blocking(move || {
            let mut summarizer = Summarizer::new().map_err(|e| e.to_string())?;
            summarizer.load_model(&path).map_err(|e| e.to_string())?;

            let mut guard = SUMMARIZER.lock().unwrap();
            *guard = Some(summarizer);
            println!("[AI] Model loaded successfully in fallback mode");
            Ok::<bool, String>(true)
        })
        .await
        .map_err(|e| e.to_string())?;

        // Clear loading flag
        {
            let mut loading_guard = MODEL_LOADING.lock().unwrap();
            *loading_guard = false;
        }

        result
    } else {
        // No model downloaded, use fallback summarizer (no LLM)
        println!("[AI] No model downloaded, using keyword-based fallback");
        let summarizer = Summarizer::new().map_err(|e| e.to_string())?;
        let mut guard = SUMMARIZER.lock().unwrap();
        *guard = Some(summarizer);

        // Clear loading flag
        {
            let mut loading_guard = MODEL_LOADING.lock().unwrap();
            *loading_guard = false;
        }

        Ok(false) // Model not loaded, using fallback
    }
}

/// Summarize an email
#[tauri::command]
pub async fn summarize_email(
    subject: String,
    from: String,
    body: String,
) -> Result<EmailSummary, String> {
    let guard = SUMMARIZER.lock().unwrap();
    let summarizer = guard
        .as_ref()
        .ok_or("AI not initialized. Call init_ai first.")?;

    let summary = summarizer
        .summarize_email(&subject, &from, &body)
        .map_err(|e| e.to_string())?;

    let insights = summarizer
        .generate_insights(&subject, &body)
        .map_err(|e| e.to_string())?;

    let priority = summarizer
        .classify_priority(&subject, &body)
        .map_err(|e| e.to_string())?;

    Ok(EmailSummary {
        summary,
        insights,
        priority,
    })
}

/// Summarize an email with streaming output
#[tauri::command]
pub async fn summarize_email_stream(
    app: AppHandle,
    subject: String,
    from: String,
    body: String,
) -> Result<EmailSummary, String> {
    // Clone data for the blocking task
    let subject_clone = subject.clone();
    let from_clone = from.clone();
    let body_clone = body.clone();
    let app_clone = app.clone();

    // Run summarization in blocking task for streaming
    let summary = tokio::task::spawn_blocking(move || {
        let guard = SUMMARIZER.lock().unwrap();
        let summarizer = guard
            .as_ref()
            .ok_or_else(|| "AI not initialized".to_string())?;

        summarizer
            .summarize_email_stream(&subject_clone, &from_clone, &body_clone, |token| {
                let _ = app_clone.emit("ai:token", token);
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Emit completion
    app.emit("ai:complete", ()).map_err(|e| e.to_string())?;

    // Get insights and priority (non-streaming)
    let (insights, priority) = {
        let guard = SUMMARIZER.lock().unwrap();
        let summarizer = guard.as_ref().ok_or("AI not initialized")?;

        let insights = summarizer
            .generate_insights(&subject, &body)
            .map_err(|e| e.to_string())?;

        let priority = summarizer
            .classify_priority(&subject, &body)
            .map_err(|e| e.to_string())?;

        (insights, priority)
    };

    Ok(EmailSummary {
        summary,
        insights,
        priority,
    })
}

/// Get quick insights about an email
#[tauri::command]
pub async fn get_email_insights(subject: String, body: String) -> Result<Vec<String>, String> {
    let guard = SUMMARIZER.lock().unwrap();
    let summarizer = guard.as_ref().ok_or("AI not initialized")?;

    summarizer
        .generate_insights(&subject, &body)
        .map_err(|e| e.to_string())
}

/// Classify email priority
#[tauri::command]
pub async fn classify_priority(subject: String, body: String) -> Result<String, String> {
    let guard = SUMMARIZER.lock().unwrap();
    let summarizer = guard.as_ref().ok_or("AI not initialized")?;

    summarizer
        .classify_priority(&subject, &body)
        .map_err(|e| e.to_string())
}

/// Get model information (for the default/recommended model)
#[tauri::command]
pub async fn get_model_info() -> Result<ModelInfo, String> {
    Ok(ModelInfo {
        repo: DEFAULT_MODEL_REPO.to_string(),
        filename: DEFAULT_MODEL_FILE.to_string(),
        size_mb: 731, // LFM2.5 Q4_K_M size
    })
}

/// Get currently selected model ID
#[tauri::command]
pub async fn get_current_model_id() -> Result<Option<String>, String> {
    let guard = CURRENT_MODEL_ID.lock().unwrap();
    Ok(guard.clone())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub repo: String,
    pub filename: String,
    pub size_mb: u32,
}

/// Get list of downloaded models
#[tauri::command]
pub async fn get_downloaded_models() -> Result<Vec<ModelOption>, String> {
    ensure_model_manager()?;

    let guard = MODEL_MANAGER.lock().unwrap();
    let manager = guard.as_ref().ok_or("Model manager not initialized")?;

    Ok(manager.get_downloaded_models())
}

/// Delete a model by ID
#[tauri::command]
pub async fn delete_model(model_id: String) -> Result<(), String> {
    ensure_model_manager()?;

    // Check if this is the currently active model
    {
        let current_guard = CURRENT_MODEL_ID.lock().unwrap();
        if let Some(ref current_id) = *current_guard {
            if current_id == &model_id {
                // Unload the model first
                let mut summarizer_guard = SUMMARIZER.lock().unwrap();
                *summarizer_guard = None;
            }
        }
    }

    // Delete the model file
    let guard = MODEL_MANAGER.lock().unwrap();
    let manager = guard.as_ref().ok_or("Model manager not initialized")?;
    manager.delete_model(&model_id).map_err(|e| e.to_string())?;

    // Clear current model ID if it was the deleted one
    {
        let mut current_guard = CURRENT_MODEL_ID.lock().unwrap();
        if let Some(ref current_id) = *current_guard {
            if current_id == &model_id {
                *current_guard = None;
            }
        }
    }

    Ok(())
}

/// Activate a specific model by ID (load it into memory)
#[tauri::command]
pub async fn activate_model(model_id: String) -> Result<(), String> {
    println!("[AI] Activating model: {}", model_id);

    // Check if loading is already in progress
    let is_loading = {
        let loading_guard = MODEL_LOADING.lock().unwrap();
        *loading_guard
    };

    if is_loading {
        println!("[AI] Model loading already in progress, waiting before activating...");
        // Wait for loading to complete
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let loading = {
                let guard = MODEL_LOADING.lock().unwrap();
                *guard
            };
            if !loading {
                break;
            }
        }
    }

    // Now acquire the loading flag for ourselves
    {
        let mut loading_guard = MODEL_LOADING.lock().unwrap();
        *loading_guard = true;
    }

    ensure_model_manager()?;

    // Get model info and path
    let model_path = {
        let guard = MODEL_MANAGER.lock().unwrap();
        let manager = guard.as_ref().ok_or("Model manager not initialized")?;

        let model = manager
            .get_model_by_id(&model_id)
            .ok_or_else(|| format!("Unknown model: {}", model_id))?;

        let path = manager.get_model_path(&model.filename);
        if !path.exists() {
            let mut loading_guard = MODEL_LOADING.lock().unwrap();
            *loading_guard = false;
            return Err(format!("Model not downloaded: {}", model_id));
        }

        println!("[AI] Model path: {:?}", path);
        path
    };

    let model_id_clone = model_id.clone();

    // Load model in blocking task
    let result = tokio::task::spawn_blocking(move || {
        println!("[AI] Starting model load in blocking task...");
        let mut summarizer = Summarizer::new().map_err(|e| e.to_string())?;
        summarizer
            .load_model(&model_path)
            .map_err(|e| e.to_string())?;

        let mut guard = SUMMARIZER.lock().unwrap();
        *guard = Some(summarizer);

        // Update current model ID
        let mut model_id_guard = CURRENT_MODEL_ID.lock().unwrap();
        *model_id_guard = Some(model_id_clone);

        println!("[AI] Model activated successfully");
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())?;

    // Clear loading flag
    {
        let mut loading_guard = MODEL_LOADING.lock().unwrap();
        *loading_guard = false;
    }

    result
}

/// Get the active model ID (the one currently loaded)
#[tauri::command]
pub async fn get_active_model_id() -> Result<Option<String>, String> {
    let guard = CURRENT_MODEL_ID.lock().unwrap();
    Ok(guard.clone())
}
