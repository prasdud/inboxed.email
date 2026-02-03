use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use rand::Rng;
use std::sync::Mutex;
use tokio::sync::oneshot;

use super::storage::{store_tokens, TokenData};

// OAuth configuration
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_URI: &str = "http://localhost:3000/callback";
const GMAIL_SCOPE: &str = "https://www.googleapis.com/auth/gmail.modify";

// For desktop applications using PKCE (Proof Key for Code Exchange):
// - CLIENT_ID is public and safe to embed in the application
// - CLIENT_SECRET is not required with PKCE, but some OAuth providers still expect it
// - We load from environment variables in development, fall back to build-time values for production
//
// Security Note: For native/desktop apps, Google's OAuth documentation states that
// client secrets cannot be kept confidential and PKCE provides the security instead.
fn get_client_id() -> String {
    // Try environment variable first (development)
    if let Ok(client_id) = std::env::var("GOOGLE_CLIENT_ID") {
        return client_id;
    }

    // Fall back to compile-time environment variable (production builds)
    // Set via: cargo build --release (with .env file present)
    option_env!("GOOGLE_CLIENT_ID")
        .expect("GOOGLE_CLIENT_ID must be set in environment or .env file")
        .to_string()
}

// For desktop apps with PKCE, client secret is not security-critical
// Google OAuth allows desktop apps to use a non-confidential client secret
fn get_client_secret() -> Option<String> {
    // Try environment variable first
    if let Ok(secret) = std::env::var("GOOGLE_CLIENT_SECRET") {
        if !secret.is_empty() {
            return Some(secret);
        }
    }

    // For desktop apps with PKCE, we don't need a client secret
    // However, if your OAuth provider requires one, you can set it here
    None
}

/// OAuth state stored during the flow
pub struct OAuthState {
    pub pkce_verifier: PkceCodeVerifier,
    pub csrf_token: CsrfToken,
    pub callback_receiver: Option<oneshot::Receiver<Result<String>>>,
}

lazy_static::lazy_static! {
    static ref OAUTH_STATE: Mutex<Option<OAuthState>> = Mutex::new(None);
}

/// Generate PKCE code verifier and challenge
fn generate_pkce() -> (PkceCodeVerifier, PkceCodeChallenge) {
    // Generate a random code verifier
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let code_verifier = URL_SAFE_NO_PAD.encode(random_bytes);
    let verifier = PkceCodeVerifier::new(code_verifier);

    // Create SHA256 hash for PKCE challenge
    let code_challenge = PkceCodeChallenge::from_code_verifier_sha256(&verifier);

    (verifier, code_challenge)
}

/// Initialize OAuth client
fn create_oauth_client() -> Result<BasicClient> {
    let client = BasicClient::new(
        ClientId::new(get_client_id()),
        get_client_secret().map(ClientSecret::new),
        AuthUrl::new(GOOGLE_AUTH_URL.to_string())
            .context("Failed to create auth URL")?,
        Some(
            TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                .context("Failed to create token URL")?,
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new(REDIRECT_URI.to_string())
            .context("Failed to create redirect URL")?,
    );

    Ok(client)
}

/// Start the OAuth flow and return the authorization URL
pub fn start_oauth_flow() -> Result<String> {
    let client = create_oauth_client()?;

    // Generate PKCE challenge
    let (pkce_verifier, pkce_challenge) = generate_pkce();

    // Generate the authorization URL
    let (authorize_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(GMAIL_SCOPE.to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Create a channel for receiving the callback
    let (tx, rx) = oneshot::channel();

    // Store the state
    let mut state = OAUTH_STATE.lock().unwrap();
    *state = Some(OAuthState {
        pkce_verifier,
        csrf_token,
        callback_receiver: Some(rx),
    });

    // Start the callback server
    start_callback_server(tx);

    Ok(authorize_url.to_string())
}

/// Start a simple HTTP server to receive the OAuth callback
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

        println!("Callback server listening on http://localhost:3000");

        // Accept a single connection
        if let Ok((mut stream, _)) = listener.accept() {
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_ok() {
                // Parse the request line to get the query parameters
                if let Some(query_start) = request_line.find("?") {
                    if let Some(query_end) = request_line.find(" HTTP/") {
                        let query = &request_line[query_start + 1..query_end];

                        // Send a success response to the browser
                        let response = "HTTP/1.1 200 OK\r\n\r\n\
                            <html><body>\
                            <h1>Authentication Successful!</h1>\
                            <p>You can close this window and return to Inboxed.</p>\
                            <script>window.close();</script>\
                            </body></html>";
                        let _ = stream.write_all(response.as_bytes());

                        // Send the query string back
                        let _ = tx.send(Ok(query.to_string()));
                        return;
                    }
                }
            }
            let _ = tx.send(Err(anyhow::anyhow!("Failed to parse callback request")));
        }
    });
}

/// Handle the OAuth callback and exchange code for tokens
pub async fn handle_oauth_callback() -> Result<TokenData> {
    // Get the stored state and extract it from the mutex in a scope
    let (pkce_verifier, callback_receiver) = {
        let mut state_lock = OAUTH_STATE.lock().unwrap();
        let state = state_lock
            .take()
            .context("No OAuth flow in progress")?;

        let OAuthState {
            pkce_verifier,
            csrf_token: _csrf_token,
            callback_receiver,
        } = state;

        (pkce_verifier, callback_receiver)
    }; // Mutex guard dropped here

    // Wait for the callback
    let query_string = callback_receiver
        .context("No callback receiver")?
        .await
        .context("Failed to receive callback")??;

    // Parse query parameters
    let params: std::collections::HashMap<_, _> = url::form_urlencoded::parse(query_string.as_bytes())
        .into_owned()
        .collect();

    let code = params
        .get("code")
        .context("No authorization code in callback")?;

    // Exchange the code for tokens
    let client = create_oauth_client()?;

    let token_response = client
        .exchange_code(AuthorizationCode::new(code.clone()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .context("Failed to exchange authorization code for tokens")?;

    // Calculate expiry time
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

    // Store tokens in keychain
    store_tokens(&token_data)?;

    Ok(token_data)
}

/// Refresh the access token using refresh token
pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenData> {
    let client = create_oauth_client()?;

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
        refresh_token: Some(refresh_token.to_string()), // Keep the same refresh token
        expires_at,
    };

    store_tokens(&token_data)?;

    Ok(token_data)
}
