import { useEffect } from 'react'
import { useAiStore, ModelOption } from '../../stores/aiStore'

interface ModelSettingsProps {
  onClose: () => void
}

export default function ModelSettings({ onClose }: ModelSettingsProps) {
  const {
    modelStatus,
    downloadProgress,
    isModelLoaded,
    isAiReady,
    availableModels,
    selectedModelId,
    error,
    downloadModel,
    getAvailableModels,
    setSelectedModel,
    initAi,
    checkModelStatus,
  } = useAiStore()

  useEffect(() => {
    getAvailableModels()
    checkModelStatus()
  }, [getAvailableModels, checkModelStatus])

  const handleDownload = async (modelId: string) => {
    setSelectedModel(modelId)
    try {
      await downloadModel(modelId)
      await initAi()
    } catch (err) {
      console.error('Download failed:', err)
    }
  }

  const formatSize = (mb: number) => {
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`
    }
    return `${mb} MB`
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
          {isLoading && (
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
                isSelected={selectedModelId === model.id}
                isDownloading={isDownloading && selectedModelId === model.id}
                isReady={isAiReady && isModelLoaded && selectedModelId === model.id}
                onDownload={() => handleDownload(model.id)}
                disabled={isDownloading || isLoading}
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

interface ModelCardProps {
  model: ModelOption
  isSelected: boolean
  isDownloading: boolean
  isReady: boolean
  onDownload: () => void
  disabled: boolean
}

function ModelCard({
  model,
  isSelected,
  isDownloading,
  isReady,
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
      className={`p-4 border-[2px] transition-all ${
        isSelected ? 'border-foreground bg-muted' : 'border-borderLight'
      }`}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-mono text-sm font-medium">{model.name}</h3>
            {isReady && (
              <span className="px-2 py-0.5 bg-foreground text-background font-mono text-[10px] uppercase">
                Active
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
          {isReady ? (
            <span className="px-4 py-2 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest opacity-50">
              In Use
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
