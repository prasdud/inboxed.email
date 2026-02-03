# OAuth Setup Guide for Inboxed

## Overview

Inboxed uses **OAuth 2.0 with PKCE** (Proof Key for Code Exchange) for secure authentication with Google. This is the recommended approach for desktop applications.

## Security Model

For desktop applications:
- **Client ID** is public and can be embedded in the application
- **Client Secret** is NOT required when using PKCE
- **PKCE** provides the security instead of the client secret
- This follows [Google's OAuth 2.0 for Mobile & Desktop Apps](https://developers.google.com/identity/protocols/oauth2/native-app) guidelines

## Development Setup

### 1. Get OAuth Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
2. Create a new project or select an existing one
3. Enable the **Gmail API**
4. Create **OAuth 2.0 Client ID** credentials
5. Choose application type: **Desktop app**
6. Note your Client ID (looks like: `xxxxx.apps.googleusercontent.com`)

### 2. Configure Environment Variables

Copy the example environment file:
```bash
cp .env.example .env
```

Edit `.env` and add your Client ID:
```
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
```

**Note**: You do NOT need a `GOOGLE_CLIENT_SECRET` for PKCE-based desktop apps. If you have one, you can optionally add it, but it's not security-critical.

### 3. Run the Application

```bash
npm run tauri dev
```

The app will automatically load the OAuth credentials from the `.env` file.

## Production Builds

For production builds, you have two options:

### Option 1: Environment Variable at Build Time

Set the environment variable when building:
```bash
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com npm run tauri build
```

### Option 2: Embed in Code (Recommended for Desktop Apps)

Since the client ID is public for desktop apps, you can embed it directly in the code:

Edit `src-tauri/src/auth/oauth.rs` and update the fallback value in the `get_client_id()` function:

```rust
option_env!("GOOGLE_CLIENT_ID")
    .unwrap_or("your-production-client-id.apps.googleusercontent.com")
    .to_string()
```

## Why This is Secure

1. **PKCE Protection**: Each OAuth flow generates a unique code verifier and challenge
2. **No Client Secret Needed**: Desktop apps can't keep secrets (they're distributed to users)
3. **Google's Recommendation**: This follows Google's official OAuth guidelines for native apps
4. **Redirect to Localhost**: The OAuth callback goes to localhost, which only the local app can intercept

## Testing OAuth Flow

1. Run the app in development mode
2. Click "Sign In with Google"
3. Complete the OAuth flow in your browser
4. You'll be redirected to `http://localhost:3000/callback`
5. The app will exchange the code for tokens using PKCE
6. Tokens are securely stored in the system keychain

## Security Checklist

- [x] No client secrets in git
- [x] `.env` file in `.gitignore`
- [x] Using PKCE for OAuth flow
- [x] Tokens stored in system keychain (not plaintext)
- [x] Client ID is public (safe to embed)

## Troubleshooting

### "Failed to bind to port 3000"
Another application is using port 3000. Either:
- Stop the other application
- Change `REDIRECT_URI` in `oauth.rs` to use a different port

### "Invalid client ID"
- Double-check your client ID in `.env`
- Ensure you enabled Gmail API in Google Cloud Console
- Verify the OAuth client type is "Desktop app"

### "redirect_uri_mismatch"
- In Google Cloud Console, add `http://localhost:3000/callback` to authorized redirect URIs

## References

- [Google OAuth 2.0 for Mobile & Desktop Apps](https://developers.google.com/identity/protocols/oauth2/native-app)
- [OAuth 2.0 PKCE RFC](https://tools.ietf.org/html/rfc7636)
- [Why Desktop Apps Don't Need Client Secrets](https://www.oauth.com/oauth2-servers/mobile-and-native-apps/)
