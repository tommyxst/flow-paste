import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { commands } from '@/lib/tauri'
import type {
  ActionChip,
  PrivacyStatus,
  PanelMode,
  MaskMapping,
  AIConfig,
  AppConfig,
  ClipboardContent,
} from '@/types'

export const useAppStore = defineStore('app', () => {
  // Panel State
  const isVisible = ref(false)
  const panelMode = ref<PanelMode>('idle')

  // Content
  const clipboardContent = ref<ClipboardContent | null>(null)
  const processedContent = ref('')
  const streamingContent = ref('')

  // AI
  const actionChips = ref<ActionChip[]>([])
  const selectedChipIndex = ref(0)
  const currentRequestId = ref<string | null>(null)

  // Privacy
  const privacyStatus = ref<PrivacyStatus>({ type: 'local' })
  const maskedMapping = ref<MaskMapping>({ mappings: {} })

  // Config
  const config = ref<AppConfig | null>(null)

  // Error
  const errorMessage = ref<string | null>(null)

  // Computed
  const clipboardText = computed(() => clipboardContent.value?.text ?? '')
  const hasContent = computed(() => !!clipboardContent.value?.text)
  const isProcessing = computed(() => panelMode.value === 'processing')

  // Panel Actions
  async function showPanel() {
    isVisible.value = true
    panelMode.value = 'preview'
    await refreshClipboard()
  }

  function hidePanel() {
    if (currentRequestId.value) {
      cancelAI()
    }
    isVisible.value = false
    panelMode.value = 'idle'
    errorMessage.value = null
  }

  // Clipboard Actions
  async function refreshClipboard() {
    try {
      const content = await commands.readClipboard()
      clipboardContent.value = content
      if (content.text) {
        panelMode.value = 'preview'
        await scanPrivacy(content.text)
      }
    } catch (e) {
      setError(`Failed to read clipboard: ${e}`)
    }
  }

  async function scanPrivacy(text: string) {
    try {
      const result = await commands.scanPii(text)
      if (result.hasPii) {
        privacyStatus.value = { type: 'cloud-masked', maskedCount: result.items.length }
      } else {
        privacyStatus.value = { type: 'cloud-safe' }
      }
    } catch (e) {
      privacyStatus.value = { type: 'local' }
    }
  }

  // Rule Processing
  async function processWithRule(ruleId: string) {
    if (!clipboardText.value) return

    startProcessing()
    try {
      const result = await commands.applyRule(clipboardText.value, ruleId)
      finishProcessing(result)
    } catch (e) {
      setError(`Rule processing failed: ${e}`)
    }
  }

  // AI Processing
  async function processWithAI(prompt: string, aiConfig?: Partial<AIConfig>) {
    if (!clipboardText.value) return

    startProcessing()
    const requestId = crypto.randomUUID()
    currentRequestId.value = requestId

    try {
      const fullConfig: AIConfig = {
        provider: config.value?.aiProvider ?? 'Ollama',
        baseUrl: config.value?.aiProvider === 'OpenAI'
          ? config.value?.openaiBaseUrl ?? 'https://api.openai.com/v1'
          : config.value?.ollamaBaseUrl ?? 'http://localhost:11434',
        model: config.value?.modelName ?? 'llama3.2',
        maxTokens: 2048,
        temperature: 0.7,
        ...aiConfig,
      }

      // Get API key if using OpenAI
      if (fullConfig.provider === 'OpenAI') {
        const apiKey = await commands.getApiKey('openai')
        if (apiKey) {
          fullConfig.apiKey = apiKey
        }
      }

      const usePrivacyShield = privacyStatus.value.type === 'cloud-masked'
      const fullPrompt = `${prompt}\n\nContent:\n${clipboardText.value}`

      await commands.sendAiRequest(fullPrompt, fullConfig, requestId, usePrivacyShield)
    } catch (e) {
      setError(`AI request failed: ${e}`)
      currentRequestId.value = null
    }
  }

  async function cancelAI() {
    if (currentRequestId.value) {
      try {
        await commands.cancelAiRequest(currentRequestId.value)
      } catch (e) {
        // Ignore cancel errors
      }
      currentRequestId.value = null
      panelMode.value = 'preview'
    }
  }

  // Paste Action
  async function confirmPaste() {
    const contentToPaste = processedContent.value || clipboardText.value
    if (!contentToPaste) return

    try {
      await commands.writeClipboard(contentToPaste)
      hidePanel()
      reset()
    } catch (e) {
      setError(`Failed to paste: ${e}`)
    }
  }

  // Config Actions
  async function loadConfig() {
    try {
      config.value = await commands.getConfig()
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }

  async function saveConfig(newConfig: AppConfig) {
    try {
      await commands.setConfig(newConfig)
      config.value = newConfig
    } catch (e) {
      setError(`Failed to save config: ${e}`)
    }
  }

  // Internal Actions
  function startProcessing() {
    panelMode.value = 'processing'
    streamingContent.value = ''
    processedContent.value = ''
    errorMessage.value = null
  }

  function appendStreamContent(content: string) {
    streamingContent.value += content
  }

  function finishProcessing(result: string) {
    processedContent.value = result
    panelMode.value = 'result'
    streamingContent.value = ''
    currentRequestId.value = null
  }

  function setError(message: string) {
    errorMessage.value = message
    panelMode.value = 'preview'
    currentRequestId.value = null
  }

  function reset() {
    clipboardContent.value = null
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

  // AI Event Handlers (to be called from component setup)
  function handleAIChunk(payload: { content: string; done: boolean; requestId: string }) {
    if (payload.requestId !== currentRequestId.value) return

    if (payload.done) {
      finishProcessing(payload.content)
    } else {
      appendStreamContent(payload.content)
    }
  }

  function handleAIError(payload: { code: string; message: string; requestId: string }) {
    if (payload.requestId !== currentRequestId.value) return
    setError(`AI Error [${payload.code}]: ${payload.message}`)
  }

  return {
    // State
    isVisible,
    panelMode,
    clipboardContent,
    clipboardText,
    processedContent,
    streamingContent,
    actionChips,
    selectedChipIndex,
    privacyStatus,
    maskedMapping,
    currentRequestId,
    errorMessage,
    config,
    // Computed
    hasContent,
    isProcessing,
    // Panel Actions
    showPanel,
    hidePanel,
    // Clipboard Actions
    refreshClipboard,
    // Processing Actions
    processWithRule,
    processWithAI,
    cancelAI,
    confirmPaste,
    // Config Actions
    loadConfig,
    saveConfig,
    // Internal Actions
    startProcessing,
    appendStreamContent,
    finishProcessing,
    setError,
    reset,
    // Event Handlers
    handleAIChunk,
    handleAIError,
  }
})
