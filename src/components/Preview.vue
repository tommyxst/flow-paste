<script setup lang="ts">
import { computed } from 'vue'

export interface PreviewProps {
  content: string
  mode?: 'preview' | 'result' | 'streaming'
  highlight?: boolean
  truncated?: boolean
}

const props = withDefaults(defineProps<PreviewProps>(), {
  mode: 'preview',
  highlight: false,
  truncated: false,
})

const displayContent = computed(() => {
  if (props.mode === 'preview') {
    const lines = props.content.split('\n')
    // Show a few more lines for context if short, but cap it
    return lines.slice(0, 5).join('\n')
  }
  return props.content
})

const showEllipsis = computed(() => {
  if (props.mode === 'preview') {
    return props.content.split('\n').length > 5
  }
  return false
})

// Streaming typewriter effect cursor
// const cursorVisible = ref(true)

const streamingDisplay = computed(() => {
  if (props.mode === 'streaming') {
    return props.content
  }
  return ''
})

const containerClasses = computed(() => {
  const base = 'w-full rounded-lg overflow-hidden transition-colors duration-200'
  
  // In spotlight design, the preview often blends in, or has very subtle background
  if (props.mode === 'preview') {
    return `${base} bg-gray-50/50 dark:bg-black/20`
  }
  return `${base} bg-[var(--accent-subtle)]/30 border border-[var(--accent-primary)]/20`
})

const textClasses = computed(() => {
  const base = 'font-mono text-sm whitespace-pre-wrap break-words p-3 overflow-y-auto custom-scrollbar'
  
  if (props.mode === 'preview') {
    return `${base} text-[var(--text-secondary)] max-h-32 opacity-80`
  }
  // Result or Streaming
  return `${base} text-[var(--text-primary)] max-h-[60vh]`
})
</script>

<template>
  <div :class="containerClasses">
    <div :class="textClasses">
      <template v-if="mode === 'streaming'">
        {{ streamingDisplay }}<span class="inline-block w-1.5 h-4 bg-[var(--accent-primary)] ml-0.5 animate-pulse align-middle"></span>
      </template>
      <template v-else>
        {{ displayContent }}
      </template>
    </div>

    <!-- Footer info for Preview -->
    <div
      v-if="(mode === 'preview' && showEllipsis) || truncated"
      class="px-3 pb-2 pt-0 text-xs text-[var(--text-tertiary)] flex items-center gap-1"
    >
      <span>... {{ content.split('\n').length }} lines total</span>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--text-tertiary); /* fallback opacity handled via color */
  opacity: 0.2;
  border-radius: 4px;
}
</style>
