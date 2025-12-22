<script setup lang="ts">
import { computed, ref } from 'vue'

export interface PreviewProps {
  content: string
  mode?: 'preview' | 'result' | 'streaming'
  highlight?: boolean
  truncated?: boolean
}

const props = withDefaults(defineProps<PreviewProps>(), {
  mode: 'preview',
  highlight: false,
  truncated: false,
})

const displayContent = computed(() => {
  if (props.mode === 'preview') {
    const lines = props.content.split('\n')
    return lines.slice(0, 3).join('\n')
  }
  return props.content
})

const showEllipsis = computed(() => {
  if (props.mode === 'preview') {
    return props.content.split('\n').length > 3
  }
  return false
})

// Streaming typewriter effect
const cursorVisible = ref(true)

const streamingDisplay = computed(() => {
  if (props.mode === 'streaming') {
    return props.content
  }
  return ''
})

const contentClasses = computed(() => {
  const base = 'text-sm font-mono whitespace-pre-wrap break-words p-3 rounded-lg overflow-y-auto'

  switch (props.mode) {
    case 'preview':
      return `${base} text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800/50 max-h-24`
    case 'result':
      return `${base} text-gray-800 dark:text-gray-200 bg-blue-50 dark:bg-blue-900/20 max-h-64`
    case 'streaming':
      return `${base} text-gray-800 dark:text-gray-200 bg-gray-50 dark:bg-gray-800/50 max-h-64`
    default:
      return base
  }
})
</script>

<template>
  <div class="preview-container">
    <div :class="contentClasses">
      <template v-if="mode === 'streaming'">
        {{ streamingDisplay }}<span
          v-if="cursorVisible"
          class="inline-block w-2 h-4 bg-blue-500 ml-0.5 animate-pulse"
        ></span>
      </template>
      <template v-else>
        {{ displayContent }}
      </template>
    </div>

    <!-- Truncation indicator -->
    <div
      v-if="truncated || showEllipsis"
      class="mt-1 text-xs text-gray-400 dark:text-gray-500"
    >
      <template v-if="mode === 'preview'">
        ... (共 {{ content.split('\n').length }} 行)
      </template>
      <template v-else-if="truncated">
        ⚠️ 内容过长已截断
      </template>
    </div>
  </div>
</template>

<style scoped>
.preview-container ::-webkit-scrollbar {
  width: 4px;
}

.preview-container ::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 2px;
}

.preview-container ::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.5);
}
</style>
