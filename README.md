# Inboxed

AI-powered email desktop client with local LLM inference.

## Overview

Inboxed is a lightweight, privacy-focused email client that runs a local LLM natively on Apple Silicon to intelligently sort, summarize, and manage emails. Built with Tauri + React for a native experience with minimal overhead.

## Tech Stack

- **Framework**: Tauri 2.0 (Rust backend + React frontend)
- **Frontend**: React 19 + TypeScript + Tailwind CSS
- **State Management**: Zustand
- **Build Tool**: Vite
- **Design**: Minimalist Monochrome aesthetic
- **LLM**: llama-cpp-2 with Metal acceleration

## Features

- **Gmail Integration**: OAuth 2.0 PKCE authentication with secure token storage
- **Full Email Operations**: Read, compose, reply, archive, delete, star emails
- **Smart Inbox**: AI-powered priority sorting with local SQLite database
- **Intelligent Categorization**: Automatic email categorization (meetings, financial, newsletters, etc.)
- **Local AI Summaries**: On-device LLM with streaming output
- **Adaptive Summaries**: Length adjusts based on email size
- **Priority Classification**: HIGH/MEDIUM/LOW with AI analysis
- **Smart Insights**: Action items, deadlines, meetings, financial mentions
- **Natural Language Chat**: Ask questions about your emails in plain English
- **Background Indexing**: Non-blocking email processing with real-time progress
- **Model Management**: Download and switch between AI models
- **Privacy First**: All AI processing and data storage happens locally

## Development Status

### ✅ Phase 1: Foundation (COMPLETED)

- [x] Tauri 2.0 + React + TypeScript project setup
- [x] Tailwind CSS with monochrome design system
- [x] Gmail OAuth 2.0 PKCE flow
- [x] Secure token storage (keychain + file fallback)
- [x] Gmail API client in Rust
- [x] Email list with avatars and read/unread indicators
- [x] Email viewer with HTML rendering

### ✅ Phase 2: Email Operations (COMPLETED)

- [x] Compose modal with editorial design
- [x] Send emails via Gmail API
- [x] Reply functionality
- [x] Archive emails
- [x] Delete/Trash emails
- [x] Star/Unstar emails
- [x] Mark as read/unread

### ✅ Phase 3: AI Intelligence (COMPLETED)

- [x] llama-cpp-2 integration with Metal GPU acceleration
- [x] HuggingFace model downloads
- [x] Model manager with progress tracking
- [x] Streaming text generation
- [x] Adaptive summary length based on email size
- [x] Priority classification with LLM
- [x] Smart insights extraction
- [x] Model settings page
- [x] Sidebar AI status indicator
- [x] Fallback keyword-based analysis

### ✅ Phase 4: Smart Inbox (COMPLETED)

- [x] SQLite database for local email storage
- [x] AI-powered priority scoring (HIGH/MEDIUM/LOW)
- [x] Automatic email categorization (meetings, financial, newsletters, etc.)
- [x] Background email indexing with progress tracking
- [x] Smart Inbox view sorted by importance
- [x] Natural language chat interface for email queries
- [x] Insight detection (deadlines, meetings, financial mentions)
- [x] Toggle between Smart Inbox and Classic views
- [x] Search and filter capabilities

### 📋 Future Phases

- **Phase 5**: Windows support (Vulkan backend)
- **Phase 6**: Multi-provider support (Outlook, Yahoo, IMAP)

## Getting Started

### Prerequisites

- macOS (Apple Silicon recommended)
- Node.js 20.19+ or 22.12+
- Rust 1.70+
- cmake (`brew install cmake`)
- Xcode Command Line Tools

### Install Dependencies

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cmake (required for llama.cpp)
brew install cmake

# Install Node dependencies
npm install
```

### Development

```bash
# Start Tauri app
npm run tauri dev

# Start dev server (frontend only)
npm run dev

