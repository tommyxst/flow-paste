<script setup lang="ts">
import { computed } from 'vue'
import type { AIProvider } from '@/types'

const props = defineProps<{
  provider: AIProvider
  isConnected?: boolean
}>()

const badgeConfig = computed(() => {
  if (props.provider === 'Ollama') {
    return {
      label: 'Local',
      color: 'text-green-600 dark:text-green-400',
      bgColor: 'bg-green-100 dark:bg-green-900/30',
      icon: '💻',
    }
  }
  return {
    label: 'Cloud',
    color: 'text-blue-600 dark:text-blue-400',
    bgColor: 'bg-blue-100 dark:bg-blue-900/30',
    icon: '☁️',
  }
})
</script>

<template>
  <div
    class="model-badge inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium"
    :class="badgeConfig.bgColor"
  >
    <span>{{ badgeConfig.icon }}</span>
    <span :class="badgeConfig.color">{{ badgeConfig.label }}</span>
    <span
      v-if="isConnected !== undefined"
      class="w-1.5 h-1.5 rounded-full"
      :class="isConnected ? 'bg-green-500' : 'bg-red-500'"
    />
  </div>
</template>
