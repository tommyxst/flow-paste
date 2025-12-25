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
  ActionSnapshot,
  ErrorInfo,
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

  // Error & Retry (improved from review)
  const errorInfo = ref<ErrorInfo | null>(null)
  const lastAction = ref<ActionSnapshot | null>(null)

  // Computed
  const clipboardText = computed(() => clipboardContent.value?.text ?? '')
  const hasContent = computed(() => !!clipboardContent.value?.text)
  const isProcessing = computed(() => panelMode.value === 'processing')
  const errorMessage = computed(() => errorInfo.value?.message ?? null)
  const canRetry = computed(() => {
    if (!lastAction.value || !errorInfo.value?.recoverable) return false
    return lastAction.value.retryCount < lastAction.value.maxRetries
  })

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
    clearError()
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

      // Generate action chips based on content
      const chips = await commands.detectContentIntent(text)
      actionChips.value = chips
    } catch (e) {
      privacyStatus.value = { type: 'local' }
      actionChips.value = []
    }
  }

  // Rule Processing
  async function processWithRule(ruleId: string) {
    if (!clipboardText.value) return

    // Track action for retry
    recordAction('rule', ruleId)

    startProcessing()
    try {
      const result = await commands.applyRule(clipboardText.value, ruleId)
      finishProcessing(result)
    } catch (e) {
      setError(`Rule processing failed: ${e}`, true)
    }
  }

  // AI Processing
  async function processWithAI(prompt: string, aiConfig?: Partial<AIConfig>) {
    if (!clipboardText.value) return

    // Track action for retry
    recordAction('ai', prompt)

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
      setError(`AI request failed: ${e}`, true)
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
      // Use paste-to-cursor with keyboard simulation
      const result = await commands.pasteToCursor(contentToPaste)

      if (result.success) {
        if (!result.usedSimulation && result.message) {
          // Fallback was used, show info message
          setError(result.message, false)
          // Still hide after a delay since content is in clipboard
          setTimeout(() => {
            hidePanel()
            reset()
          }, 1500)
        } else {
          // Full success with simulation
          hidePanel()
          reset()
        }
      } else {
        // Complete failure
        setError(result.message || 'Failed to paste', false)
      }
    } catch (e) {
      setError(`Failed to paste: ${e}`, false)
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
    clearError()
  }

  function appendStreamContent(content: string) {
    streamingContent.value += content
  }

  function finishProcessing(result: string) {
    processedContent.value = result
    panelMode.value = 'result'
    streamingContent.value = ''
    currentRequestId.value = null
    // Clear action tracking on success
    lastAction.value = null
  }

  // Action tracking for retry mechanism
  function recordAction(type: 'ai' | 'rule', payload: string) {
    lastAction.value = {
      type,
      payload,
      timestamp: Date.now(),
      retryCount: lastAction.value?.payload === payload ? (lastAction.value.retryCount) : 0,
      maxRetries: 3,
    }
  }

  function setError(message: string, recoverable = false) {
    errorInfo.value = {
      message,
      recoverable,
      severity: 'error',
    }
    panelMode.value = 'preview'
    currentRequestId.value = null

    // Update last action with error
    if (lastAction.value) {
      lastAction.value.lastError = message
    }
  }

  function clearError() {
    errorInfo.value = null
  }

  // Retry mechanism with exponential backoff
  async function retryLastAction() {
    if (!lastAction.value || !canRetry.value) return

    clearError()
    lastAction.value.retryCount++

    // Exponential backoff: 1s, 2s, 4s
    const backoffMs = 1000 * Math.pow(2, lastAction.value.retryCount - 1)
    await new Promise(resolve => setTimeout(resolve, Math.min(backoffMs, 4000)))

    if (lastAction.value.type === 'ai') {
      await processWithAI(lastAction.value.payload)
    } else if (lastAction.value.type === 'rule') {
      await processWithRule(lastAction.value.payload)
    }
  }

  function reset() {
    clipboardContent.value = null
    processedContent.value = ''
    streamingContent.value = ''
    actionChips.value = []
    selectedChipIndex.value = 0
    privacyStatus.value = { type: 'local' }
    maskedMapping.value = { mappings: {} }
    errorInfo.value = null
    lastAction.value = null
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

    // AI errors are typically recoverable (network, timeout, etc.)
    // Only non-recoverable for config/model issues
    const nonRecoverableCodes = ['invalid_config', 'model_not_found', 'invalid_api_key']
    const isRecoverable = !nonRecoverableCodes.includes(payload.code)

    setError(`AI Error [${payload.code}]: ${payload.message}`, isRecoverable)
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
    errorInfo,
    lastAction,
    config,
    // Computed
    hasContent,
    isProcessing,
    canRetry,
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
    clearError,
    retryLastAction,
    reset,
    // Event Handlers
    handleAIChunk,
    handleAIError,
  }
})
