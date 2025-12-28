<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { commands } from '@/lib/tauri'
import type { AppConfig, ModelInfo, Rule } from '@/types'

const store = useAppStore()

const emit = defineEmits<{
  close: []
}>()

const formData = ref<AppConfig>({
  hotkey: 'Ctrl+Shift+V',
  aiProvider: 'Ollama',
  ollamaBaseUrl: 'http://localhost:11434',
  openaiBaseUrl: 'https://api.openai.com/v1',
  modelName: 'llama3.2',
  theme: 'system',
  pinnedRuleIds: [],
  customRules: [],
  enableAIRuleLearning: true,
})

const apiKey = ref('')
const availableModels = ref<ModelInfo[]>([])
const isTesting = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)
const isSaving = ref(false)
const errors = ref<Record<string, string>>({})
const selectedRuleToAdd = ref('')

const currentProvider = computed(() => formData.value.aiProvider)
const currentBaseUrl = computed(() =>
  formData.value.aiProvider === 'OpenAI'
    ? formData.value.openaiBaseUrl
    : formData.value.ollamaBaseUrl
)

const requiresApiKey = computed(() => formData.value.aiProvider === 'OpenAI')

const availableRulesToAdd = computed(() => {
  const pinned = new Set(formData.value.pinnedRuleIds)
  return store.allRules.filter((r: Rule) => !pinned.has(r.id))
})

function moveRuleUp(index: number) {
  if (index <= 0) return
  const arr = [...formData.value.pinnedRuleIds]
  ;[arr[index - 1], arr[index]] = [arr[index], arr[index - 1]]
  formData.value.pinnedRuleIds = arr
}

function moveRuleDown(index: number) {
  if (index >= formData.value.pinnedRuleIds.length - 1) return
  const arr = [...formData.value.pinnedRuleIds]
  ;[arr[index], arr[index + 1]] = [arr[index + 1], arr[index]]
  formData.value.pinnedRuleIds = arr
}

function removeFromPinned(index: number) {
  formData.value.pinnedRuleIds = formData.value.pinnedRuleIds.filter((_, i) => i !== index)
}

function addToPinned() {
  if (!selectedRuleToAdd.value) return
  formData.value.pinnedRuleIds = [...formData.value.pinnedRuleIds, selectedRuleToAdd.value]
  selectedRuleToAdd.value = ''
}

onMounted(async () => {
  if (store.config) {
    formData.value = { ...store.config }
  }

  // Load API key if using OpenAI
  if (requiresApiKey.value) {
    try {
      const key = await commands.getApiKey('openai')
      if (key) {
        apiKey.value = key
      }
    } catch (e) {
      console.error('Failed to load API key:', e)
    }
  }

  // Load available models if using Ollama
  if (formData.value.aiProvider === 'Ollama') {
    await loadOllamaModels()
  }
})

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

function validateForm(): boolean {
  errors.value = {}

  // Validate hotkey
  if (!formData.value.hotkey.trim()) {
    errors.value.hotkey = '热键不能为空'
  }

  // Validate base URL
  const baseUrl = currentBaseUrl.value
  if (!baseUrl.trim()) {
    errors.value.baseUrl = 'URL 不能为空'
  } else if (formData.value.aiProvider === 'OpenAI' && !baseUrl.startsWith('https://')) {
    errors.value.baseUrl = 'OpenAI API 必须使用 HTTPS'
  }

  // Validate API key for OpenAI
  if (requiresApiKey.value && !apiKey.value.trim()) {
    errors.value.apiKey = 'OpenAI 需要 API Key'
  }

  // Validate model
  if (!formData.value.modelName.trim()) {
    errors.value.model = '模型名称不能为空'
  }

  return Object.keys(errors.value).length === 0
}

async function testConnection() {
  if (!validateForm()) return

  isTesting.value = true
  testResult.value = null

  try {
    if (formData.value.aiProvider === 'Ollama') {
      const healthy = await commands.checkOllamaHealth(formData.value.ollamaBaseUrl)
      testResult.value = {
        success: healthy,
        message: healthy ? '✅ Ollama 连接成功' : '❌ Ollama 服务未响应',
      }

      if (healthy) {
        await loadOllamaModels()
      }
    } else {
      // For OpenAI, we can't test without making an actual API call
      // Just validate the format
      testResult.value = {
        success: true,
        message: '✅ 配置格式正确',
      }
    }
  } catch (e) {
    testResult.value = {
      success: false,
      message: `❌ 连接失败: ${e}`,
    }
  } finally {
    isTesting.value = false
  }
}

