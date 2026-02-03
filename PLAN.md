# Inboxed - AI-Powered Email Desktop Client

## Project Overview

A lightweight, privacy-focused email desktop client that runs a local LLM natively on Apple Silicon (and later Windows) to intelligently sort, summarize, and manage emails. The AI model downloads on first run—no cloud dependency for AI features.

---

## Technical Decisions

### Framework: **Tauri** (not Electron)

| Criteria | Tauri | Electron |
|----------|-------|----------|
| Bundle size | ~10-15MB | ~150-200MB |
| Memory usage | Low (native webview) | High (Chromium) |
| Startup time | Fast | Slower |
| Native integrations | Excellent (Rust) | Good (Node.js) |
| LLM integration | Native via Rust bindings | Via child process |
| Cross-platform | ✅ macOS, Windows, Linux | ✅ macOS, Windows, Linux |

**Verdict**: Tauri is significantly lighter and Rust integrates seamlessly with `llama.cpp` bindings for local LLM inference.

### Local LLM Stack

- **Runtime**: `llama-cpp-2` with Metal support (Apple Silicon) / Vulkan (Windows)
- **Rust bindings**: `llama-cpp-2` crate
- **Models**:
  - LFM2.5 1.2B (731 MB) - Recommended
  - Qwen 2.5 3B (2 GB) - Better reasoning
- **Download**: HuggingFace via `hf-hub` crate

### Frontend

- **Framework**: React 19 + TypeScript
- **UI Library**: Tailwind CSS (custom monochrome design)
- **State Management**: Zustand

### Email Integration

- **Gmail**: OAuth 2.0 PKCE + Gmail API
- **Future providers**: IMAP/SMTP standard protocol support

---

## Multi-Phase Development Plan

---

## ✅ Phase 1: Foundation & Basic Email Client (COMPLETED)
**Goal**: Working email client that can read Gmail emails

### Tasks

#### 1.1 Project Setup
- [x] Initialize Tauri project with React + TypeScript frontend
- [x] Configure build system for macOS (Apple Silicon)
- [x] Set up project structure and linting (ESLint, Prettier)
- [x] Create basic window with titlebar and layout

#### 1.2 Gmail OAuth Integration
- [x] Register app with Google Cloud Console
- [x] Implement OAuth 2.0 PKCE flow in Tauri
- [x] Securely store tokens in system keychain (keyring crate)
- [x] Handle token refresh logic
- [x] File-based token storage for dev mode

#### 1.3 Email Fetching & Display
- [x] Implement Gmail API client in Rust backend
- [x] Fetch inbox emails (paginated)
- [x] Parse email headers and body (HTML/plain text)
- [x] Display email list in React frontend
- [x] Implement email detail view with proper HTML rendering

#### 1.4 Basic UI Shell
- [x] Create sidebar with folders (Inbox, Sent, Drafts, Trash)
- [x] Implement email list component with sender, subject, date
- [x] Create email viewer with proper formatting
- [x] Add basic loading states and error handling

### Deliverable
✅ **Working prototype**: App that authenticates with Gmail and displays inbox emails in a clean interface.

---

## ✅ Phase 2: Full Email Operations (COMPLETED)
**Goal**: Complete email client functionality (send, reply, delete, search)

### Tasks

#### 2.1 Compose & Send
- [x] Create compose email modal/view
- [x] Implement send functionality via Gmail API
- [ ] Add attachment support (file picker + upload) - *deferred*
- [ ] Save drafts locally and to Gmail - *deferred*

#### 2.2 Email Actions
- [x] Reply / Reply All
- [x] Move to trash / Delete
- [x] Archive emails
- [x] Mark as read/unread
- [x] Star/flag emails

#### 2.3 Labels & Folders
- [ ] Fetch and display Gmail labels - *deferred*
- [ ] Apply/remove labels from emails - *deferred*
- [ ] Filter emails by label - *deferred*

#### 2.4 Search
- [ ] Implement local search (cached emails) - *deferred*
- [ ] Implement Gmail API search - *deferred*

