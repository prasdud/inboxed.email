import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface StorageInfo {
    database_size_bytes: number
    media_cache_size_bytes: number
    total_emails_cached: number
    total_indexed_emails: number
    data_directory: string
}

interface CacheSettings {
    cache_enabled: boolean
    auto_sync_on_start: boolean
    cache_media_assets: boolean
    max_cache_age_days: number
}

interface StorageSettingsProps {
    onClose: () => void
}

export default function StorageSettings({ onClose }: StorageSettingsProps) {
    const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null)
    const [cacheSettings, setCacheSettings] = useState<CacheSettings | null>(null)
    const [loading, setLoading] = useState(true)
    const [clearing, setClearing] = useState<string | null>(null)
    const [error, setError] = useState<string | null>(null)
    const [showConfirm, setShowConfirm] = useState<string | null>(null)
    const [savingSettings, setSavingSettings] = useState(false)

    useEffect(() => {
        loadData()
    }, [])

    const loadData = async () => {
        try {
            setLoading(true)
            setError(null)
            const [info, settings] = await Promise.all([
                invoke<StorageInfo>('get_storage_info'),
                invoke<CacheSettings>('get_cache_settings'),
            ])
            setStorageInfo(info)
            setCacheSettings(settings)
        } catch (err) {
            setError((err as Error).toString())
        } finally {
            setLoading(false)
        }
    }

    const formatBytes = (bytes: number): string => {
        if (bytes === 0) return '0 Bytes'
        const k = 1024
        const sizes = ['Bytes', 'KB', 'MB', 'GB']
        const i = Math.floor(Math.log(bytes) / Math.log(k))
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }

    const handleClearEmailCache = async () => {
        try {
            setClearing('email')
            await invoke('clear_email_cache')
            await loadData()
            setShowConfirm(null)
        } catch (err) {
            setError((err as Error).toString())
        } finally {
            setClearing(null)
        }
    }

    const handleClearMediaCache = async () => {
        try {
            setClearing('media')
            await invoke('clear_media_cache')
            await loadData()
            setShowConfirm(null)
        } catch (err) {
            setError((err as Error).toString())
        } finally {
            setClearing(null)
        }
    }

    const handleClearAllCaches = async () => {
        try {
            setClearing('all')
            await invoke('clear_all_caches')
            await loadData()
            setShowConfirm(null)
        } catch (err) {
            setError((err as Error).toString())
        } finally {
            setClearing(null)
        }
    }

    const handleClearModels = async () => {
        try {
            setClearing('models')
            await invoke('clear_ai_models')
            setShowConfirm(null)
        } catch (err) {
            setError((err as Error).toString())
        } finally {
            setClearing(null)
        }
    }

    const handleCompleteReset = async () => {
        try {
            setClearing('reset')
            // Clear all app data (emails, cache, settings)
            await invoke('clear_all_app_data')
            // Clear AI models
            await invoke('clear_ai_models')
            // Sign out (clears OAuth tokens)
            await invoke('sign_out')
            // Close settings and let the app redirect to login
            onClose()
            // Reload the page to reset the app state
            window.location.reload()
        } catch (err) {
            setError((err as Error).toString())
        } finally {
            setClearing(null)
        }
    }

    const handleSettingChange = async (key: keyof CacheSettings, value: boolean | number) => {
        if (!cacheSettings) return

        const newSettings = { ...cacheSettings, [key]: value }
        setCacheSettings(newSettings)

        try {
            setSavingSettings(true)
            await invoke('save_cache_settings', { settings: newSettings })
        } catch (err) {
            setError((err as Error).toString())
            // Revert on error
            setCacheSettings(cacheSettings)
        } finally {
            setSavingSettings(false)
        }
    }

    const totalCacheSize = (storageInfo?.database_size_bytes || 0) + (storageInfo?.media_cache_size_bytes || 0)

    if (loading) {
        return (
            <div className="fixed inset-0 bg-background/95 z-50 flex items-center justify-center">
                <div className="text-center">
                    <div className="w-12 h-12 border-[2px] border-foreground border-t-transparent animate-spin mx-auto mb-4" />
                    <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground">
                        Loading storage info
                    </p>
                </div>
            </div>
        )
    }

    return (
        <div className="fixed inset-0 bg-background/95 z-50 overflow-y-auto">
            <div className="max-w-3xl mx-auto p-8">
                {/* Header */}
                <div className="flex items-center justify-between mb-8">
                    <div>
                        <h1 className="font-display text-3xl tracking-tight mb-2">
                            Storage & Cache
                        </h1>
                        <p className="font-serif text-mutedForeground">
                            Manage your local email cache and media storage
                        </p>
                    </div>
                    <button
                        onClick={onClose}
                        className="px-6 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all"
                    >
                        Close
                    </button>
                </div>

                {/* Error Display */}
                {error && (
                    <div className="border-[2px] border-red-500 p-4 mb-8 bg-red-50">
                        <p className="font-mono text-sm text-red-600">{error}</p>
                        <button
                            onClick={() => setError(null)}
                            className="mt-2 text-sm underline text-red-600"
                        >
                            Dismiss
                        </button>
                    </div>
                )}

                {/* Storage Overview */}
                <div className="border-[2px] border-foreground p-6 mb-8">
                    <h2 className="font-mono text-xs uppercase tracking-widest mb-6">
                        Storage Overview
                    </h2>

                    <div className="grid grid-cols-2 gap-6 mb-6">
                        {/* Total Cache Size */}
                        <div className="p-4 bg-muted border border-borderLight">
                            <p className="font-mono text-xs uppercase tracking-widest mb-2 text-mutedForeground">
                                Total Cache Size
                            </p>
                            <p className="font-display text-3xl">{formatBytes(totalCacheSize)}</p>
                        </div>

                        {/* Emails Cached */}
                        <div className="p-4 bg-muted border border-borderLight">
                            <p className="font-mono text-xs uppercase tracking-widest mb-2 text-mutedForeground">
                                Emails Cached
                            </p>
                            <p className="font-display text-3xl">{storageInfo?.total_emails_cached || 0}</p>
                            <p className="font-serif text-sm text-mutedForeground mt-1">
                                {storageInfo?.total_indexed_emails || 0} indexed with AI insights
                            </p>
                        </div>
                    </div>

                    {/* Breakdown */}
                    <div className="space-y-4">
                        {/* Database */}
                        <div className="flex items-center justify-between p-4 border border-borderLight">
                            <div>
                                <p className="font-mono text-sm font-medium">Email Database</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    Cached emails, threads, and AI insights
                                </p>
                            </div>
                            <div className="flex items-center gap-4">
                                <span className="font-mono text-lg">
                                    {formatBytes(storageInfo?.database_size_bytes || 0)}
                                </span>
                                {showConfirm === 'email' ? (
                                    <div className="flex gap-2">
                                        <button
                                            onClick={handleClearEmailCache}
                                            disabled={clearing !== null}
                                            className="px-3 py-2 bg-red-500 text-white font-mono text-xs uppercase tracking-widest hover:bg-red-600 transition-all disabled:opacity-50"
                                        >
                                            {clearing === 'email' ? 'Clearing...' : 'Confirm'}
                                        </button>
                                        <button
                                            onClick={() => setShowConfirm(null)}
                                            disabled={clearing !== null}
                                            className="px-3 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50"
                                        >
                                            Cancel
                                        </button>
                                    </div>
                                ) : (
                                    <button
                                        onClick={() => setShowConfirm('email')}
                                        disabled={clearing !== null || (storageInfo?.total_emails_cached || 0) === 0}
                                        className="px-4 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                                    >
                                        Clear
                                    </button>
                                )}
                            </div>
                        </div>

                        {/* Media Cache */}
                        <div className="flex items-center justify-between p-4 border border-borderLight">
                            <div>
                                <p className="font-mono text-sm font-medium">Media Cache</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    Cached images and attachments from emails
                                </p>
                            </div>
                            <div className="flex items-center gap-4">
                                <span className="font-mono text-lg">
                                    {formatBytes(storageInfo?.media_cache_size_bytes || 0)}
                                </span>
                                {showConfirm === 'media' ? (
                                    <div className="flex gap-2">
                                        <button
                                            onClick={handleClearMediaCache}
                                            disabled={clearing !== null}
                                            className="px-3 py-2 bg-red-500 text-white font-mono text-xs uppercase tracking-widest hover:bg-red-600 transition-all disabled:opacity-50"
                                        >
                                            {clearing === 'media' ? 'Clearing...' : 'Confirm'}
                                        </button>
                                        <button
                                            onClick={() => setShowConfirm(null)}
                                            disabled={clearing !== null}
                                            className="px-3 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50"
                                        >
                                            Cancel
                                        </button>
                                    </div>
                                ) : (
                                    <button
                                        onClick={() => setShowConfirm('media')}
                                        disabled={clearing !== null || (storageInfo?.media_cache_size_bytes || 0) === 0}
                                        className="px-4 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                                    >
                                        Clear
                                    </button>
                                )}
                            </div>
                        </div>
                    </div>
                </div>

                {/* Cache Settings */}
                <div className="border-[2px] border-foreground p-6 mb-8">
                    <div className="flex items-center justify-between mb-6">
                        <h2 className="font-mono text-xs uppercase tracking-widest">
                            Cache Settings
                        </h2>
                        {savingSettings && (
                            <span className="font-mono text-xs text-mutedForeground">Saving...</span>
                        )}
                    </div>

                    <div className="space-y-4">
                        {/* Enable Cache */}
                        <label className="flex items-center justify-between p-4 border border-borderLight cursor-pointer hover:bg-muted transition-colors">
                            <div>
                                <p className="font-mono text-sm font-medium">Enable Email Cache</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    Store emails locally for offline access and faster loading
                                </p>
                            </div>
                            <input
                                type="checkbox"
                                checked={cacheSettings?.cache_enabled ?? true}
                                onChange={(e) => handleSettingChange('cache_enabled', e.target.checked)}
                                className="w-5 h-5 accent-foreground"
                            />
                        </label>

                        {/* Auto Sync on Start */}
                        <label className="flex items-center justify-between p-4 border border-borderLight cursor-pointer hover:bg-muted transition-colors">
                            <div>
                                <p className="font-mono text-sm font-medium">Auto-sync on Start</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    Automatically fetch new emails when the app starts
                                </p>
                            </div>
                            <input
                                type="checkbox"
                                checked={cacheSettings?.auto_sync_on_start ?? false}
                                onChange={(e) => handleSettingChange('auto_sync_on_start', e.target.checked)}
                                className="w-5 h-5 accent-foreground"
                            />
                        </label>

                        {/* Cache Media Assets */}
                        <label className="flex items-center justify-between p-4 border border-borderLight cursor-pointer hover:bg-muted transition-colors">
                            <div>
                                <p className="font-mono text-sm font-medium">Cache Media Assets</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    Save images and attachments locally for faster viewing
                                </p>
                            </div>
                            <input
                                type="checkbox"
                                checked={cacheSettings?.cache_media_assets ?? true}
                                onChange={(e) => handleSettingChange('cache_media_assets', e.target.checked)}
                                className="w-5 h-5 accent-foreground"
                            />
                        </label>

                        {/* Max Cache Age */}
                        <div className="flex items-center justify-between p-4 border border-borderLight">
                            <div>
                                <p className="font-mono text-sm font-medium">Cache Retention</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    How long to keep cached data
                                </p>
                            </div>
                            <select
                                value={cacheSettings?.max_cache_age_days ?? 30}
                                onChange={(e) => handleSettingChange('max_cache_age_days', parseInt(e.target.value))}
                                className="px-4 py-2 border-[2px] border-foreground bg-background font-mono text-sm focus:outline-none"
                            >
                                <option value={7}>7 days</option>
                                <option value={14}>14 days</option>
                                <option value={30}>30 days</option>
                                <option value={60}>60 days</option>
                                <option value={90}>90 days</option>
                                <option value={365}>1 year</option>
                            </select>
                        </div>
                    </div>
                </div>

                {/* Danger Zone */}
                <div className="border-[2px] border-red-500 p-6">
                    <h2 className="font-mono text-xs uppercase tracking-widest mb-4 text-red-600">
                        Danger Zone
                    </h2>

                    <div className="space-y-4">
                        {/* Clear All Caches */}
                        <div className="flex items-center justify-between">
                            <div>
                                <p className="font-mono text-sm font-medium">Clear All Caches</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    Remove all cached emails and media. This cannot be undone.
                                </p>
                            </div>
                            {showConfirm === 'all' ? (
                                <div className="flex gap-2">
                                    <button
                                        onClick={handleClearAllCaches}
                                        disabled={clearing !== null}
                                        className="px-4 py-2 bg-red-500 text-white font-mono text-xs uppercase tracking-widest hover:bg-red-600 transition-all disabled:opacity-50"
                                    >
                                        {clearing === 'all' ? 'Clearing...' : 'Yes, Clear All'}
                                    </button>
                                    <button
                                        onClick={() => setShowConfirm(null)}
                                        disabled={clearing !== null}
                                        className="px-4 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50"
                                    >
                                        Cancel
                                    </button>
                                </div>
                            ) : (
                                <button
                                    onClick={() => setShowConfirm('all')}
                                    disabled={clearing !== null || totalCacheSize === 0}
                                    className="px-4 py-2 border-[2px] border-red-500 text-red-500 font-mono text-xs uppercase tracking-widest hover:bg-red-500 hover:text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    Clear All
                                </button>
                            )}
                        </div>

                        {/* Clear AI Models */}
                        <div className="flex items-center justify-between pt-4 border-t border-red-200">
                            <div>
                                <p className="font-mono text-sm font-medium">Clear AI Models</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    Delete downloaded AI models (697 MB). You'll need to re-download them.
                                </p>
                            </div>
                            {showConfirm === 'models' ? (
                                <div className="flex gap-2">
                                    <button
                                        onClick={handleClearModels}
                                        disabled={clearing !== null}
                                        className="px-4 py-2 bg-red-500 text-white font-mono text-xs uppercase tracking-widest hover:bg-red-600 transition-all disabled:opacity-50"
                                    >
                                        {clearing === 'models' ? 'Clearing...' : 'Confirm'}
                                    </button>
                                    <button
                                        onClick={() => setShowConfirm(null)}
                                        disabled={clearing !== null}
                                        className="px-4 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50"
                                    >
                                        Cancel
                                    </button>
                                </div>
                            ) : (
                                <button
                                    onClick={() => setShowConfirm('models')}
                                    disabled={clearing !== null}
                                    className="px-4 py-2 border-[2px] border-red-500 text-red-500 font-mono text-xs uppercase tracking-widest hover:bg-red-500 hover:text-white transition-all disabled:opacity-50"
                                >
                                    Clear Models
                                </button>
                            )}
                        </div>

                        {/* Complete Reset */}
                        <div className="flex items-center justify-between pt-4 border-t border-red-200">
                            <div>
                                <p className="font-mono text-sm font-medium">Complete Reset</p>
                                <p className="font-serif text-sm text-mutedForeground">
                                    Clear everything: emails, cache, AI models, and sign out. Start fresh.
                                </p>
                            </div>
                            {showConfirm === 'reset' ? (
                                <div className="flex gap-2">
                                    <button
                                        onClick={handleCompleteReset}
                                        disabled={clearing !== null}
                                        className="px-4 py-2 bg-red-600 text-white font-mono text-xs uppercase tracking-widest hover:bg-red-700 transition-all disabled:opacity-50"
                                    >
                                        {clearing === 'reset' ? 'Resetting...' : 'Yes, Reset All'}
                                    </button>
                                    <button
                                        onClick={() => setShowConfirm(null)}
                                        disabled={clearing !== null}
                                        className="px-4 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50"
                                    >
                                        Cancel
                                    </button>
                                </div>
                            ) : (
                                <button
                                    onClick={() => setShowConfirm('reset')}
                                    disabled={clearing !== null}
                                    className="px-4 py-2 bg-red-600 text-white font-mono text-xs uppercase tracking-widest hover:bg-red-700 transition-all disabled:opacity-50"
                                >
                                    Reset App
                                </button>
                            )}
                        </div>
                    </div>
                </div>

                {/* Data Location */}
                <div className="mt-8 text-center">
                    <p className="font-serif text-sm text-mutedForeground">
                        Data stored in: <code className="font-mono text-xs bg-muted px-2 py-1">{storageInfo?.data_directory}</code>
                    </p>
                </div>
            </div>
        </div>
    )
}
