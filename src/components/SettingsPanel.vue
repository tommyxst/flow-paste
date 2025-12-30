<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { Settings, X, Loader2, Check, AlertCircle } from 'lucide-vue-next'
import { useSettingsDraft } from '@/composables/useSettingsDraft'
import { useSettingsSecrets } from '@/composables/useSettingsSecrets'
import { useSettingsValidation, type FlatSettingsErrors } from '@/composables/useSettingsValidation'
import { useSettingsSaveFlow } from '@/composables/useSettingsSaveFlow'
import GeneralSection from './settings/GeneralSection.vue'
import AIConfigSection from './settings/AIConfigSection.vue'
import QuickActionsSection from './settings/QuickActionsSection.vue'

const store = useAppStore()
const emit = defineEmits<{ close: [] }>()

const { draft: formData, applyConfig, commitDraft } = useSettingsDraft(store.config)
const { apiKey, loadApiKey, saveApiKeyIfNeeded } = useSettingsSecrets()
const { validateAll, setSaveError, getFlatErrors } = useSettingsValidation()
const { isSaving, save } = useSettingsSaveFlow()

const isVisible = ref(false)
const errors = ref<FlatSettingsErrors>({})

onMounted(async () => {
  if (store.config) applyConfig(store.config)
  await loadApiKey()
  requestAnimationFrame(() => { isVisible.value = true })
})

async function handleSave() {
  const result = await save(
    formData.value,
    apiKey.value,
    () => {
      const valid = validateAll(formData.value, apiKey.value)
      errors.value = getFlatErrors()
      return valid
    },
    saveApiKeyIfNeeded
  )

  if (result.success) {
    commitDraft()
    emit('close')
  } else if (result.error) {
    setSaveError(result.error)
    errors.value = getFlatErrors()
  }
}

function handleCancel() { emit('close') }
</script>

<template>
  <div class="settings-panel" :class="{ 'is-visible': isVisible }">
    <header class="panel-header">
      <div class="header-title">
        <div class="header-icon"><Settings :size="18" /></div>
        <h2>设置</h2>
      </div>
      <button @click="handleCancel" class="close-btn" aria-label="关闭"><X :size="18" /></button>
    </header>
    
    <form @submit.prevent="handleSave" class="panel-content">
      <GeneralSection :form-data="formData" :errors="errors" style="--delay: 0" />
      <AIConfigSection
        :form-data="formData"
        v-model:api-key="apiKey"
        :errors="errors"
        style="--delay: 1"
      />
      <QuickActionsSection :form-data="formData" style="--delay: 2" />

      <div v-if="errors.save" class="save-error"><AlertCircle :size="16" /><span>{{ errors.save }}</span></div>
      
      <footer class="panel-footer">
        <button type="button" @click="handleCancel" class="btn btn-secondary">取消</button>
        <button type="submit" :disabled="isSaving" class="btn btn-primary">
          <Loader2 v-if="isSaving" :size="14" class="animate-spin" /><Check v-else :size="14" />
          <span>{{ isSaving ? '保存中...' : '保存设置' }}</span>
        </button>
      </footer>
    </form>
  </div>
</template>

<style scoped>
.settings-panel { width:100%; max-width:420px; max-height:100%; display:flex; flex-direction:column; background:#ffffff; border:1px solid var(--panel-border); border-radius:16px; box-shadow:0 0 0 1px rgba(255,255,255,0.05) inset,0 25px 50px -12px rgba(0,0,0,0.25),0 0 100px rgba(59,130,246,0.05); overflow:hidden; opacity:0; transform:scale(0.96) translateY(8px); transition:all 0.3s cubic-bezier(0.16,1,0.3,1); }
.dark .settings-panel { background:#1a1a1a; }
.settings-panel.is-visible { opacity:1; transform:scale(1) translateY(0); }
.panel-header { display:flex; align-items:center; justify-content:space-between; padding:16px 20px; border-bottom:1px solid var(--panel-border); background:linear-gradient(to bottom,rgba(255,255,255,0.03),transparent); }
.dark .panel-header { background:linear-gradient(to bottom,rgba(255,255,255,0.02),transparent); }
.header-title { display:flex; align-items:center; gap:10px; }
.header-icon { display:flex; align-items:center; justify-content:center; width:32px; height:32px; background:linear-gradient(135deg,var(--accent-primary),#8b5cf6); border-radius:8px; color:white; box-shadow:0 2px 8px rgba(59,130,246,0.3); }
.header-title h2 { font-size:16px; font-weight:600; color:var(--text-primary); margin:0; }
.close-btn { display:flex; align-items:center; justify-content:center; width:28px; height:28px; border-radius:8px; border:none; background:transparent; color:var(--text-tertiary); cursor:pointer; transition:all 0.15s ease; }
.close-btn:hover { background:rgba(0,0,0,0.05); color:var(--text-primary); }
.dark .close-btn:hover { background:rgba(255,255,255,0.1); }
.panel-content { flex:1; min-height:0; overflow-y:auto; overflow-x:hidden; padding:16px; display:flex; flex-direction:column; gap:16px; }
.panel-content::-webkit-scrollbar { width:6px; }
.panel-content::-webkit-scrollbar-track { background:transparent; }
.panel-content::-webkit-scrollbar-thumb { background:rgba(0,0,0,0.15); border-radius:3px; }
.dark .panel-content::-webkit-scrollbar-thumb { background:rgba(255,255,255,0.15); }

.save-error { display:flex; align-items:center; gap:8px; padding:12px; background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.2); border-radius:8px; color:#ef4444; font-size:12px; }
.panel-footer { display:flex; gap:10px; padding:16px 20px; border-top:1px solid var(--panel-border); background:linear-gradient(to top,rgba(0,0,0,0.02),transparent); }
.dark .panel-footer { background:linear-gradient(to top,rgba(255,255,255,0.02),transparent); }
.btn { display:flex; align-items:center; justify-content:center; gap:6px; padding:10px 18px; font-size:13px; font-weight:500; border-radius:8px; border:none; cursor:pointer; transition:all 0.15s ease; }
.btn-primary { flex:1; background:linear-gradient(135deg,var(--accent-primary),#6366f1); color:white; box-shadow:0 2px 8px rgba(59,130,246,0.3); }
.btn-primary:hover:not(:disabled) { transform:translateY(-1px); box-shadow:0 4px 12px rgba(59,130,246,0.4); }
.btn-primary:disabled { opacity:0.6; cursor:not-allowed; transform:none; }
.btn-secondary { background:rgba(0,0,0,0.04); color:var(--text-secondary); border:1px solid rgba(0,0,0,0.06); }
.dark .btn-secondary { background:rgba(255,255,255,0.06); border-color:rgba(255,255,255,0.08); }
.btn-secondary:hover { background:rgba(0,0,0,0.08); color:var(--text-primary); }
.dark .btn-secondary:hover { background:rgba(255,255,255,0.1); }
@keyframes spin { from { transform:rotate(0deg); } to { transform:rotate(360deg); } }
.animate-spin { animation:spin 1s linear infinite; }
</style>
