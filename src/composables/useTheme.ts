/**
 * Theme management composable
 *
 * Handles system theme detection and applies dark/light mode.
 * Fixed issues from review:
 * - Proper cleanup of media query listeners (no memory leak)
 * - Single shared instance via module-level state
 * - prefers-reduced-motion support for accessibility
 */
import { ref, readonly, onMounted, onUnmounted } from 'vue'

export type ThemeMode = 'system' | 'light' | 'dark'

// Module-level state to ensure single instance
let initialized = false
const isDark = ref(false)
const prefersReducedMotion = ref(false)
let currentThemeMode: ThemeMode = 'system'

// Store cleanup functions
let themeCleanup: (() => void) | null = null
let motionCleanup: (() => void) | null = null

function initTheme() {
  if (initialized || typeof window === 'undefined') return

  // Theme detection - only apply system preference if mode is 'system'
  const themeQuery = window.matchMedia('(prefers-color-scheme: dark)')

  if (currentThemeMode === 'system') {
    isDark.value = themeQuery.matches
    applyTheme(isDark.value)
  }

  const themeListener = (e: MediaQueryListEvent) => {
    // Only respond to system changes when in 'system' mode
    if (currentThemeMode === 'system') {
      isDark.value = e.matches
      applyTheme(e.matches)
    }
  }
  themeQuery.addEventListener('change', themeListener)
  themeCleanup = () => themeQuery.removeEventListener('change', themeListener)

  // Reduced motion detection
  const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
  prefersReducedMotion.value = motionQuery.matches
  applyReducedMotion(motionQuery.matches)

  const motionListener = (e: MediaQueryListEvent) => {
    prefersReducedMotion.value = e.matches
    applyReducedMotion(e.matches)
  }
  motionQuery.addEventListener('change', motionListener)
  motionCleanup = () => motionQuery.removeEventListener('change', motionListener)

  initialized = true
}

function applyTheme(dark: boolean) {
  const html = document.documentElement
  html.classList.toggle('dark', dark)
}

function applyReducedMotion(reduced: boolean) {
  const html = document.documentElement
  html.classList.toggle('reduce-motion', reduced)
}

function cleanup() {
  themeCleanup?.()
  motionCleanup?.()
  themeCleanup = null
  motionCleanup = null
  initialized = false
}

/**
 * Set theme mode - call this when user changes theme preference
 */
export function setTheme(mode: ThemeMode) {
  currentThemeMode = mode

  if (typeof window === 'undefined') return

  if (mode === 'system') {
    const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    isDark.value = systemDark
    applyTheme(systemDark)
  } else {
    const dark = mode === 'dark'
    isDark.value = dark
    applyTheme(dark)
  }
}

/**
 * Theme composable for detecting and applying system theme
 *
 * Usage:
 * ```ts
 * const { isDark, prefersReducedMotion } = useTheme()
 * ```
 */
export function useTheme() {
  onMounted(() => {
    initTheme()
  })

  // Note: We don't cleanup on unmount because theme should persist
  // Only cleanup when app is destroyed

  return {
    isDark: readonly(isDark),
    prefersReducedMotion: readonly(prefersReducedMotion),
  }
}

/**
 * Provider version - call once in App.vue
 * Handles cleanup on app unmount
 */
export function useThemeProvider() {
  onMounted(() => {
    initTheme()
  })

  onUnmounted(() => {
    cleanup()
  })

  return {
    isDark: readonly(isDark),
    prefersReducedMotion: readonly(prefersReducedMotion),
  }
}
