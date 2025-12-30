<script setup lang="ts">
import { computed, watch, toRef } from 'vue'
import { Bot } from 'lucide-vue-next'
import type { AppConfig } from '@/types'
import { useAIConfig } from '@/composables/useAIConfig'
import type { FlatSettingsErrors } from '@/composables/useSettingsValidation'
import SettingsSectionShell from './SettingsSectionShell.vue'
import AIProviderSelector from './ai/AIProviderSelector.vue'
import AIBaseUrlField from './ai/AIBaseUrlField.vue'
import AIApiKeyField from './ai/AIApiKeyField.vue'
import AIModelField from './ai/AIModelField.vue'
import AIConnectionTest from './ai/AIConnectionTest.vue'

const props = defineProps<{
  formData: AppConfig
  apiKey: string
  errors: FlatSettingsErrors
}>()

const emit = defineEmits<{
  'update:apiKey': [value: string]
}>()

const localApiKey = computed({
  get: () => props.apiKey,
  set: (val) => emit('update:apiKey', val)
})

const formDataRef = toRef(props, 'formData')

const {
  availableModels,
  isTesting,
  testResult,
  requiresApiKey,
  loadOllamaModels,
  testConnection,
  handleProviderChange
} = useAIConfig(formDataRef)

watch(() => props.formData.aiProvider, async (newProvider) => {
  handleProviderChange()
  if (newProvider === 'Ollama') {
    await loadOllamaModels()
  }
}, { immediate: true })

function onTestConnection() {
  void testConnection(localApiKey.value)
}
</script>

<template>
  <SettingsSectionShell title="AI 配置" :icon="Bot" icon-class="ai">
    <AIProviderSelector
      v-model="formData.aiProvider"
    />

    <AIBaseUrlField
      :provider="formData.aiProvider"
      v-model:ollama-url="formData.ollamaBaseUrl"
      v-model:openai-url="formData.openaiBaseUrl"
      :error="errors.baseUrl"
    />

    <AIApiKeyField
      v-if="requiresApiKey"
      v-model="localApiKey"
      :error="errors.apiKey"
    />

    <AIModelField
      :provider="formData.aiProvider"
      v-model="formData.modelName"
      :available-models="availableModels"
      :error="errors.model"
    />

    <AIConnectionTest
      :is-testing="isTesting"
      :test-result="testResult"
      @test="onTestConnection"
    />
  </SettingsSectionShell>
</template>

<style src="./settings.shared.css"></style>
