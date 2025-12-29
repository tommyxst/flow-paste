<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Bot, Server, Shield, Cpu, Key, Wifi, Loader2, Check, AlertCircle } from 'lucide-vue-next'
import { commands } from '@/lib/tauri'
import type { AppConfig, ModelInfo } from '@/types'

const props = defineProps<{
  formData: AppConfig
  apiKey: string
  errors: Record<string, string>
}>()

const emit = defineEmits<{
  'update:apiKey': [value: string]
}>()

const localApiKey = computed({
  get: () => props.apiKey,
  set: (val) => emit('update:apiKey', val)
})

const availableModels = ref<ModelInfo[]>([])
const isTesting = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)

const currentProvider = computed(() => props.formData.aiProvider)
const currentBaseUrl = computed(() => props.formData.aiProvider === 'OpenAI' ? props.formData.openaiBaseUrl : props.formData.ollamaBaseUrl)
const requiresApiKey = computed(() => props.formData.aiProvider === 'OpenAI')

onMounted(async () => {
    if (props.formData.aiProvider === 'Ollama') {
        await loadOllamaModels()
    }
})

watch(() => props.formData.aiProvider, async (newProvider) => {
    testResult.value = null
    if (newProvider === 'Ollama') {
        await loadOllamaModels()
    }
})

async function loadOllamaModels() {
  try {
    const models = await commands.listLocalModels()
    availableModels.value = models
    if (models.length > 0 && !props.formData.modelName) {
        // Mutating prop is not recommended usually but here formData is a reactive object from parent
        props.formData.modelName = models[0].id
    }
  } catch (e) { 
    console.error('Failed to load Ollama models:', e) 
  }
}

async function testConnection() {
  // Simple local validation before testing
  if (!currentBaseUrl.value.trim()) return
  if (requiresApiKey.value && !localApiKey.value.trim()) return

  isTesting.value = true
  testResult.value = null
  try {
    if (props.formData.aiProvider === 'Ollama') {
      const healthy = await commands.checkOllamaHealth(props.formData.ollamaBaseUrl)
      testResult.value = { success: healthy, message: healthy ? 'Ollama 连接成功' : 'Ollama 服务未响应' }
      if (healthy) await loadOllamaModels()
    } else {
      try {
        const success = await commands.testOpenaiConnection(props.formData.openaiBaseUrl, localApiKey.value, props.formData.modelName)
        testResult.value = { success, message: success ? 'API 连接成功' : 'API 连接失败' }
      } catch (e) { 
          testResult.value = { success: false, message: `连接失败: ${e}` } 
      }
    }
  } catch (e) { 
      testResult.value = { success: false, message: `连接失败: ${e}` } 
  }
  finally { 
      isTesting.value = false 
  }
}

function handleProviderChange() {
    // This is handled by the watch, but explicit click handler can clear test result too
    testResult.value = null
    if (props.formData.aiProvider === 'Ollama') {
        loadOllamaModels()
    }
}
</script>

