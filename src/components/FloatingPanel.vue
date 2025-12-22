<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useAppStore } from '@/stores/app'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import Preview from './Preview.vue'
import ActionChips from './ActionChips.vue'
import SettingsPanel from './SettingsPanel.vue'
import type { ActionChip } from '@/types'

const store = useAppStore()

const inputRef = ref<HTMLInputElement | null>(null)
const commandInput = ref('')
const isDragging = ref(false)
const showSettings = ref(false)
let unlistenFocus: (() => void) | null = null
let unlistenShow: (() => void) | null = null
let unlistenAIChunk: (() => void) | null = null
let unlistenAIError: (() => void) | null = null

const previewMode = computed(() => {
  if (store.panelMode === 'processing') return 'streaming'
  if (store.panelMode === 'result') return 'result'
  return 'preview'
})

const previewContent = computed(() => {
  if (store.panelMode === 'processing') return store.streamingContent
  if (store.panelMode === 'result') return store.processedContent
  return store.clipboardText
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
    if (showSettings.value) {
      showSettings.value = false
      return
    }
    if (store.isProcessing) {
      store.cancelAI()
    } else {
      await hideWindow()
    }
  } else if (e.key === 'Enter' && !e.shiftKey && !showSettings.value) {
    // Only handle Enter if not from input field (avoid double submit)
    if (e.target instanceof HTMLInputElement) {
      e.preventDefault()
      handleSubmit()
    }
  } else if (e.key === ',' && e.ctrlKey) {
    e.preventDefault()
    showSettings.value = !showSettings.value
  }
}

function handleSubmit() {
  if (!commandInput.value.trim()) {
    // If no command, confirm paste
    if (store.panelMode === 'result') {
      confirmAndClose()
    }
    return
  }
  store.processWithAI(commandInput.value)
  commandInput.value = ''
}

async function confirmAndClose() {
  await store.confirmPaste()
  await hideWindow()
}

function handleChipSelect(chip: ActionChip) {
  if (chip.actionType === 'LocalRule') {
    store.processWithRule(chip.payload)
  } else {
    store.processWithAI(chip.payload)
  }
}

function handleSettingsClose() {
  showSettings.value = false
  inputRef.value?.focus()
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
    } else if (!isDragging.value && !store.isProcessing && !showSettings.value) {
      appWindow.hide()
    }
  })

  unlistenShow = await listen('panel:show', () => {
    store.reset()
    store.showPanel()
    showSettings.value = false
    inputRef.value?.focus()
  })

  unlistenAIChunk = await listen<{ content: string; done: boolean; requestId: string }>('ai:chunk', (event) => {
    store.handleAIChunk(event.payload)
  })

  unlistenAIError = await listen<{ code: string; message: string; requestId: string }>('ai:error', (event) => {
    store.handleAIError(event.payload)
  })
})

onUnmounted(() => {
  unlistenFocus?.()
  unlistenShow?.()
  unlistenAIChunk?.()
  unlistenAIError?.()
})
</script>

<template>
  <div
    v-if="!showSettings"
    class="floating-panel w-full h-full flex flex-col rounded-xl overflow-hidden"
    style="background: var(--panel-bg); border: 1px solid var(--panel-border)"
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
        @keydown="handleKeydown"
      />
    </div>

    <!-- Content Area -->
    <div class="flex-1 px-4 py-2 overflow-hidden flex flex-col">
      <!-- Preview -->
      <div class="preview-section flex-1 min-h-0 mb-3">
        <div class="flex items-center justify-between mb-2">
          <div class="text-xs text-gray-500">
            <template v-if="store.panelMode === 'processing'">AI 处理中...</template>
            <template v-else-if="store.panelMode === 'result'">处理结果</template>
            <template v-else>剪贴板内容</template>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs text-gray-400">{{ store.clipboardText.length }} 字符</span>
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
        <Preview
          :content="previewContent"
          :mode="previewMode"
        />
      </div>

      <!-- Action Chips -->
      <ActionChips
        v-if="!store.isProcessing && store.actionChips.length > 0"
        :chips="store.actionChips"
        :selected-index="store.selectedChipIndex"
        @select="handleChipSelect"
        class="mb-3"
      />

      <!-- Result Actions -->
      <div class="mb-3" v-if="store.panelMode === 'result'">
        <div class="flex gap-2">
          <button
            @click="confirmAndClose"
            class="flex-1 px-3 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
          >
            确认并粘贴 (Enter)
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
      <div v-if="store.errorMessage" class="mb-3 p-2 bg-red-50 dark:bg-red-900/20 rounded-lg animate-shake">
        <span class="text-sm text-red-600 dark:text-red-400">{{ store.errorMessage }}</span>
      </div>
    </div>

    <!-- Footer: Status -->
    <div class="px-4 py-2 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
      <div class="text-xs text-gray-400">
        {{ store.isProcessing ? 'ESC 取消' : 'ESC 关闭' }} · Ctrl+, 设置
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

  <!-- Settings Panel Overlay -->
  <div v-else class="w-full h-full flex items-center justify-center bg-gray-900/50">
    <SettingsPanel @close="handleSettingsClose" />
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

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-4px); }
  75% { transform: translateX(4px); }
}

.animate-shake {
  animation: shake 0.3s ease-in-out;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
