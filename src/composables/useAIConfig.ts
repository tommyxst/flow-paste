import { ref, computed, type Ref } from 'vue'
import { commands } from '@/lib/tauri'
import type { AppConfig, ModelInfo } from '@/types'

export function useAIConfig(formData: Ref<AppConfig>) {
  const availableModels = ref<ModelInfo[]>([])
  const isTesting = ref(false)
  const testResult = ref<{ success: boolean; message: string } | null>(null)

  const currentProvider = computed(() => formData.value.aiProvider)
  const currentBaseUrl = computed(() =>
    formData.value.aiProvider === 'OpenAI'
      ? formData.value.openaiBaseUrl
      : formData.value.ollamaBaseUrl
  )
  const requiresApiKey = computed(() => formData.value.aiProvider === 'OpenAI')

  async function loadOllamaModels() {
    try {
      const models = await commands.listLocalModels()
      availableModels.value = models
      if (models.length > 0 && !formData.value.modelName) {
        formData.value.modelName = models[0].id
      }
    } catch (e) {
      console.error('Failed to load Ollama models:', e)
    }
  }

  async function testConnection(apiKey: string) {
    const baseUrl = currentBaseUrl.value.trim()
    const trimmedKey = apiKey.trim()

    if (!baseUrl) {
      testResult.value = { success: false, message: '请先填写服务 URL' }
      return
    }
    if (requiresApiKey.value && !trimmedKey) {
      testResult.value = { success: false, message: '请先填写 API Key' }
      return
    }

    isTesting.value = true
    testResult.value = null

    try {
      if (formData.value.aiProvider === 'Ollama') {
        const healthy = await commands.checkOllamaHealth(formData.value.ollamaBaseUrl.trim())
        testResult.value = {
          success: healthy,
          message: healthy ? 'Ollama 连接成功' : 'Ollama 服务未响应'
        }
        if (healthy) await loadOllamaModels()
      } else {
        const success = await commands.testOpenaiConnection(
          formData.value.openaiBaseUrl.trim(),
          trimmedKey,
          formData.value.modelName.trim()
        )
        testResult.value = {
          success,
          message: success ? 'API 连接成功' : 'API 连接失败'
        }
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      testResult.value = { success: false, message: `连接失败: ${msg}` }
    } finally {
      isTesting.value = false
    }
  }

  function clearTestResult() {
    testResult.value = null
  }

  function handleProviderChange() {
    clearTestResult()
    if (formData.value.aiProvider === 'Ollama') {
      loadOllamaModels()
    }
  }

  return {
    availableModels,
    isTesting,
    testResult,
    currentProvider,
    currentBaseUrl,
    requiresApiKey,
    loadOllamaModels,
    testConnection,
    clearTestResult,
    handleProviderChange
  }
}
