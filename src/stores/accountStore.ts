import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface Account {
  id: string
  email: string
  display_name: string
  provider: string
  imap_host: string
  imap_port: number
  smtp_host: string
  smtp_port: number
  auth_type: string
  is_active: boolean
  created_at: number
  last_synced_at: number | null
}

interface AccountStore {
  accounts: Account[]
  activeAccountId: string | null
  loading: boolean
  error: string | null

  fetchAccounts: () => Promise<void>
  addAccount: (params: {
    email: string
    displayName: string
    provider: string
    authType: string
    imapHost?: string
    imapPort?: number
    smtpHost?: string
    smtpPort?: number
  }) => Promise<Account>
  removeAccount: (accountId: string) => Promise<void>
  setActiveAccount: (accountId: string) => Promise<void>
  connectAccount: (accountId: string) => Promise<void>
}

export const useAccountStore = create<AccountStore>((set, get) => ({
  accounts: [],
  activeAccountId: null,
  loading: false,
  error: null,

  fetchAccounts: async () => {
    try {
      set({ loading: true, error: null })
      const accounts = await invoke<Account[]>('list_accounts')
      const active = accounts.find((a) => a.is_active)
      set({
        accounts,
        activeAccountId: active?.id || null,
        loading: false,
      })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },

  addAccount: async (params) => {
    try {
      set({ loading: true, error: null })
      const account = await invoke<Account>('add_account', {
        email: params.email,
        displayName: params.displayName,
        provider: params.provider,
        authType: params.authType,
        imapHost: params.imapHost,
        imapPort: params.imapPort,
        smtpHost: params.smtpHost,
        smtpPort: params.smtpPort,
      })

      await get().fetchAccounts()
      set({ loading: false })
      return account
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
      throw error
    }
  },

  removeAccount: async (accountId: string) => {
    try {
      set({ loading: true, error: null })
      await invoke('remove_account', { accountId })
      await get().fetchAccounts()
      set({ loading: false })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },

  setActiveAccount: async (accountId: string) => {
    try {
      await invoke('set_active_account', { accountId })
      set({ activeAccountId: accountId })
      await get().fetchAccounts()
    } catch (error) {
      set({ error: (error as Error).toString() })
    }
  },

  connectAccount: async (accountId: string) => {
    try {
      set({ error: null })
      await invoke('connect_account', { accountId })
    } catch (error) {
      set({ error: (error as Error).toString() })
      throw error
    }
  },
}))
