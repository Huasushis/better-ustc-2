<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { NavBar, Button, showToast, showDialog } from 'vant'
import { useLogStore } from '../stores/logs'
import { useRouter } from 'vue-router'
import { requestPermission, sendNotification, isPermissionGranted } from '@tauri-apps/plugin-notification'

const logStore = useLogStore()
const router = useRouter()
const logs = ref<string[]>([])

// 开发者模式
const isDevMode = computed(() => logStore.devMode)

const refresh = async () => {
  await logStore.load()
  logs.value = [...logStore.entries].reverse()
}

onMounted(refresh)

const clear = async () => {
  await showDialog({ title: '确认', message: '确定清空日志吗？', showCancelButton: true })
  await logStore.clear()
  await refresh()
  showToast('已清空')
}

const save = async () => {
  try {
    await logStore.saveToFile()
    showToast('已保存到下载目录')
  } catch (e: any) {
    showToast(e?.toString?.() || '保存失败')
  }
}

// 通知测试功能（开发者模式）
const testNotification = async () => {
  logStore.add('测试通知功能')
  try {
    // 检查权限
    let granted = await isPermissionGranted()
    logStore.add(`当前通知权限: ${granted}`)
    
    if (!granted) {
      const permission = await requestPermission()
      logStore.add(`请求通知权限结果: ${permission}`)
      granted = permission === 'granted'
    }
    
    if (granted) {
      await sendNotification({
        title: '测试通知',
        body: `这是一条测试通知 - ${new Date().toLocaleTimeString()}`,
      })
      logStore.add('通知已发送')
      showToast('通知已发送')
    } else {
      logStore.add('通知权限被拒绝')
      showToast('通知权限被拒绝')
    }
  } catch (e: any) {
    logStore.add(`通知测试失败: ${e?.toString?.()}`)
    showToast(e?.toString?.() || '通知测试失败')
  }
  await refresh()
}
</script>

<template>
  <div class="min-h-screen bg-white flex flex-col">
    <NavBar title="应用日志" left-arrow @click-left="router.back()" fixed placeholder safe-area-inset-top />
    <div class="flex-1 overflow-hidden flex flex-col">
      <div class="p-2 flex gap-2 border-b flex-wrap">
        <Button size="small" type="primary" @click="refresh">刷新</Button>
        <Button size="small" type="warning" @click="save">保存到文件</Button>
        <Button size="small" type="danger" @click="clear">清空</Button>
        <Button v-if="isDevMode" size="small" type="success" @click="testNotification">测试通知</Button>
      </div>
      <div class="flex-1 overflow-y-auto p-2 text-[12px] font-mono bg-gray-50">
        <div v-for="(line, i) in logs" :key="i" class="break-all mb-1 border-b border-gray-100 pb-1">
          {{ line }}
        </div>
        <div v-if="!logs.length" class="text-center text-gray-400 mt-10">暂无日志</div>
      </div>
    </div>
  </div>
</template>