# Build for production
npm run tauri build
```

### First Run

1. **Sign in** with your Google account
2. **Select AI Model** to download:
   - **LFM2.5 1.2B** (731 MB) - Recommended, ~200 tok/s
   - **LFM2.5 1.2B Q8** (1.25 GB) - Higher quality
   - **Qwen 2.5 3B** (2 GB) - Better reasoning
3. **Wait** for model download to complete
4. **Start Indexing** - Click "Start Indexing" in Smart Inbox to process emails
5. **Use Smart Inbox** - View priority-sorted emails with AI summaries
6. **Ask AI** - Use the chat interface for natural language queries

## AI Models

Models are downloaded from HuggingFace and stored locally:

| Model | Size | Speed | RAM | Best For |
|-------|------|-------|-----|----------|
| LFM2.5 1.2B Q4 | 731 MB | 200+ tok/s | 2 GB | Fast summaries |
| LFM2.5 1.2B Q8 | 1.25 GB | 150+ tok/s | 4 GB | Better quality |
| Qwen 2.5 3B | 2 GB | 70-90 tok/s | 8 GB | Complex reasoning |

**Storage Location**: `~/Library/Application Support/inboxed/models/`

## Adaptive Summary Length

Summaries automatically adjust based on email length:

| Email Length | Summary |
|--------------|---------|
| 0-50 words | 1 sentence |
| 51-150 words | 1-2 sentences |
| 151-400 words | 2-3 sentences |
| 401-800 words | 3-4 sentences |
| 800+ words | 4-5 comprehensive sentences |

## Smart Inbox

The Smart Inbox provides AI-powered email management:

**Priority Sorting:**
- 🔴 **HIGH** - Urgent emails (score >= 0.7)
- 🟡 **MEDIUM** - Normal emails (score >= 0.4)
- ⚪ **LOW** - Less important emails (score < 0.4)

**Categories:**
- conversation, meetings, financial, newsletters, notifications, general

**Chat Interface:**
- Natural language queries: "Show me today's emails", "Find important emails"
- Quick action buttons
- Email summaries in responses

**Database Location:** `~/Library/Application Support/inboxed/emails.db`

## Project Structure

```
emailApp/
├── src/                      # React frontend
│   ├── components/
│   │   ├── Auth/             # Login screen
│   │   ├── Sidebar/          # Navigation + AI status
│   │   ├── EmailList/        # Inbox list
│   │   ├── EmailViewer/      # Email display + AI panel
│   │   ├── SmartInbox/       # Smart inbox view + chat
│   │   ├── Compose/          # Compose modal
│   │   └── Settings/         # Model settings page
│   ├── stores/
│   │   ├── authStore.ts      # Auth state
│   │   ├── emailStore.ts     # Email state
│   │   ├── aiStore.ts        # AI/model state
│   │   └── smartInboxStore.ts # Smart inbox state
│   └── App.tsx
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── commands/         # Tauri commands
│   │   │   ├── ai.rs         # AI commands
│   │   │   ├── auth.rs       # Auth commands
│   │   │   ├── email.rs      # Email commands
│   │   │   └── db.rs         # Database commands
│   │   ├── db/
│   │   │   ├── schema.rs     # Database schema
│   │   │   └── email_db.rs   # Database operations
│   │   ├── llm/
│   │   │   ├── engine.rs     # LLM inference
│   │   │   ├── model_manager.rs  # Downloads
│   │   │   └── summarizer.rs # Summarization
│   │   ├── auth/             # OAuth + storage
│   │   └── email/            # Gmail API
│   └── Cargo.toml
├── PHASE_4_IMPLEMENTATION.md # Phase 4 detailed docs
├── PHASE_4_QUICKSTART.md     # Phase 4 quick start
├── TASKS.md                  # Implementation progress
└── README.md
```

## Debugging

```bash
# Check stored tokens (dev mode)
cat ~/.inboxed/tokens.json

# View Rust logs
RUST_LOG=debug npm run tauri dev

# Check model directory
ls ~/Library/Application\ Support/inboxed/models/

# Check database
ls ~/Library/Application\ Support/inboxed/emails.db

# View database contents (requires sqlite3)
sqlite3 ~/Library/Application\ Support/inboxed/emails.db "SELECT COUNT(*) FROM emails"
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/)
- [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT
