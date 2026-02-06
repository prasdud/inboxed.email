import { useEffect } from 'react'
import { useEmailStore } from '../../stores/emailStore'

export default function EmailList() {
  const { emails, selectedEmail, loading, error, fetchEmails, selectEmail } =
    useEmailStore()

  useEffect(() => {
    fetchEmails(50)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr)
      const now = new Date()
      const diff = now.getTime() - date.getTime()
      const days = Math.floor(diff / (1000 * 60 * 60 * 24))

      if (days === 0) {
        return date.toLocaleTimeString('en-US', {
          hour: 'numeric',
          minute: '2-digit',
        })
      } else if (days < 7) {
        return date.toLocaleDateString('en-US', { weekday: 'short' })
      } else {
        return date.toLocaleDateString('en-US', {
          month: 'short',
          day: 'numeric',
        })
      }
    } catch {
      return dateStr
    }
  }

  if (loading && emails.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center">
          <div className="w-8 h-8 border-[2px] border-foreground border-t-transparent animate-spin mx-auto mb-4" />
          <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground">
            Loading
          </p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex-1 flex items-center justify-center p-8">
        <div className="text-center border-[2px] border-foreground p-8">
          <p className="font-mono text-sm uppercase tracking-wider mb-4">
            Error
          </p>
          <p className="text-mutedForeground mb-6">{error}</p>
          <button
            onClick={() => fetchEmails(50)}
            className="px-8 py-3 bg-foreground text-background font-mono text-xs uppercase tracking-widest hover:bg-background hover:text-foreground border-[2px] border-transparent hover:border-foreground transition-all duration-100"
          >
            Retry
          </button>
        </div>
      </div>
    )
  }

  if (emails.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center p-8">
        <div className="text-center">
          <p className="font-display text-4xl mb-4">—</p>
          <p className="font-serif text-mutedForeground">No emails</p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-y-auto">
      {emails.map((email) => (
        <button
          key={email.id}
          onClick={() => selectEmail(email.id)}
          className={`group w-full text-left px-6 py-6 border-b border-borderLight transition-all duration-100 hover:bg-foreground hover:text-background focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-[-3px] ${selectedEmail?.id === email.id
              ? 'bg-foreground text-background'
              : ''
            }`}
        >
          <div className="flex items-start gap-4">
            {/* Decorative element for unread */}
            <div className="flex-shrink-0 w-2">
              {!email.is_read && (
                <div className="w-2 h-2 bg-current" />
              )}
            </div>

            {/* Email content */}
            <div className="flex-1 min-w-0">
              {/* Header: From + Date */}
              <div className="flex items-baseline justify-between mb-2 gap-4">
                <span className={`font-serif text-base ${!email.is_read ? 'font-semibold' : ''}`}>
                  {email.from.split('<')[0].trim()}
                </span>
                <span className="font-mono text-xs tracking-wider opacity-60 flex-shrink-0">
                  {formatDate(email.date)}
                </span>
              </div>

              {/* Subject */}
              <div className={`font-display text-lg leading-tight mb-2 truncate ${!email.is_read ? 'font-semibold' : ''}`}>
                {email.subject}
              </div>

              {/* Snippet */}
              <div className="font-serif text-sm opacity-60 truncate">
                {email.snippet}
              </div>

              {/* Meta */}
              {(email.is_starred || email.has_attachments) && (
                <div className="flex items-center gap-3 mt-3">
                  {email.is_starred && (
                    <span className="font-mono text-xs uppercase tracking-widest opacity-60">
                      Starred
                    </span>
                  )}
                  {email.has_attachments && (
                    <span className="font-mono text-xs uppercase tracking-widest opacity-60">
                      Attachment
                    </span>
                  )}
                </div>
              )}
            </div>
          </div>
        </button>
      ))}
    </div>
  )
}
