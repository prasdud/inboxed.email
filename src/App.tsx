import { useEffect, useState } from 'react'
import Sidebar from './components/Sidebar'
import { LoginScreen } from './components/Auth'
import EmailList from './components/EmailList'
import EmailViewer from './components/EmailViewer'
import { ComposeModal } from './components/Compose'
import { ModelDownload } from './components/Setup'
import { ModelSettings, StorageSettings } from './components/Settings'
import { SmartInbox } from './components/SmartInbox'
import { useAuthStore } from './stores/authStore'
import { useAiStore } from './stores/aiStore'

type AppState = 'loading' | 'login' | 'setup' | 'ready'
type ViewMode = 'smart' | 'classic'

function App() {
  const [activeFolder, setActiveFolder] = useState('inbox')
  const [showCompose, setShowCompose] = useState(false)
  const [showModelSettings, setShowModelSettings] = useState(false)
  const [showStorageSettings, setShowStorageSettings] = useState(false)
  const [appState, setAppState] = useState<AppState>('loading')
  const [viewMode, setViewMode] = useState<ViewMode>('smart')
  const { authenticated, loading: authLoading, checkAuth } = useAuthStore()
  const { modelStatus, checkModelStatus, initAi } = useAiStore()

  useEffect(() => {
    checkAuth()
  }, [checkAuth])

  // Check model status after authentication
  useEffect(() => {
    if (authenticated) {
      checkModelStatus()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [authenticated])

  // Initialize AI system after model status is checked (works with or without downloaded model)
  useEffect(() => {
    if (authenticated && (modelStatus.status === 'downloaded' || modelStatus.status === 'not_downloaded')) {
      initAi().catch(console.error)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [authenticated, modelStatus.status])

  // Determine app state based on auth and model status
  useEffect(() => {
    if (authLoading) {
      setAppState('loading')
    } else if (!authenticated) {
      setAppState('login')
    } else if (modelStatus.status === 'not_downloaded') {
      setAppState('setup')
    } else if (modelStatus.status === 'loading') {
      // Show loading while model is being loaded
      setAppState('loading')
    } else {
      setAppState('ready')
    }
  }, [authLoading, authenticated, modelStatus])

  // Loading state
  if (appState === 'loading') {
    return (
      <div className="flex items-center justify-center h-screen bg-background">
        <div className="text-center">
          <div className="w-12 h-12 border-[2px] border-foreground border-t-transparent animate-spin mx-auto mb-4" />
          <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground">
            Loading
          </p>
        </div>
      </div>
    )
  }

  // Login screen
  if (appState === 'login') {
    return <LoginScreen />
  }

  // Model download setup screen
  if (appState === 'setup') {
    return (
      <ModelDownload
        onComplete={() => setAppState('ready')}
        onSkip={() => setAppState('ready')}
      />
    )
  }

  // Main app
  return (
    <>
      <div className="flex h-screen bg-background">
        <Sidebar
          onFolderSelect={setActiveFolder}
          onCompose={() => setShowCompose(true)}
          onOpenModelSettings={() => setShowModelSettings(true)}
          onOpenStorageSettings={() => setShowStorageSettings(true)}
        />

        <div className="flex-1 flex flex-col">
          {/* View Toggle Header */}
          <div className="px-6 py-3 border-b-[2px] border-foreground flex items-center justify-between">
            <h2 className="font-display text-2xl tracking-tight capitalize">
              {viewMode === 'smart' ? 'Smart Inbox' : activeFolder}
            </h2>
            <div className="flex gap-2">
              <button
                onClick={() => setViewMode('smart')}
                className={`px-4 py-1.5 font-mono text-xs uppercase tracking-wider transition-colors ${viewMode === 'smart'
                  ? 'bg-foreground text-background'
                  : 'border-[2px] border-foreground hover:bg-foreground/10'
                  }`}
              >
                Smart
              </button>
              <button
                onClick={() => setViewMode('classic')}
                className={`px-4 py-1.5 font-mono text-xs uppercase tracking-wider transition-colors ${viewMode === 'classic'
                  ? 'bg-foreground text-background'
                  : 'border-[2px] border-foreground hover:bg-foreground/10'
                  }`}
              >
                Classic
              </button>
            </div>
          </div>

          {/* Content based on view mode */}
          {viewMode === 'smart' ? (
            <SmartInbox onCompose={() => setShowCompose(true)} />
          ) : (
            <div className="flex-1 flex overflow-hidden">
              {/* Email List */}
              <div className="w-[32rem] border-r-[2px] border-foreground flex flex-col">
                <EmailList />
              </div>

              {/* Email Viewer */}
              <EmailViewer />
            </div>
          )}
        </div>
      </div>

      <ComposeModal isOpen={showCompose} onClose={() => setShowCompose(false)} />

      {/* Model Settings */}
      {showModelSettings && (
        <ModelSettings onClose={() => setShowModelSettings(false)} />
      )}

      {/* Storage Settings */}
      {showStorageSettings && (
        <StorageSettings onClose={() => setShowStorageSettings(false)} />
      )}
    </>
  )
}

export default App

