mod auth;
mod commands;
mod db;
mod email;
mod llm;

use commands::account::AccountManager;
use directories::ProjectDirs;
use email::idle::IdleManager;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment variables from .env file (development only)
    let _ = dotenvy::dotenv();

    // Initialize database
    let project_dirs =
        ProjectDirs::from("com", "inboxed", "inboxed").expect("Failed to get project directory");
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
    let db_path = data_dir.join("emails.db");
    let database = db::EmailDatabase::new(db_path).expect("Failed to initialize database");
    let db_state = Arc::new(Mutex::new(Some(database)));

    // Initialize account manager and IDLE manager
    let account_manager = AccountManager::new();
    let idle_manager = IdleManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(db_state)
        .manage(account_manager)
        .manage(idle_manager)
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            commands::check_auth_status,
            commands::start_auth,
            commands::complete_auth,
            commands::refresh_token,
            commands::sign_out,
            commands::get_access_token,
            // Account commands
            commands::add_account,
            commands::remove_account,
            commands::list_accounts,
            commands::set_active_account,
            commands::connect_account,
            // Email commands
            commands::fetch_emails,
            commands::get_email,
            commands::send_email,
            commands::mark_email_read,
            commands::star_email,
            commands::trash_email,
            commands::archive_email,
            // AI commands
            commands::check_model_status,
            commands::is_model_loading,
            commands::download_model,
            commands::download_model_by_id,
            commands::init_ai,
            commands::init_ai_fallback,
            commands::summarize_email,
            commands::summarize_email_stream,
            commands::get_email_insights,
            commands::classify_priority,
            commands::get_model_info,
            commands::get_available_ai_models,
            commands::get_current_model_id,
            commands::get_downloaded_models,
            commands::delete_model,
            commands::activate_model,
            commands::get_active_model_id,
            // Database commands
            commands::init_database,
            commands::get_smart_inbox,
            commands::get_emails_by_category,
            commands::get_indexing_status,
            commands::reset_indexing_status,
            commands::start_email_indexing,
            commands::search_smart_emails,
            commands::chat_query,
            // Cache commands
            commands::get_storage_info,
            commands::get_cache_settings,
            commands::save_cache_settings,
            commands::clear_email_cache,
            commands::clear_media_cache,
            commands::clear_all_caches,
            commands::cache_media_asset,
            commands::get_cached_media_asset,
            commands::get_cached_emails_count,
            commands::has_cached_emails,
            commands::clear_all_app_data,
            commands::clear_ai_models,
            // RAG commands
            commands::init_rag,
            commands::is_rag_ready,
            commands::is_embedding_model_downloaded,
            commands::get_embedding_status,
            commands::embed_email,
            commands::embed_all_emails,
            commands::search_emails_semantic,
            commands::find_similar_emails,
            commands::get_embedded_count,
            commands::clear_embeddings,
            commands::chat_with_context,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