async function handleSave() {
  if (!validateForm()) return

  isSaving.value = true

  try {
    // Save config
    await store.saveConfig(formData.value)

    // Save API key if provided
    if (requiresApiKey.value && apiKey.value.trim()) {
      await commands.setApiKey('openai', apiKey.value)
    }

    // Re-register hotkey if changed
    if (store.config && formData.value.hotkey !== store.config.hotkey) {
      await commands.registerHotkey(formData.value.hotkey)
    }

    emit('close')
  } catch (e) {
    errors.value.save = `保存失败: ${e}`
  } finally {
    isSaving.value = false
  }
}

function handleCancel() {
  emit('close')
}

function handleProviderChange() {
  testResult.value = null
  if (formData.value.aiProvider === 'Ollama') {
    loadOllamaModels()
  }
}
</script>

<template>
  <div class="settings-panel p-6 bg-white dark:bg-gray-900 rounded-lg shadow-xl max-w-md w-full max-h-full overflow-y-auto">
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">设置</h2>
      <button
        @click="handleCancel"
        class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <form @submit.prevent="handleSave" class="space-y-4">
      <!-- Hotkey -->
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          全局热键
        </label>
        <input
          v-model="formData.hotkey"
          type="text"
          class="w-full px-3 py-2 border rounded-lg text-sm dark:bg-gray-800 dark:border-gray-700 dark:text-gray-200"
          :class="errors.hotkey ? 'border-red-500' : 'border-gray-300'"
          placeholder="Ctrl+Shift+V"
        />
        <p v-if="errors.hotkey" class="text-xs text-red-500 mt-1">{{ errors.hotkey }}</p>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
          支持: Ctrl, Shift, Alt, Meta (Cmd), CommandOrControl
        </p>
      </div>

      <!-- AI Provider -->
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          AI 提供商
        </label>
        <select
          v-model="formData.aiProvider"
          @change="handleProviderChange"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-lg text-sm dark:bg-gray-800 dark:text-gray-200"
        >
          <option value="Ollama">Ollama (本地)</option>
          <option value="OpenAI">OpenAI (云端)</option>
        </select>
      </div>

      <!-- Base URL -->
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          API 地址
        </label>
        <input
          v-if="currentProvider === 'Ollama'"
          v-model="formData.ollamaBaseUrl"
          type="url"
          class="w-full px-3 py-2 border rounded-lg text-sm dark:bg-gray-800 dark:border-gray-700 dark:text-gray-200"
          :class="errors.baseUrl ? 'border-red-500' : 'border-gray-300'"
          placeholder="http://localhost:11434"
        />
        <input
          v-else
          v-model="formData.openaiBaseUrl"
          type="url"
          class="w-full px-3 py-2 border rounded-lg text-sm dark:bg-gray-800 dark:border-gray-700 dark:text-gray-200"
          :class="errors.baseUrl ? 'border-red-500' : 'border-gray-300'"
          placeholder="https://api.openai.com/v1"
        />
        <p v-if="errors.baseUrl" class="text-xs text-red-500 mt-1">{{ errors.baseUrl }}</p>
      </div>

      <!-- API Key (OpenAI only) -->
      <div v-if="requiresApiKey">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          API Key
        </label>
        <input
          v-model="apiKey"
          type="password"
          class="w-full px-3 py-2 border rounded-lg text-sm dark:bg-gray-800 dark:border-gray-700 dark:text-gray-200"
          :class="errors.apiKey ? 'border-red-500' : 'border-gray-300'"
          placeholder="sk-..."
        />
        <p v-if="errors.apiKey" class="text-xs text-red-500 mt-1">{{ errors.apiKey }}</p>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
          🔒 安全存储在系统密钥链中
        </p>
      </div>

      <!-- Model Selection -->
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          模型
        </label>
        <select
          v-if="currentProvider === 'Ollama' && availableModels.length > 0"
          v-model="formData.modelName"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-lg text-sm dark:bg-gray-800 dark:text-gray-200"
        >
          <option v-for="model in availableModels" :key="model.id" :value="model.id">
            {{ model.name }}
          </option>
        </select>
        <input
          v-else
          v-model="formData.modelName"
          type="text"
          class="w-full px-3 py-2 border rounded-lg text-sm dark:bg-gray-800 dark:border-gray-700 dark:text-gray-200"
          :class="errors.model ? 'border-red-500' : 'border-gray-300'"
          :placeholder="currentProvider === 'OpenAI' ? 'gpt-4o-mini' : 'llama3.2'"
        />
        <p v-if="errors.model" class="text-xs text-red-500 mt-1">{{ errors.model }}</p>
      </div>

      <!-- Theme -->
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          主题
        </label>
        <select
          v-model="formData.theme"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-lg text-sm dark:bg-gray-800 dark:text-gray-200"
        >
          <option value="system">跟随系统</option>
          <option value="light">浅色</option>
          <option value="dark">深色</option>
        </select>
      </div>

      <!-- Quick Actions Management -->
      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          快捷操作 (拖拽排序，前3个显示在主面板)
        </label>
        <div class="space-y-1">
          <div
            v-for="(ruleId, index) in formData.pinnedRuleIds"
            :key="ruleId"
            class="flex items-center gap-2 p-2 rounded-lg border"
            :class="index < 3 ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800' : 'bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700'"
          >
            <span class="text-xs font-mono w-5 text-center" :class="index < 3 ? 'text-blue-500' : 'text-gray-400'">
              {{ index < 3 ? index + 1 : '-' }}
            </span>
            <span class="flex-1 text-sm text-gray-700 dark:text-gray-300">
              {{ store.allRules.find((r: Rule) => r.id === ruleId)?.name || ruleId }}
            </span>
            <button
              type="button"
              @click="moveRuleUp(index)"
              :disabled="index === 0"
              class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30"
            >↑</button>
            <button
              type="button"
              @click="moveRuleDown(index)"
              :disabled="index === formData.pinnedRuleIds.length - 1"
              class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30"
            >↓</button>
            <button
              type="button"
              @click="removeFromPinned(index)"
              class="p-1 text-red-400 hover:text-red-600"
            >×</button>
          </div>
        </div>
        <!-- Add Rule -->
        <div class="mt-2">
          <select
            v-model="selectedRuleToAdd"
            @change="addToPinned"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-lg text-sm dark:bg-gray-800 dark:text-gray-200"
          >
            <option value="">添加快捷操作...</option>
            <option
              v-for="rule in availableRulesToAdd"
              :key="rule.id"
              :value="rule.id"
            >{{ rule.name }}</option>
          </select>
        </div>
      </div>

      <!-- Test Connection -->
      <div>
        <button
          type="button"
          @click="testConnection"
          :disabled="isTesting"
          class="w-full px-4 py-2 text-sm border border-gray-300 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors disabled:opacity-50"
        >
          {{ isTesting ? '测试中...' : '测试连接' }}
        </button>
        <div
          v-if="testResult"
          class="mt-2 text-sm p-2 rounded"
          :class="testResult.success ? 'bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400' : 'bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400'"
        >
          {{ testResult.message }}
        </div>
      </div>

      <!-- Error Display -->
      <div v-if="errors.save" class="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
        <p class="text-sm text-red-600 dark:text-red-400">{{ errors.save }}</p>
      </div>

      <!-- Action Buttons -->
      <div class="flex gap-3 pt-2">
        <button
          type="submit"
          :disabled="isSaving"
          class="flex-1 px-4 py-2 bg-blue-500 text-white text-sm rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
        >
          {{ isSaving ? '保存中...' : '保存' }}
        </button>
        <button
          type="button"
          @click="handleCancel"
          class="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 text-sm rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
        >
          取消
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
input:focus,
select:focus {
  outline: none;
  border-color: #3b82f6;
  ring: 2px;
  ring-color: rgba(59, 130, 246, 0.3);
}
</style>
