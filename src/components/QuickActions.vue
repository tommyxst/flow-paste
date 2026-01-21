<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useAppStore } from '@/stores/app'
import { Command, MoreHorizontal } from 'lucide-vue-next'

const store = useAppStore()
const showOverflow = ref(false)
const menuRef = ref<HTMLElement | null>(null)

function handleRuleClick(ruleId: string) {
  showOverflow.value = false
  store.processWithRule(ruleId)
}

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    showOverflow.value = false
  }
}

// Add/remove click outside listener when menu opens/closes
watch(showOverflow, (isOpen) => {
  if (isOpen) {
    document.addEventListener('click', handleClickOutside, true)
  } else {
    document.removeEventListener('click', handleClickOutside, true)
  }
})

function handleKeydown(e: KeyboardEvent) {
  // if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

  // Alt+1, Alt+2, Alt+3 for quick actions
  const key = e.key
  if (e.altKey && key >= '1' && key <= '3') {
    const index = parseInt(key) - 1
    const rules = store.visibleRules
    if (index < rules.length) {
      e.preventDefault()
      handleRuleClick(rules[index].id)
    }
  }
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('click', handleClickOutside, true)
})
</script>

<template>
  <div v-if="store.visibleRules.length > 0" class="flex flex-wrap gap-2 items-center">
    <!-- Visible Rules (max 3) -->
    <button
      v-for="(rule, index) in store.visibleRules"
      :key="rule.id"
      class="group pl-3 pr-3 py-1.5 text-sm rounded-lg transition-all duration-200 border flex items-center gap-2 cursor-pointer select-none bg-transparent border-[var(--panel-border)] text-[var(--text-secondary)] hover:bg-gray-100/50 dark:hover:bg-white/5 hover:border-[var(--text-tertiary)]"
      @click="handleRuleClick(rule.id)"
      :title="rule.description"
    >
      <Command class="w-3.5 h-3.5 opacity-70" />
      <span class="font-medium">{{ rule.name }}</span>
      <span class="ml-1 text-[10px] opacity-50 font-mono bg-black/5 dark:bg-white/10 px-1 rounded">
        Alt+{{ index + 1 }}
      </span>
    </button>

    <!-- Overflow Menu -->
    <div v-if="store.overflowRules.length > 0" class="relative" ref="menuRef">
      <button
        class="p-1.5 rounded-lg border border-[var(--panel-border)] text-[var(--text-tertiary)] hover:bg-gray-100/50 dark:hover:bg-white/5 transition-colors"
        @click="showOverflow = !showOverflow"
        title="More actions"
      >
        <MoreHorizontal class="w-4 h-4" />
      </button>

      <!-- Dropdown -->
      <div
        v-if="showOverflow"
        class="absolute bottom-full right-0 mb-1 py-1 bg-[var(--panel-bg)] border border-[var(--panel-border)] rounded-lg shadow-lg min-w-[160px] z-10"
      >
        <button
          v-for="rule in store.overflowRules"
          :key="rule.id"
          class="w-full px-3 py-2 text-sm text-left text-[var(--text-secondary)] hover:bg-gray-100/50 dark:hover:bg-white/5 flex items-center gap-2"
          @click="handleRuleClick(rule.id)"
        >
          <Command class="w-3.5 h-3.5 opacity-70" />
          {{ rule.name }}
        </button>
      </div>
    </div>
  </div>
</template>