#### 2.5 Sync & Caching
- [ ] Implement local SQLite database - *deferred*
- [ ] Background sync with Gmail - *deferred*

### Deliverable
✅ **Functional email client**: Can send, receive, and organize emails.

---

## ✅ Phase 3: Local LLM Integration (COMPLETED)
**Goal**: Integrate local LLM for email summarization

### Tasks

#### 3.1 LLM Runtime Setup
- [x] Integrate `llama-cpp-2` into Tauri backend
- [x] Configure Metal acceleration for Apple Silicon
- [x] Create model loading/unloading system
- [x] Implement inference in background thread

#### 3.2 Model Management
- [x] Implement HuggingFace model downloads via `hf-hub`
- [x] Model manager with progress tracking
- [x] Support multiple model options (LFM2.5, Qwen)
- [x] Model settings page in UI
- [x] Sidebar AI status indicator

#### 3.3 Email Summarization
- [x] Design adaptive summarization prompts
- [x] Implement single email summarization
- [x] Add "AI Summary" button to email view
- [x] Streaming token output
- [x] Fallback keyword-based analysis

#### 3.4 Smart Analysis
- [x] Priority classification (HIGH/MEDIUM/LOW)
- [x] Insights extraction (meetings, deadlines, financial)
- [x] Adaptive summary length based on email size

#### 3.5 Performance Optimization
- [x] Background inference (Rust threads)
- [x] Progress indicators for downloads
- [x] Disable AI button while not ready

### Deliverable
✅ **AI-powered summaries**: Click any email to get an instant local AI summary. No data leaves the device.

### Features Implemented
- Dynamic summary length (1-5 sentences based on email length)
- Multiple model options with download management
- Streaming text generation
- Real-time download progress in sidebar
- Model settings page

---

## 📋 Phase 4: Smart Sorting & Categorization
**Goal**: AI automatically categorizes and prioritizes emails

### Tasks

#### 4.1 Email Classification
- [ ] Design classification prompt (categories: Important, Updates, Social, Promotions, Spam)
- [ ] Implement batch classification for inbox
- [ ] Store categories in local database
- [ ] Add category filters to UI

#### 4.2 Priority Scoring
- [ ] Design priority scoring system (1-5 or High/Medium/Low)
- [ ] Factors: sender importance, content urgency, keywords
- [ ] Learn from user behavior (which emails they open/reply to)
- [ ] Display priority indicators in email list

#### 4.3 Smart Inbox Views
- [ ] "Priority Inbox" - sorted by AI priority
- [ ] "Focus" view - only important emails
- [ ] "Daily Digest" - summary of day's emails
- [ ] Custom smart filters

#### 4.4 Action Suggestions
- [ ] Suggest replies for simple emails
- [ ] Identify emails that need response
- [ ] Flag potential deadlines/action items
- [ ] "Needs Reply" indicator

### Deliverable
✅ **Smart inbox**: Emails automatically sorted by importance with AI-suggested actions.

---

## 📋 Phase 5: Windows Support & Polish
**Goal**: Cross-platform release with polished UX

### Tasks

#### 5.1 Windows Build
- [ ] Configure Tauri for Windows builds
- [ ] Set up Vulkan/DirectML for LLM inference on Windows
- [ ] Test on various Windows hardware (with/without GPU)
- [ ] Create Windows installer (MSI/NSIS)

#### 5.2 CPU Fallback
- [ ] Implement CPU-only inference path
- [ ] Use smaller model for CPU
- [ ] Auto-detect hardware capabilities
- [ ] Settings to force CPU/GPU mode

#### 5.3 UI/UX Polish
- [ ] Dark/light theme with system preference sync
- [ ] Keyboard shortcuts
- [ ] Notification system (new emails, summaries ready)
- [ ] Settings panel (sync frequency, AI options, appearance)
- [ ] Onboarding flow for new users

#### 5.4 Performance & Stability
- [ ] Memory leak auditing
- [ ] Crash reporting (opt-in)
- [ ] Performance profiling and optimization
- [ ] Reduce bundle size where possible

