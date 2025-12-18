import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { useLogStore } from './logs'

export interface UserInfo {
  id: string
  name: string
  grade: string
  classes: string
  college?: string | null
  scientific_value?: number
  avatar?: string | null
}

interface LoginState {
  logged_in: boolean
  has_stored_creds: boolean
  username?: string | null
  user?: UserInfo | null
}

export const useUserStore = defineStore('user', {
  state: () => ({
    status: { logged_in: false, has_stored_creds: false, username: null, user: null } as LoginState,
    loading: false,
    error: '' as string,
  }),
  getters: {
    isLoggedIn: (state) => state.status.logged_in,
    displayName: (state) => state.status.user?.name || state.status.username || '未登录',
  },
  actions: {
    async fetchStatus() {
      this.loading = true
      this.error = ''
      try {
        const res = (await invoke('get_login_status')) as LoginState
        this.status = res
        useLogStore().add(`Fetch status: logged_in=${res.logged_in}, user=${res.user?.name}`)
      } catch (e: any) {
        this.error = e?.toString?.() || '获取状态失败'
        useLogStore().add(`Fetch status failed: ${this.error}`)
      } finally {
        this.loading = false
      }
    },
    async login(username: string, password: string, save: boolean) {
      this.loading = true
      this.error = ''
      useLogStore().add(`Attempting login for user: ${username}`)
      try {
        const user = (await invoke('login', { username, password, save })) as UserInfo
        this.status = { logged_in: true, has_stored_creds: save, username, user }
        useLogStore().add(`Login success: ${user.name}`)
        return true
      } catch (e: any) {
        this.error = e?.toString?.() || '登录失败'
        useLogStore().add(`Login failed: ${this.error}`)
        return false
      } finally {
        this.loading = false
      }
    },
    async logout() {
      useLogStore().add('Logging out')
      try {
        await invoke('logout')
        // Refresh status to get the saved username (if any)
        await this.fetchStatus()
      } catch (e) {
        console.error(e)
        useLogStore().add(`Logout error: ${e}`)
        this.status = { logged_in: false, has_stored_creds: false, username: null, user: null }
      }
    },
  },
})
