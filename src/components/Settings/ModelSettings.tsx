import { useEffect, useState } from 'react'
import { useAiStore, ModelOption } from '../../stores/aiStore'

interface ModelSettingsProps {
  onClose: () => void
}

export default function ModelSettings({ onClose }: ModelSettingsProps) {
  const {
    modelStatus,
    downloadProgress,
    isModelLoaded,
    availableModels,
    downloadedModels,
    selectedModelId,
    activeModelId,
    error,
    isDeleting,
    isActivating,
    downloadModel,
    getAvailableModels,
    getDownloadedModels,
    getActiveModelId,
    setSelectedModel,
    deleteModel,
    activateModel,
    initAi,
    checkModelStatus,
  } = useAiStore()

  const [confirmDelete, setConfirmDelete] = useState<string | null>(null)

  useEffect(() => {
    getAvailableModels()
    getDownloadedModels()
    getActiveModelId()
    checkModelStatus()
  }, [getAvailableModels, getDownloadedModels, getActiveModelId, checkModelStatus])

  const handleDownload = async (modelId: string) => {
    setSelectedModel(modelId)
    try {
      await downloadModel(modelId)
      await initAi()
    } catch (err) {
      console.error('Download failed:', err)
    }
  }

  const handleActivate = async (modelId: string) => {
    try {
      await activateModel(modelId)
    } catch (err) {
      console.error('Activation failed:', err)
    }
  }

  const handleDelete = async (modelId: string) => {
    try {
      await deleteModel(modelId)
      setConfirmDelete(null)
    } catch (err) {
      console.error('Delete failed:', err)
    }
  }

  const formatSize = (mb: number) => {
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`
    }
    return `${mb} MB`
  }

  const isModelDownloaded = (modelId: string) => {
    return downloadedModels.some((m) => m.id === modelId)
  }

  const isModelActive = (modelId: string) => {
    return activeModelId === modelId && isModelLoaded
  }

  const getStatusText = () => {
    switch (modelStatus.status) {
      case 'not_downloaded':
        return 'No model downloaded'
      case 'downloading':
        return `Downloading... ${downloadProgress > 0 ? `${Math.round(downloadProgress)}%` : ''}`
      case 'downloaded':
        return 'Model downloaded, loading...'
      case 'loading':
        return 'Loading model into memory...'
      case 'ready':
        return isModelLoaded ? 'AI Ready (LLM)' : 'AI Ready (Fallback)'
      case 'error':
        return `Error: ${error}`
      default:
        return 'Unknown'
    }
  }

  const getStatusColor = () => {
    switch (modelStatus.status) {
      case 'ready':
        return 'bg-green-500'
      case 'downloading':
      case 'loading':
        return 'bg-yellow-500'
      case 'error':
        return 'bg-red-500'
      default:
        return 'bg-gray-500'
    }
  }

  const selectedModel = availableModels.find((m) => m.id === selectedModelId)
  const isDownloading = modelStatus.status === 'downloading'
  const isLoading = modelStatus.status === 'loading'

  // Find active model info for display
  const activeModel = activeModelId
    ? availableModels.find((m) => m.id === activeModelId)
    : null

  return (
    <div className="fixed inset-0 bg-background/95 z-50 overflow-y-auto">
      <div className="max-w-3xl mx-auto p-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="font-display text-3xl tracking-tight mb-2">
              AI Model Settings
            </h1>
            <p className="font-serif text-mutedForeground">
              Manage your local AI model for email analysis
            </p>
          </div>
          <button
            onClick={onClose}
            className="px-6 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all"
          >
            Close
          </button>
        </div>

        {/* Current Status */}
        <div className="border-[2px] border-foreground p-6 mb-8">
          <h2 className="font-mono text-xs uppercase tracking-widest mb-4">
            Current Status
          </h2>
          <div className="flex items-center gap-3 mb-4">
            <div className={`w-3 h-3 rounded-full ${getStatusColor()}`} />
            <span className="font-serif text-lg">{getStatusText()}</span>
          </div>

          {/* Active Model Info */}
          {activeModel && isModelLoaded && (
            <div className="mt-4 p-4 bg-muted border border-borderLight">
              <p className="font-mono text-xs uppercase tracking-widest mb-2 text-green-600">
                Active Model
              </p>
              <p className="font-serif font-medium">{activeModel.name}</p>
              <p className="font-serif text-sm text-mutedForeground">
                {activeModel.description}
              </p>
            </div>
          )}

          {/* Download Progress */}
          {isDownloading && (
            <div className="mt-4">
              <div className="flex items-center justify-between mb-2">
                <span className="font-mono text-xs uppercase tracking-widest">
                  Download Progress
                </span>
                <span className="font-mono text-xs">
                  {downloadProgress > 0 ? `${Math.round(downloadProgress)}%` : 'Starting...'}
                </span>
              </div>
              <div className="h-3 bg-muted border border-borderLight overflow-hidden">
                {downloadProgress > 0 ? (
                  <div
                    className="h-full bg-foreground transition-all duration-300"
                    style={{ width: `${downloadProgress}%` }}
                  />
                ) : (
                  <div className="h-full w-1/3 bg-foreground animate-pulse" />
                )}
              </div>
              {selectedModel && (
                <p className="font-serif text-sm text-mutedForeground mt-2">
                  Downloading {selectedModel.name} ({formatSize(selectedModel.size_mb)})
                </p>
              )}
            </div>
          )}

          {/* Loading Progress */}
          {(isLoading || isActivating) && (
            <div className="mt-4 flex items-center gap-3">
              <div className="w-5 h-5 border-[2px] border-foreground border-t-transparent animate-spin" />
              <span className="font-serif text-mutedForeground">
                Loading model into memory...
              </span>
            </div>
          )}

          {/* Error */}
          {modelStatus.status === 'error' && error && (
            <div className="mt-4 p-4 bg-muted border border-borderLight">
              <p className="font-mono text-xs uppercase tracking-widest mb-2 text-red-600">
                Error Details
              </p>
              <p className="font-serif text-sm">{error}</p>
            </div>
          )}
        </div>

        {/* Downloaded Models Section */}
        {downloadedModels.length > 0 && (
          <div className="border-[2px] border-foreground p-6 mb-8">
            <h2 className="font-mono text-xs uppercase tracking-widest mb-4">
              Downloaded Models ({downloadedModels.length})
            </h2>
            <div className="space-y-4">
              {downloadedModels.map((model) => (
                <DownloadedModelCard
                  key={model.id}
                  model={model}
                  isActive={isModelActive(model.id)}
                  isActivating={isActivating && selectedModelId === model.id}
                  isDeleting={isDeleting}
                  showDeleteConfirm={confirmDelete === model.id}
                  onActivate={() => handleActivate(model.id)}
                  onDelete={() => handleDelete(model.id)}
                  onShowDeleteConfirm={() => setConfirmDelete(model.id)}
                  onCancelDelete={() => setConfirmDelete(null)}
                  disabled={isDownloading || isLoading || isActivating || isDeleting}
                />
              ))}
            </div>
          </div>
        )}

        {/* Available Models */}
        <div className="border-[2px] border-foreground p-6">
          <h2 className="font-mono text-xs uppercase tracking-widest mb-4">
            Available Models
          </h2>
          <div className="space-y-4">
            {availableModels.map((model) => (
              <ModelCard
                key={model.id}
                model={model}
                isDownloaded={isModelDownloaded(model.id)}
                isActive={isModelActive(model.id)}
                isDownloading={isDownloading && selectedModelId === model.id}
                onDownload={() => handleDownload(model.id)}
                disabled={isDownloading || isLoading || isActivating || isDeleting}
              />
            ))}
          </div>
        </div>

        {/* Info */}
        <div className="mt-8 text-center">
          <p className="font-serif text-sm text-mutedForeground">
            Models are downloaded from HuggingFace and stored locally.
            <br />
            All AI processing happens on your device - your emails never leave your computer.
          </p>
        </div>
      </div>
    </div>
  )
}

interface DownloadedModelCardProps {
  model: ModelOption
  isActive: boolean
  isActivating: boolean
  isDeleting: boolean
  showDeleteConfirm: boolean
  onActivate: () => void
  onDelete: () => void
  onShowDeleteConfirm: () => void
  onCancelDelete: () => void
  disabled: boolean
}

function DownloadedModelCard({
  model,
  isActive,
  isActivating,
  isDeleting,
  showDeleteConfirm,
  onActivate,
  onDelete,
  onShowDeleteConfirm,
  onCancelDelete,
  disabled,
}: DownloadedModelCardProps) {
  const formatSize = (mb: number) => {
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`
    }
    return `${mb} MB`
  }

  return (
    <div
      className={`p-4 border-[2px] transition-all ${isActive ? 'border-green-500 bg-green-500/5' : 'border-foreground bg-muted/50'
        }`}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-mono text-sm font-medium">{model.name}</h3>
            {isActive && (
              <span className="px-2 py-0.5 bg-green-500 text-white font-mono text-[10px] uppercase">
                Active
              </span>
            )}
            <span className="px-2 py-0.5 bg-blue-500/20 text-blue-600 font-mono text-[10px] uppercase">
              Downloaded
            </span>
          </div>
          <p className="font-serif text-sm text-mutedForeground mb-2">
            {model.description}
          </p>
          <div className="flex gap-4 text-xs text-mutedForeground">
            <span className="font-mono">{formatSize(model.size_mb)}</span>
            <span className="font-mono">{model.tokens_per_sec}</span>
            <span className="font-mono">{model.min_ram_gb}GB+ RAM</span>
          </div>
        </div>
        <div className="ml-4 flex gap-2">
          {showDeleteConfirm ? (
            <>
              <button
                onClick={onDelete}
                disabled={isDeleting}
                className="px-3 py-2 bg-red-500 text-white font-mono text-xs uppercase tracking-widest hover:bg-red-600 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isDeleting ? 'Deleting...' : 'Confirm'}
              </button>
              <button
                onClick={onCancelDelete}
                disabled={isDeleting}
                className="px-3 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50"
              >
                Cancel
              </button>
            </>
          ) : (
            <>
              {!isActive && (
                <button
                  onClick={onActivate}
                  disabled={disabled}
                  className="px-3 py-2 bg-foreground text-background font-mono text-xs uppercase tracking-widest hover:opacity-80 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isActivating ? 'Loading...' : 'Activate'}
                </button>
              )}
              <button
                onClick={onShowDeleteConfirm}
                disabled={disabled || isActive}
                className="px-3 py-2 border-[2px] border-red-500 text-red-500 font-mono text-xs uppercase tracking-widest hover:bg-red-500 hover:text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                title={isActive ? 'Cannot delete active model' : 'Delete model'}
              >
                Delete
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  )
}

interface ModelCardProps {
  model: ModelOption
  isDownloaded: boolean
  isActive: boolean
  isDownloading: boolean
  onDownload: () => void
  disabled: boolean
}

function ModelCard({
  model,
  isDownloaded,
  isActive,
  isDownloading,
  onDownload,
  disabled,
}: ModelCardProps) {
  const formatSize = (mb: number) => {
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`
    }
    return `${mb} MB`
  }

  return (
    <div
      className={`p-4 border-[2px] transition-all ${isActive
        ? 'border-green-500 bg-green-500/5'
        : isDownloaded
          ? 'border-foreground bg-muted/30'
          : 'border-borderLight'
        }`}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-mono text-sm font-medium">{model.name}</h3>
            {isActive && (
              <span className="px-2 py-0.5 bg-green-500 text-white font-mono text-[10px] uppercase">
                Active
              </span>
            )}
            {isDownloaded && !isActive && (
              <span className="px-2 py-0.5 bg-blue-500/20 text-blue-600 font-mono text-[10px] uppercase">
                Downloaded
              </span>
            )}
          </div>
          <p className="font-serif text-sm text-mutedForeground mb-2">
            {model.description}
          </p>
          <div className="flex gap-4 text-xs text-mutedForeground">
            <span className="font-mono">{formatSize(model.size_mb)}</span>
            <span className="font-mono">{model.tokens_per_sec}</span>
            <span className="font-mono">{model.min_ram_gb}GB+ RAM</span>
          </div>
        </div>
        <div className="ml-4">
          {isActive ? (
            <span className="px-4 py-2 border-[2px] border-green-500 text-green-600 font-mono text-xs uppercase tracking-widest">
              In Use
            </span>
          ) : isDownloaded ? (
            <span className="px-4 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest opacity-50">
              Ready
            </span>
          ) : (
            <button
              onClick={onDownload}
              disabled={disabled}
              className="px-4 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isDownloading ? 'Downloading...' : 'Download'}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
