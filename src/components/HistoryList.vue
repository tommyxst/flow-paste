<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue'
import { History, Trash2, X } from 'lucide-vue-next'
import { useAppStore } from '@/stores/app'
import HistoryItem from './HistoryItem.vue'

const store = useAppStore()
const listRef = ref<HTMLDivElement | null>(null)

// Platform detection for keyboard hints
const isMac = computed(() => navigator.platform.toLowerCase().includes('mac'))

function handleKeydown(e: KeyboardEvent) {
  if (!store.isHistoryMode) return

  const entries = store.historyEntries
  if (entries.length === 0) return

  switch (e.key) {
    case 'ArrowUp':
    case 'k':
      e.preventDefault()
      store.selectedHistoryIndex = Math.max(0, store.selectedHistoryIndex - 1)
      scrollToSelected()
      break
    case 'ArrowDown':
    case 'j':
      e.preventDefault()
      store.selectedHistoryIndex = Math.min(entries.length - 1, store.selectedHistoryIndex + 1)
      scrollToSelected()
      break
    case 'Enter':
      e.preventDefault()
      if (entries[store.selectedHistoryIndex]) {
        store.pasteFromHistory(entries[store.selectedHistoryIndex].id)
      }
      break
    case 'Delete':
    case 'Backspace':
      if (e.metaKey || e.ctrlKey) {
        e.preventDefault()
        if (entries[store.selectedHistoryIndex]) {
          store.deleteHistoryEntry(entries[store.selectedHistoryIndex].id)
        }
      }
      break
    case 'Escape':
      e.preventDefault()
      store.hideHistory()
      break
  }
}

function scrollToSelected() {
  const container = listRef.value
  if (!container) return

  const items = container.querySelectorAll('.history-item')
  const selectedItem = items[store.selectedHistoryIndex] as HTMLElement
  if (selectedItem) {
    selectedItem.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
  }
}

function handleSelect(id: number) {
  const index = store.historyEntries.findIndex(e => e.id === id)
  if (index !== -1) {
    store.selectedHistoryIndex = index
  }
}

function handlePaste(id: number) {
  store.pasteFromHistory(id)
}

function handleDelete(id: number) {
  store.deleteHistoryEntry(id)
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

watch(() => store.isHistoryMode, (isHistory) => {
  if (isHistory) {
    store.selectedHistoryIndex = 0
  }
})
</script>

<template>
  <div class="history-panel">
    <!-- Header -->
    <div class="history-header">
      <div class="header-left">
        <History class="w-4 h-4" />
        <span>Clipboard History</span>
        <span class="count-badge">{{ store.historyEntries.length }}</span>
      </div>
      <div class="header-actions">
        <button
          v-if="store.historyEntries.length > 0"
          class="clear-btn"
          @click="store.clearHistory"
          title="Clear all history"
        >
          <Trash2 class="w-3.5 h-3.5" />
        </button>
        <button
          class="close-btn"
          @click="store.hideHistory"
          title="Close (Esc)"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- List -->
    <div
      ref="listRef"
      class="history-list"
      role="listbox"
      aria-label="Clipboard history"
    >
      <template v-if="store.historyLoading">
        <div class="empty-state">Loading...</div>
      </template>
      <template v-else-if="store.historyEntries.length === 0">
        <div class="empty-state">
          <History class="w-8 h-8 opacity-30" />
          <span>No clipboard history yet</span>
        </div>
      </template>
      <template v-else>
        <HistoryItem
          v-for="(entry, index) in store.historyEntries"
          :key="entry.id"
          :entry="entry"
          :selected="index === store.selectedHistoryIndex"
          @select="handleSelect"
          @paste="handlePaste"
          @delete="handleDelete"
        />
      </template>
    </div>

    <!-- Footer hints -->
    <div class="history-footer">
      <span><kbd>↑↓</kbd> navigate</span>
      <span><kbd>Enter</kbd> paste</span>
      <span><kbd>{{ isMac ? '⌘⌫' : 'Ctrl+Del' }}</kbd> delete</span>
      <span><kbd>Esc</kbd> close</span>
    </div>
  </div>
</template>

<style scoped>
.history-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  max-height: 400px;
}

.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border-primary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.count-badge {
  font-size: 0.6875rem;
  padding: 0.125rem 0.375rem;
  border-radius: 9999px;
  background: var(--accent-subtle);
  color: var(--accent-primary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.clear-btn,
.close-btn {
  padding: 0.375rem;
  border-radius: 0.375rem;
  color: var(--text-tertiary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.clear-btn:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

.close-btn:hover {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.history-list::-webkit-scrollbar {
  width: 4px;
}

.history-list::-webkit-scrollbar-track {
  background: transparent;
}

.history-list::-webkit-scrollbar-thumb {
  background: var(--text-tertiary);
  border-radius: 4px;
  opacity: 0.3;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: var(--text-tertiary);
  font-size: 0.8125rem;
}

.history-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 0.5rem;
  border-top: 1px solid var(--border-primary);
  font-size: 0.6875rem;
  color: var(--text-tertiary);
}

.history-footer kbd {
  display: inline-block;
  padding: 0.125rem 0.25rem;
  font-family: inherit;
  font-size: 0.625rem;
  background: var(--bg-tertiary);
  border-radius: 0.25rem;
  margin-right: 0.125rem;
}
</style>
