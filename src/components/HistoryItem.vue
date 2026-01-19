<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { Clock, Trash2, Shield } from 'lucide-vue-next'
import type { ClipboardHistoryEntry } from '@/types'

export interface HistoryItemProps {
  entry: ClipboardHistoryEntry
  selected?: boolean
}

const props = withDefaults(defineProps<HistoryItemProps>(), {
  selected: false,
})

const emit = defineEmits<{
  select: [id: number]
  paste: [id: number]
  delete: [id: number]
}>()

// Reactive timestamp for dynamic timeAgo updates
const now = ref(Date.now())
let intervalId: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  // Update timestamp every 30 seconds for dynamic relative time
  intervalId = setInterval(() => {
    now.value = Date.now()
  }, 30000)
})

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId)
    intervalId = null
  }
})

const timeAgo = computed(() => {
  const diff = now.value - props.entry.createdAtMs
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 0) return `${days}d ago`
  if (hours > 0) return `${hours}h ago`
  if (minutes > 0) return `${minutes}m ago`
  return 'just now'
})

const sizeDisplay = computed(() => {
  const bytes = props.entry.byteLength
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
})

// Preview text is now handled via CSS line-clamp for better performance

function handleClick() {
  emit('select', props.entry.id)
}

function handleDoubleClick() {
  emit('paste', props.entry.id)
}

function handleDelete(e: Event) {
  e.stopPropagation()
  emit('delete', props.entry.id)
}
</script>

<template>
  <div
    class="history-item group"
    :class="{ selected }"
    role="option"
    :aria-selected="selected"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    tabindex="0"
  >
    <!-- Preview content -->
    <div class="preview-text">{{ entry.preview }}</div>

    <!-- Meta info -->
    <div class="meta-row">
      <div class="meta-left">
        <Clock class="w-3 h-3" />
        <span>{{ timeAgo }}</span>
        <span class="meta-divider">·</span>
        <span>{{ sizeDisplay }}</span>
        <template v-if="entry.piiDetected">
          <span class="meta-divider">·</span>
          <Shield class="w-3 h-3 text-amber-500" />
          <span class="text-amber-500">PII</span>
        </template>
      </div>

      <!-- Delete button -->
      <button
        class="delete-btn"
        @click="handleDelete"
        title="Delete"
      >
        <Trash2 class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.history-item {
  padding: 0.625rem 0.75rem;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.15s ease;
  background: var(--bg-secondary);
  border: 1px solid transparent;
}

.history-item:hover {
  background: var(--bg-tertiary);
}

.history-item.selected {
  background: var(--accent-subtle);
  border-color: var(--accent-primary);
}

.history-item:focus {
  outline: none;
  box-shadow: 0 0 0 2px var(--accent-primary);
}

.preview-text {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.8125rem;
  line-height: 1.4;
  color: var(--text-primary);
  word-break: break-word;
  margin-bottom: 0.375rem;
  /* CSS line-clamp for efficient text truncation */
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.meta-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.6875rem;
  color: var(--text-tertiary);
}

.meta-left {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.meta-divider {
  opacity: 0.5;
}

.delete-btn {
  opacity: 0;
  padding: 0.25rem;
  border-radius: 0.25rem;
  color: var(--text-tertiary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.history-item:hover .delete-btn,
.history-item.selected .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}
</style>
