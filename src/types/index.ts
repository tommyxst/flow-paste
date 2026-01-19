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
export type PanelMode = 'idle' | 'preview' | 'processing' | 'result' | 'history'

// ============================================================
// Clipboard History Types
// ============================================================
export interface ClipboardHistoryEntry {
  id: number
  kind: ClipboardKind
  preview: string
  byteLength: number
  createdAtMs: number
  piiDetected: boolean
}

export interface ClipboardHistoryItem {
  id: number
  kind: ClipboardKind
  text?: string
  preview: string
  byteLength: number
  createdAtMs: number
  piiDetected: boolean
}

export interface ClipboardHistoryChangedPayload {
  action: 'insert' | 'delete' | 'clear'
  id?: number
}

// ============================================================
// Rule Types
// ============================================================
export type TransformationType = 'regex_replace' | 'json_format' | 'json_minify' | 'sort_lines' | 'dedupe_lines' | 'to_uppercase' | 'to_lowercase'

export type RuleOrigin = 'builtin' | 'user' | 'ai'

export interface Rule {
  id: string
  name: string
  description: string
  transformationType?: TransformationType
  pattern: string
  replacement: string
  isBuiltin: boolean
  origin?: RuleOrigin
  category?: string
  enabled?: boolean
  order?: number
}

export interface CustomRule extends Rule {
  sourcePrompt?: string
  createdAt?: number
  usageCount?: number
  lastUsedAt?: number
}

export interface RuleValidationResult {
  valid: boolean
  errors: string[]
  warnings: string[]
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
  CLIPBOARD_HISTORY_CHANGED: 'clipboard:history_changed',
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
  // New CRUD commands
  listAllRules: () => Promise<Rule[]>
  upsertRule: (rule: Rule) => Promise<RuleValidationResult>
  deleteRule: (ruleId: string) => Promise<boolean>
  reorderPinnedRules: (ruleIds: string[]) => Promise<void>
  validateRuleCmd: (rule: Rule) => Promise<RuleValidationResult>
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
