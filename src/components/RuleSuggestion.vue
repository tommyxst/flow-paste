<script setup lang="ts">
import { useAppStore } from '@/stores/app'
import { Sparkles, X, Check } from 'lucide-vue-next'

const store = useAppStore()
</script>

<template>
  <div
    v-if="store.ruleSuggestion"
    class="flex items-center gap-3 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg"
  >
    <Sparkles class="w-4 h-4 text-amber-500 shrink-0" />
    <div class="flex-1 min-w-0">
      <p class="text-sm text-amber-800 dark:text-amber-200 truncate">
        可保存为快捷规则: <span class="font-medium">{{ store.ruleSuggestion.name }}</span>
      </p>
    </div>
    <div class="flex gap-1 shrink-0">
      <button
        @click="store.saveRuleSuggestion()"
        class="p-1.5 rounded bg-amber-500 text-white hover:bg-amber-600 transition-colors"
        title="保存规则"
      >
        <Check class="w-3.5 h-3.5" />
      </button>
      <button
        @click="store.dismissRuleSuggestion()"
        class="p-1.5 rounded text-amber-600 hover:bg-amber-100 dark:hover:bg-amber-900/40 transition-colors"
        title="忽略"
      >
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
</template>
