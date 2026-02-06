import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useEmailStore } from '../../stores/emailStore'
import { useAiStore } from '../../stores/aiStore'
import { ComposeModal } from '../Compose'

interface EmailSummary {
  summary: string
  insights: string[]
  priority: string
}

export default function EmailViewer() {
  const { selectedEmail, fetchEmails } = useEmailStore()
  const { isModelLoaded, isAiReady, modelStatus, downloadProgress } = useAiStore()
  const [showCompose, setShowCompose] = useState(false)
  const [actionLoading, setActionLoading] = useState<string | null>(null)
  const [summary, setSummary] = useState<EmailSummary | null>(null)
  const [streamingSummary, setStreamingSummary] = useState<string>('')
  const [loadingSummary, setLoadingSummary] = useState(false)
  const [showSummary, setShowSummary] = useState(false)
  const [isStreaming, setIsStreaming] = useState(false)
  const unlistenRef = useRef<UnlistenFn | null>(null)


  // Set up streaming listener
  useEffect(() => {
    let mounted = true

    const setupListener = async () => {
      // Listen for streaming tokens
      unlistenRef.current = await listen<string>('ai:token', (event) => {
        if (mounted) {
          setStreamingSummary(prev => prev + event.payload)
        }
      })
    }

    setupListener()

    return () => {
      mounted = false
      if (unlistenRef.current) {
        unlistenRef.current()
      }
    }
  }, [])

  // Reset summary when email changes
  useEffect(() => {
    setSummary(null)
    setStreamingSummary('')
    setIsStreaming(false)
  }, [selectedEmail?.id])

  // Load summary when showSummary is toggled on
  useEffect(() => {
    if (selectedEmail && showSummary && !summary) {
      loadSummary()
    }
  }, [selectedEmail, showSummary])

  const loadSummary = async () => {
    if (!selectedEmail) return

    setLoadingSummary(true)
    setStreamingSummary('')
    setIsStreaming(true)

    try {
      // Use streaming version if model is loaded, otherwise regular
      const command = isModelLoaded ? 'summarize_email_stream' : 'summarize_email'

      const result = await invoke<EmailSummary>(command, {
        subject: selectedEmail.subject,
        from: selectedEmail.from,
        body: selectedEmail.body_html || selectedEmail.body_plain || selectedEmail.snippet,
      })

      setSummary(result)
      setStreamingSummary('')
    } catch (error) {
      console.error('Failed to load summary:', error)
      // If streaming failed, try non-streaming version
      if (isModelLoaded) {
        try {
          const result = await invoke<EmailSummary>('summarize_email', {
            subject: selectedEmail.subject,
            from: selectedEmail.from,
            body: selectedEmail.body_html || selectedEmail.body_plain || selectedEmail.snippet,
          })
          setSummary(result)
        } catch (fallbackError) {
          console.error('Fallback summary also failed:', fallbackError)
        }
      }
    } finally {
      setLoadingSummary(false)
      setIsStreaming(false)
    }
  }

  const handleAction = async (action: string, actionFn: () => Promise<void>) => {
    setActionLoading(action)
    try {
      await actionFn()
      await fetchEmails(50) // Refresh email list
    } catch (error) {
      console.error(`${action} failed:`, error)
    } finally {
      setActionLoading(null)
    }
  }

  const handleReply = () => {
    setShowCompose(true)
  }

  const handleTrash = async () => {
    if (!selectedEmail) return
    await handleAction('trash', () =>
      invoke('trash_email', { emailId: selectedEmail.id })
    )
  }

  const handleArchive = async () => {
    if (!selectedEmail) return
    await handleAction('archive', () =>
      invoke('archive_email', { emailId: selectedEmail.id })
    )
  }

  const handleStar = async () => {
    if (!selectedEmail) return
    await handleAction('star', () =>
      invoke('star_email', {
        emailId: selectedEmail.id,
        starred: !selectedEmail.is_starred,
      })
    )
  }

  const handleMarkRead = async () => {
    if (!selectedEmail) return
    await handleAction('read', () =>
      invoke('mark_email_read', {
        emailId: selectedEmail.id,
        read: !selectedEmail.is_read,
      })
    )
  }

  if (!selectedEmail) {
    return (
      <div className="flex-1 flex items-center justify-center p-12">
        <div className="text-center max-w-md">
          <div className="mb-8">
            <div className="inline-block w-16 h-[2px] bg-foreground mb-4" />
          </div>
          <p className="font-display text-3xl mb-4 tracking-tight italic">
            Select an email
          </p>
          <p className="font-serif text-mutedForeground">
            Choose a message from the list to read
          </p>
        </div>
      </div>
    )
  }

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr)
      return date.toLocaleDateString('en-US', {
        weekday: 'long',
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      }) + ' · ' + date.toLocaleTimeString('en-US', {
        hour: 'numeric',
        minute: '2-digit',
      })
    } catch {
      return dateStr
    }
  }

  // Display either streaming content or final summary
  const displaySummary = isStreaming && streamingSummary ? streamingSummary : summary?.summary

  return (
    <>
      <div className="flex-1 flex flex-col overflow-hidden bg-background">
        {/* Actions Bar */}
        <div className="px-6 lg:px-12 py-4 border-b border-borderLight flex items-center gap-3 flex-wrap">
          <button
            onClick={handleReply}
            disabled={!!actionLoading}
            className="px-6 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all duration-100 disabled:opacity-50 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
          >
            Reply
          </button>
          <button
            onClick={handleArchive}
            disabled={!!actionLoading}
            className="px-6 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all duration-100 disabled:opacity-50 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
          >
            {actionLoading === 'archive' ? 'Archiving...' : 'Archive'}
          </button>
          <button
            onClick={handleTrash}
            disabled={!!actionLoading}
            className="px-6 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all duration-100 disabled:opacity-50 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
          >
            {actionLoading === 'trash' ? 'Deleting...' : 'Delete'}
          </button>
          <div className="flex-1" />
          <button
            onClick={handleStar}
            disabled={!!actionLoading}
            className="px-6 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all duration-100 disabled:opacity-50 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
          >
            {selectedEmail.is_starred ? '★' : '☆'}
          </button>
          <button
            onClick={handleMarkRead}
            disabled={!!actionLoading}
            className="px-6 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all duration-100 disabled:opacity-50 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
          >
            {selectedEmail.is_read ? 'Unread' : 'Read'}
          </button>
          <div className="w-px h-6 bg-borderLight" />
          {/* AI Summary Button with Status */}
          <div className="relative group">
            <button
              onClick={() => setShowSummary(!showSummary)}
              disabled={loadingSummary || !isAiReady}
              className={`px-6 py-2 font-mono text-xs uppercase tracking-widest transition-all duration-100 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3 ${
                showSummary
                  ? 'bg-foreground text-background border-[2px] border-foreground'
                  : 'border-[2px] border-foreground hover:bg-foreground hover:text-background'
              }`}
            >
              {loadingSummary
                ? 'Analyzing...'
                : modelStatus.status === 'downloading'
                ? `Downloading ${Math.round(downloadProgress)}%`
                : modelStatus.status === 'loading'
                ? 'Loading AI...'
                : !isAiReady
                ? 'AI Not Ready'
                : showSummary
                ? 'Hide AI'
                : 'AI Summary'}
            </button>
            {/* Download Progress Bar */}
            {modelStatus.status === 'downloading' && (
              <div className="absolute bottom-0 left-0 right-0 h-[2px] bg-borderLight">
                <div
                  className="h-full bg-foreground transition-all duration-300"
                  style={{ width: `${downloadProgress}%` }}
                />
              </div>
            )}
            {/* Tooltip when disabled */}
            {!isAiReady && modelStatus.status !== 'downloading' && modelStatus.status !== 'loading' && (
              <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-2 bg-foreground text-background text-xs whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                Download AI model first
                <div className="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-foreground" />
              </div>
            )}
          </div>
        </div>

        {/* Header */}
        <div className="border-b-[2px] border-foreground px-6 lg:px-12 py-8">
        {/* Subject */}
        <h1 className="font-display text-3xl lg:text-5xl leading-tight tracking-tight mb-8">
          {selectedEmail.subject}
        </h1>

        {/* Meta */}
        <div className="flex items-start justify-between gap-8">
          <div>
            <p className="font-serif text-lg mb-1">
              {selectedEmail.from.split('<')[0].trim()}
            </p>
            <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground mb-2">
              {selectedEmail.from_email}
            </p>
            <p className="font-mono text-xs text-mutedForeground">
              to: {selectedEmail.to.join(', ')}
            </p>
          </div>

          <div className="text-right">
            <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground">
              {formatDate(selectedEmail.date)}
            </p>
          </div>
        </div>
      </div>

      {/* AI Summary Panel */}
      {showSummary && (displaySummary || loadingSummary) && (
        <div className="border-b-[2px] border-foreground bg-muted">
          <div className="px-6 lg:px-12 py-8">
            {/* Priority Badge */}
            {summary?.priority && (
              <div className="mb-6">
                <span
                  className={`inline-block px-4 py-2 font-mono text-xs uppercase tracking-widest border-[2px] ${
                    summary.priority === 'HIGH'
                      ? 'bg-foreground text-background border-foreground'
                      : 'border-foreground'
                  }`}
                >
                  Priority: {summary.priority}
                </span>
                {!isModelLoaded && (
                  <span className="ml-3 font-mono text-xs text-mutedForeground">
                    (keyword-based)
                  </span>
                )}
              </div>
            )}

            {/* Summary */}
            <div className="mb-6">
              <h3 className="font-mono text-xs uppercase tracking-widest mb-3">
                AI Summary
                {isStreaming && (
                  <span className="ml-2 inline-block w-2 h-2 bg-foreground animate-pulse" />
                )}
              </h3>
              {loadingSummary && !displaySummary ? (
                <div className="flex items-center gap-3">
                  <div className="w-4 h-4 border-[2px] border-foreground border-t-transparent animate-spin" />
                  <span className="font-serif text-mutedForeground">
                    Analyzing email...
                  </span>
                </div>
              ) : (
                <p className="font-serif text-lg leading-relaxed whitespace-pre-line">
                  {displaySummary}
                  {isStreaming && <span className="animate-pulse">|</span>}
                </p>
              )}
            </div>

            {/* Insights */}
            {summary?.insights && summary.insights.length > 0 && (
              <div>
                <h3 className="font-mono text-xs uppercase tracking-widest mb-3">
                  Insights
                </h3>
                <div className="space-y-2">
                  {summary.insights.map((insight, index) => (
                    <div
                      key={index}
                      className="flex items-start gap-3 font-serif"
                    >
                      <span className="text-lg">—</span>
                      <span>{insight}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Body */}
      <div className="flex-1 overflow-y-auto">
        <article className="max-w-3xl mx-auto px-6 lg:px-12 py-12">
          {selectedEmail.body_html ? (
            <div
              className="font-serif text-lg leading-relaxed email-content"
              dangerouslySetInnerHTML={{ __html: selectedEmail.body_html }}
              style={{
                color: 'var(--foreground)',
              }}
            />
          ) : selectedEmail.body_plain ? (
            <pre className="whitespace-pre-wrap font-serif text-lg leading-relaxed">
              {selectedEmail.body_plain}
            </pre>
          ) : (
            <div className="text-center py-12">
              <div className="border-[2px] border-borderLight p-8">
                <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground mb-4">
                  No Content
                </p>
                <p className="font-serif text-mutedForeground">
                  {selectedEmail.snippet}
                </p>
              </div>
            </div>
          )}
        </article>
      </div>
    </div>

    <ComposeModal
      isOpen={showCompose}
      onClose={() => setShowCompose(false)}
      replyTo={{
        to: selectedEmail.from_email,
        subject: selectedEmail.subject,
        messageId: selectedEmail.id,
      }}
    />
  </>
  )
}
