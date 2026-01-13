<script setup lang="ts">
import { ref, computed } from 'vue'
import { Zap, GripVertical, ChevronUp, ChevronDown, Trash2 } from 'lucide-vue-next'
import { useAppStore } from '@/stores/app'
import type { AppConfig, Rule } from '@/types'
import SettingsSectionShell from './SettingsSectionShell.vue'

const props = defineProps<{
  formData: AppConfig
}>()

const store = useAppStore()
const selectedRuleToAdd = ref('')

const availableRulesToAdd = computed(() => {
  const pinned = new Set(props.formData.pinnedRuleIds)
  const allRules = store.allRules || []
  return allRules.filter((r: Rule) => !pinned.has(r.id))
})

async function moveRuleUp(index: number) {
  if (index <= 0) return
  const arr = [...props.formData.pinnedRuleIds]
  ;[arr[index - 1], arr[index]] = [arr[index], arr[index - 1]]
  props.formData.pinnedRuleIds = arr
  await store.saveConfig({ ...store.config!, pinnedRuleIds: arr })
}

async function moveRuleDown(index: number) {
  if (index >= props.formData.pinnedRuleIds.length - 1) return
  const arr = [...props.formData.pinnedRuleIds]
  ;[arr[index], arr[index + 1]] = [arr[index + 1], arr[index]]
  props.formData.pinnedRuleIds = arr
  await store.saveConfig({ ...store.config!, pinnedRuleIds: arr })
}

async function removeFromPinned(index: number) {
  const newIds = props.formData.pinnedRuleIds.filter((_, i) => i !== index)
  props.formData.pinnedRuleIds = newIds
  await store.saveConfig({ ...store.config!, pinnedRuleIds: newIds })
}

async function addToPinned() {
  if (!selectedRuleToAdd.value) return
  const newIds = [...props.formData.pinnedRuleIds, selectedRuleToAdd.value]
  props.formData.pinnedRuleIds = newIds
  selectedRuleToAdd.value = ''
  await store.saveConfig({ ...store.config!, pinnedRuleIds: newIds })
}
</script>

<template>
  <SettingsSectionShell title="快捷操作" :icon="Zap" icon-class="actions" badge="前 3 个显示在主面板">
    <div class="rules-list">
        <TransitionGroup name="rule">
          <div 
            v-for="(ruleId, idx) in formData.pinnedRuleIds" 
            :key="ruleId" 
            class="rule-item" 
            :class="{ primary: idx < 3 }"
          >
            <div class="rule-grip">
              <GripVertical :size="14" />
            </div>
            <span class="rule-index" :class="{ active: idx < 3 }">{{ idx < 3 ? idx + 1 : '·' }}</span>
            <span class="rule-name">{{ store.allRules?.find((r: Rule) => r.id === ruleId)?.name || ruleId }}</span>
            <div class="rule-actions">
              <button 
                type="button" 
                @click="moveRuleUp(idx)" 
                :disabled="idx === 0" 
                class="rule-action-btn" 
                aria-label="上移"
              >
                <ChevronUp :size="14" />
              </button>
              <button 
                type="button" 
                @click="moveRuleDown(idx)" 
                :disabled="idx === formData.pinnedRuleIds.length - 1" 
                class="rule-action-btn" 
                aria-label="下移"
              >
                <ChevronDown :size="14" />
              </button>
              <button 
                type="button" 
                @click="removeFromPinned(idx)" 
                class="rule-action-btn delete" 
                aria-label="移除"
              >
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
        </TransitionGroup>
        <div v-if="formData.pinnedRuleIds.length === 0" class="rules-empty">暂无已固定的快捷操作</div>
      </div>
      <div class="add-rule">
        <select v-model="selectedRuleToAdd" @change="addToPinned" class="field-select">
          <option value="">添加快捷操作...</option>
          <option v-for="rule in availableRulesToAdd" :key="rule.id" :value="rule.id">{{ rule.name }}</option>
        </select>
      </div>
  </SettingsSectionShell>
</template>

<style src="./settings.shared.css"></style>
<style scoped>
.rules-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 40px;
}

.rules-empty {
  padding: 16px;
  text-align: center;
  font-size: 12px;
  color: var(--text-tertiary);
  background: rgba(0, 0, 0, 0.02);
  border-radius: 8px;
  border: 1px dashed rgba(0, 0, 0, 0.08);
}

.dark .rules-empty {
  background: rgba(255, 255, 255, 0.02);
  border-color: rgba(255, 255, 255, 0.08);
}

.rule-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: rgba(0, 0, 0, 0.02);
  border: 1px solid rgba(0, 0, 0, 0.04);
  border-radius: 8px;
  transition: all 0.15s ease;
}

.dark .rule-item {
  background: rgba(255, 255, 255, 0.02);
  border-color: rgba(255, 255, 255, 0.04);
}

.rule-item.primary {
  background: var(--accent-subtle);
  border-color: rgba(59, 130, 246, 0.15);
}

.rule-grip {
  color: var(--text-tertiary);
  cursor: grab;
  opacity: 0.5;
}

.rule-index {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-tertiary);
  background: rgba(0, 0, 0, 0.04);
  border-radius: 5px;
}

.dark .rule-index {
  background: rgba(255, 255, 255, 0.06);
}

.rule-index.active {
  background: var(--accent-primary);
  color: white;
}

.rule-name {
  flex: 1;
  font-size: 12px;
  color: var(--text-primary);
}

.rule-actions {
  display: flex;
  gap: 2px;
}

.rule-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.rule-action-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.06);
  color: var(--text-primary);
}

.dark .rule-action-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}

.rule-action-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.rule-action-btn.delete:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.rule-move, .rule-enter-active, .rule-leave-active {
  transition: all 0.25s ease;
}

.rule-enter-from {
  opacity: 0;
  transform: translateX(-10px);
}

.rule-leave-to {
  opacity: 0;
  transform: translateX(10px);
}

.rule-leave-active {
  position: absolute;
}

.add-rule {
  padding-top: 6px;
}

.add-rule .field-select {
  background: rgba(0, 0, 0, 0.02);
}

.dark .add-rule .field-select {
  background: rgba(255, 255, 255, 0.03);
}
</style>
