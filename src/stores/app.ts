import { defineStore } from 'pinia'
import { ref } from 'vue'
import type {
  ActionChip,
  PrivacyStatus,
  PanelMode,
  MaskMapping,
} from '@/types'

export const useAppStore = defineStore('app', () => {
  // Panel State
  const isVisible = ref(false)
  const panelMode = ref<PanelMode>('idle')

  // Content
  const clipboardContent = ref('')
  const processedContent = ref('')
  const streamingContent = ref('')

  // AI
  const actionChips = ref<ActionChip[]>([])
  const selectedChipIndex = ref(0)
  const currentRequestId = ref<string | null>(null)

  // Privacy
  const privacyStatus = ref<PrivacyStatus>({ type: 'local' })
  const maskedMapping = ref<MaskMapping>({ mappings: {} })

  // Error
  const errorMessage = ref<string | null>(null)

  // Actions
  function showPanel() {
    isVisible.value = true
    panelMode.value = 'preview'
  }

  function hidePanel() {
    isVisible.value = false
    panelMode.value = 'idle'
    errorMessage.value = null
  }

  function setClipboardContent(content: string) {
    clipboardContent.value = content
    if (content) {
      panelMode.value = 'preview'
    }
  }

  function startProcessing() {
    panelMode.value = 'processing'
    streamingContent.value = ''
    errorMessage.value = null
  }

  function appendStreamContent(content: string) {
    streamingContent.value += content
  }

  function finishProcessing(result: string) {
    processedContent.value = result
    panelMode.value = 'result'
    streamingContent.value = ''
  }

  function setError(message: string) {
    errorMessage.value = message
    panelMode.value = 'preview'
  }

  function reset() {
    clipboardContent.value = ''
    processedContent.value = ''
    streamingContent.value = ''
    actionChips.value = []
    selectedChipIndex.value = 0
    privacyStatus.value = { type: 'local' }
    maskedMapping.value = { mappings: {} }
    errorMessage.value = null
    panelMode.value = 'idle'
    currentRequestId.value = null
  }

  return {
    // State
    isVisible,
    panelMode,
    clipboardContent,
    processedContent,
    streamingContent,
    actionChips,
    selectedChipIndex,
    privacyStatus,
    maskedMapping,
    currentRequestId,
    errorMessage,
    // Actions
    showPanel,
    hidePanel,
    setClipboardContent,
    startProcessing,
    appendStreamContent,
    finishProcessing,
    setError,
    reset,
  }
})
