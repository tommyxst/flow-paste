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
  RuleValidationResult,
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
  const isSavingRule = ref(false)
  const ruleValidationError = ref<RuleValidationResult | null>(null)

  function normalizeCustomRule(rule: CustomRule): CustomRule {
    // customRules 来源于配置，语义上必然不是内置规则；这里做一次防御性归一化，避免错误持久化导致走 apply_rule
    return { ...rule, isBuiltin: false }
  }

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
      // 查找规则：区分内置规则和自定义规则
      const rule = allRules.value.find(r => r.id === ruleId)
      if (!rule) {
        throw new Error(`Rule not found: ${ruleId}`)
      }

      // 通过检查 ID 是否在内置规则列表中判定，不信任 isBuiltin 属性（防止数据污染）
      const isRealBuiltin = builtinRules.value.some(br => br.id === ruleId)

      let result: string
      if (isRealBuiltin) {
        try {
          result = await commands.applyRule(baseText, ruleId)
        } catch (e) {
          // 防御：若后端返回 rule not found，说明并非真实内置规则（可能是旧数据/污染）；回退为自定义规则执行
          const message = String(e)
          if (message.includes('rule not found')) {
            result = await commands.applyCustomRule(baseText, { ...rule, isBuiltin: false })
          } else {
            throw e
          }
        }
      } else {
        result = await commands.applyCustomRule(baseText, { ...rule, isBuiltin: false })
      }
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

      // Build prompt with system instruction for clean output
      let fullPrompt = `IMPORTANT: Output ONLY the processed result. No explanations, no descriptions, no markdown formatting, no code blocks unless the result itself is code. Just the transformed text ready to paste.

Task: ${prompt}

Content:
${clipboardText.value}`
      if (config.value?.enableAIRuleLearning) {
        fullPrompt += `

---
After completing the task, evaluate if this transformation can be automated as a reusable rule.

COMPATIBILITY CONTRACT (CRITICAL - Rust regex engine):
- Pattern: Rust regex syntax ONLY. NO lookahead (?=), NO lookbehind (?<=), NO backreferences.
- Replacement: ONLY supports $1, $2, \${name} for captures. NO \\U, \\L, \\E case modifiers!

DECISION LOGIC:
1. For case conversion (upper/lower) → USE "to_uppercase" or "to_lowercase", NOT regex_replace
2. For JSON formatting → USE "json_format" or "json_minify"
3. For line operations → USE "sort_lines" or "dedupe_lines"
4. For pattern-based find/replace → USE "regex_replace" with compatible syntax

EXAMPLES:
✅ CORRECT: {"canBeRule":true,"confidence":0.95,"name":"Convert to Uppercase","pattern":"","replacement":"","transformationType":"to_uppercase"}
✅ CORRECT: {"canBeRule":true,"confidence":0.9,"name":"Remove digits","pattern":"\\\\d+","replacement":"","transformationType":"regex_replace"}
❌ WRONG: {"transformationType":"regex_replace","pattern":"(.+)","replacement":"\\\\U$1"} (\\U not supported!)
❌ WRONG: {"transformationType":"regex_replace","pattern":"(?=foo)bar"} (lookahead not supported!)

If unsure about compatibility, DO NOT output a rule block.

Output format (only if confidence >= 0.8):
\`\`\`rule
{"canBeRule":true,"confidence":0.9,"name":"Rule Name","pattern":"regex pattern","replacement":"replacement","transformationType":"<type>"}
\`\`\`
Valid types: regex_replace, json_format, json_minify, sort_lines, dedupe_lines, to_uppercase, to_lowercase`
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
      // Hide window first to return focus to the previous app before simulating paste
      const appWindow = getCurrentWindow()
      await appWindow.hide()
      isVisible.value = false

      // Wait a moment for focus to transfer
      await new Promise(resolve => setTimeout(resolve, 150))

      // Use paste-to-cursor (may fall back to clipboard-only)
      const result = await commands.pasteToCursor(contentToPaste)

      if (result.success) {
        // Always close panel on success (even clipboard-only fallback)
        if (!result.usedSimulation && result.message) {
          console.warn('[Paste] Simulation not used:', result.message)
        }
        hidePanel()
        reset()
      } else {
        // Complete failure: show window again so user can see the error
        await appWindow.show()
        isVisible.value = true
        setError(result.message || 'Failed to paste', false)
      }
    } catch (e) {
      // Unexpected errors: show window again so user can see the error
      try {
        const appWindow = getCurrentWindow()
        await appWindow.show()
        isVisible.value = true
      } catch {
        // ignore
      }
      setError(`Failed to paste: ${e}`, false)
    }
  }

  // Config Actions
  async function loadConfig() {
    try {
      const loaded = await commands.getConfig()
      config.value = {
        ...loaded,
        customRules: (loaded.customRules ?? []).map(normalizeCustomRule),
      }
      builtinRules.value = await commands.getBuiltinRules()
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }

  async function saveConfig(newConfig: AppConfig) {
    try {
      const normalized: AppConfig = {
        ...newConfig,
        customRules: (newConfig.customRules ?? []).map(normalizeCustomRule),
      }
      await commands.setConfig(normalized)
      config.value = normalized
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
    const normalizedRule = normalizeCustomRule(rule)
    // Check if ID exists
    const exists = config.value.customRules.some(r => r.id === normalizedRule.id)
    if (exists) {
      // Update existing
      const newRules = config.value.customRules.map(r => r.id === normalizedRule.id ? normalizedRule : r)
      await saveConfig({ ...config.value, customRules: newRules })
    } else {
      // Add new
      const newRules = [...config.value.customRules, normalizedRule]
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

  async function saveRuleSuggestion(): Promise<boolean> {
    if (!ruleSuggestion.value || !config.value) return false
    if (isSavingRule.value) return false // 防止重复提交

    // 清除之前的验证错误
    ruleValidationError.value = null
    isSavingRule.value = true

    try {
      const s = ruleSuggestion.value
      const newRule: Rule = {
        id: `ai_${Date.now()}`,
        name: s.name,
        description: `AI generated rule`,
        pattern: s.pattern,
        replacement: s.replacement,
        transformationType: s.transformationType,
        isBuiltin: false,
        origin: 'ai',
        enabled: true,
      }

      // 调用后端验证并保存
      const result = await commands.upsertRule(newRule)
      if (!result.valid) {
        ruleValidationError.value = result
        return false
      }

      // 验证通过，刷新配置并固定规则
      await loadConfig()
      await pinRule(newRule.id)
      ruleSuggestion.value = null
      return true
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      ruleValidationError.value = {
        valid: false,
        errors: [message],
        warnings: [],
      }
      return false
    } finally {
      isSavingRule.value = false
    }
  }

  function dismissRuleSuggestion() {
    ruleSuggestion.value = null
    ruleValidationError.value = null
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
      // Parse rule suggestion if present (支持多种格式)
      // 格式1: ```rule\n{JSON}\n``` (标准 Markdown)
      // 格式2: rule\n{JSON} (无反引号，AI 常见输出)
      // 格式3: ---\nrule\n{JSON} (带分隔符)
      const rulePatterns = [
        /```rule\s*([\s\S]*?)```/,                    // 标准 Markdown 代码块
        /(?:^|\n)rule\s*\n?(\{[\s\S]*?"canBeRule"[\s\S]*?\})/,  // 无反引号，匹配包含 canBeRule 的 JSON
        /---\s*\n?rule\s*\n?(\{[\s\S]*?\})\s*$/,      // 带分隔符格式
      ]

      let ruleMatch: RegExpMatchArray | null = null
      for (const pattern of rulePatterns) {
        ruleMatch = payload.content.match(pattern)
        if (ruleMatch) {
          console.log('[AI Rule] Matched with pattern:', pattern)
          break
        }
      }

      if (ruleMatch) {
        try {
          let suggestion = JSON.parse(ruleMatch[1].trim()) as RuleSuggestion
          if (suggestion.canBeRule && suggestion.confidence >= 0.8) {
            // 尝试自动修复/转换不兼容的规则
            suggestion = autoFixIncompatibleRule(suggestion)

            // 验证 transformationType 有效性（优先检查）
            if (!isValidTransformationType(suggestion.transformationType)) {
              console.warn('[AI Rule] Invalid transformation type:', suggestion.transformationType)
            }
            // 检查不支持的 replacement 语法
            else if (suggestion.transformationType === 'regex_replace' && hasIncompatibleReplacementSyntax(suggestion.replacement)) {
              console.warn('[AI Rule] Incompatible replacement syntax detected, skipping suggestion')
            }
            // 按类型分流校验：仅 regex_replace 需要验证 pattern
            else if (suggestion.transformationType === 'regex_replace' && !validateRulePattern(suggestion.pattern)) {
              console.warn('[AI Rule] Invalid pattern for regex_replace, skipping suggestion')
            }
            // 检查重复规则（考虑类型）
            else if (isDuplicateRule(suggestion)) {
              console.log('[AI Rule] Duplicate rule detected, skipping suggestion')
            }
            else {
              ruleSuggestion.value = suggestion
            }
          }
        } catch (e) {
          console.warn('[AI Rule] Failed to parse suggestion:', e)
        }
        // Remove rule block from displayed content (匹配多种格式)
        let cleanContent = payload.content
          .replace(/\n*---\n*```rule[\s\S]*?```\s*$/, '')       // 标准格式 with ---
          .replace(/```rule[\s\S]*?```\s*$/, '')               // 标准格式 without ---
          .replace(/\n*---\n*rule\s*\n?\{[\s\S]*?\}\s*$/, '')  // 无反引号 with ---
          .replace(/\nrule\s*\n?\{[\s\S]*?"canBeRule"[\s\S]*?\}\s*$/, '') // 无反引号 without ---
          .trim()
        finishProcessing(cleanContent)
      } else {
        finishProcessing(payload.content)
      }
    } else {
      appendStreamContent(payload.content)
    }
  }

  // 有效的转换类型常量
  const VALID_TRANSFORM_TYPES = ['regex_replace', 'json_format', 'json_minify', 'sort_lines', 'dedupe_lines', 'to_uppercase', 'to_lowercase'] as const

  // 检测不支持的 replacement 语法（Rust regex 不支持 \U, \L, \E 等）
  function hasIncompatibleReplacementSyntax(replacement: string): boolean {
    if (!replacement) return false
    // 检测 \U, \L, \E 等 Perl 风格的大小写转换语法
    // 也检测 \1 风格的引用（Rust 用 $1）
    return /\\[ULE]|\\[0-9]/.test(replacement)
  }

  // 自动修复/转换不兼容的规则
  function autoFixIncompatibleRule(suggestion: RuleSuggestion): RuleSuggestion {
    // 场景1: AI 用了 \U$1 做大写转换 → 转为 to_uppercase
    if (suggestion.transformationType === 'regex_replace' && suggestion.replacement) {
      const replacement = suggestion.replacement
      // 检测大写转换意图: \U$1, \U$0, \U${0}, \\U$1 等
      if (/\\U\$[01]|\\U\$\{[01]\}/.test(replacement)) {
        // 检查 pattern 是否是全量捕获（如 .*, (.+), (?s)(.*) 等）
        const fullCapturePatterns = /^(\.\*|\(\.\+\)|\(\?\:?\.\*\)|\(\?s\)\(\.\*\)|\.+)$/
        if (!suggestion.pattern || fullCapturePatterns.test(suggestion.pattern)) {
          console.log('[AI Rule] Auto-converting \\U replacement to to_uppercase')
          return {
            ...suggestion,
            transformationType: 'to_uppercase',
            pattern: '',
            replacement: ''
          }
        }
      }
      // 检测小写转换意图: \L$1, \L$0 等
      if (/\\L\$[01]|\\L\$\{[01]\}/.test(replacement)) {
        const fullCapturePatterns = /^(\.\*|\(\.\+\)|\(\?\:?\.\*\)|\(\?s\)\(\.\*\)|\.+)$/
        if (!suggestion.pattern || fullCapturePatterns.test(suggestion.pattern)) {
          console.log('[AI Rule] Auto-converting \\L replacement to to_lowercase')
          return {
            ...suggestion,
            transformationType: 'to_lowercase',
            pattern: '',
            replacement: ''
          }
        }
      }
    }
    return suggestion
  }

  // 基础校验 pattern（真正的语法验证交给后端 Rust regex）
  function validateRulePattern(pattern: string): boolean {
    return !!pattern && pattern.length > 0 && pattern.length <= 500
  }

  // 检查是否为重复规则（按类型区分判重键）
  function isDuplicateRule(suggestion: RuleSuggestion): boolean {
    const customRules = config.value?.customRules ?? []

    // regex_replace: pattern + replacement + type
    if (suggestion.transformationType === 'regex_replace') {
      return customRules.some(
        r => r.transformationType === 'regex_replace' &&
             r.pattern === suggestion.pattern &&
             r.replacement === suggestion.replacement
      )
    }

    // 其他类型: name + type（同名同类型视为重复）
    return customRules.some(
      r => r.transformationType === suggestion.transformationType &&
           r.name === suggestion.name
    )
  }

  // 验证 transformationType 有效性
  function isValidTransformationType(type: string): boolean {
    return VALID_TRANSFORM_TYPES.includes(type as typeof VALID_TRANSFORM_TYPES[number])
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
    isSavingRule,
    ruleValidationError,
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
