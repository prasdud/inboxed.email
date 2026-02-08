import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'

interface AuthStatus {
  authenticated: boolean
  email: string | null
}

interface AuthStore {
  authenticated: boolean
  email: string | null
  loading: boolean
  error: string | null
  checkAuth: () => Promise<void>
  signIn: () => Promise<void>
  signOut: () => Promise<void>
}

export const useAuthStore = create<AuthStore>((set) => ({
  authenticated: false,
  email: null,
  loading: false,
  error: null,

  checkAuth: async () => {
    try {
      set({ loading: true, error: null })
      const status = await invoke<AuthStatus>('check_auth_status')
      set({
        authenticated: status.authenticated,
        email: status.email,
        loading: false,
      })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },

  signIn: async () => {
    try {
      console.log('Sign in clicked')
      set({ loading: true, error: null })

      // Start OAuth flow and get authorization URL
      console.log('Calling start_auth...')
      const authUrl = await invoke<string>('start_auth', { provider: null, accountId: null })
      console.log('Auth URL:', authUrl)

      // Open the URL in the default browser
      console.log('Opening browser...')
      await open(authUrl)

      // Wait for the OAuth callback to complete
      console.log('Waiting for callback...')
      await invoke('complete_auth')
      console.log('OAuth complete!')

      // Check auth status after completion
      const status = await invoke<AuthStatus>('check_auth_status')
      set({
        authenticated: status.authenticated,
        email: status.email,
        loading: false,
      })
    } catch (error) {
      console.error('Sign in error:', error)
      set({
        error: (error as Error).toString(),
        loading: false,
        authenticated: false,
      })
    }
  },

  signOut: async () => {
    try {
      set({ loading: true, error: null })
      await invoke('sign_out')
      set({ authenticated: false, email: null, loading: false })
    } catch (error) {
      set({ error: (error as Error).toString(), loading: false })
    }
  },
}))
