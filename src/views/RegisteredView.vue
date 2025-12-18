<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { NavBar, Tabs, Tab, Empty, Loading, showConfirmDialog, showLoadingToast, showSuccessToast, showFailToast, closeToast } from 'vant'
import ActivityCard from '../components/ActivityCard.vue'
import { useActivityStore } from '../stores/activity'
import { useLogStore } from '../stores/logs'
import { useRouter } from 'vue-router'

const store = useActivityStore()
const logStore = useLogStore()
const router = useRouter()
const active = ref('registered')

const load = async () => {
  await store.fetchMine()
}

const goDetail = (id: string) => {
  logStore.add(`点击进入已报名活动详情: ${id}`)
  router.push({ name: 'activity-detail', params: { id }, query: { from: 'registered' } })
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
    const ok = await store.cancelApply(id)
    if (ok) {
      logStore.add(`取消报名成功: ${id}`)
      closeToast()
      showSuccessToast('取消成功')
      // 更新本地活动状态
      store.updateRegistrationStatus(id, false)
      // 刷新已报名列表
      store.fetchMine()
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

onMounted(async () => {
  if (store.registered.length === 0 && store.participated.length === 0) {
    await load()
  }
})
</script>

<template>
  <div class="min-h-full flex flex-col bg-[#f7f8fa]">
    <NavBar title="我的报名" fixed placeholder safe-area-inset-top />
    <div class="flex-1 min-h-0">
      <Tabs v-model:active="active" swipeable sticky offset-top="46">
        <Tab title="报名中/已截止" name="registered">
          <div class="p-3">
            <div v-if="store.loadingMine" class="py-4 text-center text-gray-500"><Loading size="24" vertical>加载中</Loading></div>
            <template v-else>
              <Empty v-if="!store.registered.length" description="暂无" />
              <ActivityCard
                v-for="item in store.registered"
                :key="item.id"
                :activity="item"
                :registered="true"
                :show-apply="true"
                @detail="goDetail(item.id)"
                @cancel="onCancel(item.id)"
              />
            </template>
          </div>
        </Tab>
        <Tab title="已参与/已结项" name="participated">
          <div class="p-3">
            <div v-if="store.loadingMine" class="py-4 text-center text-gray-500"><Loading size="24" vertical>加载中</Loading></div>
            <template v-else>
              <Empty v-if="!store.participated.length" description="暂无" />
              <ActivityCard
                v-for="item in store.participated"
                :key="item.id"
                :activity="item"
                :registered="true"
                @detail="goDetail(item.id)"
              />
            </template>
          </div>
        </Tab>
      </Tabs>
    </div>
  </div>
</template>
