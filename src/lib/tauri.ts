import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { createModuleLogger } from '@/utils/logger'
import { IPC_EVENTS } from '@/types'
import type {
  PanelTogglePayload,
  AIChunkPayload,
  AIDonePayload,
  AIErrorPayload,
  ClipboardChangedPayload,
  ClipboardHistoryChangedPayload,
  ClipboardContent,
  ClipboardHistoryEntry,
  ClipboardHistoryItem,
  PIIScanResult,
  MaskMapping,
  ModelInfo,
  AIConfig,
  AppConfig,
  Rule,
  PerfReport,
  ActionChip,
  RuleValidationResult,
} from '@/types'

const log = createModuleLogger('tauri')

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  log.debug(`Invoking command: ${command}`, args)
  try {
    const result = await invoke<T>(command, args)
    log.debug(`Command ${command} succeeded`, result)
    return result
  } catch (error) {
    log.error(`Command ${command} failed`, error)
    throw error
  }
}

export async function listenToEvent<T>(
  event: string,
  handler: (payload: T) => void
): Promise<UnlistenFn> {
  log.info(`Listening to event: ${event}`)
  return listen<T>(event, (e) => {
    log.debug(`Received event: ${event}`, e.payload)
    handler(e.payload)
  })
}

export const tauriEvents = {
  onPanelToggle: (handler: (payload: PanelTogglePayload) => void) =>
    listenToEvent<PanelTogglePayload>(IPC_EVENTS.PANEL_TOGGLE, handler),

  onAIChunk: (handler: (payload: AIChunkPayload) => void) =>
    listenToEvent<AIChunkPayload>(IPC_EVENTS.AI_CHUNK, handler),

  onAIDone: (handler: (payload: AIDonePayload) => void) =>
    listenToEvent<AIDonePayload>(IPC_EVENTS.AI_DONE, handler),

  onAIError: (handler: (payload: AIErrorPayload) => void) =>
    listenToEvent<AIErrorPayload>(IPC_EVENTS.AI_ERROR, handler),

  onClipboardChanged: (handler: (payload: ClipboardChangedPayload) => void) =>
    listenToEvent<ClipboardChangedPayload>(IPC_EVENTS.CLIPBOARD_CHANGED, handler),

  onClipboardHistoryChanged: (handler: (payload: ClipboardHistoryChangedPayload) => void) =>
    listenToEvent<ClipboardHistoryChangedPayload>(IPC_EVENTS.CLIPBOARD_HISTORY_CHANGED, handler),
}

export interface MaskResult {
  masked: string
  mapping: MaskMapping
  scanResult: PIIScanResult
}

export interface PasteResult {
  success: boolean
  usedSimulation: boolean
  message?: string
}

export const commands = {
  greet: (name: string) => invokeCommand<string>('greet', { name }),

  // Clipboard commands
  readClipboard: () => invokeCommand<ClipboardContent>('read_clipboard'),
  writeClipboard: (text: string) => invokeCommand<void>('write_clipboard', { text }),
  pasteToCursor: (text: string, delayMs?: number) =>
    invokeCommand<PasteResult>('paste_to_cursor', { text, delayMs }),
  checkPasteCapability: () => invokeCommand<void>('check_paste_capability'),

  // Clipboard History commands
  listClipboardHistory: (limit?: number, offset?: number) =>
    invokeCommand<ClipboardHistoryEntry[]>('list_clipboard_history', { limit, offset }),
  getClipboardHistory: (id: number) =>
    invokeCommand<ClipboardHistoryItem>('get_clipboard_history', { id }),
  deleteClipboardHistory: (id: number) =>
    invokeCommand<void>('delete_clipboard_history', { id }),
  clearClipboardHistory: () =>
    invokeCommand<void>('clear_clipboard_history'),
  pasteClipboardHistory: (id: number, delayMs?: number) =>
    invokeCommand<PasteResult>('paste_clipboard_history', { id, delayMs }),

  // Privacy Shield commands
  scanPii: (text: string) => invokeCommand<PIIScanResult>('scan_pii', { text }),
  maskPii: (text: string) => invokeCommand<MaskResult>('mask_pii', { text }),
  restorePii: (text: string, mapping: MaskMapping) =>
    invokeCommand<string>('restore_pii', { text, mapping }),

  // AI commands
  listLocalModels: () => invokeCommand<ModelInfo[]>('list_local_models'),
  checkOllamaHealth: (baseUrl?: string) =>
    invokeCommand<boolean>('check_ollama_health', { baseUrl }),
  testOpenaiConnection: (baseUrl: string, apiKey: string, model: string) =>
    invokeCommand<boolean>('test_openai_connection', { baseUrl, apiKey, model }),
  sendAiRequest: (
    prompt: string,
    config: AIConfig,
    requestId: string,
    usePrivacyShield: boolean
  ) =>
    invokeCommand<void>('send_ai_request', {
      prompt,
      config,
      requestId,
      usePrivacyShield,
    }),
  cancelAiRequest: (requestId: string) =>
    invokeCommand<void>('cancel_ai_request', { requestId }),
  detectContentIntent: (text: string) =>
    invokeCommand<ActionChip[]>('detect_content_intent', { text }),

  // Config commands
  getConfig: () => invokeCommand<AppConfig>('get_config'),
  setConfig: (config: AppConfig) => invokeCommand<void>('set_config', { config }),
  getApiKey: (provider: string) => invokeCommand<string | null>('get_api_key', { provider }),
  setApiKey: (provider: string, key: string) =>
    invokeCommand<void>('set_api_key', { provider, key }),

  // Regex commands
  getBuiltinRules: () => invokeCommand<Rule[]>('get_builtin_rules'),
  applyRule: (text: string, ruleId: string) =>
    invokeCommand<string>('apply_rule', { text, ruleId }),
  applyCustomRule: (text: string, rule: Rule) =>
    invokeCommand<string>('apply_custom_rule', { text, rule }),

  // Rule CRUD commands
  listAllRules: () => invokeCommand<Rule[]>('list_all_rules'),
  upsertRule: (rule: Rule) =>
    invokeCommand<RuleValidationResult>('upsert_rule', { rule }),
  deleteRule: (ruleId: string) =>
    invokeCommand<boolean>('delete_rule', { ruleId }),
  reorderPinnedRules: (ruleIds: string[]) =>
    invokeCommand<void>('reorder_pinned_rules', { ruleIds }),
  validateRuleCmd: (rule: Rule) =>
    invokeCommand<RuleValidationResult>('validate_rule_cmd', { rule }),

  // Hotkey commands
  registerHotkey: (hotkey: string) =>
    invokeCommand<void>('register_hotkey', { hotkey }),
  unregisterHotkey: () => invokeCommand<void>('unregister_hotkey'),
  isHotkeyRegistered: () => invokeCommand<boolean>('is_hotkey_registered'),

  // Performance commands
  reportRenderTimestamp: () => invokeCommand<void>('report_render_timestamp'),
  runPerfSuite: () => invokeCommand<PerfReport>('run_perf_suite'),
}
