import { useState, useEffect } from 'react'
import { useAiStore } from '../../stores/aiStore'
import { useAccountStore } from '../../stores/accountStore'

interface Folder {
  id: string
  name: string
  count?: number
}

const folders: Folder[] = [
  { id: 'inbox', name: 'Inbox', count: 0 },
  { id: 'sent', name: 'Sent' },
  { id: 'drafts', name: 'Drafts', count: 0 },
  { id: 'trash', name: 'Trash' },
  { id: 'spam', name: 'Spam' },
]

interface SidebarProps {
  onFolderSelect: (folderId: string) => void
  onCompose: () => void
  onOpenModelSettings: () => void
  onOpenStorageSettings: () => void
}

export default function Sidebar({ onFolderSelect, onCompose, onOpenModelSettings, onOpenStorageSettings }: SidebarProps) {
  const [activeFolder, setActiveFolder] = useState('inbox')
  const { modelStatus, downloadProgress, isModelLoaded } = useAiStore()
  const { accounts, activeAccountId, fetchAccounts, setActiveAccount } = useAccountStore()
  const [showAccountMenu, setShowAccountMenu] = useState(false)

  useEffect(() => {
    fetchAccounts()
  }, [fetchAccounts])

  const handleFolderClick = (folderId: string) => {
    setActiveFolder(folderId)
    onFolderSelect(folderId)
  }

  const handleAccountSwitch = async (accountId: string) => {
    await setActiveAccount(accountId)
    setShowAccountMenu(false)
  }

  const activeAccount = accounts.find((a) => a.id === activeAccountId)

  const getAiStatusText = () => {
    switch (modelStatus.status) {
      case 'downloading':
        return `Downloading ${downloadProgress > 0 ? `${Math.round(downloadProgress)}%` : '...'}`
      case 'loading':
        return 'Loading...'
      case 'ready':
        return isModelLoaded ? 'AI Ready' : 'Fallback Mode'
      case 'error':
        return 'Error'
      default:
        return 'Not Set Up'
    }
  }

  const getAiStatusColor = () => {
    switch (modelStatus.status) {
      case 'ready':
        return isModelLoaded ? 'bg-green-500' : 'bg-yellow-500'
      case 'downloading':
      case 'loading':
        return 'bg-blue-500 animate-pulse'
      case 'error':
        return 'bg-red-500'
      default:
        return 'bg-gray-400'
    }
  }

  const getProviderColor = (provider: string) => {
    switch (provider) {
      case 'gmail':
        return 'bg-red-500'
      case 'outlook':
        return 'bg-blue-500'
      case 'yahoo':
        return 'bg-purple-500'
      default:
        return 'bg-gray-500'
    }
  }

  return (
    <div className="w-56 lg:w-64 flex-shrink-0 bg-background border-r-[2px] border-foreground flex flex-col">
      {/* Header */}
      <div className="p-6 border-b-[2px] border-foreground">
        <h1 className="font-display text-3xl tracking-tighter">Inboxed</h1>
        <p className="font-mono text-xs uppercase tracking-widest mt-2 text-mutedForeground">
          {activeAccount?.email || 'Inbox'}
        </p>
      </div>

      {/* Account Switcher */}
      {accounts.length > 0 && (
        <div className="border-b-[2px] border-foreground">
          <button
            onClick={() => setShowAccountMenu(!showAccountMenu)}
            className="w-full p-4 text-left hover:bg-muted transition-colors"
          >
            <div className="flex items-center gap-3">
              <div className={`w-2 h-2 rounded-full ${activeAccount ? getProviderColor(activeAccount.provider) : 'bg-gray-400'}`} />
              <div className="flex-1 min-w-0">
                <p className="font-serif text-sm truncate">
                  {activeAccount?.display_name || 'No account'}
                </p>
                <p className="font-mono text-xs text-mutedForeground truncate">
                  {activeAccount?.email || 'Select an account'}
                </p>
              </div>
              <span className="font-mono text-xs text-mutedForeground">
                {showAccountMenu ? '▲' : '▼'}
              </span>
            </div>
          </button>

          {showAccountMenu && (
            <div className="border-t border-borderLight">
              {accounts.map((account) => (
                <button
                  key={account.id}
                  onClick={() => handleAccountSwitch(account.id)}
                  className={`w-full p-3 text-left hover:bg-muted transition-colors flex items-center gap-3 ${
                    account.id === activeAccountId ? 'bg-muted' : ''
                  }`}
                >
                  <div className={`w-2 h-2 rounded-full ${getProviderColor(account.provider)}`} />
                  <div className="flex-1 min-w-0">
                    <p className="font-serif text-xs truncate">{account.email}</p>
                  </div>
                  {account.id === activeAccountId && (
                    <span className="font-mono text-xs">✓</span>
                  )}
                </button>
              ))}
              <button
                onClick={() => {
                  setShowAccountMenu(false)
                  // Navigate to add account — handled by parent
                }}
                className="w-full p-3 text-left hover:bg-muted transition-colors font-mono text-xs uppercase tracking-widest text-mutedForeground"
              >
                + Add Account
              </button>
            </div>
          )}
        </div>
      )}

      {/* Compose Button */}
      <div className="p-6 border-b-[2px] border-foreground">
        <button
          onClick={onCompose}
          className="w-full px-6 py-4 bg-foreground text-background font-mono text-xs uppercase tracking-widest hover:bg-background hover:text-foreground border-[2px] border-transparent hover:border-foreground transition-all duration-100 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-3"
        >
          Compose
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 py-8">
        {folders.map((folder) => (
          <button
            key={folder.id}
            onClick={() => handleFolderClick(folder.id)}
            className={`group w-full text-left px-6 py-4 border-l-[4px] transition-all duration-100 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-[-3px] ${activeFolder === folder.id
              ? 'border-foreground bg-foreground text-background'
              : 'border-transparent hover:border-foreground hover:bg-muted'
              }`}
          >
            <div className="flex items-center justify-between">
              <span className="font-serif text-lg">{folder.name}</span>
              {folder.count !== undefined && folder.count > 0 && (
                <span
                  className={`font-mono text-xs px-2 py-1 border ${activeFolder === folder.id
                    ? 'border-background text-background'
                    : 'border-foreground'
                    }`}
                >
                  {folder.count}
                </span>
              )}
            </div>
          </button>
        ))}
      </nav>

      {/* Settings Section */}
      <div className="p-4 border-t-[2px] border-foreground space-y-2">
        {/* AI Model Settings */}
        <button
          onClick={onOpenModelSettings}
          className="w-full p-3 border-[2px] border-borderLight hover:border-foreground transition-all group"
        >
          <div className="flex items-center justify-between mb-2">
            <span className="font-mono text-xs uppercase tracking-widest">
              AI Model
            </span>
            <div className={`w-2 h-2 rounded-full ${getAiStatusColor()}`} />
          </div>
          <div className="text-left">
            <p className="font-serif text-sm">{getAiStatusText()}</p>
            {modelStatus.status === 'downloading' && (
              <div className="mt-2 h-1 bg-muted overflow-hidden">
                <div
                  className="h-full bg-foreground transition-all duration-300"
                  style={{ width: `${downloadProgress}%` }}
                />
              </div>
            )}
          </div>
        </button>

        {/* Storage Settings */}
        <button
          onClick={onOpenStorageSettings}
          className="w-full p-3 border-[2px] border-borderLight hover:border-foreground transition-all group text-left"
        >
          <div className="flex items-center justify-between">
            <span className="font-mono text-xs uppercase tracking-widest">
              Storage & Cache
            </span>
            <svg
              className="w-4 h-4 text-mutedForeground group-hover:text-foreground transition-colors"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 7v10c0 2 1 3 3 3h10c2 0 3-1 3-3V7c0-2-1-3-3-3H7c-2 0-3 1-3 3z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 7h16M8 4v3m8-3v3"
              />
            </svg>
          </div>
          <p className="font-serif text-sm text-mutedForeground mt-1">
            Manage cached emails & media
          </p>
        </button>
      </div>

      {/* Footer */}
      <div className="px-6 py-4 border-t border-borderLight">
        <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground">
          Inboxed v1.0
        </p>
      </div>
    </div>
  )
}
