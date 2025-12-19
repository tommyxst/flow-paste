<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useAppStore } from '@/stores/app'
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'

const store = useAppStore()

const inputRef = ref<HTMLInputElement | null>(null)
const commandInput = ref('')
const isLoading = ref(false)
const isDragging = ref(false)
let unlistenFocus: (() => void) | null = null
let unlistenShow: (() => void) | null = null

const previewLines = computed(() => {
  if (!store.clipboardContent) return '（无剪贴板内容）'
  const lines = store.clipboardContent.split('\n')
  return lines.slice(0, 5).join('\n') + (lines.length > 5 ? `\n... (共 ${lines.length} 行)` : '')
})

const charCount = computed(() => store.clipboardContent.length)

async function loadClipboard() {
  isLoading.value = true
  store.errorMessage = null
  try {
    console.log('Reading clipboard...')
    const text = await readText()
    console.log('Clipboard content:', text ? `${text.length} chars` : 'empty')
    store.setClipboardContent(text || '')
  } catch (err: unknown) {
    const errorMsg = err instanceof Error ? err.message : String(err)
    console.error('Failed to read clipboard:', errorMsg)
    store.setError(`剪贴板读取失败: ${errorMsg}`)
  } finally {
    isLoading.value = false
  }
}

async function copyToClipboard(text: string) {
  try {
    await writeText(text)
    store.setClipboardContent(text)
  } catch (err) {
    console.error('Failed to write clipboard:', err)
    store.setError('无法写入剪贴板')
  }
}

async function hideWindow() {
  const appWindow = getCurrentWindow()
  await appWindow.hide()
}

async function startDrag() {
  isDragging.value = true
  const appWindow = getCurrentWindow()
  await appWindow.startDragging()
  // Reset after a short delay to allow focus events to settle
  setTimeout(() => { isDragging.value = false }, 200)
}

async function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    await hideWindow()
  } else if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSubmit()
  }
}

// Text processing rules
const rules = {
  removeEmptyLines: (text: string) => text.split('\n').filter(line => line.trim()).join('\n'),
  trimLines: (text: string) => text.split('\n').map(line => line.trim()).join('\n'),
  toUpperCase: (text: string) => text.toUpperCase(),
  toLowerCase: (text: string) => text.toLowerCase(),
}

async function applyRule(ruleName: keyof typeof rules) {
  if (!store.clipboardContent) return
  const processed = rules[ruleName](store.clipboardContent)
  store.processedContent = processed
  store.panelMode = 'result'
  await copyToClipboard(processed)
}

function handleSubmit() {
  if (!commandInput.value.trim()) return
  // TODO: Parse natural language commands
  console.log('Submit:', commandInput.value)
}

onMounted(async () => {
  console.log('FloatingPanel mounted')
  inputRef.value?.focus()
  store.showPanel()
  await loadClipboard()

  const appWindow = getCurrentWindow()

  // Listen for window focus changes
  unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      console.log('Window focused, refreshing clipboard')
      loadClipboard()
      inputRef.value?.focus()
    } else if (!isDragging.value) {
      console.log('Window lost focus, hiding')
      appWindow.hide()
    }
  })

  // Listen for panel:show event from global shortcut
  unlistenShow = await listen('panel:show', () => {
    console.log('Panel show event received')
    store.reset()
    store.showPanel()
    loadClipboard()
    inputRef.value?.focus()
  })

  // Global keyboard listener for ESC
  document.addEventListener('keydown', handleGlobalKeydown)
})

function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    console.log('ESC pressed, hiding window')
    hideWindow()
  }
}

onUnmounted(() => {
  unlistenFocus?.()
  unlistenShow?.()
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
        placeholder="输入处理指令（如：去空行、转大写）..."
      />
    </div>

    <!-- Content Area -->
    <div class="flex-1 px-4 py-2 overflow-hidden flex flex-col">
      <!-- Preview -->
      <div class="preview-section flex-1 min-h-0">
        <div class="flex items-center justify-between mb-2">
          <div class="text-xs text-gray-500">
            {{ store.panelMode === 'result' ? '处理结果（已复制）' : '剪贴板内容预览' }}
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs text-gray-400">{{ charCount }} 字符</span>
            <button
              @click="loadClipboard"
              class="text-xs text-blue-500 hover:text-blue-600"
              :disabled="isLoading"
            >
              {{ isLoading ? '读取中...' : '刷新' }}
            </button>
          </div>
        </div>
        <pre class="text-sm text-gray-700 dark:text-gray-300 font-mono whitespace-pre-wrap break-words h-full max-h-32 overflow-y-auto bg-gray-50 dark:bg-gray-800/50 p-3 rounded-lg">{{ store.panelMode === 'result' ? store.processedContent : previewLines }}</pre>
      </div>

      <!-- Quick Actions -->
      <div class="mt-3">
        <div class="text-xs text-gray-500 mb-2">快捷操作</div>
        <div class="flex flex-wrap gap-2">
          <button
            @click="applyRule('removeEmptyLines')"
            class="px-3 py-1.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
            :disabled="!store.clipboardContent"
          >
            去空行
          </button>
          <button
            @click="applyRule('trimLines')"
            class="px-3 py-1.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
            :disabled="!store.clipboardContent"
          >
            去首尾空格
          </button>
          <button
            @click="applyRule('toUpperCase')"
            class="px-3 py-1.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
            :disabled="!store.clipboardContent"
          >
            转大写
          </button>
          <button
            @click="applyRule('toLowerCase')"
            class="px-3 py-1.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
            :disabled="!store.clipboardContent"
          >
            转小写
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
        Ctrl+Shift+V 呼出 · ESC 关闭
      </div>
      <div class="text-xs text-gray-500">
        FlowPaste v0.1.0
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
