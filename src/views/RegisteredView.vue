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

onMounted(load)
</script>

<template>
  <div class="min-h-screen bg-[#f7f8fa] pb-4">
    <NavBar title="我的报名" left-arrow @click-left="$router.back()" fixed />
    <div class="pt-12 px-3">
      <Tabs v-model:active="active">
        <Tab title="报名中/已截止" name="registered">
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
        </Tab>
        <Tab title="已参与/已结项" name="participated">
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
        </Tab>
      </Tabs>
    </div>
  </div>
</template>
