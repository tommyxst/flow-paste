<script setup lang="ts">
import { Cpu } from 'lucide-vue-next'
import type { AIProvider, ModelInfo } from '@/types'

defineProps<{
  provider: AIProvider
  modelValue: string
  availableModels: ModelInfo[]
  error?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <div class="field-group">
    <label class="field-label">
      <Cpu :size="14" />
      <span>模型</span>
    </label>
    <select
      v-if="provider === 'Ollama' && availableModels.length > 0"
      :value="modelValue"
      @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
      class="field-select"
    >
      <option v-for="m in availableModels" :key="m.id" :value="m.id">
        {{ m.name }}
      </option>
    </select>
    <input
      v-else
      :value="modelValue"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      type="text"
      class="field-input"
      :class="{ 'has-error': error }"
      :placeholder="provider === 'OpenAI' ? 'gpt-4o-mini' : 'llama3.2'"
    />
    <p v-if="error" class="field-error">{{ error }}</p>
  </div>
</template>

<style src="../settings.shared.css"></style>
