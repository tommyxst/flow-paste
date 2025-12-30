<script setup lang="ts">
import { Server } from 'lucide-vue-next'
import type { AIProvider } from '@/types'

defineProps<{
  provider: AIProvider
  ollamaUrl: string
  openaiUrl: string
  error?: string
}>()

const emit = defineEmits<{
  'update:ollamaUrl': [value: string]
  'update:openaiUrl': [value: string]
}>()
</script>

<template>
  <div class="field-group">
    <label class="field-label">
      <Server :size="14" />
      <span>API 地址</span>
    </label>
    <input
      v-if="provider === 'Ollama'"
      :value="ollamaUrl"
      @input="emit('update:ollamaUrl', ($event.target as HTMLInputElement).value)"
      type="url"
      class="field-input"
      :class="{ 'has-error': error }"
      placeholder="http://localhost:11434"
    />
    <input
      v-else
      :value="openaiUrl"
      @input="emit('update:openaiUrl', ($event.target as HTMLInputElement).value)"
      type="url"
      class="field-input"
      :class="{ 'has-error': error }"
      placeholder="https://api.openai.com/v1"
    />
    <p v-if="error" class="field-error">{{ error }}</p>
  </div>
</template>

<style src="../settings.shared.css"></style>