<template>
  <section class="settings-section">
    <div class="section-header">
      <div class="section-icon ai">
        <Bot :size="16" />
      </div>
      <h3>AI 配置</h3>
    </div>
    <div class="section-content">
      <div class="field-group">
        <label class="field-label">
          <Cpu :size="14" />
          <span>AI 提供商</span>
        </label>
        <div class="provider-selector">
          <button 
            type="button" 
            @click="formData.aiProvider = 'Ollama'; handleProviderChange()" 
            class="provider-option" 
            :class="{ active: formData.aiProvider === 'Ollama' }"
          >
            <Shield :size="14" />
            <span>Ollama</span>
            <span class="provider-badge local">本地</span>
          </button>
          <button 
            type="button" 
            @click="formData.aiProvider = 'OpenAI'; handleProviderChange()" 
            class="provider-option" 
            :class="{ active: formData.aiProvider === 'OpenAI' }"
          >
            <Bot :size="14" />
            <span>OpenAI</span>
            <span class="provider-badge cloud">云端</span>
          </button>
        </div>
      </div>

      <div class="field-group">
        <label class="field-label">
          <Server :size="14" />
          <span>API 地址</span>
        </label>
        <input 
          v-if="currentProvider === 'Ollama'" 
          v-model="formData.ollamaBaseUrl" 
          type="url" 
          class="field-input" 
          :class="{ 'has-error': errors.baseUrl }" 
          placeholder="http://localhost:11434" 
        />
        <input 
          v-else 
          v-model="formData.openaiBaseUrl" 
          type="url" 
          class="field-input" 
          :class="{ 'has-error': errors.baseUrl }" 
          placeholder="https://api.openai.com/v1" 
        />
        <p v-if="errors.baseUrl" class="field-error">{{ errors.baseUrl }}</p>
      </div>

      <div v-if="requiresApiKey" class="field-group">
        <label class="field-label">
          <Key :size="14" />
          <span>API Key</span>
        </label>
        <div class="input-with-icon">
          <input 
            v-model="localApiKey" 
            type="password" 
            class="field-input" 
            :class="{ 'has-error': errors.apiKey }" 
            placeholder="sk-..." 
          />
          <div class="input-icon-right">
            <Shield :size="14" />
          </div>
        </div>
        <p v-if="errors.apiKey" class="field-error">{{ errors.apiKey }}</p>
        <p class="field-hint secure">安全存储在系统密钥链中</p>
      </div>

      <div class="field-group">
        <label class="field-label">
          <Cpu :size="14" />
          <span>模型</span>
        </label>
        <select 
          v-if="currentProvider === 'Ollama' && availableModels.length > 0" 
          v-model="formData.modelName" 
          class="field-select"
        >
          <option v-for="m in availableModels" :key="m.id" :value="m.id">{{ m.name }}</option>
        </select>
        <input 
          v-else 
          v-model="formData.modelName" 
          type="text" 
          class="field-input" 
          :class="{ 'has-error': errors.model }" 
          :placeholder="currentProvider === 'OpenAI' ? 'gpt-4o-mini' : 'llama3.2'" 
        />
        <p v-if="errors.model" class="field-error">{{ errors.model }}</p>
      </div>

      <div class="connection-test">
        <button type="button" @click="testConnection" :disabled="isTesting" class="test-btn">
          <Loader2 v-if="isTesting" :size="14" class="animate-spin" />
          <Wifi v-else :size="14" />
          <span>{{ isTesting ? '测试中...' : '测试连接' }}</span>
        </button>
        <div v-if="testResult" class="test-result" :class="{ success: testResult.success, error: !testResult.success }">
          <Check v-if="testResult.success" :size="14" />
          <AlertCircle v-else :size="14" />
          <span>{{ testResult.message }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.settings-section {
  background: rgba(0, 0, 0, 0.02);
  border: 1px solid rgba(0, 0, 0, 0.04);
  border-radius: 12px;
  overflow: hidden;
}

.dark .settings-section {
  background: rgba(255, 255, 255, 0.02);
  border-color: rgba(255, 255, 255, 0.04);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  background: linear-gradient(to right, rgba(0, 0, 0, 0.02), transparent);
  border-bottom: 1px solid rgba(0, 0, 0, 0.04);
  cursor: pointer;
}

.dark .section-header {
  background: linear-gradient(to right, rgba(255, 255, 255, 0.02), transparent);
  border-bottom-color: rgba(255, 255, 255, 0.04);
}

.settings-section:not(.open) .section-header {
  border-bottom-color: transparent;
}

.section-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 7px;
  color: white;
}

.section-icon.ai {
  background: linear-gradient(135deg, #3b82f6, #06b6d4);
  box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
}

.section-header h3 {
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.accordion-icon {
  color: var(--text-tertiary);
  transition: transform 0.3s ease;
}

.accordion-icon.rotate {
  transform: rotate(180deg);
}

.section-content-wrapper {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.settings-section.open .section-content-wrapper {
  grid-template-rows: 1fr;
}

.section-content {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
  overflow: hidden;
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

.field-label svg {
  opacity: 0.6;
}

.field-input, .field-select {
  width: 100%;
  padding: 10px 12px;
  font-size: 13px;
  color: var(--text-primary);
  background: rgba(0, 0, 0, 0.03);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 8px;
  outline: none;
  transition: all 0.15s ease;
}

.dark .field-input, .dark .field-select {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.08);
}

.field-input:focus, .field-select:focus {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

.field-input.has-error {
  border-color: #ef4444;
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
}

.field-error {
  font-size: 11px;
  color: #ef4444;
  margin: 0;
}

.field-hint {
  font-size: 11px;
  color: var(--text-tertiary);
  margin: 0;
}

.field-hint.secure {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #22c55e;
}

.field-hint.secure::before {
  content: '🔒';
  font-size: 10px;
}

.input-with-icon {
  position: relative;
}

.input-with-icon .field-input {
  padding-right: 36px;
}

.input-icon-right {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-tertiary);
  pointer-events: none;
}

.provider-selector {
  display: flex;
  gap: 8px;
}

.provider-option {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: rgba(0, 0, 0, 0.03);
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.dark .provider-option {
  background: rgba(255, 255, 255, 0.03);
  border-color: rgba(255, 255, 255, 0.06);
}

.provider-option:hover {
  border-color: rgba(0, 0, 0, 0.12);
}

.dark .provider-option:hover {
  border-color: rgba(255, 255, 255, 0.12);
}

.provider-option.active {
  background: var(--accent-subtle);
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.provider-badge {
  font-size: 9px;
  padding: 2px 5px;
  border-radius: 4px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.provider-badge.local {
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
}

.provider-badge.cloud {
  background: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
}

.connection-test {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 6px;
}

.test-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 16px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: rgba(0, 0, 0, 0.03);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.dark .test-btn {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.08);
}

.test-btn:hover:not(:disabled) {
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.test-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  font-size: 12px;
  border-radius: 6px;
  animation: slideInDown 0.2s ease;
}

.test-result.success {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.test-result.error {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

@keyframes slideInDown {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
