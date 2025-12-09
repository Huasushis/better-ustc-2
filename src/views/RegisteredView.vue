<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { NavBar, Tabs, Tab, Empty, Loading } from 'vant'
import ActivityCard from '../components/ActivityCard.vue'
import { useActivityStore } from '../stores/activity'
import { useRouter } from 'vue-router'

const store = useActivityStore()
const router = useRouter()
const active = ref('registered')

const load = async () => {
  await store.fetchMine()
}

onMounted(async () => {
  if (store.registered.length === 0 && store.participated.length === 0) {
    await load()
  }
})
</script>

<template>
  <div class="h-full flex flex-col bg-[#f7f8fa]">
    <NavBar title="我的报名" fixed placeholder safe-area-inset-top />
    <div class="flex-1 overflow-hidden">
      <Tabs v-model:active="active" swipeable class="h-full flex flex-col">
        <Tab title="报名中/已截止" name="registered" class="h-full overflow-y-auto">
          <div class="p-3 min-h-full">
            <div v-if="store.loadingMine" class="py-4 text-center text-gray-500"><Loading size="24" vertical>加载中</Loading></div>
            <template v-else>
              <Empty v-if="!store.registered.length" description="暂无" />
              <ActivityCard
                v-for="item in store.registered"
                :key="item.id"
                :activity="item"
                :registered="true"
                @detail="router.push({ name: 'activity-detail', params: { id: item.id } })"
              />
            </template>
          </div>
        </Tab>
        <Tab title="已参与/已结项" name="participated" class="h-full overflow-y-auto">
          <div class="p-3 min-h-full">
            <div v-if="store.loadingMine" class="py-4 text-center text-gray-500"><Loading size="24" vertical>加载中</Loading></div>
            <template v-else>
              <Empty v-if="!store.participated.length" description="暂无" />
              <ActivityCard
                v-for="item in store.participated"
                :key="item.id"
                :activity="item"
                :registered="true"
                @detail="router.push({ name: 'activity-detail', params: { id: item.id } })"
              />
            </template>
          </div>
        </Tab>
      </Tabs>
    </div>
  </div>
</template>
