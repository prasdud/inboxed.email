import { useEffect, useState } from 'react'
import { useAiStore, ModelOption } from '../../stores/aiStore'

interface ModelDownloadProps {
  onComplete: () => void
  onSkip?: () => void
}

export default function ModelDownload({ onComplete, onSkip }: ModelDownloadProps) {
  const {
    modelStatus,
    downloadProgress,
    availableModels,
    selectedModelId,
    error,
    downloadModel,
    getAvailableModels,
    setSelectedModel,
    initAi,
  } = useAiStore()

  const [isDownloading, setIsDownloading] = useState(false)

  useEffect(() => {
    getAvailableModels()
  }, [getAvailableModels])

  // Set default selection to first (recommended) model
  useEffect(() => {
    if (availableModels.length > 0 && !selectedModelId) {
      setSelectedModel(availableModels[0].id)
    }
  }, [availableModels, selectedModelId, setSelectedModel])

  const handleDownload = async () => {
    setIsDownloading(true)
    try {
      await downloadModel(selectedModelId || undefined)
      // After download, initialize AI
      await initAi()
      onComplete()
    } catch (err) {
      console.error('Download failed:', err)
    } finally {
      setIsDownloading(false)
    }
  }

  const handleSkip = async () => {
    // Initialize with fallback (no LLM, keyword-based)
    await initAi()
    onSkip?.()
  }

  const formatSize = (mb: number) => {
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`
    }
    return `${mb} MB`
  }

  const selectedModel = availableModels.find((m) => m.id === selectedModelId)
  const isError = modelStatus.status === 'error'

  return (
    <div className="flex items-center justify-center min-h-screen bg-background p-8">
      <div className="max-w-2xl w-full">
        {/* Header */}
        <div className="text-center mb-12">
          <div className="inline-block w-16 h-[2px] bg-foreground mb-6" />
          <h1 className="font-display text-4xl tracking-tight mb-4">
            AI Assistant Setup
          </h1>
          <p className="font-serif text-lg text-mutedForeground">
            Download the AI model for intelligent email analysis
          </p>
        </div>

        {/* Model Selection */}
        <div className="mb-8">
          <h2 className="font-mono text-xs uppercase tracking-widest mb-4">
            Select Model
          </h2>
          <div className="space-y-3">
            {availableModels.map((model) => (
              <ModelCard
                key={model.id}
                model={model}
                isSelected={selectedModelId === model.id}
                onSelect={() => setSelectedModel(model.id)}
                disabled={isDownloading}
              />
            ))}
          </div>
        </div>

        {/* Selected Model Details */}
        {selectedModel && (
          <div className="border-[2px] border-foreground p-8 mb-8">
            <div className="flex items-start justify-between mb-6">
              <div>
                <h2 className="font-display text-xl mb-1">{selectedModel.name}</h2>
                <p className="font-serif text-mutedForeground">
                  {selectedModel.description}
                </p>
              </div>
              <div className="text-right">
                <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground mb-1">
                  Size
                </p>
                <p className="font-display text-xl">
                  {formatSize(selectedModel.size_mb)}
                </p>
              </div>
            </div>

            <div className="flex gap-8 mb-6">
              <div>
                <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground mb-1">
                  Speed
                </p>
                <p className="font-serif">{selectedModel.tokens_per_sec}</p>
              </div>
              <div>
                <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground mb-1">
                  Min RAM
                </p>
                <p className="font-serif">{selectedModel.min_ram_gb} GB</p>
              </div>
            </div>

            {/* Progress Bar */}
            {(isDownloading || modelStatus.status === 'downloading') && (
              <div className="mb-6">
                <div className="flex items-center justify-between mb-2">
                  <span className="font-mono text-xs uppercase tracking-widest">
                    Downloading from HuggingFace
                  </span>
                  <span className="font-mono text-xs">
                    {downloadProgress > 0 && downloadProgress < 100
                      ? `${Math.round(downloadProgress)}%`
                      : 'Please wait...'}
                  </span>
                </div>
                <div className="h-2 bg-muted border border-borderLight overflow-hidden">
                  {downloadProgress > 0 ? (
                    <div
                      className="h-full bg-foreground transition-all duration-300"
                      style={{ width: `${downloadProgress}%` }}
                    />
                  ) : (
                    // Indeterminate progress bar animation
                    <div className="h-full w-1/3 bg-foreground animate-[shimmer_1.5s_ease-in-out_infinite]" />
                  )}
                </div>
                <p className="font-serif text-xs text-mutedForeground mt-2">
                  Downloading {formatSize(selectedModel?.size_mb || 0)}... This may take a few minutes.
                </p>
              </div>
            )}

            {/* Loading State */}
            {modelStatus.status === 'loading' && (
              <div className="mb-6 flex items-center gap-3">
                <div className="w-4 h-4 border-[2px] border-foreground border-t-transparent animate-spin" />
                <span className="font-mono text-xs uppercase tracking-widest">
                  Loading model into memory...
                </span>
              </div>
            )}

            {/* Error State */}
            {isError && (
              <div className="mb-6 p-4 bg-muted border border-borderLight">
                <p className="font-mono text-xs uppercase tracking-widest mb-2">
                  Error
                </p>
                <p className="font-serif text-sm">
                  {error || 'An error occurred during download'}
                </p>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex gap-4">
              <button
                onClick={handleDownload}
                disabled={isDownloading || modelStatus.status === 'loading'}
                className="flex-1 px-6 py-3 bg-foreground text-background font-mono text-xs uppercase tracking-widest hover:bg-foreground/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isDownloading || modelStatus.status === 'downloading'
                  ? 'Downloading...'
                  : modelStatus.status === 'loading'
                  ? 'Loading...'
                  : isError
                  ? 'Retry Download'
                  : 'Download Model'}
              </button>

              {onSkip && (
                <button
                  onClick={handleSkip}
                  disabled={isDownloading || modelStatus.status === 'loading'}
                  className="px-6 py-3 border-[2px] border-foreground font-mono text-xs uppercase tracking-widest hover:bg-foreground hover:text-background transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Skip
                </button>
              )}
            </div>
          </div>
        )}

        {/* Footer Note */}
        <p className="text-center font-serif text-sm text-mutedForeground">
          The model will be stored locally and runs entirely on your device.
          <br />
          You can use the app without AI features by clicking Skip.
        </p>
      </div>
    </div>
  )
}

interface ModelCardProps {
  model: ModelOption
  isSelected: boolean
  onSelect: () => void
  disabled: boolean
}

function ModelCard({ model, isSelected, onSelect, disabled }: ModelCardProps) {
  const formatSize = (mb: number) => {
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`
    }
    return `${mb} MB`
  }

  return (
    <button
      onClick={onSelect}
      disabled={disabled}
      className={`w-full text-left p-4 border-[2px] transition-all ${
        isSelected
          ? 'border-foreground bg-muted'
          : 'border-borderLight hover:border-foreground'
      } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div
            className={`w-4 h-4 border-[2px] border-foreground flex items-center justify-center ${
              isSelected ? 'bg-foreground' : ''
            }`}
          >
            {isSelected && (
              <svg
                className="w-3 h-3 text-background"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                  clipRule="evenodd"
                />
              </svg>
            )}
          </div>
          <div>
            <p className="font-mono text-sm">{model.name}</p>
            <p className="font-serif text-xs text-mutedForeground">
              {model.tokens_per_sec} · {model.min_ram_gb}GB+ RAM
            </p>
          </div>
        </div>
        <span className="font-mono text-xs text-mutedForeground">
          {formatSize(model.size_mb)}
        </span>
      </div>
    </button>
  )
}
