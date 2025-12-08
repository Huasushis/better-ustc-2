<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { computed, onMounted, watch } from 'vue'
import { Tabbar, TabbarItem, ConfigProvider } from 'vant'
import { useUserStore } from './stores/user'
import { useActivityStore } from './stores/activity'

const route = useRoute()
const router = useRouter()
const userStore = useUserStore()
const activityStore = useActivityStore()

const active = computed({
  get() {
    if (route.path.startsWith('/profile')) return 'profile'
    if (route.path.startsWith('/registered')) return 'registered'
    return 'home'
  },
  set(name: string) {
    if (name === 'home') router.replace('/')
    else if (name === 'profile') router.replace('/profile')
    else if (name === 'registered') router.replace('/registered')
  },
})

onMounted(async () => {
  await userStore.fetchStatus()
})

watch(() => userStore.isLoggedIn, (val) => {
  if (val) {
    activityStore.fetchAll()
    activityStore.fetchRecommended()
    activityStore.fetchMine()
  }
}, { immediate: true })
</script>

<template>
  <ConfigProvider :theme-vars="{ primaryColor: '#1e80ff' }">
    <div class="min-h-screen flex flex-col bg-[#f7f8fa] pt-[env(safe-area-inset-top)]">
      <div class="flex-1 overflow-hidden pb-14">
        <router-view v-slot="{ Component }">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </div>
      <Tabbar :fixed="true" :safe-area-inset-bottom="true" v-model="active">
        <TabbarItem name="home" icon="home-o">首页</TabbarItem>
        <TabbarItem name="registered" icon="todo-list-o">我的报名</TabbarItem>
        <TabbarItem name="profile" icon="user-o">个人中心</TabbarItem>
      </Tabbar>
    </div>
  </ConfigProvider>
</template>