#### 5.5 Auto-Updates
- [ ] Implement Tauri updater
- [ ] Model update mechanism (download new models)
- [ ] Changelog/release notes display

### Deliverable
✅ **Production-ready v1.0**: Polished app running on macOS and Windows with full AI features.

---

## 📋 Phase 6: Multi-Provider Support
**Goal**: Support Outlook, Yahoo, and generic IMAP accounts

### Tasks

#### 6.1 IMAP/SMTP Implementation
- [ ] Generic IMAP client in Rust (async-imap or similar)
- [ ] SMTP client for sending
- [ ] Connection pooling and keep-alive
- [ ] Handle various IMAP quirks

#### 6.2 Provider-Specific Adapters
- [ ] Outlook/Microsoft 365 (OAuth + Graph API)
- [ ] Yahoo Mail (OAuth)
- [ ] Generic IMAP/SMTP (manual server config)
- [ ] Provider auto-detection from email domain

#### 6.3 Unified Inbox
- [ ] Aggregate emails from multiple accounts
- [ ] Color coding by account
- [ ] Account-specific settings
- [ ] Cross-account search

#### 6.4 Account Management UI
- [ ] Add/remove accounts flow
- [ ] Account settings (sync frequency, notifications)
- [ ] Account health status indicators

### Deliverable
✅ **Universal email client**: Works with Gmail, Outlook, Yahoo, and any IMAP server.

---

## Project Structure

```
emailApp/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/          # Tauri commands
│   │   │   ├── ai.rs          # AI/LLM commands
│   │   │   ├── auth.rs        # Auth commands
│   │   │   └── email.rs       # Email commands
│   │   ├── email/             # Email client logic
│   │   │   ├── gmail.rs
│   │   │   └── types.rs
│   │   ├── llm/               # LLM integration
│   │   │   ├── engine.rs      # Inference engine
│   │   │   ├── model_manager.rs # Downloads
│   │   │   └── summarizer.rs  # Summarization
│   │   └── auth/              # OAuth handling
│   │       ├── oauth.rs
│   │       └── storage.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                       # React frontend
│   ├── components/
│   │   ├── Auth/
│   │   ├── EmailList/
│   │   ├── EmailViewer/
│   │   ├── Compose/
│   │   ├── Sidebar/
│   │   └── Settings/
│   ├── stores/                # Zustand stores
│   │   ├── authStore.ts
│   │   ├── emailStore.ts
│   │   └── aiStore.ts
│   ├── App.tsx
│   └── main.tsx
├── package.json
├── PLAN.md
├── TASKS.md
└── README.md
```

---

## Technology Stack Summary

| Component | Technology |
|-----------|------------|
| Framework | Tauri 2.0 |
| Frontend | React 19 + TypeScript |
| Styling | Tailwind CSS (custom) |
| State | Zustand |
| Backend | Rust |
| LLM | llama-cpp-2 with Metal |
| Models | LFM2.5 1.2B, Qwen 2.5 3B |
| Model Download | hf-hub |
| Auth | OAuth 2.0 PKCE |
| Email | Gmail API |

---

## Success Metrics

- **Phase 1**: ✅ App loads Gmail inbox in <3 seconds
- **Phase 2**: ✅ Basic email operations working
- **Phase 3**: ✅ Summarization in <3 seconds per email (LFM2.5)
- **Phase 4**: 80%+ classification accuracy
- **Phase 5**: <100MB app bundle (excluding model)
- **Phase 6**: Works with 5+ email providers

---

## AI Model Options

| Model | Size | Speed | RAM | Use Case |
|-------|------|-------|-----|----------|
| LFM2.5 1.2B Q4 | 731 MB | 200+ tok/s | 2 GB | Fast summaries (recommended) |
| LFM2.5 1.2B Q8 | 1.25 GB | 150+ tok/s | 4 GB | Higher quality |
| Qwen 2.5 3B | 2 GB | 70-90 tok/s | 8 GB | Better reasoning |

---

*Each phase is designed to be independently testable and provides value.*
