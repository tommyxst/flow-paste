<script setup lang="ts">
import { Wifi, Loader2, Check, AlertCircle } from 'lucide-vue-next'

defineProps<{
  isTesting: boolean
  testResult: { success: boolean; message: string } | null
}>()

const emit = defineEmits<{
  test: []
}>()
</script>

<template>
  <div class="connection-test">
    <button type="button" @click="emit('test')" :disabled="isTesting" class="test-btn">
      <Loader2 v-if="isTesting" :size="14" class="animate-spin" />
      <Wifi v-else :size="14" />
      <span>{{ isTesting ? '测试中...' : '测试连接' }}</span>
    </button>
    <div
      v-if="testResult"
      class="test-result"
      role="status"
      :class="{ success: testResult.success, error: !testResult.success }"
    >
      <Check v-if="testResult.success" :size="14" aria-hidden="true" />
      <AlertCircle v-else :size="14" aria-hidden="true" />
      <span>{{ testResult.message }}</span>
    </div>
  </div>
</template>

<style scoped>
.connection-test {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 6px;
}

.test-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 16px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: rgba(0, 0, 0, 0.03);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.dark .test-btn {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.08);
}

.test-btn:hover:not(:disabled) {
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.test-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  font-size: 12px;
  border-radius: 6px;
  animation: slideInDown 0.2s ease;
}

.test-result.success {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.test-result.error {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}
</style>
