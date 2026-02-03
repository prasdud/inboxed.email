# Inboxed - Development Progress

## Phase 1.1: Project Setup ✅ COMPLETED

**Completed on**: 2026-02-03

### Tasks Completed

#### 1.1.1 - Initialize Tauri Project ✅
- Created Tauri 2.0 project with React + TypeScript template
- Configured for macOS (Apple Silicon will be enabled when Rust is installed)
- Project builds and runs successfully

#### 1.1.2 - Set up Project Structure and Tooling ✅
- **ESLint**: Configured with React, TypeScript, and React Hooks rules
- **Prettier**: Configured for consistent code formatting
- **Tailwind CSS**: Configured with PostCSS
- **Folder Structure**: Created all directories as per PLAN.md:
  - Frontend: `src/components/{Sidebar,EmailList,EmailViewer,Compose,Settings}`
  - Frontend: `src/{hooks,stores,lib}`
  - Backend: `src-tauri/src/{commands,email,auth}`
- **Dependencies Installed**:
  - Zustand (state management)
  - Tailwind CSS + PostCSS + Autoprefixer
  - ESLint + Prettier + plugins

#### 1.4.1 - Create App Layout with Sidebar ✅
- Built responsive sidebar component with:
  - Folder navigation (Inbox, Sent, Drafts, Trash, Spam)
  - Collapsible functionality
  - Active folder highlighting
  - Dark mode support
- Created main app layout with three-panel structure:
  - Sidebar (collapsible)
  - Email list placeholder
  - Email viewer placeholder

### Files Created/Modified

**New Files**:
- `src/components/Sidebar/Sidebar.tsx` - Sidebar component
- `src/components/Sidebar/index.ts` - Barrel export
- `src/index.css` - Global styles with Tailwind directives
- `tailwind.config.js` - Tailwind configuration
- `postcss.config.js` - PostCSS configuration
- `.eslintrc.cjs` - ESLint configuration
- `.prettierrc` - Prettier configuration

**Modified Files**:
- `package.json` - Updated project name to "inboxed", added lint/format scripts
- `src-tauri/Cargo.toml` - Updated project name and description
- `src/App.tsx` - Replaced demo content with email client layout
- `src/main.tsx` - Added CSS import
- `README.md` - Updated with project details

**Deleted Files**:
- `src/App.css` - Replaced with Tailwind utilities

### Current State

**Running**: Vite dev server on http://localhost:1420/

**UI Shows**:
- ✅ Collapsible sidebar with email folders
- ✅ Empty email list with placeholder message
- ✅ Empty email viewer with placeholder message
- ✅ Dark/light mode support

**What Works**:
- Folder navigation (UI state only, no real data)
- Sidebar collapse/expand
- Responsive layout
- Code linting and formatting

### Next Steps (Phase 1.2)

**Task 1.2.1** - Set up Google Cloud Console OAuth (Manual):
1. Go to Google Cloud Console
2. Create new project or select existing
3. Enable Gmail API
4. Create OAuth 2.0 credentials (Desktop app)
5. Configure redirect URIs
6. Save client ID and secret

**Task 1.2.2** - Implement OAuth 2.0 PKCE Flow:
- Requires Rust installation first
- Create OAuth handler in `src-tauri/src/auth/`
- Open system browser for authorization
- Handle callback via custom protocol or localhost
- Exchange auth code for tokens

**Task 1.2.3** - Secure Token Storage:
- Use `keyring` crate for system keychain integration
- Store/retrieve access and refresh tokens
- Implement token refresh logic

### Prerequisites Still Needed

⚠️ **Rust**: Not installed yet. Required for:
- Building Tauri backend
- Running `npm run tauri dev`
- Implementing OAuth and Gmail API client

Install with:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Technology Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Framework | Tauri | 10-15MB vs 150MB for Electron, native Rust/LLM integration |
| Frontend | React + TypeScript | Familiar, mature, good tooling |
| Styling | Tailwind CSS | Rapid development, consistent design system |
| State | Zustand | Lightweight, simple API, no boilerplate |
| Bundler | Vite | Fast, modern, official Tauri support |

### Metrics

- **Bundle Size (Current)**: ~72 npm packages installed
- **Dev Server Startup**: ~366ms
- **Project Size**: ~305 npm packages (including devDependencies)
- **Lines of Code**: ~200 (mostly Sidebar component)

---

## Next: Phase 1.2 - Gmail OAuth Integration

See `TASKS.json` tasks 1.2.1, 1.2.2, 1.2.3 for detailed requirements.
