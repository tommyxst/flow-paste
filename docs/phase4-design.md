# Phase 4: Integration & Polish - Design Document

## Overview

Phase 4 focuses on completing the user experience critical features: paste-to-cursor functionality, theme support, and error handling UI.

## Task 27: Paste-to-Cursor Functionality

### Objective
Implement keyboard simulation to paste processed content directly at the cursor position in the target application.

### Technical Approach

#### Backend (Rust)

**New File:** `src-tauri/src/clipboard/paste.rs`

```rust
use enigo::{Enigo, Key, KeyboardControllable};

pub enum PasteError {
    ClipboardWrite(String),
    SimulationFailed(String),
    PermissionDenied,
}

/// Paste content to cursor by:
/// 1. Writing to clipboard
/// 2. Simulating Ctrl/Cmd+V keystroke
pub fn paste_to_cursor(app: &AppHandle, text: &str) -> Result<(), PasteError> {
    // Step 1: Write to clipboard
    write_clipboard(app, text)?;

    // Step 2: Small delay for clipboard sync
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Step 3: Simulate paste keystroke
    let mut enigo = Enigo::new();

    #[cfg(target_os = "macos")]
    enigo.key_down(Key::Meta);
    #[cfg(not(target_os = "macos"))]
    enigo.key_down(Key::Control);

    enigo.key_click(Key::Layout('v'));

    #[cfg(target_os = "macos")]
    enigo.key_up(Key::Meta);
    #[cfg(not(target_os = "macos"))]
    enigo.key_up(Key::Control);

    Ok(())
}
```

**Dependencies:**
```toml
enigo = "0.3"
```

#### Frontend Integration

Update `confirmPaste()` in `src/stores/app.ts`:
```typescript
async function confirmPaste() {
  const contentToPaste = processedContent.value || clipboardText.value
  if (!contentToPaste) return

  try {
    // Use new paste-to-cursor command
    await commands.pasteToCursor(contentToPaste)
    hidePanel()
    reset()
  } catch (e) {
    // Fallback to clipboard-only if paste simulation fails
    await commands.writeClipboard(contentToPaste)
    setError('Pasted to clipboard (simulation failed)')
  }
}
```

### Error Handling
- If keyboard simulation fails (permissions), fallback to clipboard-only
- Show user-friendly message explaining the fallback

---

## Task 28: Dark/Light Mode Support

### Objective
Follow system theme preference and apply appropriate styling throughout the UI.

### Technical Approach

#### 1. TailwindCSS Configuration

**File:** `tailwind.config.js` (or inline in `vite.config.ts`)
```js
export default {
  darkMode: 'class', // Use class-based dark mode
  // ...
}
```

#### 2. Theme Detection Composable

**New File:** `src/composables/useTheme.ts`
```typescript
import { ref, onMounted, onUnmounted } from 'vue'

export function useTheme() {
  const isDark = ref(false)

  function updateTheme() {
    isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
    document.documentElement.classList.toggle('dark', isDark.value)
  }

  onMounted(() => {
    updateTheme()
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    media.addEventListener('change', updateTheme)
  })

  onUnmounted(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    media.removeEventListener('change', updateTheme)
  })

  return { isDark }
}
```

#### 3. CSS Variables Update

**File:** `src/styles/main.css`
```css
:root {
  /* Light theme */
  --panel-bg: rgba(255, 255, 255, 0.85);
  --panel-border: rgba(0, 0, 0, 0.1);
  --text-primary: #1a1a2e;
  --text-secondary: #4a5568;
  --text-tertiary: #718096;
  --accent-primary: #6366f1;
  --glass-blur: 20px;
}

.dark {
  /* Dark theme */
  --panel-bg: rgba(30, 30, 46, 0.9);
  --panel-border: rgba(255, 255, 255, 0.1);
  --text-primary: #e2e8f0;
  --text-secondary: #a0aec0;
  --text-tertiary: #718096;
  --accent-primary: #818cf8;
}
```

#### 4. App.vue Integration

```vue
<script setup>
import { useTheme } from '@/composables/useTheme'
const { isDark } = useTheme()
</script>
```

### Component Updates Required
- All components already use CSS variables, minimal changes needed
- Ensure all hardcoded colors are replaced with variables

