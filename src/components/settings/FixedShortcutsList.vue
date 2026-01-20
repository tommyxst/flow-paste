<script setup lang="ts">
import KeyCapGroup from '@/components/common/KeyCapGroup.vue'

interface ShortcutItem {
  label: string
  keys: string[]
}

const FIXED_SHORTCUTS: ShortcutItem[] = [
  { label: '打开设置', keys: ['Ctrl', 'Alt', 'S'] },
  { label: '切换历史', keys: ['Ctrl', 'H'] },
  { label: '关闭面板', keys: ['Esc'] },
  { label: '确认粘贴', keys: ['Enter'] },
  { label: '快捷操作', keys: ['Alt', '1-3'] }
]
</script>

<template>
  <div class="fixed-shortcuts">
    <div class="shortcuts-header">
      <span class="shortcuts-title">应用内快捷键</span>
      <span class="shortcuts-badge">固定</span>
    </div>
    <div class="shortcuts-grid">
      <div v-for="item in FIXED_SHORTCUTS" :key="item.label" class="shortcut-item">
        <span class="shortcut-label">{{ item.label }}</span>
        <KeyCapGroup :keys="item.keys" size="sm" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.fixed-shortcuts {
  margin-top: 8px;
  padding-top: 14px;
  border-top: 1px dashed rgba(0, 0, 0, 0.06);
}

.dark .fixed-shortcuts {
  border-top-color: rgba(255, 255, 255, 0.06);
}

.shortcuts-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.shortcuts-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

.shortcuts-badge {
  font-size: 10px;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.04);
  color: var(--text-tertiary);
  border-radius: 8px;
  font-weight: 500;
}

.dark .shortcuts-badge {
  background: rgba(255, 255, 255, 0.06);
}

.shortcuts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.shortcut-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 6px;
  transition: background 0.15s ease;
}

.dark .shortcut-item {
  background: rgba(255, 255, 255, 0.02);
}

.shortcut-item:hover {
  background: rgba(0, 0, 0, 0.04);
}

.dark .shortcut-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.shortcut-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}
</style>
