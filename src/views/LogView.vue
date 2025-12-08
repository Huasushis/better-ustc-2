<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NavBar, Button, showToast, showDialog } from 'vant'
import { useLogStore } from '../stores/logs'
import { useRouter } from 'vue-router'

const logStore = useLogStore()
const router = useRouter()
const logs = ref<string[]>([])

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
</script>

<template>
  <div class="min-h-screen bg-white flex flex-col">
    <NavBar title="应用日志" left-arrow @click-left="router.back()" fixed />
    <div class="pt-12 flex-1 overflow-hidden flex flex-col">
      <div class="p-2 flex gap-2 border-b">
        <Button size="small" type="primary" @click="refresh">刷新</Button>
        <Button size="small" type="warning" @click="save">保存到文件</Button>
        <Button size="small" type="danger" @click="clear">清空</Button>
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

