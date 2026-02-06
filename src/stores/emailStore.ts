import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface EmailListItem {
  id: string
  thread_id: string
  subject: string
  from: string
  from_email: string
  date: string
  snippet: string
  is_read: boolean
  is_starred: boolean
  has_attachments: boolean
}

export interface Email extends EmailListItem {
  to: string[]
  body_html: string | null
  body_plain: string | null
  labels: string[]
}

interface EmailStore {
  emails: EmailListItem[]
  selectedEmail: Email | null
  loading: boolean
  error: string | null
  fetchEmails: (maxResults?: number, query?: string, forceRefresh?: boolean) => Promise<void>
  selectEmail: (emailId: string) => Promise<void>
  clearSelection: () => void
}

export const useEmailStore = create<EmailStore>((set) => ({
  emails: [],
  selectedEmail: null,
  loading: false,
  error: null,

  fetchEmails: async (maxResults = 50, query, forceRefresh = false) => {
    try {
      set({ loading: true, error: null })
      const emails = await invoke<EmailListItem[]>('fetch_emails', {
        maxResults,
        query,
        forceRefresh,
      })
      set({ emails, loading: false })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },

  selectEmail: async (emailId: string) => {
    try {
      set({ loading: true, error: null })
      const email = await invoke<Email>('get_email', { emailId })
      set({ selectedEmail: email, loading: false })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },

  clearSelection: () => {
    set({ selectedEmail: null })
  },
}))
