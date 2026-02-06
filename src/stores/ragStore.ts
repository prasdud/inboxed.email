import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

export interface EmbeddingStatus {
    is_embedding: boolean
    total_emails: number
    embedded_emails: number
    current_model: string | null
    last_embedded_at: number | null
    error_message: string | null
}

export interface SearchResult {
    email_id: string
    similarity: number
    subject: string | null
    from: string | null
    snippet: string | null
}

export interface EmbeddingProgress {
    total: number
    embedded: number
    current_email_id: string | null
}

interface RagStore {
    // State
    isInitialized: boolean
    isEmbedding: boolean
    embeddingProgress: EmbeddingProgress | null
    embeddingStatus: EmbeddingStatus | null
    searchResults: SearchResult[]
    error: string | null

    // Actions
    initRag: () => Promise<boolean>
    checkRagReady: () => Promise<boolean>
    getEmbeddingStatus: () => Promise<void>
    embedAllEmails: () => Promise<number>
    embedEmail: (emailId: string, subject: string, from: string, body: string) => Promise<void>
    searchSemantic: (query: string, limit?: number) => Promise<SearchResult[]>
    findSimilarEmails: (emailId: string, limit?: number) => Promise<SearchResult[]>
    getEmbeddedCount: () => Promise<number>
    clearEmbeddings: () => Promise<void>
    chatWithContext: (query: string, limit?: number) => Promise<string>
    reset: () => void
}

export const useRagStore = create<RagStore>((set, get) => ({
    isInitialized: false,
    isEmbedding: false,
    embeddingProgress: null,
    embeddingStatus: null,
    searchResults: [],
    error: null,

    initRag: async () => {
        try {
            set({ error: null })
            const success = await invoke<boolean>('init_rag')
            set({ isInitialized: success })

            if (success) {
                await get().getEmbeddingStatus()
            }

            return success
        } catch (error) {
            set({ error: (error as Error).toString() })
            return false
        }
    },

    checkRagReady: async () => {
        try {
            const ready = await invoke<boolean>('is_rag_ready')
            set({ isInitialized: ready })
            return ready
        } catch (error) {
            console.error('Failed to check RAG status:', error)
            return false
        }
    },

    getEmbeddingStatus: async () => {
        try {
            const status = await invoke<EmbeddingStatus>('get_embedding_status')
            set({
                embeddingStatus: status,
                isEmbedding: status.is_embedding,
            })
        } catch (error) {
            console.error('Failed to get embedding status:', error)
        }
    },

    embedAllEmails: async () => {
        let progressUnlisten: UnlistenFn | null = null
        let completeUnlisten: UnlistenFn | null = null

        try {
            set({ isEmbedding: true, error: null })

            // Listen for progress events
            progressUnlisten = await listen<EmbeddingProgress>('embedding:progress', (event) => {
                set({ embeddingProgress: event.payload })
            })

            // Listen for completion
            completeUnlisten = await listen<number>('embedding:complete', (event) => {
                set({
                    isEmbedding: false,
                    embeddingProgress: null,
                })
            })

            const count = await invoke<number>('embed_all_emails')

            // Refresh status
            await get().getEmbeddingStatus()

            return count
        } catch (error) {
            set({
                error: (error as Error).toString(),
                isEmbedding: false,
            })
            throw error
        } finally {
            if (progressUnlisten) progressUnlisten()
            if (completeUnlisten) completeUnlisten()
        }
    },

    embedEmail: async (emailId: string, subject: string, from: string, body: string) => {
        try {
            await invoke('embed_email', { emailId, subject, from, body })
        } catch (error) {
            console.error('Failed to embed email:', error)
        }
    },

    searchSemantic: async (query: string, limit = 10) => {
        try {
            set({ error: null })
            const results = await invoke<SearchResult[]>('search_emails_semantic', { query, limit })
            set({ searchResults: results })
            return results
        } catch (error) {
            set({ error: (error as Error).toString() })
            return []
        }
    },

    findSimilarEmails: async (emailId: string, limit = 5) => {
        try {
            set({ error: null })
            const results = await invoke<SearchResult[]>('find_similar_emails', { emailId, limit })
            set({ searchResults: results })
            return results
        } catch (error) {
            console.error('Failed to find similar emails:', error)
            return []
        }
    },

    getEmbeddedCount: async () => {
        try {
            return await invoke<number>('get_embedded_count')
        } catch (error) {
            console.error('Failed to get embedded count:', error)
            return 0
        }
    },

    clearEmbeddings: async () => {
        try {
            await invoke('clear_embeddings')
            await get().getEmbeddingStatus()
        } catch (error) {
            console.error('Failed to clear embeddings:', error)
            throw error
        }
    },

    chatWithContext: async (query: string, limit = 5) => {
        try {
            set({ error: null })
            return await invoke<string>('chat_with_context', { query, limit })
        } catch (error) {
            set({ error: (error as Error).toString() })
            return `Error: ${(error as Error).toString()}`
        }
    },

    reset: () => {
        set({
            isInitialized: false,
            isEmbedding: false,
            embeddingProgress: null,
            embeddingStatus: null,
            searchResults: [],
            error: null,
        })
    },
}))
