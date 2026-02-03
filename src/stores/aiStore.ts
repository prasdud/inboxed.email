import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

export type ModelStatus =
  | { status: 'not_downloaded' }
  | { status: 'downloading'; progress: number }
  | { status: 'downloaded' }
  | { status: 'loading' }
  | { status: 'ready' }
  | { status: 'error'; message: string }

export interface ModelOption {
  id: string
  name: string
  repo: string
  filename: string
  size_mb: number
  description: string
  min_ram_gb: number
  tokens_per_sec: string
}

export interface ModelInfo {
  repo: string
  filename: string
  size_mb: number
}

interface AiStore {
  modelStatus: ModelStatus
  downloadProgress: number
  isModelLoaded: boolean
  isAiReady: boolean  // True when AI can be used (model loaded or fallback ready)
  modelInfo: ModelInfo | null
  availableModels: ModelOption[]
  selectedModelId: string | null
  error: string | null

  // Actions
  checkModelStatus: () => Promise<void>
  downloadModel: (modelId?: string) => Promise<void>
  initAi: () => Promise<boolean>
  getModelInfo: () => Promise<void>
  getAvailableModels: () => Promise<void>
  setSelectedModel: (modelId: string) => void
  reset: () => void
}

export const useAiStore = create<AiStore>((set, get) => ({
  modelStatus: { status: 'not_downloaded' },
  downloadProgress: 0,
  isModelLoaded: false,
  isAiReady: false,
  modelInfo: null,
  availableModels: [],
  selectedModelId: null,
  error: null,

  checkModelStatus: async () => {
    try {
      const status = await invoke<ModelStatus>('check_model_status')
      set({
        modelStatus: status,
        isModelLoaded: status.status === 'ready',
        error: null,
      })
    } catch (error) {
      set({
        error: (error as Error).toString(),
        modelStatus: { status: 'error', message: (error as Error).toString() },
      })
    }
  },

  downloadModel: async (modelId?: string) => {
    let progressUnlisten: UnlistenFn | null = null
    let completeUnlisten: UnlistenFn | null = null
    let errorUnlisten: UnlistenFn | null = null

    try {
      set({
        modelStatus: { status: 'downloading', progress: 0 },
        downloadProgress: 0,
        error: null,
      })

      // Listen for progress events
      progressUnlisten = await listen<number>('model:progress', (event) => {
        set({
          downloadProgress: event.payload,
          modelStatus: { status: 'downloading', progress: event.payload },
        })
      })

      // Listen for completion
      completeUnlisten = await listen('model:complete', () => {
        set({
          modelStatus: { status: 'downloaded' },
          downloadProgress: 100,
        })
      })

      // Listen for errors
      errorUnlisten = await listen<string>('model:error', (event) => {
        set({
          modelStatus: { status: 'error', message: event.payload },
          error: event.payload,
        })
      })

      // Start download - use specific model ID if provided
      if (modelId) {
        await invoke('download_model_by_id', { modelId })
        set({ selectedModelId: modelId })
      } else {
        await invoke('download_model')
        set({ selectedModelId: 'lfm2.5-1.2b-q4' }) // Default model
      }

      // Update status after successful download
      set({ modelStatus: { status: 'downloaded' } })
    } catch (error) {
      set({
        error: (error as Error).toString(),
        modelStatus: { status: 'error', message: (error as Error).toString() },
      })
      throw error
    } finally {
      // Clean up listeners
      if (progressUnlisten) progressUnlisten()
      if (completeUnlisten) completeUnlisten()
      if (errorUnlisten) errorUnlisten()
    }
  },

  initAi: async () => {
    try {
      set({ modelStatus: { status: 'loading' }, error: null })

      // Use fallback init which works with or without model
      const modelLoaded = await invoke<boolean>('init_ai_fallback')

      set({
        modelStatus: { status: 'ready' },
        isModelLoaded: modelLoaded,
        isAiReady: true,  // AI is ready (either with model or fallback)
      })

      return modelLoaded
    } catch (error) {
      set({
        error: (error as Error).toString(),
        modelStatus: { status: 'error', message: (error as Error).toString() },
        isAiReady: false,
      })
      return false
    }
  },

  getModelInfo: async () => {
    try {
      const info = await invoke<ModelInfo>('get_model_info')
      set({ modelInfo: info })
    } catch (error) {
      console.error('Failed to get model info:', error)
    }
  },

  getAvailableModels: async () => {
    try {
      const models = await invoke<ModelOption[]>('get_available_ai_models')
      set({ availableModels: models })
    } catch (error) {
      console.error('Failed to get available models:', error)
    }
  },

  setSelectedModel: (modelId: string) => {
    set({ selectedModelId: modelId })
  },

  reset: () => {
    set({
      modelStatus: { status: 'not_downloaded' },
      downloadProgress: 0,
      isModelLoaded: false,
      isAiReady: false,
      error: null,
    })
  },
}))
