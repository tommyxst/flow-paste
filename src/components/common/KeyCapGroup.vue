<script setup lang="ts">
import KeyCap from './KeyCap.vue'

defineProps<{
  keys: string[]
  pressedKeys?: Set<string>
  size?: 'sm' | 'md'
}>()

const MODIFIER_KEYS = ['Ctrl', 'Control', 'Shift', 'Alt', 'Meta', 'Win', 'Cmd', 'Command']

function isModifier(key: string): boolean {
  return MODIFIER_KEYS.some(m => key.toLowerCase().includes(m.toLowerCase()))
}
</script>

<template>
  <div class="keycap-group">
    <template v-for="(key, index) in keys" :key="key">
      <span v-if="index > 0" class="keycap-separator">+</span>
      <KeyCap
        :key-name="key"
        :variant="isModifier(key) ? 'modifier' : 'primary'"
        :pressed="pressedKeys?.has(key)"
        :size="size"
      />
    </template>
  </div>
</template>

<style scoped>
.keycap-group {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.keycap-separator {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-tertiary);
  user-select: none;
}
</style>
