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
  downloadedModels: ModelOption[]
  selectedModelId: string | null
  activeModelId: string | null  // Currently loaded model
  error: string | null
  isDeleting: boolean
  isActivating: boolean

  // Actions
  checkModelStatus: () => Promise<void>
  downloadModel: (modelId?: string) => Promise<void>
  initAi: () => Promise<boolean>
  getModelInfo: () => Promise<void>
  getAvailableModels: () => Promise<void>
  getDownloadedModels: () => Promise<void>
  getActiveModelId: () => Promise<void>
  deleteModel: (modelId: string) => Promise<void>
  activateModel: (modelId: string) => Promise<void>
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
  downloadedModels: [],
  selectedModelId: null,
  activeModelId: null,
  error: null,
  isDeleting: false,
  isActivating: false,

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

      // Refresh downloaded models list
      await get().getDownloadedModels()
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

      // Verify the actual status from backend
      const actualStatus = await invoke<ModelStatus>('check_model_status')

      set({
        modelStatus: actualStatus,
        isModelLoaded: actualStatus.status === 'ready',
        isAiReady: true,  // AI is ready (either with model or fallback)
      })

      // Refresh active model ID
      await get().getActiveModelId()

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

  getDownloadedModels: async () => {
    try {
      const models = await invoke<ModelOption[]>('get_downloaded_models')
      set({ downloadedModels: models })
    } catch (error) {
      console.error('Failed to get downloaded models:', error)
    }
  },

  getActiveModelId: async () => {
    try {
      const modelId = await invoke<string | null>('get_active_model_id')
      set({ activeModelId: modelId })
    } catch (error) {
      console.error('Failed to get active model ID:', error)
    }
  },

  deleteModel: async (modelId: string) => {
    try {
      set({ isDeleting: true, error: null })
      await invoke('delete_model', { modelId })

      // Refresh lists
      await get().getDownloadedModels()
      await get().getActiveModelId()
      await get().checkModelStatus()

      set({ isDeleting: false })
    } catch (error) {
      set({
        error: (error as Error).toString(),
        isDeleting: false,
      })
      throw error
    }
  },

  activateModel: async (modelId: string) => {
    try {
      set({ isActivating: true, modelStatus: { status: 'loading' }, error: null })
      await invoke('activate_model', { modelId })

      // Verify the actual status from backend
      const actualStatus = await invoke<ModelStatus>('check_model_status')

      set({
        activeModelId: modelId,
        selectedModelId: modelId,
        isActivating: false,
        modelStatus: actualStatus,
        isModelLoaded: actualStatus.status === 'ready',
        isAiReady: actualStatus.status === 'ready',
      })
    } catch (error) {
      set({
        error: (error as Error).toString(),
        isActivating: false,
        modelStatus: { status: 'error', message: (error as Error).toString() },
      })
      throw error
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
      isDeleting: false,
      isActivating: false,
    })
  },
}))

