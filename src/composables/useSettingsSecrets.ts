import { ref } from 'vue'
import { commands } from '@/lib/tauri'
import type { AIProvider } from '@/types'

export function useSettingsSecrets() {
  const apiKey = ref('')
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  async function loadApiKey() {
    isLoading.value = true
    error.value = null
    try {
      const key = await commands.getApiKey('openai')
      if (key) apiKey.value = key
    } catch (e) {
      console.error('Failed to load API key:', e)
      error.value = `加载 API Key 失败: ${e}`
    } finally {
      isLoading.value = false
    }
  }

  async function saveApiKeyIfNeeded(provider: AIProvider, key: string): Promise<boolean> {
    if (provider !== 'OpenAI' || !key.trim()) return true

    try {
      await commands.setApiKey('openai', key)
      return true
    } catch (e) {
      console.error('[Settings] Failed to save API key:', e)
      error.value = `保存 API Key 失败: ${e}`
      return false
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    apiKey,
    isLoading,
    error,
    loadApiKey,
    saveApiKeyIfNeeded,
    clearError
  }
}
