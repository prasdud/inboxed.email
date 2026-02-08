use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tokio::sync::oneshot;

use super::storage::{store_account_tokens, store_tokens, TokenData};

// ========== OAuth Provider Configurations ==========

const REDIRECT_URI: &str = "http://localhost:3000/callback";

/// Provider-specific OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthProviderConfig {
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
    pub client_id_env: &'static str,
    pub client_secret_env: &'static str,
}

/// Get OAuth config for Google (Gmail IMAP access)
pub fn google_oauth_config() -> OAuthProviderConfig {
    OAuthProviderConfig {
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        token_url: "https://oauth2.googleapis.com/token".to_string(),
        // Use the full mail scope for IMAP access (not gmail.modify)
        scopes: vec!["https://mail.google.com/".to_string()],
        client_id_env: "GOOGLE_CLIENT_ID",
        client_secret_env: "GOOGLE_CLIENT_SECRET",
    }
}

/// Get OAuth config for Microsoft (Outlook IMAP/SMTP access)
pub fn microsoft_oauth_config() -> OAuthProviderConfig {
    OAuthProviderConfig {
        auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
        token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
        scopes: vec![
            "https://outlook.office365.com/IMAP.AccessAsUser.All".to_string(),
            "https://outlook.office365.com/SMTP.Send".to_string(),
            "offline_access".to_string(),
        ],
        client_id_env: "MICROSOFT_CLIENT_ID",
        client_secret_env: "MICROSOFT_CLIENT_SECRET",
    }
}

/// Get the provider config by name
pub fn get_provider_config(provider: &str) -> OAuthProviderConfig {
    match provider.to_lowercase().as_str() {
        "outlook" | "microsoft" | "hotmail" => microsoft_oauth_config(),
        _ => google_oauth_config(), // Default to Google
    }
}

// ========== Client ID / Secret helpers ==========

fn get_env_var(key: &str) -> Option<String> {
    // Try runtime environment first
    if let Ok(val) = std::env::var(key) {
        if !val.is_empty() {
            return Some(val);
        }
    }
    // Try compile-time
    option_env!("GOOGLE_CLIENT_ID").map(|s| s.to_string()) // Fallback for backward compat
}

fn get_client_id_for_provider(config: &OAuthProviderConfig) -> String {
    get_env_var(config.client_id_env)
        .expect(&format!("{} must be set in environment or .env file", config.client_id_env))
}

fn get_client_secret_for_provider(config: &OAuthProviderConfig) -> Option<String> {
    get_env_var(config.client_secret_env)
}

// ========== Legacy single-account helpers (backward compatible) ==========

fn get_client_id() -> String {
    get_client_id_for_provider(&google_oauth_config())
}

fn get_client_secret() -> Option<String> {
    get_client_secret_for_provider(&google_oauth_config())
}

// ========== OAuth State ==========

pub struct OAuthState {
    pub pkce_verifier: PkceCodeVerifier,
    pub csrf_token: CsrfToken,
    pub callback_receiver: Option<oneshot::Receiver<Result<String>>>,
    pub account_id: Option<String>,
    pub provider: String,
}

lazy_static::lazy_static! {
    static ref OAUTH_STATE: Mutex<Option<OAuthState>> = Mutex::new(None);
}

// ========== PKCE ==========

fn generate_pkce() -> (PkceCodeVerifier, PkceCodeChallenge) {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let code_verifier = URL_SAFE_NO_PAD.encode(random_bytes);
    let verifier = PkceCodeVerifier::new(code_verifier);
    let code_challenge = PkceCodeChallenge::from_code_verifier_sha256(&verifier);
    (verifier, code_challenge)
}

// ========== OAuth Client ==========

fn create_oauth_client_for_provider(config: &OAuthProviderConfig) -> Result<BasicClient> {
    let client_id = get_client_id_for_provider(config);
    let client_secret = get_client_secret_for_provider(config);

    let client = BasicClient::new(
        ClientId::new(client_id),
        client_secret.map(ClientSecret::new),
        AuthUrl::new(config.auth_url.clone()).context("Failed to create auth URL")?,
        Some(TokenUrl::new(config.token_url.clone()).context("Failed to create token URL")?),
    )
    .set_redirect_uri(
        RedirectUrl::new(REDIRECT_URI.to_string()).context("Failed to create redirect URL")?,
    );

    Ok(client)
}

fn create_oauth_client() -> Result<BasicClient> {
    create_oauth_client_for_provider(&google_oauth_config())
}

// ========== Parameterized OAuth Flow ==========

