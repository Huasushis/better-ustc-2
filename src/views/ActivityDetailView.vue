<script setup lang="ts">
import { onMounted, ref, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { NavBar, Tag, Cell, CellGroup, Toast, Button, Loading, showNotify } from 'vant'
import { useActivityStore, statusText, shortTime } from '../stores/activity'
import ActivityCard from '../components/ActivityCard.vue'
import { requestPermission, sendNotification } from '@tauri-apps/plugin-notification'

const route = useRoute()
const store = useActivityStore()
const id = route.params.id as string
const detail = ref<any>(null)
const loading = ref(true)

const load = async () => {
  loading.value = true
  try {
    detail.value = await store.refreshDetail(id)
  } catch (e: any) {
    Toast.fail(e?.toString?.() || '加载失败')
  } finally {
    loading.value = false
  }
}

onMounted(load)

const onApply = async () => {
  if (!detail.value) return
  Toast.loading({ message: '报名中...', duration: 0 })
  try {
    const ok = await store.apply(detail.value.id)
    if (ok) {
      Toast.success('报名成功')
      await scheduleNotification(detail.value)
    } else Toast.fail('报名失败/名额已满')
  } catch (e: any) {
    Toast.fail(e?.toString?.() || '报名失败')
  } finally {
    Toast.clear()
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
  Toast.loading({ message: '监控名额中...', duration: 0 })
  let attempts = 0
  autoTimer.value = window.setInterval(async () => {
    attempts += 1
    const latest = await store.refreshDetail(detail.value!.id)
    if (latest.apply_limit && (latest.apply_num || 0) < latest.apply_limit && latest.status_code === 26) {
      try {
        const ok = await store.apply(latest.id)
        if (ok) {
          Toast.success('抢到名额，已报名')
          await scheduleNotification(latest)
          stopAuto()
          return
        }
      } catch (e) {
        console.error(e)
      }
    }
    if (attempts >= 20) {
      Toast.fail('监控结束，仍未抢到')
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
  Toast.clear()
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
            <Button type="primary" block @click="onApply">立即报名</Button>
            <Button :loading="autoApplyLoading" block plain type="warning" @click="autoApply">名额监控</Button>
          </div>
        </div>

        <div v-if="detail.children?.length" class="mt-4">
          <div class="font-semibold mb-2">系列课子项目</div>
          <ActivityCard
            v-for="child in detail.children"
            :key="child.id"
            :activity="child"
            :show-apply="true"
            @detail="$router.push({ name: 'activity-detail', params: { id: child.id } })"
            @apply="() => onApply()"
          />
        </div>
      </template>
    </div>
  </div>
</template>
