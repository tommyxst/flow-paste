import { ref, computed, type Ref } from 'vue'

const MODIFIERS = new Set(['Ctrl', 'Shift', 'Alt', 'Win'])

const DISPLAY_MAP: Record<string, string> = {
  Control: 'Ctrl',
  Meta: 'Win',
  Command: 'Cmd',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  Escape: 'Esc',
  ' ': 'Space'
}

const MODIFIER_ORDER = ['Win', 'Meta', 'Cmd', 'Ctrl', 'Control', 'Alt', 'Shift']

function sortKeys(keys: string[]): string[] {
  return [...keys].sort((a, b) => {
    const aIdx = MODIFIER_ORDER.indexOf(a)
    const bIdx = MODIFIER_ORDER.indexOf(b)
    if (aIdx !== -1 && bIdx !== -1) return aIdx - bIdx
    if (aIdx !== -1) return -1
    if (bIdx !== -1) return 1
    return a.localeCompare(b)
  })
}

export interface HotkeyRecorderState {
  isRecording: Ref<boolean>
  pressedKeys: Ref<Set<string>>
  currentCombo: Ref<string[]>
  error: Ref<string | null>
}

export interface HotkeyRecorderActions {
  startRecording: () => void
  stopRecording: () => void
  setCombo: (keys: string[]) => void
  validateCombo: (keys: string[]) => boolean
  mapKey: (e: KeyboardEvent) => string
  resetError: () => void
}

export type UseHotkeyRecorderReturn = HotkeyRecorderState & HotkeyRecorderActions

export function useHotkeyRecorder(initialValue = ''): UseHotkeyRecorderReturn {
  const isRecording = ref(false)
  const pressedKeys = ref<Set<string>>(new Set())
  const recordedCombo = ref<string[]>(
    initialValue ? initialValue.split('+').filter(Boolean) : []
  )
  const error = ref<string | null>(null)

  const currentCombo = computed(() => {
    if (isRecording.value && pressedKeys.value.size > 0) {
      return sortKeys(Array.from(pressedKeys.value))
    }
    return sortKeys(recordedCombo.value)
  })

  function mapKey(e: KeyboardEvent): string {
    const key = e.key
    if (key in DISPLAY_MAP) {
      return DISPLAY_MAP[key]
    }
    return key.length === 1 ? key.toUpperCase() : key
  }

  function validateCombo(keys: string[]): boolean {
    if (keys.length === 0) {
      error.value = '快捷键不能为空'
      return false
    }
    if (keys.length > 3) {
      error.value = '最多支持 3 个按键组合'
      return false
    }

    const hasModifier = keys.some((k) => MODIFIERS.has(k))
    const hasMainKey = keys.some((k) => !MODIFIERS.has(k))

    if (!hasModifier) {
      error.value = '需包含修饰键 (Ctrl/Alt/Shift/Win)'
      return false
    }
    if (!hasMainKey) {
      error.value = '需包含主按键'
      return false
    }

    error.value = null
    return true
  }

  function startRecording() {
    isRecording.value = true
    pressedKeys.value = new Set()
    error.value = null
  }

  function stopRecording() {
    isRecording.value = false
    pressedKeys.value = new Set()
  }

  function setCombo(keys: string[]) {
    recordedCombo.value = sortKeys(keys)
  }

  function resetError() {
    error.value = null
  }

  return {
    isRecording,
    pressedKeys,
    currentCombo,
    error,
    startRecording,
    stopRecording,
    setCombo,
    validateCombo,
    mapKey,
    resetError
  }
}
