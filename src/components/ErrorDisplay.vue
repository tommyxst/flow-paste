<script setup lang="ts">
/**
 * ErrorDisplay Component
 *
 * Displays error messages with animations and retry functionality.
 * Improvements from review:
 * - Uses CSS variables for theming
 * - Supports severity levels (error/warning/info)
 * - Includes retry with count tracking
 * - Respects prefers-reduced-motion via CSS
 */
import { computed, onMounted, onUnmounted } from 'vue'
import { AlertCircle, AlertTriangle, Info, RefreshCw, X } from 'lucide-vue-next'

export type ErrorSeverity = 'error' | 'warning' | 'info'

export interface ErrorInfo {
  message: string
  code?: string
  severity?: ErrorSeverity
  recoverable?: boolean
  autoHideMs?: number
}

const props = withDefaults(
  defineProps<{
    error: ErrorInfo
    retryCount?: number
    maxRetries?: number
    dismissible?: boolean
  }>(),
  {
    retryCount: 0,
    maxRetries: 3,
    dismissible: true,
  }
)

const emit = defineEmits<{
  retry: []
  dismiss: []
}>()

const severity = computed(() => props.error.severity ?? 'error')

const isRecoverable = computed(
  () => props.error.recoverable !== false && props.retryCount < props.maxRetries
)

const canRetry = computed(() => isRecoverable.value && props.retryCount < props.maxRetries)

const IconComponent = computed(() => {
  switch (severity.value) {
    case 'warning':
      return AlertTriangle
    case 'info':
      return Info
    default:
      return AlertCircle
  }
})

// Auto-hide logic
let autoHideTimeout: ReturnType<typeof setTimeout> | null = null

onMounted(() => {
  if (props.error.autoHideMs && props.error.autoHideMs > 0) {
    autoHideTimeout = setTimeout(() => {
      emit('dismiss')
    }, props.error.autoHideMs)
  }
})

onUnmounted(() => {
  if (autoHideTimeout) {
    clearTimeout(autoHideTimeout)
  }
})

function handleRetry() {
  emit('retry')
}

function handleDismiss() {
  emit('dismiss')
}
</script>

<template>
  <div
    class="error-display animate-slide-in-down"
    :class="[`error-display--${severity}`]"
    role="alert"
    :aria-live="severity === 'error' ? 'assertive' : 'polite'"
  >
    <div class="flex items-center gap-3">
      <component :is="IconComponent" class="w-5 h-5 shrink-0 error-icon" />
      <div class="flex-1 min-w-0">
        <p class="font-medium text-sm truncate error-text">
          {{ error.message }}
        </p>
        <p v-if="error.code" class="text-xs mt-0.5 error-text-secondary">
          Code: {{ error.code }}
        </p>
      </div>
    </div>

    <div class="flex items-center gap-2 mt-3">
      <button
        v-if="canRetry"
        @click="handleRetry"
        class="error-button flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm transition-colors"
      >
        <RefreshCw class="w-3.5 h-3.5" />
        Retry
        <span v-if="retryCount > 0" class="opacity-70">({{ retryCount }}/{{ maxRetries }})</span>
      </button>

      <button
        v-if="dismissible"
        @click="handleDismiss"
        class="error-button flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm transition-colors"
      >
        <X class="w-3.5 h-3.5" />
        Dismiss
      </button>
    </div>
  </div>
</template>

<style scoped>
.error-display {
  padding: 1rem;
  border-radius: 0.75rem;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.2);
}

/* Error severity (default) */
.error-display--error {
  background-color: var(--error-bg);
  color: var(--error-text);
}

.error-display--error .error-icon {
  color: var(--error-icon);
}

.error-display--error .error-text-secondary {
  color: var(--error-text-secondary);
}

/* Warning severity */
.error-display--warning {
  background-color: var(--warning-bg);
  color: var(--error-text);
}

.error-display--warning .error-icon {
  color: var(--warning-icon);
}

/* Info severity */
.error-display--info {
  background-color: var(--accent-primary);
  color: var(--error-text);
}

.error-display--info .error-icon {
  color: rgba(255, 255, 255, 0.8);
}

/* Button styling */
.error-button {
  background-color: rgba(255, 255, 255, 0.1);
  color: inherit;
}

.error-button:hover {
  background-color: rgba(255, 255, 255, 0.2);
}

/* Text classes */
.error-text {
  color: inherit;
}

.error-text-secondary {
  color: var(--error-text-secondary);
}
</style>
