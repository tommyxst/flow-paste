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

// Module-level state to ensure single instance
let initialized = false
const isDark = ref(false)
const prefersReducedMotion = ref(false)

// Store cleanup functions
let themeCleanup: (() => void) | null = null
let motionCleanup: (() => void) | null = null

function initTheme() {
  if (initialized || typeof window === 'undefined') return

  // Theme detection
  const themeQuery = window.matchMedia('(prefers-color-scheme: dark)')
  isDark.value = themeQuery.matches
  applyTheme(isDark.value)

  const themeListener = (e: MediaQueryListEvent) => {
    isDark.value = e.matches
    applyTheme(e.matches)
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
