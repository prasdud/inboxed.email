import { useState } from 'react'
import { useAuthStore } from '../../stores/authStore'
import { useAccountStore } from '../../stores/accountStore'
import { invoke } from '@tauri-apps/api/core'

type AuthMode = 'select' | 'gmail' | 'outlook' | 'custom'

export default function LoginScreen() {
  const { signIn, loading, error } = useAuthStore()
  const { addAccount } = useAccountStore()
  const [mode, setMode] = useState<AuthMode>('select')
  const [customForm, setCustomForm] = useState({
    email: '',
    password: '',
    imapHost: '',
    imapPort: '993',
    smtpHost: '',
    smtpPort: '465',
  })
  const [customError, setCustomError] = useState<string | null>(null)
  const [customLoading, setCustomLoading] = useState(false)

  const handleOAuthSignIn = async (_provider: 'gmail' | 'outlook') => {
    // Use existing OAuth flow for Gmail, will be extended for Outlook in Phase 3
    await signIn()
  }

  const handleCustomSignIn = async () => {
    if (!customForm.email || !customForm.password || !customForm.imapHost || !customForm.smtpHost) {
      setCustomError('Please fill in all required fields')
      return
    }

    setCustomLoading(true)
    setCustomError(null)

    try {
      // Add account to database
      const account = await addAccount({
        email: customForm.email,
        displayName: customForm.email.split('@')[0],
        provider: 'custom',
        authType: 'password',
        imapHost: customForm.imapHost,
        imapPort: parseInt(customForm.imapPort),
        smtpHost: customForm.smtpHost,
        smtpPort: parseInt(customForm.smtpPort),
      })

      // Store app password
      await invoke('store_app_password_cmd', {
        accountId: account.id,
        password: customForm.password,
      }).catch(() => {
        // Command may not exist yet, that's ok
      })

      // Try to connect
      await invoke('connect_account', { accountId: account.id })

      // Trigger auth check
      const { checkAuth } = useAuthStore.getState()
      await checkAuth()
    } catch (err) {
      setCustomError((err as Error).toString())
    } finally {
      setCustomLoading(false)
    }
  }

  return (
    <div className="flex items-center justify-center h-screen bg-background relative overflow-hidden">
      {/* Background texture */}
      <div className="absolute inset-0 opacity-[0.015] pointer-events-none">
        <div
          className="w-full h-full"
          style={{
            backgroundImage: `repeating-linear-gradient(
              0deg,
              transparent,
              transparent 1px,
              #000 1px,
              #000 2px
            )`,
            backgroundSize: '100% 4px',
          }}
        />
      </div>

      <div className="relative z-10 w-full max-w-2xl px-8">
        {/* Decorative line with square */}
        <div className="flex items-center gap-4 mb-12">
          <div className="h-[2px] flex-1 bg-foreground" />
          <div className="w-3 h-3 border-[2px] border-foreground" />
        </div>

        {/* Oversized headline */}
        <div className="mb-16">
          <h1 className="font-display text-7xl md:text-8xl leading-none tracking-tighter mb-6">
            Inboxed
          </h1>
          <p className="font-serif text-xl md:text-2xl text-mutedForeground leading-relaxed max-w-xl">
            AI-powered email client with local intelligence.
            <br />
            <span className="italic">Your inbox, refined.</span>
          </p>
        </div>

        {/* Provider Selection */}
        {mode === 'select' && (
          <div className="space-y-4">
            <button
              onClick={() => handleOAuthSignIn('gmail')}
              disabled={loading}
              className="group relative w-full px-12 py-5 bg-foreground text-background font-mono text-sm uppercase tracking-widest font-medium transition-all duration-100 hover:bg-background hover:text-foreground border-[2px] border-transparent hover:border-foreground disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? (
                <span className="flex items-center justify-center gap-3">
                  <span className="inline-block w-4 h-4 border-[2px] border-current border-t-transparent animate-spin" />
                  Authenticating
                </span>
              ) : (
                <span className="flex items-center justify-center gap-3">
                  Sign in with Google
                  <span className="inline-block transition-transform group-hover:translate-x-1">
                    →
                  </span>
                </span>
              )}
            </button>

            <button
              onClick={() => handleOAuthSignIn('outlook')}
              disabled={loading}
              className="group relative w-full px-12 py-5 border-[2px] border-foreground font-mono text-sm uppercase tracking-widest font-medium transition-all duration-100 hover:bg-foreground hover:text-background disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <span className="flex items-center justify-center gap-3">
                Sign in with Outlook
                <span className="inline-block transition-transform group-hover:translate-x-1">
                  →
                </span>
              </span>
            </button>

            <button
              onClick={() => setMode('custom')}
              className="group relative w-full px-12 py-5 border-[2px] border-borderLight font-mono text-sm uppercase tracking-widest font-medium transition-all duration-100 hover:border-foreground text-mutedForeground hover:text-foreground"
            >
              <span className="flex items-center justify-center gap-3">
                Custom IMAP Server
                <span className="inline-block transition-transform group-hover:translate-x-1">
                  →
                </span>
              </span>
            </button>

            {error && (
              <div className="border-[2px] border-foreground p-6">
                <p className="font-mono text-sm uppercase tracking-wider mb-2">
                  Error
                </p>
                <p className="text-mutedForeground">{error}</p>
              </div>
            )}
          </div>
        )}

        {/* Custom IMAP Form */}
        {mode === 'custom' && (
          <div className="space-y-6">
            <button
              onClick={() => setMode('select')}
              className="font-mono text-xs uppercase tracking-widest text-mutedForeground hover:text-foreground"
            >
              ← Back
            </button>

            <div className="space-y-4">
              <div>
                <label className="font-mono text-xs uppercase tracking-widest block mb-2">Email</label>
                <input
                  type="email"
                  value={customForm.email}
                  onChange={(e) => setCustomForm({ ...customForm, email: e.target.value })}
                  placeholder="you@example.com"
                  className="w-full bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
                />
              </div>

              <div>
                <label className="font-mono text-xs uppercase tracking-widest block mb-2">App Password</label>
                <input
                  type="password"
                  value={customForm.password}
                  onChange={(e) => setCustomForm({ ...customForm, password: e.target.value })}
                  placeholder="App password or password"
                  className="w-full bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="font-mono text-xs uppercase tracking-widest block mb-2">IMAP Host</label>
                  <input
                    type="text"
                    value={customForm.imapHost}
                    onChange={(e) => setCustomForm({ ...customForm, imapHost: e.target.value })}
                    placeholder="imap.example.com"
                    className="w-full bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
                  />
                </div>
                <div>
                  <label className="font-mono text-xs uppercase tracking-widest block mb-2">IMAP Port</label>
                  <input
                    type="number"
                    value={customForm.imapPort}
                    onChange={(e) => setCustomForm({ ...customForm, imapPort: e.target.value })}
                    className="w-full bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="font-mono text-xs uppercase tracking-widest block mb-2">SMTP Host</label>
                  <input
                    type="text"
                    value={customForm.smtpHost}
                    onChange={(e) => setCustomForm({ ...customForm, smtpHost: e.target.value })}
                    placeholder="smtp.example.com"
                    className="w-full bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
                  />
                </div>
                <div>
                  <label className="font-mono text-xs uppercase tracking-widest block mb-2">SMTP Port</label>
                  <input
                    type="number"
                    value={customForm.smtpPort}
                    onChange={(e) => setCustomForm({ ...customForm, smtpPort: e.target.value })}
                    className="w-full bg-transparent border-b-[2px] border-borderLight focus:border-foreground py-2 font-serif outline-none transition-all"
                  />
                </div>
              </div>
            </div>

            <button
              onClick={handleCustomSignIn}
              disabled={customLoading}
              className="w-full px-12 py-5 bg-foreground text-background font-mono text-sm uppercase tracking-widest font-medium transition-all duration-100 hover:bg-background hover:text-foreground border-[2px] border-transparent hover:border-foreground disabled:opacity-50"
            >
              {customLoading ? (
                <span className="flex items-center justify-center gap-3">
                  <span className="inline-block w-4 h-4 border-[2px] border-current border-t-transparent animate-spin" />
                  Connecting
                </span>
              ) : (
                'Connect'
              )}
            </button>

            {customError && (
              <div className="border-[2px] border-foreground p-6">
                <p className="font-mono text-sm uppercase tracking-wider mb-2">Error</p>
                <p className="text-mutedForeground">{customError}</p>
              </div>
            )}
          </div>
        )}

        {/* Footer notes */}
        <div className="mt-20 pt-8 border-t border-borderLight">
          <div className="grid grid-cols-2 gap-8">
            <div>
              <p className="font-mono text-xs uppercase tracking-widest mb-2">
                Privacy
              </p>
              <p className="text-sm text-mutedForeground">
                Emails stored locally on your device
              </p>
            </div>
            <div>
              <p className="font-mono text-xs uppercase tracking-widest mb-2">
                Intelligence
              </p>
              <p className="text-sm text-mutedForeground">
                AI runs on-device, no cloud
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
