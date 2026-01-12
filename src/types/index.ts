// ============================================================
// Action Chip Types
// ============================================================
export type ActionType = 'LocalRule' | 'AIPrompt'

export interface ActionChip {
  id: string
  label: string
  actionType: ActionType
  payload: string
  shortcut?: string
}

// ============================================================
// Clipboard Types
// ============================================================
export type ClipboardKind = 'text' | 'image' | 'unknown'

export interface ClipboardImageMeta {
  width: number
  height: number
  byteLength: number
}

export interface ClipboardContent {
  kind: ClipboardKind
  text?: string
  image?: ClipboardImageMeta
}

// ============================================================
// Privacy Types
// ============================================================
export type PrivacyStatusType = 'local' | 'cloud-safe' | 'cloud-masked'

export interface PrivacyStatus {
  type: PrivacyStatusType
  maskedCount?: number
}

export type PIIType = 'Phone' | 'Email' | 'IDCard' | 'BankCard' | 'IP' | 'APIKey'

export interface PIIItem {
  piiType: PIIType
  value: string
  start: number
  end: number
}

export interface PIIScanResult {
  hasPii: boolean
  items: PIIItem[]
}

export interface MaskMapping {
  mappings: Record<string, string>
}

// ============================================================
// Panel Types
// ============================================================
export type PanelMode = 'idle' | 'preview' | 'processing' | 'result'

// ============================================================
// Rule Types
// ============================================================
export type TransformationType = 'regex_replace' | 'json_format' | 'json_minify' | 'sort_lines' | 'dedupe_lines' | 'to_uppercase' | 'to_lowercase'

export interface Rule {
  id: string
  name: string
  description: string
  transformationType?: TransformationType
  pattern: string
  replacement: string
  isBuiltin: boolean
}

export interface CustomRule extends Rule {
  sourcePrompt?: string
  createdBy?: 'builtin' | 'user' | 'ai'
  createdAt?: number
  usageCount?: number
  lastUsedAt?: number
}

// ============================================================
// AI Types
// ============================================================
export type AIProvider = 'OpenAI' | 'Ollama'

export interface AIConfig {
  provider: AIProvider
  baseUrl: string
  model: string
  apiKey?: string
  maxTokens: number
  temperature: number
}

export interface ModelInfo {
  id: string
  name: string
  provider: AIProvider
}

// ============================================================
// IPC Event Names (module:action convention)
// ============================================================
export const IPC_EVENTS = {
  PANEL_TOGGLE: 'panel:toggle',
  PANEL_HIDE: 'panel:hide',
  AI_CHUNK: 'ai:chunk',
  AI_DONE: 'ai:done',
  AI_ERROR: 'ai:error',
  AI_CANCEL: 'ai:cancel',
  CLIPBOARD_CHANGED: 'clipboard:changed',
} as const

export type IPCEventName = typeof IPC_EVENTS[keyof typeof IPC_EVENTS]

// ============================================================
// IPC Event Payloads
// ============================================================
export interface PanelTogglePayload {
  visible: boolean
}

export interface AIChunkPayload {
  content: string
  done: false
}

export interface AIDonePayload {
  content: string
  done: true
}

export interface AIErrorPayload {
  code: string
  message: string
}

export interface AICancelPayload {
  requestId: string
}

export interface ClipboardChangedPayload {
  hasText: boolean
}

// ============================================================
// Error Types
// ============================================================
export type ErrorAction = 'Retry' | 'Settings' | 'Dismiss'
export type ErrorSeverity = 'error' | 'warning' | 'info'

export interface ErrorResponse {
  code: string
  message: string
  recoverable: boolean
  action?: ErrorAction
}

export interface ErrorInfo {
  message: string
  code?: string
  severity?: ErrorSeverity
  recoverable?: boolean
  autoHideMs?: number
}

// ============================================================
// Action Tracking Types (for retry mechanism)
// ============================================================
export interface ActionSnapshot {
  type: 'ai' | 'rule'
  payload: string
  timestamp: number
  retryCount: number
  maxRetries: number
  lastError?: string
}

// ============================================================
// AI Rule Learning Types
// ============================================================
export interface RuleSuggestion {
  canBeRule: boolean
  confidence: number
  name: string
  pattern: string
  replacement: string
  transformationType: TransformationType
}

// ============================================================
// Config Types
// ============================================================
export interface AppConfig {
  hotkey: string
  aiProvider: AIProvider
  ollamaBaseUrl: string
  openaiBaseUrl: string
  modelName: string
  theme: 'system' | 'light' | 'dark'
  pinnedRuleIds: string[]
  customRules: CustomRule[]
  enableAIRuleLearning: boolean
}

// ============================================================
// Tauri Command Types
// ============================================================
export interface TauriCommands {
  greet: (name: string) => Promise<string>
  readClipboard: () => Promise<ClipboardContent>
  writeClipboard: (content: string) => Promise<void>
  scanPii: (text: string) => Promise<PIIScanResult>
  maskPii: (text: string) => Promise<{ masked: string; mapping: MaskMapping }>
  restorePii: (text: string, mapping: MaskMapping) => Promise<string>
  getConfig: () => Promise<AppConfig>
  setConfig: (config: AppConfig) => Promise<void>
  getApiKey: (provider: string) => Promise<string>
  setApiKey: (provider: string, key: string) => Promise<void>
  listLocalModels: () => Promise<ModelInfo[]>
  applyRule: (text: string, ruleId: string) => Promise<string>
  applyCustomRule: (text: string, rule: Rule) => Promise<string>
  getBuiltinRules: () => Promise<Rule[]>
}

// ============================================================
// Performance Types
// ============================================================
export interface PerfMetric {
  name: string
  valueMs?: number
  valueMb?: number
  target: number
  status: 'PASS' | 'FAIL' | 'WARN' | 'NOT_MEASURED'
}

export interface PerfReport {
  meta: {
    timestamp: number
    platform: string
  }
  metrics: PerfMetric[]
}
