use crate::auth::storage::{get_account_tokens, get_app_password};
use crate::email::imap_client::{ImapClient, ImapCredentials};
use crate::email::server_presets::{ProviderType, ServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{watch, Mutex};
use tokio::time::{sleep, Duration};

/// Event payload emitted when new mail arrives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMailEvent {
    pub account_id: String,
    pub folder: String,
}

/// Manages IMAP IDLE connections for all accounts
pub struct IdleManager {
    /// Per-account shutdown senders
    shutdown_senders: Arc<Mutex<HashMap<String, watch::Sender<bool>>>>,
}

impl IdleManager {
    pub fn new() -> Self {
        Self {
            shutdown_senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start IDLE monitoring for an account
    pub async fn start_idle<R: tauri::Runtime>(
        &self,
        app: AppHandle<R>,
        account_id: String,
        email: String,
        provider: ProviderType,
        server_config: ServerConfig,
        auth_type: String,
    ) {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        // Store shutdown sender
        {
            let mut senders = self.shutdown_senders.lock().await;
            // Stop existing IDLE for this account if any
            if let Some(old_tx) = senders.remove(&account_id) {
                let _ = old_tx.send(true);
            }
            senders.insert(account_id.clone(), shutdown_tx);
        }

        let account_id_clone = account_id.clone();

        tokio::spawn(async move {
            idle_loop(
                app,
                account_id_clone,
                email,
                provider,
                server_config,
                auth_type,
                shutdown_rx,
            )
            .await;
        });
    }

    /// Stop IDLE monitoring for an account
    pub async fn stop_idle(&self, account_id: &str) {
        let mut senders = self.shutdown_senders.lock().await;
        if let Some(tx) = senders.remove(account_id) {
            let _ = tx.send(true);
        }
    }

    /// Stop all IDLE monitors
    pub async fn stop_all(&self) {
        let mut senders = self.shutdown_senders.lock().await;
        for (_, tx) in senders.drain() {
            let _ = tx.send(true);
        }
    }
}

/// The IDLE loop for a single account
async fn idle_loop<R: tauri::Runtime>(
    app: AppHandle<R>,
    account_id: String,
    email: String,
    provider: ProviderType,
    server_config: ServerConfig,
    auth_type: String,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let folder = "INBOX";
    // RFC 2177: IDLE should be re-issued every 29 minutes max
    let idle_timeout_secs = 29 * 60;
    let retry_delay = Duration::from_secs(30);

    loop {
        // Check shutdown
        if *shutdown_rx.borrow() {
            println!("[IDLE:{}] Shutdown signal received", account_id);
            break;
        }

        // Build credentials
        let credentials = if auth_type == "oauth2" {
            match get_account_tokens(&account_id) {
                Ok(tokens) => ImapCredentials::OAuth2 {
                    user: email.clone(),
                    access_token: tokens.access_token,
                },
                Err(e) => {
                    eprintln!(
                        "[IDLE:{}] Failed to get OAuth tokens: {}. Retrying...",
                        account_id, e
                    );
                    sleep(retry_delay).await;
                    continue;
                }
            }
        } else {
            match get_app_password(&account_id) {
                Ok(password) => ImapCredentials::Password {
                    user: email.clone(),
                    password,
                },
                Err(e) => {
                    eprintln!(
                        "[IDLE:{}] Failed to get password: {}. Retrying...",
                        account_id, e
                    );
                    sleep(retry_delay).await;
                    continue;
                }
            }
        };

        let client = ImapClient::new(
            account_id.clone(),
            email.clone(),
            provider.clone(),
            server_config.clone(),
            credentials,
        );

        // Connect
        match client.reconnect().await {
            Ok(()) => {
                println!("[IDLE:{}] Connected, starting IDLE on {}", account_id, folder);
            }
            Err(e) => {
                eprintln!(
                    "[IDLE:{}] Connection failed: {}. Retrying in 30s...",
                    account_id, e
                );
                sleep(retry_delay).await;
                continue;
            }
        }

        // IDLE loop (re-issue every 29 min)
        match client.idle_wait(folder, idle_timeout_secs).await {
            Ok(true) => {
                // New mail detected
                println!("[IDLE:{}] New mail detected in {}", account_id, folder);
                let _ = app.emit(
                    "email:new_mail",
                    NewMailEvent {
                        account_id: account_id.clone(),
                        folder: folder.to_string(),
                    },
                );
            }
            Ok(false) => {
                // Timeout — re-issue IDLE
                println!("[IDLE:{}] IDLE timeout, re-issuing", account_id);
            }
            Err(e) => {
                eprintln!(
                    "[IDLE:{}] IDLE error: {}. Reconnecting in 30s...",
                    account_id, e
                );
                sleep(retry_delay).await;
            }
        }
    }

    println!("[IDLE:{}] IDLE loop exited", account_id);
}
