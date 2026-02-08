# Changelog

All notable changes to **Inboxed** are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Added
- **Multi-account support** — connect Gmail, Outlook, Yahoo, or custom IMAP accounts from a single app
  - `accounts` table in SQLite for managing multiple email accounts
  - Account switcher UI in the sidebar with provider color indicators
  - Per-account OAuth token storage (`~/.inboxed/account_tokens.json`)
  - `AccountManager` for IMAP client lifecycle and caching
- **Generic IMAP/SMTP client** (`imap_client.rs`, 734 lines) — provider-agnostic email operations
  - Full IMAP support: list folders, fetch messages, flag operations, move/delete
  - SMTP sending with HTML + plain text multipart
  - OAuth2 (XOAUTH2) and password authentication
  - Thread ID computation from In-Reply-To/References headers
- **IMAP IDLE push notifications** (`idle.rs`) — real-time email delivery without polling
  - Per-account IDLE loop with 29-minute RFC 2177 timeout compliance
  - Automatic reconnection with 30-second retry
  - Emits `email:new_mail` Tauri event to frontend
- **Server presets** for Gmail, Outlook, Yahoo with correct IMAP/SMTP hosts and ports
- **Provider-agnostic OAuth** — refactored OAuth module to support multiple providers
  - Google OAuth with Gmail IMAP scope
  - Microsoft OAuth with IMAP.AccessAsUser.All + SMTP.Send scopes
  - PKCE support for all desktop OAuth flows
- **LoginScreen multi-provider UI** — provider selection (Gmail, Outlook, Custom IMAP) with custom server form
- **Email provider trait** (`provider.rs`) — abstraction layer for plugging in new providers
- **Account store** (`accountStore.ts`) — frontend state management for multi-account

### Changed
- Email ID format changed to `{account_id}:{folder}:{uid}` for multi-account disambiguation
- `fetch_emails` now routes through active account's IMAP client with legacy Gmail API fallback
- Database schema extended: `emails` table gains `account_id`, `uid`, `folder`, `message_id` columns
- `ComposeModal` updated to pass account context
- Auth store updated for provider-aware authentication flow

### Fixed
- **Embed Emails / Build Index / Re-index never finding emails** — Tauri app identifier was `com.mohitsingh.tauri-app` causing `app.path().app_data_dir()` to resolve to a different directory than `ProjectDirs::from("com", "inboxed", "inboxed")` where emails are actually stored; changed identifier to `com.inboxed.inboxed` so both paths match
- **VectorDatabase creating its own empty `emails` table** — `VectorDatabase::new()` called `create_tables()` which created all tables (including an empty `emails` table) in the vector DB file; `get_unembedded_email_ids()` then queried this empty table and always returned 0 results. Added `create_vector_tables()` that only creates `email_embeddings` and `embedding_status` tables, and rewrote `embed_all_emails` to fetch IDs from `EmailDatabase` and filter out already-embedded ones via `VectorDatabase::get_embedded_email_ids()`
- **Errors silently swallowed on Re-index, Build Index, and Embed Emails** — all three buttons caught errors with `console.error` only; added visible dismissible error banners in Smart Inbox and Model Settings
- **No loading feedback on Re-index / Build Index buttons** — added `isReindexing` and `isBuildingIndex` loading states that disable buttons and show progress text during async operations
- **UI crash during indexing/reindexing** — `indexing:complete` listener now properly `await`s and wraps `embedAllEmails()`, `fetchSmartInbox()`, `getIndexingStatus()` in try-catch; previously unhandled promise rejections would crash React
- **AI init failure blocks indexing** — AI initialization before indexing is now wrapped in its own try-catch so indexing proceeds with fallback mode instead of failing entirely
- **Settings "Embed Models" button missing** — added the `embeddingModelDownloaded && !ragInitialized` state that previously showed no button; now displays "Initialize Model"
- **Smart Inbox limited to 50 emails** — raised default query limit from 50 to 500 across frontend stores and backend command fallbacks
- **"0/0 INDEXED" badge showing incorrectly** — embedding status badge is now hidden when both `total_emails` and `embedded_emails` are 0
- Stale indexing state now reset on failed `startIndexing` call
- **Chat panel breaking app responsiveness** — opening the AI chat panel squeezed the email list to zero width on smaller viewports; added `overflow-hidden min-w-0` to the main content wrapper in `App.tsx`, `flex-wrap` to the action bar buttons, `min-w-0` to the email list container, and removed `flex-shrink-0` from the chat panel so both panes share space proportionally

