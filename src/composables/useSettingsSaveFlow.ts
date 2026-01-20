import { ref } from 'vue'
import { useAppStore } from '@/stores/app'
import { commands } from '@/lib/tauri'
import type { AppConfig } from '@/types'

export function useSettingsSaveFlow() {
  const store = useAppStore()
  const isSaving = ref(false)

  async function save(
    draft: AppConfig,
    apiKey: string,
    validateFn: () => boolean,
    saveSecretFn: (provider: AppConfig['aiProvider'], key: string) => Promise<boolean>
  ): Promise<{ success: boolean; error?: string }> {
    if (!validateFn()) {
      return { success: false }
    }

    isSaving.value = true

    try {
      // Save old hotkey BEFORE updating config (to compare correctly)
      const oldHotkey = store.config?.hotkey

      // Save config to store
      await store.saveConfig(draft)

      // Save API key if needed
      const secretSaved = await saveSecretFn(draft.aiProvider, apiKey)
      if (!secretSaved) {
        return { success: false, error: '保存 API Key 失败' }
      }

      // Register hotkey if changed (compare with saved old value)
      if (draft.hotkey !== oldHotkey) {
        await commands.registerHotkey(draft.hotkey)
      }

      return { success: true }
    } catch (e) {
      return { success: false, error: `保存失败: ${e}` }
    } finally {
      isSaving.value = false
    }
  }

  return {
    isSaving,
    save
  }
}
