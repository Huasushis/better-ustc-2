<script setup lang="ts">
import { ref, onUnmounted, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { NavBar, Tag, Cell, CellGroup, Button, Loading, showNotify, showConfirmDialog, showLoadingToast, showSuccessToast, showFailToast, closeToast } from 'vant'
import { useActivityStore, statusText, shortTime } from '../stores/activity'
import { useLogStore } from '../stores/logs'
import ActivityCard from '../components/ActivityCard.vue'
import { requestPermission, sendNotification } from '@tauri-apps/plugin-notification'

const route = useRoute()
const store = useActivityStore()
const logStore = useLogStore()

// 使用 computed 监听路由参数变化
const currentId = computed(() => route.params.id as string)
const detail = ref<any>(null)
const loading = ref(true)

// 判断是否已报名
const isRegistered = computed(() => detail.value?.boolean_registration === 1)
// 判断是否是系列活动
const isSeries = computed(() => detail.value?.item_category === '1')

const load = async (id: string) => {
  loading.value = true
  logStore.add(`开始加载活动详情: ${id}`)
  try {
    detail.value = await store.refreshDetail(id)
    logStore.add(`活动详情加载成功: ${JSON.stringify(detail.value).slice(0, 500)}...`)
  } catch (e: any) {
    logStore.add(`活动详情加载失败: ${e?.toString?.() || '未知错误'}`)
    showFailToast(e?.toString?.() || '加载失败')
  } finally {
    loading.value = false
  }
}

// 监听路由参数变化，重新加载数据
watch(currentId, (newId) => {
  if (newId) {
    load(newId)
  }
}, { immediate: true })

const onApply = async (autoCancel: boolean = false) => {
  if (!detail.value) return
  logStore.add(`点击报名按钮: ${detail.value.id}, autoCancel=${autoCancel}`)
  showLoadingToast({ message: '报名中...', duration: 0, forbidClick: true })
  try {
    const result = await store.apply(detail.value.id, autoCancel)
    if (result === true) {
      logStore.add(`报名成功: ${detail.value.id}`)
      closeToast()
      showSuccessToast('报名成功')
      await scheduleNotification(detail.value)
      // 更新本地状态
      store.updateRegistrationStatus(detail.value.id, true)
      // 更新当前详情
      detail.value = { ...detail.value, boolean_registration: 1, apply_num: (detail.value.apply_num || 0) + 1 }
      // 刷新已报名列表
      store.fetchMine()
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
          await onApply(true)
        } catch {
          // 用户取消
          showFailToast('已取消报名')
        }
      } else {
        logStore.add(`报名失败: ${result}`)
        closeToast()
        showFailToast(result || '报名失败')
      }
    } else {
      logStore.add(`报名失败/名额已满: ${detail.value.id}`)
      closeToast()
      showFailToast('报名失败/名额已满')
    }
  } catch (e: any) {
    logStore.add(`报名异常: ${e?.toString?.()}`)
    closeToast()
    showFailToast(e?.toString?.() || '报名失败')
  }
}

const onCancelApply = async () => {
  if (!detail.value) return
  logStore.add(`点击取消报名按钮: ${detail.value.id}`)
  try {
    await showConfirmDialog({
      title: '确认取消',
      message: '确定要取消报名该活动吗？',
      confirmButtonText: '确定取消',
      cancelButtonText: '返回',
    })
    showLoadingToast({ message: '取消中...', duration: 0, forbidClick: true })
    const ok = await store.cancelApply(detail.value.id)
    if (ok) {
      logStore.add(`取消报名成功: ${detail.value.id}`)
      closeToast()
      showSuccessToast('取消成功')
      // 更新本地状态
      store.updateRegistrationStatus(detail.value.id, false)
      // 更新当前详情
      detail.value = { ...detail.value, boolean_registration: 0, apply_num: Math.max(0, (detail.value.apply_num || 1) - 1) }
      // 刷新已报名列表
      store.fetchMine()
    } else {
      logStore.add(`取消报名失败: ${detail.value.id}`)
      closeToast()
      showFailToast('取消失败')
    }
  } catch (e: any) {
    if (e !== 'cancel') {
      logStore.add(`取消报名异常: ${e?.toString?.()}`)
      closeToast()
      showFailToast(e?.toString?.() || '取消失败')
    }
  }
}

const scheduleNotification = async (act: any) => {
  if (!act.start_time) return
  const permission = await requestPermission()
  if (permission === 'granted') {
    const start = new Date(act.start_time).getTime()
    const fireAt = start - 20 * 60 * 1000
    if (fireAt > Date.now()) {
      const delay = fireAt - Date.now()
      setTimeout(() => {
        sendNotification({ title: '活动即将开始', body: `${act.name} 将于20分钟后开始` })
      }, delay)
      showNotify({ type: 'success', message: '已设置开场前20分钟提醒' })
    }
  }
}

