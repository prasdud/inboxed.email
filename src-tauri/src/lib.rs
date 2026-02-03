mod auth;
mod commands;
mod email;
mod llm;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            commands::check_auth_status,
            commands::start_auth,
            commands::complete_auth,
            commands::refresh_token,
            commands::sign_out,
            commands::get_access_token,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
