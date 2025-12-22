<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import type { ActionChip } from '@/types'

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
  // Don't interfere with input fields
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
  const base = 'px-4 py-2 text-sm rounded-full transition-all duration-200 flex items-center gap-2'
  const isSelected = index === props.selectedIndex

  if (isSelected) {
    return `${base} bg-blue-500 text-white ring-2 ring-blue-300 dark:ring-blue-600`
  }
  return `${base} bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 cursor-pointer`
}
</script>

<template>
  <div v-if="chips.length > 0" class="action-chips-container">
    <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">
      智能推荐操作
    </div>
    <div class="flex flex-wrap gap-2">
      <button
        v-for="(chip, index) in chips"
        :key="chip.id"
        :class="chipClasses(index)"
        @click="handleClick(chip)"
      >
        <span
          v-if="chip.shortcut"
          class="inline-flex items-center justify-center w-5 h-5 text-xs font-bold rounded"
          :class="index === selectedIndex ? 'bg-white/20' : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400'"
        >
          {{ chip.shortcut }}
        </span>
        <span>{{ chip.label }}</span>
      </button>
    </div>
    <div class="text-xs text-gray-400 dark:text-gray-500 mt-2">
      💡 按数字键 1-3 快速选择
    </div>
  </div>
</template>

<style scoped>
.action-chips-container button {
  user-select: none;
}

.action-chips-container button:active {
  transform: scale(0.98);
}
</style>