const autoApplyLoading = ref(false)
const autoTimer = ref<number | null>(null)
const autoApply = async () => {
  if (!detail.value || autoTimer.value) return
  autoApplyLoading.value = true
  showLoadingToast({ message: '监控名额中...', duration: 0, forbidClick: true })
  let attempts = 0
  autoTimer.value = window.setInterval(async () => {
    attempts += 1
    try {
      const latest = await store.refreshDetail(detail.value!.id)
      detail.value = latest // 更新详情
      if (latest.apply_limit && (latest.apply_num || 0) < latest.apply_limit && latest.status_code === 26) {
        const result = await store.apply(latest.id)
        if (result === true) {
          closeToast()
          showSuccessToast('抢到名额，已报名')
          await scheduleNotification(latest)
          // 更新本地状态
          store.updateRegistrationStatus(latest.id, true)
          detail.value = { ...latest, boolean_registration: 1, apply_num: (latest.apply_num || 0) + 1 }
          store.fetchMine()
          stopAuto()
          return
        }
      }
    } catch (e) {
      console.error(e)
    }
    if (attempts >= 20) {
      closeToast()
      showFailToast('监控结束，仍未抢到')
      stopAuto()
    }
  }, 30_000)
}

const stopAuto = () => {
  if (autoTimer.value) {
    clearInterval(autoTimer.value)
    autoTimer.value = null
  }
  autoApplyLoading.value = false
  // 不在这里关闭toast，因为可能会覆盖成功/失败提示
}

onUnmounted(stopAuto)
</script>

<template>
  <div class="min-h-screen bg-[#f7f8fa] pb-4">
    <NavBar title="活动详情" left-arrow @click-left="$router.back()" fixed placeholder safe-area-inset-top />
    <div class="px-3 pt-3">
      <div v-if="loading" class="py-10 text-center text-gray-500"><Loading size="24" vertical>加载中</Loading></div>
      <template v-else-if="detail">
        <div class="bg-white rounded-lg p-4 shadow-sm">
          <div class="flex items-start gap-3">
            <img :src="detail.pic ? `https://young.ustc.edu.cn/login/${detail.pic}` : 'https://via.placeholder.com/120'" class="w-24 h-24 rounded-md object-cover" />
            <div class="flex-1 min-w-0">
              <div class="text-lg font-semibold leading-tight">{{ detail.name }}</div>
              <div class="mt-2 flex gap-2 flex-wrap">
                <Tag type="primary" plain>{{ statusText(detail.status_code) }}</Tag>
                <Tag v-if="detail.apply_limit" plain type="warning">{{ detail.apply_num || 0 }}/{{ detail.apply_limit }}</Tag>
                <Tag v-if="detail.valid_hour" plain type="success">{{ detail.valid_hour }} 学时</Tag>
                <Tag v-if="detail.item_category === '1'" plain>系列课</Tag>
              </div>
            </div>
          </div>
          <CellGroup inset class="mt-3">
            <Cell title="时间" :value="`${shortTime(detail.start_time)} - ${shortTime(detail.end_time)}`" />
            <Cell title="地点" :value="detail.placeInfo || '未提供'" />
            <Cell title="主办方" :value="detail.organizer_dictText || detail.businessDeptName || '未知'" />
            <Cell title="报名截止" :value="shortTime(detail.apply_end)" />
            <Cell title="联系电话" :value="detail.tel || '无'" />
          </CellGroup>
          <div class="mt-3 text-sm leading-relaxed text-gray-700" v-html="detail.baseContent || detail.conceive || '暂无详情'" />
          <div class="mt-4 flex gap-2">
            <!-- 系列活动不显示报名按钮 -->
            <template v-if="!isSeries">
              <template v-if="isRegistered">
                <Button type="danger" block @click="onCancelApply">取消报名</Button>
              </template>
              <template v-else>
                <Button type="primary" block @click="() => onApply()">立即报名</Button>
                <Button :loading="autoApplyLoading" block plain type="warning" @click="autoApply">名额监控</Button>
              </template>
            </template>
            <template v-else>
              <div class="text-center text-gray-500 text-sm w-full">系列活动请在下方选择子项目报名</div>
            </template>
          </div>
        </div>

        <div v-if="detail.children?.length" class="mt-4">
          <div class="font-semibold mb-2">系列课子项目</div>
          <ActivityCard
            v-for="child in detail.children"
            :key="child.id"
            :activity="child"
            :show-apply="true"
            @detail="$router.push({ name: 'activity-detail', params: { id: child.id }, query: { from: $route.query.from } })"
            @apply="() => onApply()"
          />
        </div>
      </template>
    </div>
  </div>
</template>
