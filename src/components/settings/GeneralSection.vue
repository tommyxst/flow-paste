<script setup lang="ts">
import { Keyboard, Palette } from 'lucide-vue-next'
import type { AppConfig } from '@/types'

defineProps<{
  formData: AppConfig
  errors: Record<string, string>
}>()
</script>

<template>
  <section class="settings-section">
    <div class="section-header">
      <div class="section-icon general">
        <Keyboard :size="16" />
      </div>
      <h3>通用设置</h3>
    </div>
    <div class="section-content">
      <div class="field-group">
        <label class="field-label">
          <Keyboard :size="14" />
          <span>全局热键</span>
        </label>
        <input
          v-model="formData.hotkey"
          type="text"
          class="field-input"
          :class="{ 'has-error': errors.hotkey }"
          placeholder="Ctrl+Shift+V"
        />
        <p v-if="errors.hotkey" class="field-error">{{ errors.hotkey }}</p>
        <p class="field-hint">支持: Ctrl, Shift, Alt, Meta, CommandOrControl</p>
      </div>

      <div class="field-group">
        <label class="field-label">
          <Palette :size="14" />
          <span>外观主题</span>
        </label>
        <div class="theme-selector">
          <button
            type="button"
            v-for="opt in [
              {value: 'system', label: '自动'},
              {value: 'light', label: '浅色'},
              {value: 'dark', label: '深色'}
            ]"
            :key="opt.value"
            @click="formData.theme = opt.value as 'system' | 'light' | 'dark'"
            class="theme-option"
            :class="{ active: formData.theme === opt.value }"
          >
            {{ opt.label }}
          </button>
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

.section-icon.general {
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  box-shadow: 0 2px 6px rgba(99, 102, 241, 0.3);
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

.field-input {
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

.dark .field-input {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.08);
}

.field-input:focus {
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

.theme-selector {
  display: flex;
  gap: 6px;
  background: rgba(0, 0, 0, 0.03);
  padding: 4px;
  border-radius: 8px;
}

.dark .theme-selector {
  background: rgba(255, 255, 255, 0.05);
}

.theme-option {
  flex: 1;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.theme-option:hover {
  color: var(--text-primary);
}

.theme-option.active {
  background: white;
  color: var(--text-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.dark .theme-option.active {
  background: rgba(255, 255, 255, 0.1);
}
</style>
