<script setup lang="ts">
import { onMounted } from 'vue'
import { NavBar, List, Empty, Button, showConfirmDialog } from 'vant'
import { useLogStore } from '../stores/logs'

const logStore = useLogStore()

onMounted(() => {
  logStore.load()
})
</script>

<template>
  <div class="min-h-screen bg-[#f7f8fa] pb-4">
    <NavBar title="日志" left-arrow @click-left="$router.back()" fixed />
    <div class="pt-12 px-3">
      <div class="bg-white rounded-lg p-3 shadow-sm">
        <List :finished="true" finished-text="到底了">
          <div v-if="!logStore.entries.length" class="py-6"><Empty description="暂无日志" /></div>
          <div v-else class="space-y-2 text-[12px] text-gray-700">
            <div v-for="(line, idx) in logStore.entries" :key="idx" class="border-b pb-2">{{ line }}</div>
          </div>
        </List>
        <Button
          block
          plain
          type="danger"
          class="mt-2"
          @click="
            showConfirmDialog({ title: '清空日志', message: '确定要删除所有日志吗？' })
              .then(() => logStore.clear())
              .catch(() => {})
          "
        >
          清空日志
        </Button>
      </div>
    </div>
  </div>
</template>
