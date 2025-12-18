<script setup lang="ts">
import { Tag, Icon, Button } from 'vant'
import { computed } from 'vue'
import type { Activity } from '../stores/activity'
import { statusText, shortTime } from '../stores/activity'

const props = defineProps<{ activity: Activity; showApply?: boolean; registered?: boolean; showApplyCount?: boolean }>()
const emit = defineEmits<{
  (e: 'detail'): void
  (e: 'apply'): void
  (e: 'cancel'): void
}>()

const cover = computed(() => props.activity.pic ? `https://young.ustc.edu.cn/login/${props.activity.pic}` : 'https://via.placeholder.com/120')
const isSeries = computed(() => props.activity.item_category === '1')
const registered = computed(() => props.registered || props.activity.boolean_registration === 1)
</script>

<template>
  <div class="flex gap-3 bg-white rounded-lg p-3 shadow-sm mb-3" @click.stop="emit('detail')">
    <img :src="cover" alt="cover" class="w-24 h-24 rounded-md object-cover" />
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1">
        <span class="font-semibold text-[15px] line-clamp-2 flex-1">{{ activity.name }}</span>
        <Tag v-if="registered" type="success" size="medium">已报名</Tag>
        <Tag v-if="isSeries" type="primary" plain size="medium">系列</Tag>
      </div>
      <div class="text-[12px] text-gray-500 mt-1 flex items-center gap-1">
        <Icon name="clock-o" />
        <span>{{ shortTime(activity.start_time) }} - {{ shortTime(activity.end_time) }}</span>
      </div>
      <div class="text-[12px] text-gray-500 mt-1 flex items-center gap-1">
        <Icon name="location-o" />
        <span class="truncate">{{ activity.placeInfo || '未提供地点' }}</span>
      </div>
      <div class="text-[12px] text-gray-500 mt-1 flex items-center gap-1">
        <Icon name="friends-o" />
        <span class="truncate">{{ activity.organizer_dictText || activity.businessDeptName || '主办方未知' }}</span>
      </div>

      <div class="mt-1 flex items-center gap-2">
        <Tag plain type="warning" size="medium">截止 {{ shortTime(activity.apply_end) }}</Tag>
        <Tag plain type="primary" size="medium">{{ statusText(activity.status_code) }}</Tag>
        <Tag v-if="showApplyCount && activity.apply_limit" plain size="medium">{{ activity.apply_num || 0 }}/{{ activity.apply_limit }}</Tag>
      </div>
      <!-- 系列活动 -->
      <div v-if="showApply && isSeries" class="mt-2">
        <Button size="small" plain block @click.stop="emit('detail')">查看系列子项目</Button>
      </div>
      <!-- 已报名的非系列活动 -->
      <div v-else-if="showApply && !isSeries && registered" class="mt-2 flex gap-2">
        <Button size="small" type="danger" block @click.stop="emit('cancel')">取消报名</Button>
        <Button size="small" plain block @click.stop="emit('detail')">详情</Button>
      </div>
      <!-- 未报名的非系列活动 -->
      <div v-else-if="showApply && !isSeries" class="mt-2 flex gap-2">
        <Button size="small" type="primary" block @click.stop="emit('apply')">报名/抢位</Button>
        <Button size="small" plain block @click.stop="emit('detail')">详情</Button>
      </div>
    </div>
  </div>
</template>
