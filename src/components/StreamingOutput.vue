<script setup lang="ts">
import { ref, watch, onUnmounted, computed } from 'vue'

const props = defineProps<{
  content: string
  isStreaming: boolean
}>()

const emit = defineEmits<{
  cancel: []
}>()

const displayedContent = ref('')
const cursorVisible = ref(true)
let animationFrame: number | null = null
let cursorInterval: number | null = null

const isOverflowing = computed(() => displayedContent.value.length > 500)

watch(
  () => props.content,
  (newContent) => {
    if (props.isStreaming) {
      displayedContent.value = newContent
    } else {
      displayedContent.value = newContent
    }
  },
  { immediate: true }
)

watch(
  () => props.isStreaming,
  (streaming) => {
    if (streaming) {
      cursorInterval = window.setInterval(() => {
        cursorVisible.value = !cursorVisible.value
      }, 530)
    } else {
      if (cursorInterval) {
        clearInterval(cursorInterval)
        cursorInterval = null
      }
      cursorVisible.value = false
    }
  },
  { immediate: true }
)

onUnmounted(() => {
  if (animationFrame) cancelAnimationFrame(animationFrame)
  if (cursorInterval) clearInterval(cursorInterval)
})

function handleCancel() {
  emit('cancel')
}
</script>

<template>
  <div class="streaming-output relative">
    <div
      class="output-content font-mono text-sm leading-relaxed whitespace-pre-wrap break-words"
      :class="{ 'max-h-64 overflow-y-auto': isOverflowing }"
    >
      <span class="text-gray-800 dark:text-gray-200">{{ displayedContent }}</span>
      <span
        v-if="isStreaming && cursorVisible"
        class="cursor inline-block w-2 h-4 bg-blue-500 ml-0.5 align-middle animate-pulse"
      />
    </div>

    <div
      v-if="isStreaming"
      class="cancel-section mt-3 flex items-center justify-between"
    >
      <span class="text-xs text-gray-500">正在生成...</span>
      <button
        class="cancel-btn px-3 py-1 text-xs text-gray-600 dark:text-gray-400 hover:text-red-500 dark:hover:text-red-400 bg-gray-100 dark:bg-gray-800 rounded-md transition-colors"
        @click="handleCancel"
      >
        取消 (Esc)
      </button>
    </div>
  </div>
</template>

<style scoped>
.output-content::-webkit-scrollbar {
  width: 6px;
}

.output-content::-webkit-scrollbar-track {
  background: transparent;
}

.output-content::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 3px;
}

.output-content::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.5);
}

.cursor {
  animation: blink 1s step-end infinite;
}

@keyframes blink {
  50% {
    opacity: 0;
  }
}
</style>
