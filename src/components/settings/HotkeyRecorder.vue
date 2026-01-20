<script setup lang="ts">
import { watch, onBeforeUnmount, computed } from 'vue'
import KeyCapGroup from '@/components/common/KeyCapGroup.vue'
import { useHotkeyRecorder } from '@/composables/useHotkeyRecorder'

const props = defineProps<{
  modelValue: string
  disabled?: boolean
  error?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const {
  isRecording,
  pressedKeys,
  currentCombo,
  error: internalError,
  startRecording,
  stopRecording,
  validateCombo,
  mapKey,
  setCombo
} = useHotkeyRecorder(props.modelValue)

watch(
  () => props.modelValue,
  (newVal) => {
    if (!isRecording.value) {
      setCombo(newVal ? newVal.split('+').filter(Boolean) : [])
    }
  }
)

function handleKeyDown(e: KeyboardEvent) {
  if (!isRecording.value) return
  e.preventDefault()
  e.stopPropagation()

  if (e.key === 'Escape') {
    stopRecording()
    setCombo(props.modelValue ? props.modelValue.split('+').filter(Boolean) : [])
    return
  }

  const key = mapKey(e)
  pressedKeys.value.add(key)
  setCombo(Array.from(pressedKeys.value))
}

function handleKeyUp(e: KeyboardEvent) {
  if (!isRecording.value) return
  e.preventDefault()

  const key = mapKey(e)
  pressedKeys.value.delete(key)

  if (pressedKeys.value.size === 0) {
    if (validateCombo(currentCombo.value)) {
      emit('update:modelValue', currentCombo.value.join('+'))
      stopRecording()
    }
  }
}

function toggleRecording() {
  if (props.disabled) return

  if (isRecording.value) {
    stopRecording()
  } else {
    startRecording()
    window.addEventListener('keydown', handleKeyDown)
    window.addEventListener('keyup', handleKeyUp)
  }
}

function clear() {
  if (props.disabled) return
  emit('update:modelValue', '')
  setCombo([])
}

watch(isRecording, (recording) => {
  if (!recording) {
    window.removeEventListener('keydown', handleKeyDown)
    window.removeEventListener('keyup', handleKeyUp)
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeyDown)
  window.removeEventListener('keyup', handleKeyUp)
})

const displayError = computed(() => props.error || internalError.value)
</script>

<template>
  <div class="hotkey-recorder">
    <div
      class="recorder-display"
      :class="{
        'is-recording': isRecording,
        'has-error': !!displayError
      }"
    >
      <div class="recorder-keys">
        <KeyCapGroup
          v-if="currentCombo.length > 0"
          :keys="currentCombo"
          :pressed-keys="pressedKeys"
        />
        <span v-else class="placeholder">{{ isRecording ? '按下快捷键...' : '未设置' }}</span>
      </div>

      <div class="recorder-actions">
        <button
          v-if="!isRecording"
          type="button"
          class="recorder-btn"
          :disabled="disabled"
          @click="toggleRecording"
        >
          {{ currentCombo.length ? '修改' : '录入' }}
        </button>
        <button
          v-else
          type="button"
          class="recorder-btn cancel"
          @click="toggleRecording"
        >
          取消
        </button>

        <button
          v-if="currentCombo.length > 0 && !isRecording"
          type="button"
          class="recorder-btn danger"
          :disabled="disabled"
          @click="clear"
        >
          清除
        </button>
      </div>
    </div>

    <p v-if="displayError" class="recorder-error">{{ displayError }}</p>
    <p v-else-if="isRecording" class="recorder-hint">按下快捷键组合，松开以确认 (Esc 取消)</p>
    <p v-else class="recorder-hint">支持: Ctrl, Shift, Alt, Win</p>
  </div>
</template>

<style scoped>
.hotkey-recorder {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.recorder-display {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  min-height: 44px;
  background: rgba(0, 0, 0, 0.03);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 8px;
  transition: all 0.15s ease;
}

.dark .recorder-display {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.08);
}

.recorder-display.is-recording {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-subtle);
  background: rgba(99, 102, 241, 0.05);
}

.recorder-display.has-error {
  border-color: #ef4444;
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
}

.recorder-keys {
  display: flex;
  align-items: center;
  min-height: 28px;
}

.placeholder {
  font-size: 13px;
  color: var(--text-tertiary);
}

.recorder-actions {
  display: flex;
  gap: 6px;
  margin-left: 12px;
}

.recorder-btn {
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.recorder-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.05);
  color: var(--text-primary);
}

.dark .recorder-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
}

.recorder-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.recorder-btn.cancel {
  color: var(--accent-primary);
}

.recorder-btn.danger:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

.recorder-error {
  font-size: 11px;
  color: #ef4444;
  margin: 0;
}

.recorder-hint {
  font-size: 11px;
  color: var(--text-tertiary);
  margin: 0;
}
</style>
