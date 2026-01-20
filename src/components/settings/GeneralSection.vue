<script setup lang="ts">
import { Keyboard, Palette } from 'lucide-vue-next'
import type { AppConfig } from '@/types'
import SettingsSectionShell from './SettingsSectionShell.vue'
import HotkeyRecorder from './HotkeyRecorder.vue'
import FixedShortcutsList from './FixedShortcutsList.vue'

defineProps<{
  formData: AppConfig
  errors: Record<string, string>
}>()
</script>

<template>
  <SettingsSectionShell title="通用设置" :icon="Keyboard" icon-class="general">
      <div class="field-group">
        <label class="field-label">
          <Keyboard :size="14" />
          <span>全局热键</span>
        </label>
        <HotkeyRecorder
          v-model="formData.hotkey"
          :error="errors.hotkey"
        />
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

      <FixedShortcutsList />
  </SettingsSectionShell>
</template>

<style src="./settings.shared.css"></style>
<style scoped>
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
