import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useAiStore } from './aiStore'
import { useRagStore } from './ragStore'

export interface EmailWithInsight {
  id: string
  thread_id: string
  subject: string
  from_name: string
  from_email: string
  to_emails: string[]
  date: number
  snippet: string
  is_read: boolean
  is_starred: boolean
  has_attachments: boolean
  priority: string
  priority_score: number
  category: string | null
  summary: string | null
}

export interface IndexingStatus {
  is_indexing: boolean
  total_emails: number
  processed_emails: number
  last_indexed_at: number | null
  error_message: string | null
}

interface SmartInboxStore {
  emails: EmailWithInsight[]
  loading: boolean
  error: string | null
  indexingStatus: IndexingStatus | null
  indexingProgress: number

  // Actions
  fetchSmartInbox: (limit?: number, offset?: number) => Promise<void>
  getEmailsByCategory: (category: string, limit?: number) => Promise<void>
  searchEmails: (query: string, limit?: number) => Promise<void>
  getIndexingStatus: () => Promise<void>
  resetIndexingStatus: () => Promise<void>
  startIndexing: (maxEmails?: number) => Promise<void>
  initDatabase: () => Promise<void>
  setupIndexingListeners: () => Promise<() => void>
}

export const useSmartInboxStore = create<SmartInboxStore>((set, get) => ({
  emails: [],
  loading: false,
  error: null,
  indexingStatus: null,
  indexingProgress: 0,

  initDatabase: async () => {
    try {
      await invoke('init_database')
    } catch (error) {
      console.error('Failed to initialize database:', error)
      set({ error: (error as Error).toString() })
    }
  },

  fetchSmartInbox: async (limit = 500, offset = 0) => {
    try {
      set({ loading: true, error: null })
      const emails = await invoke<EmailWithInsight[]>('get_smart_inbox', {
        limit,
        offset,
      })
      set({ emails, loading: false })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },

  getEmailsByCategory: async (category: string, limit = 500) => {
    try {
      set({ loading: true, error: null })
      const emails = await invoke<EmailWithInsight[]>('get_emails_by_category', {
        category,
        limit,
      })
      set({ emails, loading: false })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },

  searchEmails: async (query: string, limit = 500) => {
    try {
      set({ loading: true, error: null })
      const emails = await invoke<EmailWithInsight[]>('search_smart_emails', {
        query,
        limit,
      })
      set({ emails, loading: false })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },

  getIndexingStatus: async () => {
    try {
      const status = await invoke<IndexingStatus>('get_indexing_status')
      set({ indexingStatus: status })
    } catch (error) {
      console.error('Failed to get indexing status:', error)
    }
  },

  resetIndexingStatus: async () => {
    try {
      await invoke('reset_indexing_status')
      await get().getIndexingStatus()
    } catch (error) {
      console.error('Failed to reset indexing status:', error)
    }
  },

  startIndexing: async (maxEmails = 100) => {
    try {
      set({ error: null })

      // Try to ensure AI is initialized before indexing, but don't block on failure
      try {
        const aiStore = useAiStore.getState()
        if (!aiStore.isAiReady) {
          console.log('[SmartInbox] Initializing AI before indexing...')
          await aiStore.initAi()
        }
        await aiStore.checkModelStatus()
        console.log('[SmartInbox] Model status:', aiStore.modelStatus)
      } catch (aiError) {
        console.warn('[SmartInbox] AI init failed, indexing will use fallback:', aiError)
      }

      await invoke('start_email_indexing', { maxEmails })
    } catch (error) {
      set({ error: (error as Error).toString() })
      throw error
    }
  },

  setupIndexingListeners: async () => {
    const unlisteners: UnlistenFn[] = []

    // Listen for indexing started
    const startedUnlisten = await listen('indexing:started', () => {
      set({ indexingProgress: 0 })
      get().getIndexingStatus()
    })
    unlisteners.push(startedUnlisten)

    // Listen for progress
    const progressUnlisten = await listen<number>('indexing:progress', (event) => {
      set({ indexingProgress: event.payload })
    })
    unlisteners.push(progressUnlisten)

    // Listen for completion
    const completeUnlisten = await listen('indexing:complete', async () => {
      set({ indexingProgress: 100 })
      try {
        await get().getIndexingStatus()
      } catch (e) {
        console.error('[SmartInbox] Failed to get indexing status after complete:', e)
      }
      try {
        await get().fetchSmartInbox()
      } catch (e) {
        console.error('[SmartInbox] Failed to fetch smart inbox after complete:', e)
      }

      // Auto-embed after indexing completes if RAG is initialized
      try {
        const ragStore = useRagStore.getState()
        if (ragStore.isInitialized) {
          await ragStore.embedAllEmails()
        }
      } catch (e) {
        console.error('[SmartInbox] Failed to auto-embed after indexing:', e)
      }
    })
    unlisteners.push(completeUnlisten)

    // Listen for errors
    const errorUnlisten = await listen<string>('indexing:error', (event) => {
      set({ error: `Indexing failed: ${event.payload}` })
      get().getIndexingStatus()
    })
    unlisteners.push(errorUnlisten)

    // Return cleanup function
    return () => {
      unlisteners.forEach((fn) => fn())
    }
  },
}))
