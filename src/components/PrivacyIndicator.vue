<script setup lang="ts">
import { computed } from 'vue'
import type { PrivacyStatus } from '@/types'

const props = defineProps<{
  status: PrivacyStatus
}>()

const statusConfig = computed(() => {
  switch (props.status.type) {
    case 'local':
      return {
        icon: '🛡️',
        color: 'text-green-500',
        bgColor: 'bg-green-500/10',
        borderColor: 'border-green-500/20',
        tooltip: '本地处理，数据不出设备',
      }
    case 'cloud-safe':
      return {
        icon: '🛡️',
        color: 'text-blue-500',
        bgColor: 'bg-blue-500/10',
        borderColor: 'border-blue-500/20',
        tooltip: '云端处理，未检测到敏感信息',
      }
    case 'cloud-masked':
      return {
        icon: '🛡️',
        color: 'text-amber-500',
        bgColor: 'bg-amber-500/10',
        borderColor: 'border-amber-500/20',
        tooltip: `隐私盾已激活：${props.status.maskedCount} 项敏感信息已脱敏`,
      }
  }
})
</script>

<template>
  <div
    class="privacy-indicator flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium transition-all duration-200 cursor-default"
    :class="[statusConfig.bgColor, statusConfig.borderColor, 'border']"
    :title="statusConfig.tooltip"
  >
    <span class="text-sm">{{ statusConfig.icon }}</span>
    <span :class="statusConfig.color">
      <template v-if="status.type === 'local'">本地</template>
      <template v-else-if="status.type === 'cloud-safe'">安全</template>
      <template v-else>{{ status.maskedCount }} 项已脱敏</template>
    </span>
  </div>
</template>

<style scoped>
.privacy-indicator:hover {
  transform: scale(1.02);
}
</style>
