import { ref, computed } from 'vue'
import type { AppConfig } from '@/types'

const defaultConfig: AppConfig = {
  hotkey: 'Ctrl+Shift+V',
  aiProvider: 'Ollama',
  ollamaBaseUrl: 'http://localhost:11434',
  openaiBaseUrl: 'https://api.openai.com/v1',
  modelName: 'llama3.2',
  theme: 'system',
  pinnedRuleIds: [],
  customRules: [],
  enableAIRuleLearning: true
}

function deepClone<T>(obj: T): T {
  // Vue reactive objects cannot be cloned with structuredClone
  return JSON.parse(JSON.stringify(obj)) as T
}

function deepEqual(a: unknown, b: unknown): boolean {
  return JSON.stringify(a) === JSON.stringify(b)
}

export function useSettingsDraft(initialConfig: AppConfig | null) {
  const base = ref<AppConfig>(deepClone(initialConfig ?? defaultConfig))
  const draft = ref<AppConfig>(deepClone(initialConfig ?? defaultConfig))

  const isDirty = computed(() => !deepEqual(draft.value, base.value))

  const dirtyFields = computed(() => {
    const fields: Record<string, boolean> = {}
    const keys = Object.keys(draft.value) as (keyof AppConfig)[]
    for (const key of keys) {
      fields[key] = !deepEqual(draft.value[key], base.value[key])
    }
    return fields
  })

  function applyConfig(config: AppConfig) {
    base.value = deepClone(config)
    draft.value = deepClone(config)
  }

  function resetDraft() {
    draft.value = deepClone(base.value)
  }

  function commitDraft() {
    base.value = deepClone(draft.value)
  }

  return {
    draft,
    isDirty,
    dirtyFields,
    applyConfig,
    resetDraft,
    commitDraft
  }
}
