import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

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

  fetchSmartInbox: async (limit = 50, offset = 0) => {
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

  getEmailsByCategory: async (category: string, limit = 50) => {
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

  searchEmails: async (query: string, limit = 50) => {
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

  startIndexing: async (maxEmails = 100) => {
    try {
      set({ error: null })
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
    const completeUnlisten = await listen('indexing:complete', () => {
      set({ indexingProgress: 100 })
      get().getIndexingStatus()
      get().fetchSmartInbox()
    })
    unlisteners.push(completeUnlisten)

    // Return cleanup function
    return () => {
      unlisteners.forEach((fn) => fn())
    }
  },
}))
