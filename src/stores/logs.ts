import { defineStore } from 'pinia'
import { Store } from '@tauri-apps/plugin-store'
import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs'

let storePromise: Promise<Store> | null = null
const getStore = () => {
  if (!storePromise) storePromise = Store.load('logs.json')
  return storePromise
}

export const useLogStore = defineStore('logs', {
  state: () => ({
    entries: [] as string[],
    devMode: false,
  }),
  actions: {
    async load() {
      try {
        const store = await getStore()
        const savedEntries = await store.get<string[]>('entries')
        const savedDev = await store.get<boolean>('devMode')
        this.entries = savedEntries ?? []
        this.devMode = savedDev ?? false
      } catch (_) {
        this.entries = []
      }
    },
    async add(message: string) {
      const now = new Date()
      const localTime = now.toLocaleString('zh-CN', { 
        year: 'numeric', 
        month: '2-digit', 
        day: '2-digit', 
        hour: '2-digit', 
        minute: '2-digit', 
        second: '2-digit',
        hour12: false 
      })
      const line = `[${localTime}] ${message}`
      this.entries.push(line)
      try {
        const store = await getStore()
        await store.set('entries', this.entries)
        await store.save()
      } catch (e) {
        console.error('write log failed', e)
      }
    },
    enableDevMode() {
      this.devMode = true
      getStore()
        .then(async (store) => {
          await store.set('devMode', true)
          await store.save()
        })
        .catch((e: unknown) => console.error('persist devMode failed', e))
      this.add('开发者模式开启')
    },
    async clear() {
      this.entries = []
      try {
        const store = await getStore()
        await store.set('entries', [])
        await store.save()
      } catch (e) {
        console.error('clear log failed', e)
      }
    },
    async saveToFile() {
      const content = this.entries.join('\n')
      const filename = `better-ustc-logs-${Date.now()}.txt`
      await writeTextFile(filename, content, { baseDir: BaseDirectory.Download })
    },
  },
})
