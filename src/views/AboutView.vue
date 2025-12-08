<script setup lang="ts">
import { NavBar, Cell, CellGroup, Image as VanImage } from 'vant'
import pkg from '../../package.json'
</script>

<template>
  <div class="min-h-screen bg-[#f7f8fa] pb-4">
    <NavBar title="关于" left-arrow @click-left="$router.back()" fixed />
    <div class="pt-12 px-3">
      <div class="bg-white rounded-lg p-4 shadow-sm flex items-center gap-3">
        <VanImage width="64" height="64" round src="/logo.png" fit="cover" />
        <div>
          <div class="text-lg font-semibold">Better USTC 2</div>
          <div class="text-gray-500 text-sm">跨平台二课助手</div>
        </div>
      </div>
      <CellGroup inset class="mt-4">
        <Cell title="版本" :value="pkg.version" />
        <Cell title="许可证" value="MIT" />
        <Cell title="作者" value="陈鑫 / 范祎博涵 / 周映诚" />
      </CellGroup>
    </div>
  </div>
</template>