---

## Task 29: Error Handling UI

### Objective
Create a dedicated error display component with animations and retry functionality.

### Technical Approach

#### New Component: `src/components/ErrorDisplay.vue`

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { AlertCircle, RefreshCw, X } from 'lucide-vue-next'

interface ErrorInfo {
  message: string
  code?: string
  recoverable?: boolean
}

const props = defineProps<{
  error: ErrorInfo
}>()

const emit = defineEmits<{
  retry: []
  dismiss: []
}>()

const isRecoverable = computed(() => props.error.recoverable !== false)
</script>

<template>
  <div class="error-display animate-shake">
    <div class="flex items-center gap-3">
      <AlertCircle class="w-5 h-5 text-red-400 shrink-0" />
      <div class="flex-1 min-w-0">
        <p class="text-white font-medium text-sm truncate">
          {{ error.message }}
        </p>
        <p v-if="error.code" class="text-red-300/70 text-xs mt-0.5">
          Code: {{ error.code }}
        </p>
      </div>
    </div>

    <div class="flex items-center gap-2 mt-3">
      <button
        v-if="isRecoverable"
        @click="emit('retry')"
        class="flex items-center gap-1.5 px-3 py-1.5 bg-white/10 hover:bg-white/20 rounded-lg text-sm transition-colors"
      >
        <RefreshCw class="w-3.5 h-3.5" />
        Retry
      </button>
      <button
        @click="emit('dismiss')"
        class="flex items-center gap-1.5 px-3 py-1.5 bg-white/10 hover:bg-white/20 rounded-lg text-sm transition-colors"
      >
        <X class="w-3.5 h-3.5" />
        Dismiss
      </button>
    </div>
  </div>
</template>

<style scoped>
.error-display {
  @apply p-4 bg-red-500/90 text-white rounded-xl shadow-lg backdrop-blur-md;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  10%, 30%, 50%, 70%, 90% { transform: translateX(-4px); }
  20%, 40%, 60%, 80% { transform: translateX(4px); }
}

.animate-shake {
  animation: shake 0.5s ease-in-out;
}
</style>
```

#### Integration in FloatingPanel

Replace inline error toast with ErrorDisplay component:

```vue
<template>
  <!-- ... existing code ... -->

  <!-- Error Display -->
  <ErrorDisplay
    v-if="store.errorMessage"
    :error="{ message: store.errorMessage, recoverable: true }"
    @retry="handleRetry"
    @dismiss="store.clearError()"
    class="absolute bottom-12 left-4 right-4"
  />
</template>
```

#### Store Updates

Add to `src/stores/app.ts`:
```typescript
// New state
const lastAction = ref<{ type: string; payload?: any } | null>(null)

// New actions
function clearError() {
  errorMessage.value = null
}

async function retryLastAction() {
  if (!lastAction.value) return
  clearError()

  if (lastAction.value.type === 'ai') {
    await processWithAI(lastAction.value.payload)
  } else if (lastAction.value.type === 'rule') {
    await processWithRule(lastAction.value.payload)
  }
}
```

---

## Implementation Order

1. **Task 28: Dark/Light Mode** (Foundation)
   - Update CSS variables
   - Create useTheme composable
   - Test across all components

2. **Task 29: Error Handling UI** (UX Polish)
   - Create ErrorDisplay component
   - Add retry mechanism to store
   - Integrate into FloatingPanel

3. **Task 27: Paste-to-Cursor** (Core Feature)
   - Add enigo dependency
   - Implement paste_to_cursor in Rust
   - Add Tauri command
   - Update frontend confirmPaste
   - Test cross-platform

---

## Testing Checklist

### Dark/Light Mode
- [ ] Theme follows system preference
- [ ] Transition is smooth
- [ ] All components render correctly in both modes
- [ ] CSS variables applied consistently

### Error Handling
- [ ] Error displays with shake animation
- [ ] Retry works for recoverable errors
- [ ] Dismiss clears error state
- [ ] Non-blocking (panel still usable)

### Paste-to-Cursor
- [ ] Works on Windows (Ctrl+V)
- [ ] Works on macOS (Cmd+V)
- [ ] Fallback works when simulation fails
- [ ] User notified of fallback
