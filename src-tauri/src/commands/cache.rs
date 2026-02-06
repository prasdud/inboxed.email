use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::db::EmailDatabase;

type DbState = Arc<Mutex<Option<EmailDatabase>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub database_size_bytes: u64,
    pub media_cache_size_bytes: u64,
    pub total_emails_cached: i64,
    pub total_indexed_emails: i64,
    pub data_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    pub cache_enabled: bool,
    pub auto_sync_on_start: bool,
    pub cache_media_assets: bool,
    pub max_cache_age_days: u32,
}

/// Get the project data directory
fn get_data_dir() -> Result<PathBuf, String> {
    let project_dirs =
        ProjectDirs::from("com", "inboxed", "inboxed").ok_or("Failed to get project directory")?;
    Ok(project_dirs.data_dir().to_path_buf())
}

/// Get the media cache directory
fn get_media_cache_dir() -> Result<PathBuf, String> {
    let data_dir = get_data_dir()?;
    Ok(data_dir.join("media_cache"))
}

/// Calculate total size of a directory recursively
fn get_dir_size(path: &PathBuf) -> u64 {
    if !path.exists() {
        return 0;
    }

    let mut total_size = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            } else if entry_path.is_dir() {
                total_size += get_dir_size(&entry_path);
            }
        }
    }
    total_size
}

/// Get storage information including database size and media cache size
#[tauri::command]
pub async fn get_storage_info(db: State<'_, DbState>) -> Result<StorageInfo, String> {
    let data_dir = get_data_dir()?;
    let db_path = data_dir.join("emails.db");
    let media_cache_dir = get_media_cache_dir()?;

    // Get database file size
    let database_size_bytes = if db_path.exists() {
        fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    // Get media cache directory size
    let media_cache_size_bytes = get_dir_size(&media_cache_dir);

    // Get email counts from database
    let (total_emails_cached, total_indexed_emails) = {
        let db_lock = db.lock().unwrap();
        if let Some(database) = db_lock.as_ref() {
            let cached = database.get_email_count().unwrap_or(0);
            let indexed = database.get_indexed_count().unwrap_or(0);
            (cached, indexed)
        } else {
            (0, 0)
        }
    };

    Ok(StorageInfo {
        database_size_bytes,
        media_cache_size_bytes,
        total_emails_cached,
        total_indexed_emails,
        data_directory: data_dir.to_string_lossy().to_string(),
    })
}

/// Get current cache settings
#[tauri::command]
pub async fn get_cache_settings() -> Result<CacheSettings, String> {
    let data_dir = get_data_dir()?;
    let settings_path = data_dir.join("cache_settings.json");

    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read cache settings: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse cache settings: {}", e))
    } else {
        // Return default settings
        Ok(CacheSettings {
            cache_enabled: true,
            auto_sync_on_start: false,
            cache_media_assets: true,
            max_cache_age_days: 30,
        })
    }
}

/// Save cache settings
#[tauri::command]
pub async fn save_cache_settings(settings: CacheSettings) -> Result<(), String> {
    let data_dir = get_data_dir()?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let settings_path = data_dir.join("cache_settings.json");
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize cache settings: {}", e))?;

    fs::write(&settings_path, content).map_err(|e| format!("Failed to write cache settings: {}", e))
}

/// Clear the email database (keeps the schema)
#[tauri::command]
pub async fn clear_email_cache(db: State<'_, DbState>) -> Result<(), String> {
    let db_lock = db.lock().unwrap();
    let database = db_lock.as_ref().ok_or("Database not initialized")?;

    database.clear_all_emails().map_err(|e| e.to_string())
}

/// Clear the media cache directory
#[tauri::command]
pub async fn clear_media_cache() -> Result<(), String> {
    let media_cache_dir = get_media_cache_dir()?;

    if media_cache_dir.exists() {
        fs::remove_dir_all(&media_cache_dir)
            .map_err(|e| format!("Failed to clear media cache: {}", e))?;
        fs::create_dir_all(&media_cache_dir)
            .map_err(|e| format!("Failed to recreate media cache directory: {}", e))?;
    }

    Ok(())
}

/// Clear all caches (emails and media)
#[tauri::command]
pub async fn clear_all_caches(db: State<'_, DbState>) -> Result<(), String> {
    // Clear email cache
    clear_email_cache(db).await?;

    // Clear media cache
    clear_media_cache().await?;

    Ok(())
}

/// Store a media asset in the cache
#[tauri::command]
pub async fn cache_media_asset(
    email_id: String,
    asset_url: String,
    content_type: String,
    data: Vec<u8>,
) -> Result<String, String> {
    let media_cache_dir = get_media_cache_dir()?;
    let email_cache_dir = media_cache_dir.join(&email_id);

    fs::create_dir_all(&email_cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    // Generate a safe filename from the URL
    let url_hash = format!("{:x}", md5::compute(asset_url.as_bytes()));
    let extension = content_type
        .split('/')
        .last()
        .unwrap_or("bin")
        .split(';')
        .next()
        .unwrap_or("bin");
    let filename = format!("{}.{}", url_hash, extension);
    let file_path = email_cache_dir.join(&filename);

    fs::write(&file_path, data).map_err(|e| format!("Failed to write cached asset: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Check if a media asset is cached and return its path
#[tauri::command]
pub async fn get_cached_media_asset(
    email_id: String,
    asset_url: String,
) -> Result<Option<String>, String> {
    let media_cache_dir = get_media_cache_dir()?;
    let email_cache_dir = media_cache_dir.join(&email_id);

    if !email_cache_dir.exists() {
        return Ok(None);
    }

    let url_hash = format!("{:x}", md5::compute(asset_url.as_bytes()));

    // Look for any file starting with this hash
    if let Ok(entries) = fs::read_dir(&email_cache_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with(&url_hash) {
                return Ok(Some(entry.path().to_string_lossy().to_string()));
            }
        }
    }

    Ok(None)
}

/// Get cached emails count
#[tauri::command]
pub async fn get_cached_emails_count(db: State<'_, DbState>) -> Result<i64, String> {
    let db_lock = db.lock().unwrap();
    let database = db_lock.as_ref().ok_or("Database not initialized")?;

    database.get_email_count().map_err(|e| e.to_string())
}

/// Check if any emails are cached
#[tauri::command]
pub async fn has_cached_emails(db: State<'_, DbState>) -> Result<bool, String> {
    let count = get_cached_emails_count(db).await?;
    Ok(count > 0)
}

/// Clear all app data including database, cache, and settings
/// This does NOT clear OAuth tokens - use sign_out for that
#[tauri::command]
pub async fn clear_all_app_data(db: State<'_, DbState>) -> Result<(), String> {
    // Clear email cache and media cache
    clear_all_caches(db).await?;

    // Clear cache settings file
    let data_dir = get_data_dir()?;
    let settings_path = data_dir.join("cache_settings.json");
    if settings_path.exists() {
        fs::remove_file(&settings_path)
            .map_err(|e| format!("Failed to clear cache settings: {}", e))?;
    }

    Ok(())
}

/// Delete downloaded AI models
#[tauri::command]
pub async fn clear_ai_models() -> Result<(), String> {
    let data_dir = get_data_dir()?;
    let models_dir = data_dir.join("models");

    if models_dir.exists() {
        fs::remove_dir_all(&models_dir)
            .map_err(|e| format!("Failed to clear AI models: {}", e))?;
        fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to recreate models directory: {}", e))?;
    }

    Ok(())
}
