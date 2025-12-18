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
    // 优先检查是否是活动详情页，并且有from参数
    if (route.path.startsWith('/activity')) {
      const from = route.query.from as string
      if (from === 'registered') return 'registered'
      if (from === 'profile') return 'profile'
      return 'home'
    }
    if (route.path.startsWith('/profile') || route.path.startsWith('/about') || route.path.startsWith('/logs')) return 'profile'
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
    <div class="min-h-screen flex flex-col bg-[#f7f8fa]">
      <div class="flex-1 pb-14 overflow-y-auto">
        <router-view v-slot="{ Component, route }">
          <!-- 详情页不使用 keep-alive，其他页面使用 -->
          <keep-alive :exclude="['ActivityDetailView']">
            <component :is="Component" :key="route.path.startsWith('/activity') ? route.fullPath : undefined" />
          </keep-alive>
        </router-view>
      </div>
      <Tabbar :fixed="true" :safe-area-inset-bottom="true" v-model="active">
        <TabbarItem name="home" icon="home-o">首页</TabbarItem>
        <TabbarItem name="registered" icon="records">已报</TabbarItem>
        <TabbarItem name="profile" icon="user-o">我的</TabbarItem>
      </Tabbar>
    </div>
  </ConfigProvider>
</template>

<style>
:root {
  --van-nav-bar-height: 44px;
  --van-nav-bar-background: #fff;
}
.van-nav-bar__content {
  align-items: center !important;
  height: 100%;
}
.van-nav-bar__title {
  font-weight: 600;
  font-size: 17px;
}
.van-nav-bar .van-icon {
  font-size: 20px;
  color: #333;
}
.van-nav-bar__text {
  color: #333;
}
.van-nav-bar__left, .van-nav-bar__right {
  align-items: center !important;
}
</style>