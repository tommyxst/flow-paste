<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, nextTick } from 'vue'
import { useAppStore } from '@/stores/app'
import { useThemeProvider } from '@/composables/useTheme'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Search, Settings, Loader2, RefreshCw, ShieldCheck, ShieldAlert, ClipboardPaste } from 'lucide-vue-next'
import Preview from './Preview.vue'
import QuickActions from './QuickActions.vue'
import RuleSuggestion from './RuleSuggestion.vue'
import SettingsPanel from './SettingsPanel.vue'
import ErrorDisplay from './ErrorDisplay.vue'

const store = useAppStore()

// Initialize theme detection
useThemeProvider()

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

function handleSubmit() {
  if (!commandInput.value.trim()) {
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
}


function handleSettingsClose() {
  showSettings.value = false
  inputRef.value?.focus()
}

function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.key === ',' && e.ctrlKey) {
    e.preventDefault()
    showSettings.value = !showSettings.value
    return
  }

  if (e.key === 'Escape') {
    if (showSettings.value) {
      showSettings.value = false
      return
    }
    if (store.isProcessing) {
      store.cancelAI()
    } else {
      hideWindow()
    }
    return
  }

  if (e.key === 'Enter' && !e.shiftKey && !showSettings.value) {
    if (e.target instanceof HTMLInputElement) {
      e.preventDefault()
      handleSubmit()
    }
  }
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

  unlistenShow = await listen('panel:show', async () => {
    store.reset()
    store.showPanel()
    showSettings.value = false
    inputRef.value?.focus()

    // Performance: Report render completion (code review fix: wait for Vue DOM updates)
    await nextTick()
    requestAnimationFrame(() => invoke('report_render_timestamp'))
  })

  unlistenAIChunk = await listen<{ content: string; done: boolean; requestId: string }>('ai:chunk', (event) => {
    store.handleAIChunk(event.payload)
  })

  unlistenAIError = await listen<{ code: string; message: string; requestId: string }>('ai:error', (event) => {
    store.handleAIError(event.payload)
  })

  window.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  unlistenFocus?.()
  unlistenShow?.()
  unlistenAIChunk?.()
  unlistenAIError?.()
  window.removeEventListener('keydown', handleGlobalKeydown)
})
</script>

<template>
  <div
    v-if="!showSettings"
    class="floating-panel w-full flex flex-col rounded-xl overflow-hidden animate-scale-in"
  >
    <!-- Header Input Area -->
    <div 
      class="flex items-center px-4 py-3 gap-3 border-b border-gray-100/10"
      @mousedown="startDrag"
    >
      <Search class="w-5 h-5 text-[var(--accent-primary)] shrink-0" v-if="!store.isProcessing" />
      <Loader2 class="w-5 h-5 text-[var(--accent-primary)] animate-spin shrink-0" v-else />
      
      <input
        ref="inputRef"
        v-model="commandInput"
        type="text"
        class="flex-1 bg-transparent text-xl text-[var(--text-primary)] placeholder-[var(--text-tertiary)] outline-none min-w-0 font-medium"
        placeholder="Describe format..."
        :disabled="store.isProcessing"
      />
      
      <div class="flex items-center gap-2 shrink-0">
        <div 
           class="px-2 py-1 rounded text-xs font-medium bg-gray-100/50 dark:bg-white/5 text-[var(--text-secondary)] border border-[var(--panel-border)]"
           v-if="store.clipboardText"
        >
          {{ store.clipboardText.length }} chars
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-col flex">
      <!-- Preview / Result Area -->
      <div class="px-4 py-3">
        <Preview
          :content="previewContent"
          :mode="previewMode"
        />
      </div>

      <!-- Quick Actions -->
      <div
        class="px-4 pb-3"
        v-if="!store.isProcessing && store.hasContent"
      >
        <QuickActions />
      </div>
      
      <!-- Result Confirmation Actions -->
      <div class="px-4 pb-3 pt-1 border-t border-[var(--panel-border)] bg-gray-50/30 dark:bg-black/10 flex justify-between items-center" v-if="store.panelMode === 'result'">
         <div class="text-xs text-[var(--text-secondary)]">Press <span class="font-bold">Enter</span> to paste</div>
         <div class="flex gap-2">
            <button
              @click="confirmAndClose()"
              class="p-1.5 hover:bg-[var(--accent-primary)]/10 rounded text-[var(--accent-primary)] transition-colors"
              title="Paste to cursor"
            >
              <ClipboardPaste class="w-4 h-4" />
            </button>
            <button
              @click="store.reset(); store.refreshClipboard()"
              class="p-1.5 hover:bg-black/5 dark:hover:bg-white/10 rounded text-[var(--text-tertiary)] transition-colors"
              title="Reset"
            >
              <RefreshCw class="w-4 h-4" />
            </button>
         </div>
      </div>

      <!-- Rule Suggestion -->
      <div class="px-4 pb-3" v-if="store.panelMode === 'result' && store.ruleSuggestion">
        <RuleSuggestion />
      </div>

      <!-- Footer / Status Bar -->
      <div class="px-4 py-2 bg-gray-50/50 dark:bg-black/20 border-t border-[var(--panel-border)] flex justify-between items-center text-xs text-[var(--text-tertiary)] select-none">
        
        <!-- Privacy Status -->
        <div class="flex items-center gap-1.5" :class="store.privacyStatus.type === 'cloud-safe' ? 'text-emerald-500' : 'text-amber-500'">
          <ShieldCheck v-if="store.privacyStatus.type === 'cloud-safe'" class="w-3.5 h-3.5" />
          <ShieldAlert v-else class="w-3.5 h-3.5" />
          <span class="font-medium">
             {{ store.privacyStatus.type === 'cloud-safe' ? 'Secure Mode' : `PII Masked (${store.privacyStatus.maskedCount})` }}
          </span>
        </div>

        <div class="flex items-center gap-3">
           <button 
             @click="showSettings = true" 
             class="hover:text-[var(--text-primary)] transition-colors flex items-center gap-1"
           >
             <Settings class="w-3.5 h-3.5" />
           </button>
        </div>
      </div>
    </div>

     <!-- Error Display -->
      <ErrorDisplay
        v-if="store.errorInfo"
        :error="store.errorInfo"
        :retry-count="store.lastAction?.retryCount ?? 0"
        :max-retries="store.lastAction?.maxRetries ?? 3"
        @retry="store.retryLastAction()"
        @dismiss="store.clearError()"
        class="absolute bottom-12 left-4 right-4"
      />
  </div>

  <!-- Settings Panel Overlay -->
  <div v-else class="settings-overlay">
    <SettingsPanel @close="handleSettingsClose" />
  </div>
</template>

<style scoped>
.floating-panel {
  background: var(--panel-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--panel-border);
  box-shadow: 0 20px 50px -12px rgba(0, 0, 0, 0.3);
}

.settings-overlay {
  width: 100%;
  height: 100vh;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 16px;
  background: #141414;
  overflow-y: auto;
}
</style>
