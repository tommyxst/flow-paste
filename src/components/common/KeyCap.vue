<script setup lang="ts">
defineProps<{
  keyName: string
  variant?: 'modifier' | 'primary'
  pressed?: boolean
  size?: 'sm' | 'md'
}>()
</script>

<template>
  <span
    class="keycap"
    :class="[
      variant === 'primary' ? 'keycap-primary' : 'keycap-modifier',
      { 'keycap-pressed': pressed },
      size === 'sm' ? 'keycap-sm' : ''
    ]"
  >
    {{ keyName }}
  </span>
</template>

<style scoped>
.keycap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 600;
  font-family: system-ui, -apple-system, sans-serif;
  border-radius: 6px;
  transition: all 0.1s ease;
  user-select: none;
  white-space: nowrap;
}

.keycap-sm {
  padding: 2px 6px;
  font-size: 11px;
  border-radius: 4px;
}

.keycap-modifier {
  background: linear-gradient(180deg, #f5f5f5 0%, #e8e8e8 100%);
  color: #374151;
  border: 1px solid rgba(0, 0, 0, 0.12);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06), inset 0 1px 0 rgba(255, 255, 255, 0.8);
}

.dark .keycap-modifier {
  background: linear-gradient(180deg, #3f3f46 0%, #27272a 100%);
  color: #e4e4e7;
  border-color: rgba(255, 255, 255, 0.1);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

.keycap-primary {
  background: linear-gradient(135deg, #8b5cf6 0%, #6366f1 100%);
  color: white;
  border: 1px solid rgba(99, 102, 241, 0.3);
  box-shadow: 0 1px 3px rgba(99, 102, 241, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.keycap-pressed {
  transform: translateY(1px) scale(0.98);
  box-shadow: none;
}
</style>
