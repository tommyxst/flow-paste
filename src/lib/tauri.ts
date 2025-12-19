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
  PIIScanResult,
  MaskMapping,
  ModelInfo,
  AIConfig,
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
}

export interface MaskResult {
  masked: string
  mapping: MaskMapping
  scanResult: PIIScanResult
}

export const commands = {
  greet: (name: string) => invokeCommand<string>('greet', { name }),

  // Privacy Shield commands
  scanPii: (text: string) => invokeCommand<PIIScanResult>('scan_pii', { text }),
  maskPii: (text: string) => invokeCommand<MaskResult>('mask_pii', { text }),
  restorePii: (text: string, mapping: MaskMapping) =>
    invokeCommand<string>('restore_pii', { text, mapping }),

  // AI commands
  listLocalModels: () => invokeCommand<ModelInfo[]>('list_local_models'),
  checkOllamaHealth: (baseUrl?: string) =>
    invokeCommand<boolean>('check_ollama_health', { baseUrl }),
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
}