---

## [0.5.0] - 2026-02-08

### Added
- **Embedding model support** (all-MiniLM-L6-v2, ~33 MB, 384-dim vectors)
  - `EmbeddingEngine` with Metal acceleration on macOS via Candle
  - Model download from HuggingFace Hub with direct HTTP fallback
  - `email_embeddings` table for vector storage
  - `embedding_status` table for progress tracking
- **RAG (Retrieval-Augmented Generation) system**
  - `RagEngine` for semantic email search using cosine similarity
  - `embed_all_emails` batch command with progress events
  - `search_emails_semantic` for vector-based email search
  - `find_similar_emails` to discover related emails
  - `chat_with_context` for RAG-powered AI chat
- **Model Settings UI** — download, activate, and delete LLM models; manage embedding model; embed emails with progress tracking
- **ChatPanel** component for conversational AI queries over email data
- **Smart Inbox embedding integration** — "Build Index" / "Setup AI Index" buttons, embedding progress bar, indexed email count badge

---

## [0.4.0] - 2026-02-07

### Changed
- **Complete website landing page redesign**
  - Hero: app screenshot placeholder with macOS window mockup, fade-in animations, overline label
  - Navbar: mobile hamburger menu with smooth transitions, Blog link
  - Features: card numbering, accent lines, scroll-triggered animations, improved "Engineered for Silicon" section
  - Pricing: editorial dashes, subtitles, hover shadows; free tier now includes all AI models
  - Footer: CTA section, back-to-top button, underline-from-left hover effect, 5-column grid
  - Comparisons: updated Superhuman and 0.email pages to reflect Inboxed as free

---

## [0.3.0] - 2026-02-06

### Added
- **Responsive layout** for all screen sizes

### Fixed
- Classic view email list scrolling — added `overflow-hidden` to flex wrapper so height constraint propagates to `EmailList`'s `overflow-y-auto` container

---

## [0.2.0] - 2026-02-03

### Added
- **Phase 4: Smart Inbox with AI-powered email management**
  - SQLite database (`emails.db`) for local email storage and caching
  - AI-powered priority scoring: HIGH / MEDIUM / LOW with keyword-based classification
  - Automatic email categorization: conversation, meetings, financial, newsletters, notifications, general
  - Background email indexing with real-time progress tracking via Tauri events
  - Smart Inbox UI with priority-sorted email list, unread highlighting, category badges
  - Natural language chat interface for email queries with intent detection
  - Insight detection: deadlines, meetings, financial content
  - Toggle between Smart Inbox and Classic views
  - `indexing_status` table for tracking processing progress
  - LLM summarization with `spawn_blocking` isolation and fallback to keyword extraction

---

## [0.1.0] - 2026-02-03

### Added
- Initial project: Tauri + React + TypeScript email client
- Gmail OAuth2 integration with PKCE for desktop security
- Email list view with read/unread, starred, attachment indicators
- Email detail viewer with HTML rendering
- Compose modal for sending emails via Gmail API
- Sidebar with folder navigation (Inbox, Sent, Drafts, Trash, Spam)

### Changed
- **Rebrand from LocalMail to Inboxed** — updated package names, service identifiers (`com.inboxed.app`), directory paths, UI components, and documentation

### Security
- Moved OAuth `CLIENT_ID` and `CLIENT_SECRET` to environment variables (`.env`)
- Added `.env.example` with setup documentation
- Updated `.gitignore` to exclude credential files
