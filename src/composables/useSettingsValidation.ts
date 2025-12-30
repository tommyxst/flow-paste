import { ref } from 'vue'
import type { AppConfig } from '@/types'

export interface SettingsErrors {
  general?: { hotkey?: string }
  ai?: { baseUrl?: string; apiKey?: string; model?: string }
  quickActions?: Record<string, string>
  save?: string
}

export type FlatSettingsErrors = Partial<
  Record<'hotkey' | 'baseUrl' | 'apiKey' | 'model' | 'save', string>
>

export function useSettingsValidation() {
  const errors = ref<SettingsErrors>({})

  function validateAll(draft: AppConfig, apiKey: string): boolean {
    errors.value = {}

    // General validation
    if (!draft.hotkey.trim()) {
      errors.value.general = { hotkey: '热键不能为空' }
    }

    // AI validation
    const aiErrors: SettingsErrors['ai'] = {}
    const baseUrl = draft.aiProvider === 'OpenAI'
      ? draft.openaiBaseUrl
      : draft.ollamaBaseUrl

    if (!baseUrl.trim()) {
      aiErrors.baseUrl = 'URL 不能为空'
    } else if (draft.aiProvider === 'OpenAI' && !baseUrl.startsWith('https://')) {
      aiErrors.baseUrl = 'OpenAI API 必须使用 HTTPS'
    }

    if (draft.aiProvider === 'OpenAI' && !apiKey.trim()) {
      aiErrors.apiKey = 'OpenAI 需要 API Key'
    }

    if (!draft.modelName.trim()) {
      aiErrors.model = '模型名称不能为空'
    }

    if (Object.keys(aiErrors).length > 0) {
      errors.value.ai = aiErrors
    }

    return Object.keys(errors.value).length === 0
  }

  function setSaveError(message: string) {
    errors.value.save = message
  }

  function clearErrors() {
    errors.value = {}
  }

  function clearSaveError() {
    delete errors.value.save
  }

  // Flatten errors for backward compatibility
  function getFlatErrors(): FlatSettingsErrors {
    const flat: FlatSettingsErrors = {}
    if (errors.value.general?.hotkey) flat.hotkey = errors.value.general.hotkey
    if (errors.value.ai?.baseUrl) flat.baseUrl = errors.value.ai.baseUrl
    if (errors.value.ai?.apiKey) flat.apiKey = errors.value.ai.apiKey
    if (errors.value.ai?.model) flat.model = errors.value.ai.model
    if (errors.value.save) flat.save = errors.value.save
    return flat
  }

  return {
    errors,
    validateAll,
    setSaveError,
    clearErrors,
    clearSaveError,
    getFlatErrors
  }
}
