<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { NavBar, Cell, CellGroup, Button, Field, Form, Toast, showToast, showNotify } from 'vant'
import { useUserStore } from '../stores/user'
import { useRouter } from 'vue-router'
import { useLogStore } from '../stores/logs'

const userStore = useUserStore()
const router = useRouter()
const logStore = useLogStore()
const username = ref('')
const password = ref('')
const save = ref(true)
const tapCount = ref(0)

const handleDevTap = () => {
  tapCount.value += 1
  if (tapCount.value >= 7 && !logStore.devMode) {
    logStore.enableDevMode()
    showNotify({ type: 'success', message: '开发者模式已开启' })
  }
}

onMounted(async () => {
  await Promise.all([userStore.fetchStatus(), logStore.load()])
})

// Auto-fill username if available and field is empty
watch(() => userStore.status.username, (val) => {
  if (val && !username.value) {
    username.value = val
  }
}, { immediate: true })

const onLogin = async () => {
  if (!username.value || !password.value) return showToast('请输入账号密码')
  const ok = await userStore.login(username.value, password.value, save.value)
  if (ok) {
    Toast.success('登录成功')
  } else {
    showNotify({ type: 'danger', message: userStore.error || '登录失败' })
  }
}

const gotoMine = () => router.push('/registered')
const gotoAbout = () => router.push('/about')
const gotoLogs = () => logStore.devMode && router.push('/logs')
</script>

<template>
  <div class="min-h-screen bg-[#f7f8fa] pb-6">
    <NavBar title="个人中心" fixed />
    <div class="pt-12 px-3">
      <div class="bg-white rounded-xl p-4 shadow-sm mb-4" @click="handleDevTap">
        <div class="flex items-center gap-3">
          <div class="w-12 h-12 rounded-full bg-blue-100 flex items-center justify-center text-blue-600 text-lg font-semibold">
            {{ userStore.displayName?.[0] || '未' }}
          </div>
          <div class="flex-1 min-w-0">
            <div class="font-semibold text-[16px]">{{ userStore.displayName }}</div>
            <div class="text-gray-500 text-[12px]">
              {{ userStore.status.user?.classes || '未登录' }}
            </div>
          </div>
          <Button size="small" type="primary" plain @click="gotoMine">我的报名</Button>
        </div>
        <div class="mt-3 flex gap-2">
          <Button block type="primary" :disabled="userStore.isLoggedIn" @click="onLogin">{{ userStore.isLoggedIn ? '已登录' : '登录' }}</Button>
          <Button block plain type="danger" v-if="userStore.isLoggedIn" @click="userStore.logout">退出</Button>
        </div>
        <div class="mt-3 flex gap-2">
          <Button block plain type="primary" @click="gotoAbout">关于</Button>
          <Button block plain type="warning" v-if="logStore.devMode" @click="gotoLogs">查看日志</Button>
        </div>
      </div>

      <div class="bg-white rounded-xl p-4 shadow-sm" v-if="!userStore.isLoggedIn">
        <div class="font-semibold mb-3">账号登录</div>
        <Form @submit="onLogin">
          <Field v-model="username" name="username" label="学号" placeholder="请输入学号" />
          <Field v-model="password" type="password" name="password" label="密码" placeholder="请输入密码" />
          <div class="mt-2 text-gray-500 text-[12px]">密码将使用本机密钥加密后存储（可选）。</div>
          <div class="mt-3 flex items-center gap-2">
            <input type="checkbox" id="save" v-model="save" />
            <label for="save" class="text-[13px]">保存账号密码</label>
          </div>
          <Button round block type="primary" class="mt-3" native-type="submit" :loading="userStore.loading">登录</Button>
        </Form>
      </div>

      <CellGroup inset class="mt-4">
        <Cell title="登录状态" :value="userStore.isLoggedIn ? '已登录' : '未登录'" />
        <Cell title="已存储账号" :value="userStore.status.has_stored_creds ? '是' : '否'" />
        <Cell title="课程/学时" :value="userStore.status.user?.scientific_value ?? '-'" />
      </CellGroup>
    </div>
  </div>
</template>
