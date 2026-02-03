import { useEffect, useState } from 'react'
import Sidebar from './components/Sidebar'
import { LoginScreen } from './components/Auth'
import EmailList from './components/EmailList'
import EmailViewer from './components/EmailViewer'
import { ComposeModal } from './components/Compose'
import { ModelDownload } from './components/Setup'
import { ModelSettings } from './components/Settings'
import { useAuthStore } from './stores/authStore'
import { useAiStore } from './stores/aiStore'

type AppState = 'loading' | 'login' | 'setup' | 'ready'

function App() {
  const [activeFolder, setActiveFolder] = useState('inbox')
  const [showCompose, setShowCompose] = useState(false)
  const [showModelSettings, setShowModelSettings] = useState(false)
  const [appState, setAppState] = useState<AppState>('loading')
  const { authenticated, loading: authLoading, checkAuth } = useAuthStore()
  const { modelStatus, checkModelStatus } = useAiStore()

  useEffect(() => {
    checkAuth()
  }, [checkAuth])

  // Check model status after authentication
  useEffect(() => {
    if (authenticated) {
      checkModelStatus()
    }
  }, [authenticated, checkModelStatus])

  // Determine app state based on auth and model status
  useEffect(() => {
    if (authLoading) {
      setAppState('loading')
    } else if (!authenticated) {
      setAppState('login')
    } else if (modelStatus.status === 'not_downloaded') {
      setAppState('setup')
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
        />

        <div className="flex-1 flex">
          {/* Email List */}
          <div className="w-[32rem] border-r-[2px] border-foreground flex flex-col">
            <div className="px-6 py-5 border-b-[2px] border-foreground">
              <h2 className="font-display text-2xl tracking-tight capitalize">
                {activeFolder}
              </h2>
            </div>
            <EmailList />
          </div>

          {/* Email Viewer */}
          <EmailViewer />
        </div>
      </div>

      <ComposeModal isOpen={showCompose} onClose={() => setShowCompose(false)} />

      {/* Model Settings */}
      {showModelSettings && (
        <ModelSettings onClose={() => setShowModelSettings(false)} />
      )}
    </>
  )
}

export default App
