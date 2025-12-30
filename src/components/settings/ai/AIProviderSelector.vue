<script setup lang="ts">
import { Bot, Shield } from 'lucide-vue-next'
import type { AIProvider } from '@/types'

const props = defineProps<{
  modelValue: AIProvider
}>()

const emit = defineEmits<{
  'update:modelValue': [value: AIProvider]
  change: []
}>()

function selectProvider(provider: AIProvider) {
  emit('update:modelValue', provider)
  emit('change')
}
</script>

<template>
  <div class="field-group">
    <label class="field-label">
      <Bot :size="14" />
      <span>AI 提供商</span>
    </label>
    <div class="provider-selector">
      <button
        type="button"
        @click="selectProvider('Ollama')"
        class="provider-option"
        :class="{ active: modelValue === 'Ollama' }"
      >
        <Shield :size="14" />
        <span>Ollama</span>
        <span class="provider-badge local">本地</span>
      </button>
      <button
        type="button"
        @click="selectProvider('OpenAI')"
        class="provider-option"
        :class="{ active: modelValue === 'OpenAI' }"
      >
        <Bot :size="14" />
        <span>OpenAI</span>
        <span class="provider-badge cloud">云端</span>
      </button>
    </div>
  </div>
</template>

<style src="../settings.shared.css"></style>
<style scoped>
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
</style>