/// Start OAuth flow for a specific provider and optional account
pub fn start_oauth_flow_for_provider(provider: &str, account_id: Option<&str>) -> Result<String> {
    let config = get_provider_config(provider);
    let client = create_oauth_client_for_provider(&config)?;

    let (pkce_verifier, pkce_challenge) = generate_pkce();

    let mut auth_request = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge);

    for scope in &config.scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }

    let (authorize_url, csrf_token) = auth_request.url();

    let (tx, rx) = oneshot::channel();

    let mut state = OAUTH_STATE.lock().unwrap();
    *state = Some(OAuthState {
        pkce_verifier,
        csrf_token,
        callback_receiver: Some(rx),
        account_id: account_id.map(|s| s.to_string()),
        provider: provider.to_string(),
    });

    start_callback_server(tx);

    Ok(authorize_url.to_string())
}

/// Start the legacy OAuth flow (backward compatible — uses Google)
pub fn start_oauth_flow() -> Result<String> {
    start_oauth_flow_for_provider("gmail", None)
}

// ========== Callback Server ==========

fn start_callback_server(tx: oneshot::Sender<Result<String>>) {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    tokio::spawn(async move {
        let listener = match TcpListener::bind("127.0.0.1:3000") {
            Ok(l) => l,
            Err(e) => {
                let _ = tx.send(Err(anyhow::anyhow!("Failed to bind to port 3000: {}", e)));
                return;
            }
        };

        if let Ok((mut stream, _)) = listener.accept() {
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_ok() {
                if let Some(query_start) = request_line.find("?") {
                    if let Some(query_end) = request_line.find(" HTTP/") {
                        let query = &request_line[query_start + 1..query_end];

                        let response = "HTTP/1.1 200 OK\r\n\r\n\
                            <html><body>\
                            <h1>Authentication Successful!</h1>\
                            <p>You can close this window and return to Inboxed.</p>\
                            <script>window.close();</script>\
                            </body></html>";
                        let _ = stream.write_all(response.as_bytes());

                        let _ = tx.send(Ok(query.to_string()));
                        return;
                    }
                }
            }
            let _ = tx.send(Err(anyhow::anyhow!("Failed to parse callback request")));
        }
    });
}

// ========== Token Exchange ==========

/// Handle OAuth callback — exchanges code for tokens, stores them
pub async fn handle_oauth_callback() -> Result<TokenData> {
    let (pkce_verifier, callback_receiver, account_id, provider) = {
        let mut state_lock = OAUTH_STATE.lock().unwrap();
        let state = state_lock.take().context("No OAuth flow in progress")?;

        (
            state.pkce_verifier,
            state.callback_receiver,
            state.account_id,
            state.provider,
        )
    };

    let query_string = callback_receiver
        .context("No callback receiver")?
        .await
        .context("Failed to receive callback")??;

    let params: std::collections::HashMap<_, _> =
        url::form_urlencoded::parse(query_string.as_bytes())
            .into_owned()
            .collect();

    let code = params
        .get("code")
        .context("No authorization code in callback")?;

    let config = get_provider_config(&provider);
    let client = create_oauth_client_for_provider(&config)?;

    let token_response = client
        .exchange_code(AuthorizationCode::new(code.clone()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .context("Failed to exchange authorization code for tokens")?;

    let expires_at = Utc::now()
        + Duration::seconds(
            token_response
                .expires_in()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(3600),
        );

    let token_data = TokenData {
        access_token: token_response.access_token().secret().clone(),
        refresh_token: token_response
            .refresh_token()
            .map(|t| t.secret().clone()),
        expires_at,
    };

    // Store tokens: per-account if account_id is set, otherwise legacy
    if let Some(ref aid) = account_id {
        store_account_tokens(aid, &token_data)?;
    } else {
        store_tokens(&token_data)?;
    }

    Ok(token_data)
}

/// Refresh access token (parameterized by provider and optional account)
pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenData> {
    refresh_access_token_for_provider(refresh_token, "gmail", None).await
}

pub async fn refresh_access_token_for_provider(
    refresh_token: &str,
    provider: &str,
    account_id: Option<&str>,
) -> Result<TokenData> {
    let config = get_provider_config(provider);
    let client = create_oauth_client_for_provider(&config)?;

    let token_response = client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
        .request_async(async_http_client)
        .await
        .context("Failed to refresh access token")?;

    let expires_at = Utc::now()
        + Duration::seconds(
            token_response
                .expires_in()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(3600),
        );

    let token_data = TokenData {
        access_token: token_response.access_token().secret().clone(),
        refresh_token: Some(refresh_token.to_string()),
        expires_at,
    };

    if let Some(aid) = account_id {
        store_account_tokens(aid, &token_data)?;
    } else {
        store_tokens(&token_data)?;
    }

    Ok(token_data)
}
