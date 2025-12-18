import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { format } from 'date-fns'
import { useLogStore } from './logs'

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
  // Flattened fields from backend
  pic?: string
  placeInfo?: string
  organizer_dictText?: string
  businessDeptName?: string
  bussinessDeptName?: string
  businessDeptId_dictText?: string
  module?: string
  [key: string]: any
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
  a.businessDeptId_dictText || a.businessDeptName || a.bussinessDeptName || ''

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
      useLogStore().add('Fetching all activities')
      try {
        const res = (await invoke('get_unended_activities')) as Activity[]
        this.all = res
        useLogStore().add(`Fetched ${res.length} activities`)
      } catch (e: any) {
        this.error = e?.toString?.() || '获取活动失败'
        useLogStore().add(`Fetch all failed: ${this.error}`)
      } finally {
        this.loadingAll = false
      }
    },
    async fetchRecommended() {
      this.loadingRec = true
      this.error = ''
      useLogStore().add('Fetching recommended activities')
      try {
        const res = (await invoke('get_recommended_activities')) as Activity[]
        this.recommended = res
        useLogStore().add(`Fetched ${res.length} recommended activities`)
      } catch (e: any) {
        this.error = e?.toString?.() || '获取推荐失败'
        useLogStore().add(`Fetch recommended failed: ${this.error}`)
      } finally {
        this.loadingRec = false
      }
    },
    async fetchMine() {
      this.loadingMine = true
      this.error = ''
      useLogStore().add('Fetching my activities')
      try {
        const reg = (await invoke('get_registered_activities')) as Activity[]
        const part = (await invoke('get_participated_activities')) as Activity[]
        this.registered = reg as any
        this.participated = part as any
        useLogStore().add(`Fetched ${reg.length} registered, ${part.length} participated`)
      } catch (e: any) {
        this.error = e?.toString?.() || '获取我的活动失败'
        useLogStore().add(`Fetch mine failed: ${this.error}`)
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
      useLogStore().add(`Fetching detail for ${id}`)
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
        useLogStore().add(`Fetch detail failed: ${this.error}`)
        throw e
      }
    },
    async refreshDetail(id: string) {
      useLogStore().add(`Refreshing detail for ${id}`)
      try {
        useLogStore().add(`Calling get_activity_detail with activity_id: ${id}`)
        const res = (await invoke('get_activity_detail', { activity_id: id })) as Activity
        useLogStore().add(`get_activity_detail response: ${JSON.stringify(res).slice(0, 800)}`)
        if (res.item_category === '1') {
          useLogStore().add(`Activity is series, fetching children`)
          const children = (await invoke('get_activity_children', { activity_id: id })) as Activity[]
          res.children = children
          useLogStore().add(`Fetched ${children.length} children`)
        }
        this.detail.set(id, res)
        return res
      } catch (e: any) {
        this.error = e?.toString?.() || '刷新详情失败'
        useLogStore().add(`Refresh detail failed: ${this.error}`)
        throw e
      }
    },
    async apply(id: string, autoCancel: boolean = false): Promise<boolean | string> {
      useLogStore().add(`Applying for activity ${id}, autoCancel=${autoCancel}`)
      try {
        const res = await invoke('register_for_activity', { activity_id: id, auto_cancel: autoCancel }) as boolean
        useLogStore().add(`Apply result: ${res}`)
        return res
      } catch (e: any) {
        const errStr = e?.toString?.() || ''
        useLogStore().add(`Apply failed: ${errStr}`)
        // 返回错误字符串而不是throw，让调用方判断是否是时间冲突
        return errStr
      }
    },
    async cancelApply(id: string) {
      useLogStore().add(`Canceling activity ${id}`)
      try {
        const res = await invoke('cancel_activity', { activity_id: id }) as boolean
        useLogStore().add(`Cancel result: ${res}`)
        return res
      } catch (e: any) {
        useLogStore().add(`Cancel failed: ${e}`)
        throw e
      }
    },
  },
})
