import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
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
  CustomRule,
  Rule,
  RuleSuggestion,
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
  const builtinRules = ref<Rule[]>([])

  // Error & Retry (improved from review)
  const errorInfo = ref<ErrorInfo | null>(null)
  const lastAction = ref<ActionSnapshot | null>(null)

  // AI Rule Learning
  const ruleSuggestion = ref<RuleSuggestion | null>(null)

  // Computed
  const clipboardText = computed(() => clipboardContent.value?.text ?? '')
  const hasContent = computed(() => !!clipboardContent.value?.text)
  const isProcessing = computed(() => panelMode.value === 'processing')
  const errorMessage = computed(() => errorInfo.value?.message ?? null)
  const canRetry = computed(() => {
    if (!lastAction.value || !errorInfo.value?.recoverable) return false
    return lastAction.value.retryCount < lastAction.value.maxRetries
  })

  // All rules (builtin + custom)
  const allRules = computed(() => {
    const custom = config.value?.customRules ?? []
    return [...builtinRules.value, ...custom]
  })

  // Visible rules (top 3 pinned)
  const visibleRules = computed(() => {
    const pinnedIds = config.value?.pinnedRuleIds ?? []
    return pinnedIds
      .slice(0, 3)
      .map(id => allRules.value.find(r => r.id === id))
      .filter((r): r is Rule => !!r)
  })

  // Overflow rules (not in top 3)
  const overflowRules = computed(() => {
    const pinnedIds = config.value?.pinnedRuleIds ?? []
    const visibleIds = new Set(pinnedIds.slice(0, 3))
    return allRules.value.filter(r => !visibleIds.has(r.id))
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
    // Use processed text if available, support chain formatting (Bug 3 fix)
    const baseText = processedContent.value || clipboardText.value
    if (!baseText) return

    // Guard against concurrent clicks (Code review fix)
    if (isProcessing.value) return

    // Track action for retry
    recordAction('rule', ruleId)

    startProcessing()
    try {
      const result = await commands.applyRule(baseText, ruleId)
      finishProcessing(result)

      // Write to clipboard first
      clipboardContent.value = { kind: 'text', text: result }
      await commands.writeClipboard(result)

      // Hide window and wait for focus to transfer before pasting
      const appWindow = getCurrentWindow()
      await appWindow.hide()
      isVisible.value = false

      // Wait for window to fully hide and focus to transfer
      await new Promise(resolve => setTimeout(resolve, 150))

      // Now paste to cursor
      const pasteResult = await commands.pasteToCursor(result)
      if (!pasteResult.success) {
        // Show window again to display error
        await appWindow.show()
        isVisible.value = true
        setError(pasteResult.message || 'Failed to paste', false)
      } else {
        reset()
      }
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
        console.log('[AI] Fetching API key for OpenAI provider...')
        try {
          const apiKey = await commands.getApiKey('openai')
          console.log('[AI] API key result:', apiKey ? 'found (len=' + apiKey.length + ')' : 'NOT FOUND')
          if (apiKey) {
            fullConfig.apiKey = apiKey
          } else {
            console.warn('[AI] No API key found in keyring!')
          }
        } catch (e) {
          console.error('[AI] Failed to get API key:', e)
        }
      }

      const usePrivacyShield = privacyStatus.value.type === 'cloud-masked'

      // Build prompt with optional rule learning
      let fullPrompt = `${prompt}\n\nContent:\n${clipboardText.value}`
      if (config.value?.enableAIRuleLearning) {
        fullPrompt += `\n\n---\nAfter completing the task, evaluate if this transformation can be automated as a reusable rule.
If yes, append a JSON block at the end in this exact format:
\`\`\`rule
{"canBeRule":true,"confidence":0.9,"name":"Rule Name","pattern":"regex pattern","replacement":"replacement string","transformationType":"regex_replace"}
\`\`\`
Only include this if confidence >= 0.8. Valid transformationType: regex_replace, json_format, json_minify, sort_lines, dedupe_lines.`
      }

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
      builtinRules.value = await commands.getBuiltinRules()
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

  // Rule Management
  async function reorderRules(newOrder: string[]) {
    if (!config.value) return
    await saveConfig({ ...config.value, pinnedRuleIds: newOrder })
  }

  async function pinRule(ruleId: string) {
    if (!config.value) return
    if (config.value.pinnedRuleIds.includes(ruleId)) return

    // We allow more than 3 in the list, UI decides how to show them
    const newPinned = [...config.value.pinnedRuleIds, ruleId]
    await saveConfig({ ...config.value, pinnedRuleIds: newPinned })
  }

  async function unpinRule(ruleId: string) {
    if (!config.value) return
    const newPinned = config.value.pinnedRuleIds.filter(id => id !== ruleId)
    await saveConfig({ ...config.value, pinnedRuleIds: newPinned })
  }

  async function saveCustomRule(rule: CustomRule) {
    if (!config.value) return
    // Check if ID exists
    const exists = config.value.customRules.some(r => r.id === rule.id)
    if (exists) {
      // Update existing
      const newRules = config.value.customRules.map(r => r.id === rule.id ? rule : r)
      await saveConfig({ ...config.value, customRules: newRules })
    } else {
      // Add new
      const newRules = [...config.value.customRules, rule]
      await saveConfig({ ...config.value, customRules: newRules })
    }
  }

  async function deleteCustomRule(ruleId: string) {
    if (!config.value) return
    const newRules = config.value.customRules.filter(r => r.id !== ruleId)
    // Also remove from pinned if present
    const newPinned = config.value.pinnedRuleIds.filter(id => id !== ruleId)

    await saveConfig({
      ...config.value,
      customRules: newRules,
      pinnedRuleIds: newPinned
    })
  }

  async function saveRuleSuggestion() {
    if (!ruleSuggestion.value || !config.value) return
    const s = ruleSuggestion.value
    const newRule: CustomRule = {
      id: `ai_${Date.now()}`,
      name: s.name,
      description: `AI generated rule`,
      pattern: s.pattern,
      replacement: s.replacement,
      transformationType: s.transformationType,
      isBuiltin: false,
      createdBy: 'ai',
      createdAt: Date.now(),
      usageCount: 0,
    }
    await saveCustomRule(newRule)
    await pinRule(newRule.id)
    ruleSuggestion.value = null
  }

  function dismissRuleSuggestion() {
    ruleSuggestion.value = null
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
    ruleSuggestion.value = null
    panelMode.value = 'idle'
    currentRequestId.value = null
  }

  // AI Event Handlers (to be called from component setup)
  function handleAIChunk(payload: { content: string; done: boolean; requestId: string }) {
    if (payload.requestId !== currentRequestId.value) return

    if (payload.done) {
      // Parse rule suggestion if present
      const ruleMatch = payload.content.match(/```rule\s*([\s\S]*?)```/)
      if (ruleMatch) {
        try {
          const suggestion = JSON.parse(ruleMatch[1].trim()) as RuleSuggestion
          if (suggestion.canBeRule && suggestion.confidence >= 0.8) {
            ruleSuggestion.value = suggestion
          }
        } catch { /* ignore parse errors */ }
        // Remove rule block from displayed content
        const cleanContent = payload.content.replace(/\n*---\n*```rule[\s\S]*?```\s*$/, '').trim()
        finishProcessing(cleanContent)
      } else {
        finishProcessing(payload.content)
      }
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
    builtinRules,
    ruleSuggestion,
    // Computed
    hasContent,
    isProcessing,
    canRetry,
    allRules,
    visibleRules,
    overflowRules,
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
    reorderRules,
    pinRule,
    unpinRule,
    saveCustomRule,
    deleteCustomRule,
    saveRuleSuggestion,
    dismissRuleSuggestion,
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
