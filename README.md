# LocalMail

AI-powered email desktop client with local LLM inference.

## Overview

LocalMail is a lightweight, privacy-focused email client that runs a local LLM natively on Apple Silicon to intelligently sort, summarize, and manage emails. Built with Tauri + React for a native experience with minimal overhead.

## Tech Stack

- **Framework**: Tauri 2.0 (Rust backend + React frontend)
- **Frontend**: React 19 + TypeScript + Tailwind CSS
- **State Management**: Zustand
- **Build Tool**: Vite
- **Design**: Minimalist Monochrome aesthetic
- **LLM**: Keyword-based analysis (llama.cpp integration ready)

## Features

- **Gmail Integration**: OAuth 2.0 PKCE authentication with secure keychain storage
- **Full Email Operations**: Read, compose, reply, archive, delete, star emails
- **AI-Powered Summaries**: Priority classification, smart insights, and natural language summaries
- **Minimalist Monochrome UI**: Editorial typography, clean borders, pure black & white

## Development Status

### ✅ Phase 1: Foundation (COMPLETED)

- [x] Tauri 2.0 + React + TypeScript project setup
- [x] Tailwind CSS with monochrome design system
- [x] Gmail OAuth 2.0 PKCE flow
- [x] Secure token storage in macOS Keychain
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

- [x] LLM module structure (ready for llama.cpp)
- [x] Email summarization engine
- [x] Priority classification (HIGH/MEDIUM/LOW)
- [x] Smart insights extraction
- [x] AI Summary panel in email viewer
- [x] Keyword-based analysis (demo mode)

### 📋 Future Phases

- **Phase 4**: Smart inbox with auto-sorting by priority
- **Phase 5**: Windows support and polish
- **Phase 6**: Multi-provider support (Outlook, Yahoo, IMAP)

## Getting Started

### Prerequisites

- Node.js 20.19+ or 22.12+
- Rust 1.70+
- Xcode Command Line Tools (macOS)

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Development

```bash
# Install dependencies
npm install

# Start Tauri app
npm run tauri dev

# Start dev server (frontend only)
npm run dev

# Lint code
npm run lint

# Format code
npm run format
```

### Using the App

1. **Login**: Click "Sign in with Gmail" to authenticate
2. **Browse**: View your inbox with email list and viewer
3. **Compose**: Click "COMPOSE" to write a new email
4. **Reply**: Open an email and click "REPLY"
5. **AI Summary**: Click "AI SUMMARY" to see insights and priority
6. **Actions**: Archive, delete, star, or mark emails read/unread

## Project Structure

```
emailApp/
├── src/                      # React frontend
│   ├── components/
│   │   ├── Auth/            # Login screen
│   │   ├── Sidebar/         # Navigation sidebar
│   │   ├── EmailList/       # Inbox list with avatars
│   │   ├── EmailViewer/     # Email display with AI panel
│   │   └── Compose/         # Compose modal
│   ├── stores/              # Zustand state stores
│   │   ├── authStore.ts     # Auth state
│   │   └── emailStore.ts    # Email state
│   ├── hooks/               # Custom React hooks
│   ├── lib/                 # Utility functions
│   └── App.tsx              # Main app component
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── commands/        # Tauri commands (auth, email, AI)
│   │   ├── email/           # Gmail API client
│   │   ├── auth/            # OAuth PKCE + Keychain storage
│   │   └── llm/             # AI summarization engine
│   └── Cargo.toml
├── PLAN.md                  # Detailed project plan
└── TASKS.json               # Task breakdown
```

## AI Features

The AI Summary feature provides:

- **Priority Classification**: HIGH, MEDIUM, or LOW based on urgency keywords
- **Smart Insights**: Detects meetings, deadlines, questions, and financial content
- **Natural Summaries**: Extracts key information from email body

Current implementation uses keyword-based analysis. Full LLM integration (llama.cpp with Metal acceleration) is ready to enable.

## Debugging

```bash
# Check if tokens are stored
security find-generic-password -s com.localmail.app

# View Rust logs
RUST_LOG=debug npm run tauri dev

# Clear stored tokens (force re-login)
security delete-generic-password -s com.localmail.app -a gmail_access_token
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT
