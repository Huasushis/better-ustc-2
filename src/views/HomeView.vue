<script setup lang="ts">
import { onMounted, ref, computed, onUnmounted } from 'vue'
import { NavBar, Collapse, CollapseItem, Empty, Loading, showToast, Search, DropdownMenu, DropdownItem, Button, showConfirmDialog, showLoadingToast, showSuccessToast, showFailToast, closeToast } from 'vant'
import ActivityCard from '../components/ActivityCard.vue'
import { useActivityStore, moduleDict, activityDeptName } from '../stores/activity'
import { useLogStore } from '../stores/logs'
import { useRouter } from 'vue-router'
import { requestPermission, sendNotification } from '@tauri-apps/plugin-notification'

const activityStore = useActivityStore()
const logStore = useLogStore()
const router = useRouter()
const activeNames = ref(['rec', 'all'])
const refreshing = ref(false)
const timerDaily = ref<number | null>(null)

// 本地更新活动的报名状态，避免整体刷新
const updateActivityRegistration = (id: string, registered: boolean) => {
  const updateList = (list: any[]) => {
    const item = list.find(a => a.id === id)
    if (item) {
      item.boolean_registration = registered ? 1 : 0
      if (registered && item.apply_num != null) {
        item.apply_num = (item.apply_num || 0) + 1
      } else if (!registered && item.apply_num != null && item.apply_num > 0) {
        item.apply_num = item.apply_num - 1
      }
    }
  }
  updateList(activityStore.recommended)
  updateList(activityStore.all)
}

const loadAll = async () => {
  refreshing.value = true
  await Promise.all([
    activityStore.fetchRecommended(),
    activityStore.fetchAll(),
    activityStore.fetchMine()
  ]).catch((e) =>
    showToast(e?.toString?.() || '加载失败')
  )
  refreshing.value = false
}

onMounted(loadAll)


const keyword = computed({
  get: () => activityStore.filter.keyword,
  set: (v: string) => activityStore.setFilter({ keyword: v }),
})
const moduleSelected = computed({
  get: () => activityStore.filter.modules,
  set: (v: string[]) => activityStore.setFilter({ modules: v }),
})
const organizerValue = computed({
  get: () => activityStore.filter.organizer,
  set: (v: string) => activityStore.setFilter({ organizer: v }),
})
const deptValue = computed({
  get: () => activityStore.filter.department,
  set: (v: string) => activityStore.setFilter({ department: v }),
})

const startAfter = computed({
  get: () => activityStore.filter.startAfter,
  set: (v: string) => activityStore.setFilter({ startAfter: v }),
})
const endBefore = computed({
  get: () => activityStore.filter.endBefore,
  set: (v: string) => activityStore.setFilter({ endBefore: v }),
})

const moduleOptions = computed(() => ['d', 'z', 't', 'm', 'l'])

const deptOptions = computed(() => {
  const set = new Set<string>()
  ;[...activityStore.all, ...activityStore.recommended].forEach((a) => {
    const name = activityDeptName(a)
    if (name) set.add(name)
  })
  return Array.from(set)
})

const filtered = (list: any[]) => {
  const kw = keyword.value.trim().toLowerCase()
  const mods = moduleSelected.value
  const dept = deptValue.value.toLowerCase()
  const org = organizerValue.value.trim().toLowerCase()
  const start = startAfter.value ? new Date(startAfter.value).getTime() : 0
  const end = endBefore.value ? new Date(endBefore.value).getTime() : Number.MAX_SAFE_INTEGER

  return list.filter((a) => {
    if (kw) {
      const txt = `${a.name} ${a.organizer_dictText || ''} ${a.placeInfo || ''}`.toLowerCase()
      if (!txt.includes(kw)) return false
    }
    if (org) {
      const o = `${a.organizer_dictText || a.businessDeptName || ''}`.toLowerCase()
      if (!o.includes(org)) return false
    }
    if (mods && mods.length) {
      const m = a.module
      if (!m || !mods.includes(m)) return false
    }
    if (dept) {
      const name = activityDeptName(a).toLowerCase()
      if (!name.includes(dept)) return false
    }
    const st = a.start_time ? new Date(a.start_time).getTime() : 0
    const et = a.end_time ? new Date(a.end_time).getTime() : st
    if (start && et < start) return false
    if (end !== Number.MAX_SAFE_INTEGER && st > end) return false
    return true
  })
}

const toggleModule = (code: string) => {
  const set = new Set(moduleSelected.value)
  if (set.has(code)) set.delete(code)
  else set.add(code)
  moduleSelected.value = Array.from(set)
}

const filteredRec = computed(() => filtered(activityStore.recommended))
const filteredAll = computed(() => filtered(activityStore.all))

const goDetail = (id: string) => {
  logStore.add(`点击进入活动详情: ${id}`)
  router.push({ name: 'activity-detail', params: { id }, query: { from: 'home' } })
}

const onApply = async (id: string, autoCancel: boolean = false) => {
  logStore.add(`点击报名活动: ${id}, autoCancel=${autoCancel}`)
  showLoadingToast({ message: '正在尝试报名...', duration: 0, forbidClick: true })
  try {
    const result = await activityStore.apply(id, autoCancel)
    if (result === true) {
      closeToast()
      showSuccessToast('报名成功')
      // 更新本地活动状态，避免整体刷新
      updateActivityRegistration(id, true)
      // 刷新已报名列表
      activityStore.fetchMine()
    } else if (typeof result === 'string') {
      // 检查是否是时间冲突错误
      if (result.includes('时间冲突') || result.includes('冲突')) {
        closeToast()
        try {
          await showConfirmDialog({
            title: '时间冲突',
            message: '该活动与已报名活动时间冲突，是否自动取消冲突活动并重新报名？',
            confirmButtonText: '确定',
            cancelButtonText: '取消',
          })
          // 用户选择确定，使用autoCancel=true重试
          await onApply(id, true)
        } catch {
          // 用户取消
          showFailToast('已取消报名')
        }
      } else {
        closeToast()
        showFailToast(result || '报名失败')
      }
    } else {
      closeToast()
      showFailToast('报名失败或名额已满')
    }
  } catch (e: any) {
    closeToast()
    showFailToast(e?.toString?.() || '报名失败')
  }
}

const onCancel = async (id: string) => {
  logStore.add(`点击取消报名活动: ${id}`)
  try {
    await showConfirmDialog({
      title: '确认取消',
      message: '确定要取消报名该活动吗？',
      confirmButtonText: '确定取消',
      cancelButtonText: '返回',
    })
    showLoadingToast({ message: '取消中...', duration: 0, forbidClick: true })
    const ok = await activityStore.cancelApply(id)
    if (ok) {
      logStore.add(`取消报名成功: ${id}`)
      closeToast()
      showSuccessToast('取消成功')
      // 更新本地活动状态，避免整体刷新
      updateActivityRegistration(id, false)
      // 刷新已报名列表
      activityStore.fetchMine()
    } else {
      logStore.add(`取消报名失败: ${id}`)
      closeToast()
      showFailToast('取消失败')
    }
  } catch (e: any) {
    closeToast()
    if (e !== 'cancel') {
      logStore.add(`取消报名异常: ${e?.toString?.()}`)
      showFailToast(e?.toString?.() || '取消失败')
    }
  }
}

// 定时每日刷新（应用运行期间），并提醒
const scheduleDaily = async () => {
  const perm = await requestPermission()
  const doRefresh = async () => {
    await loadAll()
    if (perm === 'granted') {
      sendNotification({ title: '二课更新', body: '已自动刷新今日活动列表' })
    }
  }
  const now = new Date()
  const nextMidnight = new Date(now)
  nextMidnight.setHours(24, 0, 0, 0)
  const ms = nextMidnight.getTime() - now.getTime()
  setTimeout(() => {
    doRefresh()
    timerDaily.value = window.setInterval(doRefresh, 24 * 60 * 60 * 1000)
  }, ms)
}

onMounted(scheduleDaily)
onUnmounted(() => {
  if (timerDaily.value) window.clearInterval(timerDaily.value)
})
</script>

<template>
  <div class="h-full flex flex-col bg-[#f7f8fa]">
    <NavBar title="首页" fixed placeholder safe-area-inset-top :z-index="100">
      <template #right>
        <Button size="small" :loading="refreshing" @click="loadAll" icon="replay">刷新</Button>
      </template>
    </NavBar>
    <div class="flex-1 overflow-y-auto">
      <div class="px-3 py-3 space-y-3 min-h-full">
        <div class="bg-white rounded-lg p-3 shadow-sm space-y-2">
          <Search v-model="keyword" placeholder="搜索名称/主办方/地点" />

          <div class="text-[13px] text-gray-700">模块筛选（多选）</div>
          <div class="flex flex-wrap gap-2">
            <Button
              v-for="code in moduleOptions"
              :key="code"
              size="small"
              :type="moduleSelected.includes(code) ? 'primary' : 'default'"
              plain
              @click="toggleModule(code)"
            >{{ moduleDict[code] }}</Button>
          </div>

          <DropdownMenu>
            <DropdownItem v-model="deptValue" :options="[{ text: '全部部门', value: '' }, ...deptOptions.map((d) => ({ text: d, value: d }))]" title="主办方" />
          </DropdownMenu>

          <div class="grid grid-cols-2 gap-2 text-[12px] text-gray-600">
            <input type="datetime-local" class="van-field__control border rounded px-2 py-1" v-model="startAfter" />
            <input type="datetime-local" class="van-field__control border rounded px-2 py-1" v-model="endBefore" />
          </div>
          <div class="flex gap-2">
            <input type="text" class="van-field__control border rounded px-2 py-1 flex-1 text-[12px]" v-model="organizerValue" placeholder="主办方关键词" />
            <Button size="small" type="primary" plain @click="activityStore.clearFilter">清空</Button>
          </div>
        </div>

        <Collapse v-model="activeNames">
          <CollapseItem title="推荐活动" name="rec">
            <div v-if="activityStore.loadingRec" class="py-4 text-center text-gray-500"><Loading size="24" vertical>加载中</Loading></div>
            <template v-else>
              <Empty v-if="!filteredRec.length" description="暂无推荐" />
              <ActivityCard
                v-for="item in filteredRec"
                :key="item.id"
                :activity="item"
                :show-apply="true"
                @detail="() => goDetail(item.id)"
                @apply="() => onApply(item.id)"
                @cancel="() => onCancel(item.id)"
              />
            </template>
          </CollapseItem>
          <CollapseItem title="全部未结束活动" name="all">
            <div v-if="activityStore.loadingAll" class="py-4 text-center text-gray-500"><Loading size="24" vertical>加载中</Loading></div>
            <template v-else>
              <Empty v-if="!filteredAll.length" description="暂无数据" />
              <ActivityCard
                v-for="item in filteredAll"
                :key="item.id"
                :activity="item"
                :show-apply="true"
                @detail="() => goDetail(item.id)"
                @apply="() => onApply(item.id)"
                @cancel="() => onCancel(item.id)"
              />
            </template>
          </CollapseItem>
        </Collapse>
        <div class="mt-3 text-center text-[12px] text-gray-500">点击右上角刷新 · 数据来自二课平台</div>
      </div>
    </div>
  </div>
</template>
