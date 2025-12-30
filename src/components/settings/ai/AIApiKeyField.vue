<script setup lang="ts">
import { Key, Shield } from 'lucide-vue-next'

defineProps<{
  modelValue: string
  error?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <div class="field-group">
    <label class="field-label">
      <Key :size="14" />
      <span>API Key</span>
    </label>
    <div class="input-with-icon">
      <input
        :value="modelValue"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        type="password"
        class="field-input"
        :class="{ 'has-error': error }"
        placeholder="sk-..."
      />
      <div class="input-icon-right">
        <Shield :size="14" />
      </div>
    </div>
    <p v-if="error" class="field-error">{{ error }}</p>
    <p class="field-hint secure">安全存储在系统密钥链中</p>
  </div>
</template>

<style src="../settings.shared.css"></style>
