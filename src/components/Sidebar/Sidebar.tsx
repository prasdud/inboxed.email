import { useState } from 'react'
import { useAiStore } from '../../stores/aiStore'

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
}

export default function Sidebar({ onFolderSelect, onCompose, onOpenModelSettings }: SidebarProps) {
  const [activeFolder, setActiveFolder] = useState('inbox')
  const { modelStatus, downloadProgress, isModelLoaded, isAiReady } = useAiStore()

  const handleFolderClick = (folderId: string) => {
    setActiveFolder(folderId)
    onFolderSelect(folderId)
  }

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

  return (
    <div className="w-64 bg-background border-r-[2px] border-foreground flex flex-col">
      {/* Header */}
      <div className="p-6 border-b-[2px] border-foreground">
        <h1 className="font-display text-3xl tracking-tighter">Inboxed</h1>
        <p className="font-mono text-xs uppercase tracking-widest mt-2 text-mutedForeground">
          Inbox
        </p>
      </div>

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
        {folders.map((folder, index) => (
          <button
            key={folder.id}
            onClick={() => handleFolderClick(folder.id)}
            className={`group w-full text-left px-6 py-4 border-l-[4px] transition-all duration-100 focus-visible:outline focus-visible:outline-3 focus-visible:outline-foreground focus-visible:outline-offset-[-3px] ${
              activeFolder === folder.id
                ? 'border-foreground bg-foreground text-background'
                : 'border-transparent hover:border-foreground hover:bg-muted'
            }`}
          >
            <div className="flex items-center justify-between">
              <span className="font-serif text-lg">{folder.name}</span>
              {folder.count !== undefined && folder.count > 0 && (
                <span
                  className={`font-mono text-xs px-2 py-1 border ${
                    activeFolder === folder.id
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

      {/* AI Status & Settings */}
      <div className="p-4 border-t-[2px] border-foreground">
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
