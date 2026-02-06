import { useEffect, useState } from 'react'
import { useSmartInboxStore } from '../../stores/smartInboxStore'
import { useEmailStore } from '../../stores/emailStore'
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
    startIndexing,
    initDatabase,
    setupIndexingListeners,
  } = useSmartInboxStore()

  const { selectEmail } = useEmailStore()
  const [showChat, setShowChat] = useState(false)
  const [hasIndexed, setHasIndexed] = useState(false)

  useEffect(() => {
    // Initialize database and fetch status
    const init = async () => {
      await initDatabase()
      await getIndexingStatus()

      // Check if we have any indexed emails
      if (emails.length === 0) {
        await fetchSmartInbox()
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
    try {
      await startIndexing(100)
    } catch (error) {
      console.error('Failed to start indexing:', error)
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
          <button
            onClick={handleStartIndexing}
            className="px-6 py-3 bg-foreground text-background hover:bg-gray-700 transition-colors font-mono text-sm uppercase tracking-wider"
          >
            Start Indexing
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 flex flex-col bg-background h-full overflow-hidden">
      {/* Action Bar */}
      <div className="px-6 py-3 border-b-[2px] border-foreground flex items-center justify-between flex-shrink-0">
        <p className="text-sm text-mutedForeground">
          {indexingStatus?.is_indexing
            ? `Indexing emails... ${indexingProgress}%`
            : `${emails.length} emails sorted by importance`}
        </p>
        <div className="flex items-center gap-2">
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
          {!indexingStatus?.is_indexing && (
            <button
              onClick={handleStartIndexing}
              className="px-4 py-2 border-[2px] border-foreground hover:bg-foreground hover:text-background text-sm font-mono uppercase tracking-wider transition-colors"
            >
              Re-index
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

      <div className="flex-1 flex overflow-hidden">
        {/* Email List */}
        <div className="flex-1 overflow-y-auto">
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
          <div className="w-[30%] min-w-[280px] max-w-[24rem] flex-shrink-0 border-l-[2px] border-foreground">
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
