<script setup lang="ts">
import { NavBar, Cell, CellGroup } from 'vant'
import { useRouter } from 'vue-router'

const router = useRouter()
const version = '0.1.0' // Should match package.json
</script>

<template>
  <div class="min-h-screen bg-[#f7f8fa]">
    <NavBar title="关于" left-arrow @click-left="router.back()" fixed placeholder safe-area-inset-top />
    <div class="px-4 flex flex-col items-center">
      <div class="mt-10 mb-6 flex flex-col items-center">
        <div class="w-20 h-20 bg-blue-500 rounded-xl flex items-center justify-center text-white text-2xl font-bold shadow-lg">
          USTC
        </div>
        <div class="mt-3 text-lg font-semibold">Better USTC II</div>
        <div class="text-gray-500 text-sm">v{{ version }}</div>
      </div>

      <CellGroup inset class="w-full">
        <Cell title="开发者" value="Huasushis" />
        <Cell title="开源协议" value="MIT" />
        <Cell title="GitHub" value="https://github.com/Huasushis/better-ustc-2" is-link url="https://github.com/Huasushis/better-ustc-2" />
      </CellGroup>

      <div class="mt-auto mb-10 text-center text-gray-400 text-xs">
        <p>Copyright © 2025 Huasushis, Jason, 近彩休蔽日</p>
        <p>All Rights Reserved</p>
      </div>
    </div>
  </div>
</template>

