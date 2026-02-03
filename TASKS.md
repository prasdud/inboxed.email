# Inboxed - Implementation Tasks

## Phase 3: Local LLM Integration

### Completed Tasks

#### Backend (Rust)

- [x] **Add LLM Dependencies** - `Cargo.toml`
  - Added `llama-cpp-2` for inference with Metal support
  - Added `hf-hub` for HuggingFace model downloads
  - Added `futures` for async operations
  - Added `directories` for cross-platform paths

- [x] **Create Model Manager** - `src/llm/model_manager.rs`
  - Download models from HuggingFace
  - Cache models in `~/Library/Application Support/inboxed/models/`
  - Support multiple model options
  - Track download progress

- [x] **Create LLM Engine** - `src/llm/engine.rs`
  - Load GGUF models with Metal GPU acceleration
  - Streaming token generation
  - Configurable sampling (temperature, top_p)
  - Stop sequence support

- [x] **Update Summarizer** - `src/llm/summarizer.rs`
  - Dynamic summary length based on email size
  - Support for LFM2.5 and Qwen prompt formats
  - Streaming summarization
  - Fallback keyword-based analysis when no model

- [x] **AI Commands** - `src/commands/ai.rs`
  - `check_model_status` - Check if model is downloaded/ready
  - `download_model` - Download default model
  - `download_model_by_id` - Download specific model
  - `get_available_ai_models` - List model options
  - `init_ai` / `init_ai_fallback` - Initialize AI system
  - `summarize_email` / `summarize_email_stream` - Generate summaries
  - `get_email_insights` - Extract insights
  - `classify_priority` - Classify priority level

- [x] **Token Persistence Fix** - `src/auth/storage.rs`
  - Store tokens in `~/.inboxed/tokens.json` instead of temp dir
  - Auto-refresh expired tokens using refresh token

#### Frontend (React/TypeScript)

- [x] **AI Store** - `src/stores/aiStore.ts`
  - Track model status (not_downloaded, downloading, ready, etc.)
  - Download progress tracking
  - Model selection state

- [x] **Model Download Screen** - `src/components/Setup/ModelDownload.tsx`
  - First-run model selection
  - Download progress display
  - Model comparison cards

- [x] **Model Settings Page** - `src/components/Settings/ModelSettings.tsx`
  - View current AI status
  - Download progress monitoring
  - Switch between models
  - Accessible from sidebar

- [x] **Sidebar Update** - `src/components/Sidebar/Sidebar.tsx`
  - AI status indicator with color coding
  - Download progress bar
  - Quick access to model settings

- [x] **Email Viewer Update** - `src/components/EmailViewer/EmailViewer.tsx`
  - Streaming summary display
  - Disable AI Summary when not ready
  - Show download progress in button
  - Tooltip when AI unavailable

- [x] **App Flow Update** - `src/App.tsx`
  - Model download screen on first launch
  - Model settings modal integration

---

## Available AI Models

| Model ID | Name | Size | Speed | Min RAM |
|----------|------|------|-------|---------|
| `lfm2.5-1.2b-q4` | LFM2.5 1.2B (Recommended) | 731 MB | 200+ tok/s | 2 GB |
| `lfm2.5-1.2b-q8` | LFM2.5 1.2B High Quality | 1.25 GB | 150+ tok/s | 4 GB |
| `qwen2.5-3b-q4` | Qwen 2.5 3B | 2 GB | 70-90 tok/s | 8 GB |

---

## Dynamic Summary Length

| Email Words | Summary Length | Max Tokens |
|-------------|----------------|------------|
| 0-50 | 1 sentence | 50 |
| 51-150 | 1-2 sentences | 80 |
| 151-400 | 2-3 sentences | 120 |
| 401-800 | 3-4 sentences | 180 |
| 800+ | 4-5 sentences (comprehensive) | 250 |

---

## File Structure

```
src-tauri/
├── src/
│   ├── llm/
│   │   ├── mod.rs              # Module exports
│   │   ├── model_manager.rs    # Model download & caching
│   │   ├── engine.rs           # LLM inference engine
│   │   └── summarizer.rs       # Email summarization
│   ├── commands/
│   │   └── ai.rs               # Tauri AI commands
│   └── auth/
│       └── storage.rs          # Token persistence
│
src/
├── stores/
│   └── aiStore.ts              # AI state management
├── components/
│   ├── Setup/
│   │   └── ModelDownload.tsx   # First-run download UI
│   ├── Settings/
│   │   └── ModelSettings.tsx   # Model management page
│   ├── Sidebar/
│   │   └── Sidebar.tsx         # AI status indicator
│   └── EmailViewer/
│       └── EmailViewer.tsx     # Summary display
└── App.tsx                     # App routing
```

---

## Future Tasks

- [ ] Model deletion/cleanup
- [ ] Custom model import
- [ ] Batch email summarization
- [ ] Smart inbox sorting based on AI classification
- [ ] Email reply suggestions
- [ ] Windows support (Vulkan backend)
