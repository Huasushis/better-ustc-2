import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { format } from 'date-fns'

export interface Activity {
  id: string
  name: string
  status_code: number
  valid_hour?: number | null
  apply_num?: number | null
  apply_limit?: number | null
  boolean_registration?: number | null
  need_sign_info_str?: string | null
  conceive?: string | null
  base_content?: string | null
  item_category?: string | null
  create_time_str?: string | null
  apply_start?: string | null
  apply_end?: string | null
  start_time?: string | null
  end_time?: string | null
  tel?: string | null
  raw?: any
  // children optional (series)
  children?: Activity[]
}

export const statusText = (code: number) => {
  const map: Record<number, string> = {
    26: '报名中',
    28: '报名结束',
    30: '学时公示',
    31: '追加学时',
    32: '公示结束',
    33: '学时申请',
    34: '学时通过',
    35: '学时驳回',
    40: '结项',
    [-3]: '异常结项',
  }
  return map[code] || '未知'
}

export const moduleDict: Record<string, string> = {
  d: '德',
  z: '智',
  t: '体',
  m: '美',
  l: '劳',
}

export const activityDeptName = (a: Activity) =>
  a.raw?.businessDeptId_dictText || a.raw?.businessDeptName || a.raw?.bussinessDeptName || ''

export const shortTime = (t?: string | null) => (t ? format(new Date(t), 'MM-dd HH:mm') : '')

export const useActivityStore = defineStore('activity', {
  state: () => ({
    recommended: [] as Activity[],
    all: [] as Activity[],
    registered: [] as Activity[],
    participated: [] as Activity[],
    detail: new Map<string, Activity>(),
    loadingAll: false,
    loadingRec: false,
    loadingMine: false,
    error: '',
    // front-end filters (类似 SCFilter，但仅本地过滤，不递归 children)
    filter: {
      keyword: '',
      modules: [] as string[], // d z t m l
      department: '', // 展示名
      organizer: '',
      startAfter: '',
      endBefore: '',
    },
  }),
  actions: {
    async fetchAll() {
      this.loadingAll = true
      this.error = ''
      try {
        const res = (await invoke('get_unended_activities')) as Activity[]
        this.all = res
      } catch (e: any) {
        this.error = e?.toString?.() || '获取活动失败'
      } finally {
        this.loadingAll = false
      }
    },
    async fetchRecommended() {
      this.loadingRec = true
      this.error = ''
      try {
        const res = (await invoke('get_recommended_activities')) as Activity[]
        this.recommended = res
      } catch (e: any) {
        this.error = e?.toString?.() || '获取推荐失败'
      } finally {
        this.loadingRec = false
      }
    },
    async fetchMine() {
      this.loadingMine = true
      this.error = ''
      try {
        const reg = (await invoke('get_registered_activities')) as Activity[]
        const part = (await invoke('get_participated_activities')) as Activity[]
        this.registered = reg as any
        this.participated = part as any
      } catch (e: any) {
        this.error = e?.toString?.() || '获取我的活动失败'
      } finally {
        this.loadingMine = false
      }
    },
    setFilter(partial: Partial<typeof this.filter>) {
      this.filter = { ...this.filter, ...partial }
    },
    clearFilter() {
      this.filter = { keyword: '', modules: [], department: '', organizer: '', startAfter: '', endBefore: '' }
    },
    async fetchDetail(id: string) {
      if (this.detail.has(id)) return this.detail.get(id)!
      try {
        const res = (await invoke('get_activity_detail', { activity_id: id })) as Activity
        if (res.item_category === '1') {
          const children = (await invoke('get_activity_children', { activity_id: id })) as Activity[]
          res.children = children
        }
        this.detail.set(id, res)
        return res
      } catch (e: any) {
        this.error = e?.toString?.() || '获取详情失败'
        throw e
      }
    },
    async refreshDetail(id: string) {
      try {
        const res = (await invoke('get_activity_detail', { activity_id: id })) as Activity
        if (res.item_category === '1') {
          const children = (await invoke('get_activity_children', { activity_id: id })) as Activity[]
          res.children = children
        }
        this.detail.set(id, res)
        return res
      } catch (e: any) {
        this.error = e?.toString?.() || '刷新详情失败'
        throw e
      }
    },
    async apply(id: string) {
      return invoke('register_for_activity', { activity_id: id }) as Promise<boolean>
    },
  },
})
