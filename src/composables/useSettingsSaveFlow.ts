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
      // Save config to store
      await store.saveConfig(draft)

      // Save API key if needed
      const secretSaved = await saveSecretFn(draft.aiProvider, apiKey)
      if (!secretSaved) {
        return { success: false, error: '保存 API Key 失败' }
      }

      // Register hotkey if changed
      if (store.config && draft.hotkey !== store.config.hotkey) {
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
