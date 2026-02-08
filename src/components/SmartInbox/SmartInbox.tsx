import { useEffect, useState } from 'react'
import { useSmartInboxStore } from '../../stores/smartInboxStore'
import { useEmailStore } from '../../stores/emailStore'
import { useRagStore } from '../../stores/ragStore'
import { ChatPanel } from './ChatPanel'

const PRIORITY_LABELS = {
  HIGH: '🔴',
  MEDIUM: '🟡',
  LOW: '⚪',
}

interface SmartInboxProps {
  onCompose?: () => void
}

export function SmartInbox({ onCompose }: SmartInboxProps) {
  const {
    emails,
    loading,
    error,
    indexingStatus,
    indexingProgress,
    fetchSmartInbox,
    getIndexingStatus,
    resetIndexingStatus,
    startIndexing,
    initDatabase,
    setupIndexingListeners,
  } = useSmartInboxStore()

  const {
    isInitialized: ragReady,
    isModelDownloaded,
    isEmbedding,
    embeddingProgress,
    embeddingStatus,
    allEmailsEmbedded,
    error: ragError,
    embedAllEmails,
    downloadAndInitRag,
    getEmbeddingStatus,
  } = useRagStore()

  const { selectEmail } = useEmailStore()
  const [showChat, setShowChat] = useState(false)
  const [hasIndexed, setHasIndexed] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const [isReindexing, setIsReindexing] = useState(false)
  const [isBuildingIndex, setIsBuildingIndex] = useState(false)

  useEffect(() => {
    // Initialize database and fetch status
    const init = async () => {
      await initDatabase()
      await getIndexingStatus()

      // Reset stale indexing state from a previous interrupted run
      const currentStatus = useSmartInboxStore.getState().indexingStatus
      if (currentStatus?.is_indexing) {
        await resetIndexingStatus()
      }

      // Check if we have any indexed emails
      if (emails.length === 0) {
        await fetchSmartInbox()
      }

      // Auto-init RAG if embedding model is already downloaded
      const ragStore = useRagStore.getState()
      const downloaded = await ragStore.checkModelDownloaded()
      if (downloaded && !ragStore.isInitialized) {
        try {
          await ragStore.initRag()
        } catch (e) {
          console.error('[SmartInbox] Auto-init RAG failed:', e)
        }
      }

      // Fetch embedding status to know if all emails are indexed
      if (ragStore.isInitialized || downloaded) {
        await ragStore.getEmbeddingStatus()
      }
    }
    init()

    // Setup indexing listeners
    const cleanup = setupIndexingListeners()
    return () => {
      cleanup.then((fn) => fn())
    }
  }, [])

  useEffect(() => {
    if (indexingStatus && indexingStatus.last_indexed_at) {
      setHasIndexed(true)
    }
  }, [indexingStatus])

  const handleStartIndexing = async () => {
    setActionError(null)
    setIsReindexing(true)
    try {
      await startIndexing(100)
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error)
      setActionError(`Indexing failed: ${msg}`)
      console.error('Failed to start indexing:', error)
      await resetIndexingStatus().catch(() => {})
    } finally {
      setIsReindexing(false)
    }
  }

  const handleBuildIndex = async () => {
    setActionError(null)
    setIsBuildingIndex(true)
    try {
      if (!ragReady) {
        const initialized = await downloadAndInitRag()
        if (!initialized) return
      }
      await embedAllEmails()
      await getEmbeddingStatus()
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error)
      setActionError(`Build index failed: ${msg}`)
      console.error('Failed to build index:', error)
    } finally {
      setIsBuildingIndex(false)
    }
  }

  const handleEmailClick = (emailId: string) => {
    selectEmail(emailId)
  }

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000)
    const now = new Date()
    const diffHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60)

    if (diffHours < 24) {
      return date.toLocaleTimeString('en-US', {
        hour: 'numeric',
        minute: '2-digit',
      })
    } else if (diffHours < 24 * 7) {
      return date.toLocaleDateString('en-US', { weekday: 'short' })
    } else {
      return date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
      })
    }
  }

  if (!hasIndexed && !indexingStatus?.is_indexing) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center bg-background p-8">
        <div className="max-w-md text-center">
          <h1 className="font-display text-3xl mb-4">Welcome to Smart Inbox</h1>
          <p className="text-mutedForeground mb-8">
            Let's index your emails to unlock AI-powered sorting, categorization, and smart search.
          </p>
          {actionError && (
            <div className="mb-4 p-3 bg-red-50 border border-red-300 text-red-600 text-sm font-mono text-left">
              {actionError}
            </div>
          )}
          <button
            onClick={handleStartIndexing}
            disabled={isReindexing}
            className="px-6 py-3 bg-foreground text-background hover:bg-gray-700 transition-colors font-mono text-sm uppercase tracking-wider disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isReindexing ? 'Starting...' : 'Start Indexing'}
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 flex flex-col bg-background h-full overflow-hidden">
      {/* Action Bar */}
      <div className="px-6 py-3 border-b-[2px] border-foreground flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-3">
          <p className="text-sm text-mutedForeground">
            {indexingStatus?.is_indexing
              ? `Indexing emails... ${indexingProgress}%`
              : `${emails.length} emails sorted by importance`}
          </p>
          {ragReady && !indexingStatus?.is_indexing && !isEmbedding && embeddingStatus && (embeddingStatus.total_emails > 0 || embeddingStatus.embedded_emails > 0) && (
            <span
              className={`inline-flex items-center gap-1 px-2 py-0.5 text-xs font-mono uppercase tracking-wider ${
                allEmailsEmbedded
                  ? 'bg-green-100 text-green-700'
                  : 'bg-yellow-100 text-yellow-700'
              }`}
            >
              {allEmailsEmbedded
                ? `${embeddingStatus.embedded_emails} indexed`
                : `${embeddingStatus.embedded_emails}/${embeddingStatus.total_emails} indexed`}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          {onCompose && (
            <button
              onClick={onCompose}
              className="px-4 py-2 bg-foreground text-background hover:bg-gray-700 text-sm font-mono uppercase tracking-wider transition-colors"
            >
              Compose
            </button>
          )}
          <button
            onClick={() => setShowChat(!showChat)}
            className={`px-4 py-2 text-sm font-mono uppercase tracking-wider transition-colors ${showChat
                ? 'bg-foreground text-background'
                : 'border-[2px] border-foreground hover:bg-foreground hover:text-background'
              }`}
          >
            {showChat ? 'Hide Chat' : 'Ask AI'}
          </button>
          {!indexingStatus?.is_indexing && !isEmbedding && !allEmailsEmbedded && hasIndexed && (
            <button
              onClick={handleBuildIndex}
              disabled={isBuildingIndex}
              className="px-4 py-2 border-[2px] border-foreground hover:bg-foreground hover:text-background text-sm font-mono uppercase tracking-wider transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isBuildingIndex ? 'Building...' : isModelDownloaded ? 'Build Index' : 'Setup AI Index'}
            </button>
          )}
          {!indexingStatus?.is_indexing && (
            <button
              onClick={handleStartIndexing}
              disabled={isReindexing}
              className="px-4 py-2 border-[2px] border-foreground hover:bg-foreground hover:text-background text-sm font-mono uppercase tracking-wider transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isReindexing ? 'Re-indexing...' : 'Re-index'}
            </button>
          )}
        </div>
      </div>

      {/* Indexing Progress */}
      {indexingStatus?.is_indexing && (
        <div className="px-6 py-3 bg-gray-100 border-b-[2px] border-foreground flex-shrink-0">
          <div className="flex items-center justify-between text-sm mb-2">
            <span className="font-mono">
              Processing: {indexingStatus.processed_emails} / {indexingStatus.total_emails}
            </span>
            <span className="font-mono">{indexingProgress}%</span>
          </div>
          <div className="w-full h-2 bg-gray-300">
            <div
              className="h-full bg-foreground transition-all duration-300"
              style={{ width: `${indexingProgress}%` }}
            />
          </div>
        </div>
      )}

      {/* Action Error Banner */}
      {actionError && (
        <div className="px-6 py-3 bg-red-50 border-b-[2px] border-red-600 flex-shrink-0 flex items-center justify-between">
          <span className="text-red-600 text-sm font-mono">{actionError}</span>
          <button
            onClick={() => setActionError(null)}
            className="text-red-600 hover:text-red-800 font-mono text-xs uppercase tracking-wider ml-4"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* RAG Error Banner */}
      {ragError && !actionError && (
        <div className="px-6 py-3 bg-red-50 border-b-[2px] border-red-600 flex-shrink-0 flex items-center justify-between">
          <span className="text-red-600 text-sm font-mono">RAG: {ragError}</span>
          <button
            onClick={() => useRagStore.setState({ error: null })}
            className="text-red-600 hover:text-red-800 font-mono text-xs uppercase tracking-wider ml-4"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Embedding Progress */}
      {isEmbedding && embeddingProgress && (
        <div className="px-6 py-3 bg-blue-50 border-b-[2px] border-foreground flex-shrink-0">
          <div className="flex items-center justify-between text-sm mb-2">
            <span className="font-mono">
              Building AI index: {embeddingProgress.embedded} / {embeddingProgress.total}
            </span>
            <span className="font-mono">
              {embeddingProgress.total > 0
                ? Math.round((embeddingProgress.embedded / embeddingProgress.total) * 100)
                : 0}%
            </span>
          </div>
          <div className="w-full h-2 bg-blue-200">
            <div
              className="h-full bg-blue-600 transition-all duration-300"
              style={{
                width: `${embeddingProgress.total > 0
                  ? (embeddingProgress.embedded / embeddingProgress.total) * 100
                  : 0}%`,
              }}
            />
          </div>
        </div>
      )}

      <div className="flex-1 flex overflow-hidden">
        {/* Email List */}
        <div className="flex-1 overflow-y-auto min-w-0">
          {loading && emails.length === 0 ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center">
                <div className="w-12 h-12 border-[2px] border-foreground border-t-transparent animate-spin mx-auto mb-4" />
                <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground">
                  Loading
                </p>
              </div>
            </div>
          ) : emails.length === 0 ? (
            <div className="flex items-center justify-center h-full">
              <p className="text-mutedForeground font-mono text-sm">No emails found</p>
            </div>
          ) : (
            <div className="divide-y-[2px] divide-foreground">
              {emails.map((email) => (
                <div
                  key={email.id}
                  onClick={() => handleEmailClick(email.id)}
                  className={`px-6 py-4 hover:bg-gray-50 cursor-pointer transition-colors ${!email.is_read ? 'bg-blue-50' : ''
                    }`}
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-lg" title={email.priority}>
                          {PRIORITY_LABELS[email.priority as keyof typeof PRIORITY_LABELS]}
                        </span>
                        <span className="font-semibold truncate">{email.from_name}</span>
                        {email.category && (
                          <span className="px-2 py-0.5 text-xs bg-gray-200 font-mono uppercase">
                            {email.category}
                          </span>
                        )}
                      </div>
                      <h3
                        className={`font-medium mb-1 truncate ${!email.is_read ? 'font-bold' : ''
                          }`}
                      >
                        {email.subject}
                      </h3>
                      <p className="text-sm text-mutedForeground line-clamp-2">
                        {email.summary || email.snippet}
                      </p>
                    </div>
                    <div className="text-right flex-shrink-0">
                      <div className="text-xs text-mutedForeground mb-1">
                        {formatDate(email.date)}
                      </div>
                      {email.is_starred && <span className="text-yellow-500">★</span>}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Chat Panel */}
        {showChat && (
          <div className="w-[30%] min-w-[240px] max-w-[24rem] border-l-[2px] border-foreground">
            <ChatPanel onClose={() => setShowChat(false)} />
          </div>
        )}
      </div>

      {error && (
        <div className="px-6 py-3 bg-red-100 border-t-[2px] border-red-600 text-red-600 text-sm font-mono">
          {error}
        </div>
      )}
    </div>
  )
}
