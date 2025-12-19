<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useAppStore } from '@/stores/app'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'

const store = useAppStore()

const inputRef = ref<HTMLInputElement | null>(null)
const commandInput = ref('')
const isDragging = ref(false)
let unlistenFocus: (() => void) | null = null
let unlistenShow: (() => void) | null = null
let unlistenAIChunk: (() => void) | null = null

const previewLines = computed(() => {
  if (!store.clipboardText) return '（无剪贴板内容）'
  const lines = store.clipboardText.split('\n')
  return lines.slice(0, 5).join('\n') + (lines.length > 5 ? `\n... (共 ${lines.length} 行)` : '')
})

const charCount = computed(() => store.clipboardText.length)

const displayContent = computed(() => {
  if (store.panelMode === 'processing') {
    return store.streamingContent || '处理中...'
  }
  if (store.panelMode === 'result') {
    return store.processedContent
  }
  return previewLines.value
})

async function hideWindow() {
  const appWindow = getCurrentWindow()
  await appWindow.hide()
}

async function startDrag() {
  isDragging.value = true
  const appWindow = getCurrentWindow()
  await appWindow.startDragging()
  setTimeout(() => { isDragging.value = false }, 200)
}

async function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (store.isProcessing) {
      store.cancelAI()
    } else {
      await hideWindow()
    }
  } else if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSubmit()
  }
}

// Quick actions using backend regex rules
async function applyQuickRule(ruleId: string) {
  await store.processWithRule(ruleId)
}

function handleSubmit() {
  if (!commandInput.value.trim()) return
  store.processWithAI(commandInput.value)
  commandInput.value = ''
}

async function confirmAndClose() {
  await store.confirmPaste()
  await hideWindow()
}

onMounted(async () => {
  inputRef.value?.focus()
  store.showPanel()
  await store.loadConfig()

  const appWindow = getCurrentWindow()

  unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      store.refreshClipboard()
      inputRef.value?.focus()
    } else if (!isDragging.value && !store.isProcessing) {
      appWindow.hide()
    }
  })

  unlistenShow = await listen('panel:show', () => {
    store.reset()
    store.showPanel()
    inputRef.value?.focus()
  })

  // Listen for AI streaming events
  unlistenAIChunk = await listen<{ content: string; done: boolean; requestId: string }>('ai:chunk', (event) => {
    store.handleAIChunk(event.payload)
  })

  document.addEventListener('keydown', handleGlobalKeydown)
})

function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (store.isProcessing) {
      store.cancelAI()
    } else {
      hideWindow()
    }
  }
}

onUnmounted(() => {
  unlistenFocus?.()
  unlistenShow?.()
  unlistenAIChunk?.()
  document.removeEventListener('keydown', handleGlobalKeydown)
})
</script>

<template>
  <div
    class="floating-panel w-full h-full flex flex-col rounded-xl overflow-hidden"
    style="background: var(--panel-bg); border: 1px solid var(--panel-border)"
    @keydown="handleKeydown"
  >
    <!-- Drag Handle -->
    <div
      class="drag-handle h-6 flex items-center justify-center cursor-move shrink-0"
      @mousedown="startDrag"
    >
      <div class="w-10 h-1 bg-gray-300 dark:bg-gray-600 rounded-full"></div>
    </div>

    <!-- Header: Input -->
    <div class="px-4 pb-2">
      <input
        ref="inputRef"
        v-model="commandInput"
        type="text"
        class="w-full px-3 py-2 bg-gray-100 dark:bg-gray-800 rounded-lg text-sm text-gray-800 dark:text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50"
        placeholder="输入处理指令（如：翻译成英文、总结要点）..."
        :disabled="store.isProcessing"
      />
    </div>

    <!-- Content Area -->
    <div class="flex-1 px-4 py-2 overflow-hidden flex flex-col">
      <!-- Preview -->
      <div class="preview-section flex-1 min-h-0">
        <div class="flex items-center justify-between mb-2">
          <div class="text-xs text-gray-500">
            <template v-if="store.panelMode === 'processing'">AI 处理中...</template>
            <template v-else-if="store.panelMode === 'result'">处理结果</template>
            <template v-else>剪贴板内容预览</template>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs text-gray-400">{{ charCount }} 字符</span>
            <button
              v-if="store.isProcessing"
              @click="store.cancelAI"
              class="text-xs text-red-500 hover:text-red-600"
            >
              取消
            </button>
            <button
              v-else
              @click="store.refreshClipboard"
              class="text-xs text-blue-500 hover:text-blue-600"
            >
              刷新
            </button>
          </div>
        </div>
        <pre class="text-sm text-gray-700 dark:text-gray-300 font-mono whitespace-pre-wrap break-words h-full max-h-32 overflow-y-auto bg-gray-50 dark:bg-gray-800/50 p-3 rounded-lg">{{ displayContent }}</pre>
      </div>

      <!-- Quick Actions -->
      <div class="mt-3" v-if="!store.isProcessing">
        <div class="text-xs text-gray-500 mb-2">快捷操作</div>
        <div class="flex flex-wrap gap-2">
          <button
            @click="applyQuickRule('remove_empty_lines')"
            class="px-3 py-1.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
            :disabled="!store.hasContent"
          >
            去空行
          </button>
          <button
            @click="applyQuickRule('trim_whitespace')"
            class="px-3 py-1.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
            :disabled="!store.hasContent"
          >
            去首尾空格
          </button>
          <button
            @click="applyQuickRule('collapse_spaces')"
            class="px-3 py-1.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
            :disabled="!store.hasContent"
          >
            合并空格
          </button>
          <button
            @click="applyQuickRule('cjk_spacing')"
            class="px-3 py-1.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
            :disabled="!store.hasContent"
          >
            中英间距
          </button>
        </div>
      </div>

      <!-- Result Actions -->
      <div class="mt-3" v-if="store.panelMode === 'result'">
        <div class="flex gap-2">
          <button
            @click="confirmAndClose"
            class="flex-1 px-3 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
          >
            确认并粘贴
          </button>
          <button
            @click="store.reset(); store.refreshClipboard()"
            class="px-3 py-2 text-sm bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
          >
            重置
          </button>
        </div>
      </div>

      <!-- Error Display -->
      <div v-if="store.errorMessage" class="error-section mt-3 p-2 bg-red-50 dark:bg-red-900/20 rounded-lg">
        <span class="text-sm text-red-600 dark:text-red-400">{{ store.errorMessage }}</span>
      </div>
    </div>

    <!-- Footer: Status -->
    <div class="px-4 py-2 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
      <div class="text-xs text-gray-400">
        Ctrl+Shift+V 呼出 · ESC {{ store.isProcessing ? '取消' : '关闭' }}
      </div>
      <div class="flex items-center gap-2">
        <span
          v-if="store.privacyStatus.type !== 'local'"
          class="text-xs px-2 py-0.5 rounded-full"
          :class="store.privacyStatus.type === 'cloud-safe' ? 'bg-green-100 text-green-600 dark:bg-green-900/30 dark:text-green-400' : 'bg-yellow-100 text-yellow-600 dark:bg-yellow-900/30 dark:text-yellow-400'"
        >
          {{ store.privacyStatus.type === 'cloud-safe' ? '云端安全' : `已脱敏 ${store.privacyStatus.maskedCount}项` }}
        </span>
        <span class="text-xs text-gray-500">FlowPaste v0.1.0</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.floating-panel {
  animation: fadeIn 150ms ease-out;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.98);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.preview-section pre::-webkit-scrollbar {
  width: 4px;
}

.preview-section pre::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 2px;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
