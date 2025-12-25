<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import type { ActionChip } from '@/types'
import { Sparkles, Command } from 'lucide-vue-next'

export interface ActionChipsProps {
  chips: ActionChip[]
  selectedIndex?: number
}

const props = withDefaults(defineProps<ActionChipsProps>(), {
  selectedIndex: 0,
})

const emit = defineEmits<{
  select: [chip: ActionChip]
}>()

function handleClick(chip: ActionChip) {
  emit('select', chip)
}

function handleKeydown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
    return
  }

  const key = e.key
  if (key >= '1' && key <= '3') {
    const index = parseInt(key) - 1
    if (index < props.chips.length) {
      e.preventDefault()
      emit('select', props.chips[index])
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

const chipClasses = (index: number) => {
  const base = 'group relative pl-3 pr-3 py-1.5 text-sm rounded-lg transition-all duration-200 border flex items-center gap-2 cursor-pointer select-none'
  const isSelected = index === props.selectedIndex

  if (isSelected) {
    return `${base} bg-[var(--accent-subtle)] border-[var(--accent-primary)] text-[var(--accent-primary)] shadow-sm`
  }
  return `${base} bg-transparent border-[var(--panel-border)] text-[var(--text-secondary)] hover:bg-gray-100/50 dark:hover:bg-white/5 hover:border-[var(--text-tertiary)]`
}
</script>

<template>
  <div v-if="chips.length > 0" class="flex flex-wrap gap-2">
    <button
      v-for="(chip, index) in chips"
      :key="chip.id"
      :class="chipClasses(index)"
      @click="handleClick(chip)"
    >
      <Sparkles class="w-3.5 h-3.5 opacity-70" v-if="chip.actionType === 'AIPrompt'" />
      <Command class="w-3.5 h-3.5 opacity-70" v-else />
      
      <span class="font-medium">{{ chip.label }}</span>
      
      <!-- Shortcut Hint -->
      <span 
        v-if="index < 3"
        class="ml-1 text-[10px] opacity-50 font-mono bg-black/5 dark:bg-white/10 px-1 rounded"
      >
        {{ index + 1 }}
      </span>
    </button>
  </div>
</template>
