import { useAuthStore } from '../../stores/authStore'

export default function LoginScreen() {
  const { signIn, loading, error } = useAuthStore()

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

        {/* CTA Button */}
        <div className="space-y-6">
          <button
            onClick={signIn}
            disabled={loading}
            className="group relative px-12 py-5 bg-foreground text-background font-mono text-sm uppercase tracking-widest font-medium transition-all duration-100 hover:bg-background hover:text-foreground border-[2px] border-transparent hover:border-foreground disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
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

          {error && (
            <div className="border-[2px] border-foreground p-6">
              <p className="font-mono text-sm uppercase tracking-wider mb-2">
                Error
              </p>
              <p className="text-mutedForeground">{error}</p>
            </div>
          )}
        </div>

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
