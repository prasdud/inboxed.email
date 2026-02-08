import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
// Account store available for multi-account "From" dropdown
// import { useAccountStore } from '../../stores/accountStore'

interface ComposeModalProps {
  isOpen: boolean
  onClose: () => void
  replyTo?: {
    to: string
    subject: string
    messageId: string
  }
}

export default function ComposeModal({
  isOpen,
  onClose,
  replyTo,
}: ComposeModalProps) {
  const [to, setTo] = useState(replyTo?.to || '')
  const [cc, setCc] = useState('')
  const [bcc, setBcc] = useState('')
  const [subject, setSubject] = useState(
    replyTo?.subject.startsWith('Re:')
      ? replyTo.subject
      : replyTo
      ? `Re: ${replyTo.subject}`
      : ''
  )
  const [body, setBody] = useState('')
  const [sending, setSending] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showCc, setShowCc] = useState(false)

  if (!isOpen) return null

  const handleSend = async () => {
    if (!to || !subject) {
      setError('Please fill in recipient and subject')
      return
    }

    setSending(true)
    setError(null)

    try {
      const toEmails = to.split(',').map((e) => e.trim())
      const ccEmails = cc ? cc.split(',').map((e) => e.trim()) : undefined
      const bccEmails = bcc ? bcc.split(',').map((e) => e.trim()) : undefined

      await invoke('send_email', {
        to: toEmails,
        subject,
        body: body.replace(/\n/g, '<br>'),
        cc: ccEmails,
        bcc: bccEmails,
      })

      onClose()
    } catch (err) {
      setError((err as Error).toString())
    } finally {
      setSending(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-foreground/80">
      <div className="w-full max-w-4xl h-[90vh] bg-background border-[4px] border-foreground flex flex-col">
        {/* Header */}
        <div className="px-8 py-6 border-b-[2px] border-foreground flex items-center justify-between">
          <h2 className="font-display text-3xl tracking-tight">
            {replyTo ? 'Reply' : 'New Message'}
          </h2>
          <button
            onClick={onClose}
            className="w-10 h-10 border-[2px] border-foreground hover:bg-foreground hover:text-background transition-all duration-100 flex items-center justify-center font-mono text-xl focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
          >
            ×
          </button>
        </div>

        {/* Form */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="px-8 py-6 space-y-4 border-b-[2px] border-foreground">
            {/* To */}
            <div className="flex items-center gap-4">
              <label className="font-mono text-xs uppercase tracking-widest w-16">
                To
              </label>
              <input
                type="text"
                value={to}
                onChange={(e) => setTo(e.target.value)}
                placeholder="recipient@example.com"
                className="flex-1 bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
              />
              {!showCc && (
                <button
                  onClick={() => setShowCc(true)}
                  className="font-mono text-xs uppercase tracking-widest text-mutedForeground hover:text-foreground"
                >
                  Cc/Bcc
                </button>
              )}
            </div>

            {/* Cc/Bcc */}
            {showCc && (
              <>
                <div className="flex items-center gap-4">
                  <label className="font-mono text-xs uppercase tracking-widest w-16">
                    Cc
                  </label>
                  <input
                    type="text"
                    value={cc}
                    onChange={(e) => setCc(e.target.value)}
                    placeholder="cc@example.com"
                    className="flex-1 bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
                  />
                </div>
                <div className="flex items-center gap-4">
                  <label className="font-mono text-xs uppercase tracking-widest w-16">
                    Bcc
                  </label>
                  <input
                    type="text"
                    value={bcc}
                    onChange={(e) => setBcc(e.target.value)}
                    placeholder="bcc@example.com"
                    className="flex-1 bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
                  />
                </div>
              </>
            )}

            {/* Subject */}
            <div className="flex items-center gap-4">
              <label className="font-mono text-xs uppercase tracking-widest w-16">
                Subject
              </label>
              <input
                type="text"
                value={subject}
                onChange={(e) => setSubject(e.target.value)}
                placeholder="Email subject"
                className="flex-1 bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-display text-xl outline-none transition-all"
              />
            </div>
          </div>

          {/* Body */}
          <div className="flex-1 px-8 py-6 overflow-y-auto">
            <textarea
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="Write your message..."
              className="w-full h-full bg-transparent font-serif text-lg leading-relaxed resize-none outline-none"
            />
          </div>
        </div>

        {/* Footer */}
        <div className="px-8 py-6 border-t-[2px] border-foreground flex items-center justify-between">
          {error && (
            <p className="text-sm text-mutedForeground">{error}</p>
          )}
          <div className="flex-1" />
          <div className="flex gap-4">
            <button
              onClick={onClose}
              disabled={sending}
              className="px-8 py-3 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-muted transition-all duration-100 disabled:opacity-50 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
            >
              Cancel
            </button>
            <button
              onClick={handleSend}
              disabled={sending}
              className="px-8 py-3 bg-foreground text-background font-mono text-xs uppercase tracking-widest hover:bg-background hover:text-foreground border-[2px] border-transparent hover:border-foreground transition-all duration-100 disabled:opacity-50 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
            >
              {sending ? 'Sending...' : 'Send'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
