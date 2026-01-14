<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Zap, GripVertical, ChevronUp, ChevronDown, Pin, PinOff, Library, Sparkles, Trash2 } from 'lucide-vue-next'
import { useAppStore } from '@/stores/app'
import { commands } from '@/lib/tauri'
import type { AppConfig, Rule } from '@/types'
import SettingsSectionShell from './SettingsSectionShell.vue'

const props = defineProps<{
  formData: AppConfig
}>()

const store = useAppStore()
const allRules = ref<Rule[]>([])
const activeTab = ref<'pinned' | 'library'>('pinned')
const selectedCategory = ref<string>('all')

// 加载所有规则
onMounted(async () => {
  try {
    allRules.value = await commands.listAllRules()
  } catch (e) {
    console.error('Failed to load rules:', e)
    allRules.value = store.allRules || []
  }
})

// 按类别分组的规则
const categories = computed(() => {
  const cats = new Map<string, Rule[]>()
  cats.set('all', [])

  for (const rule of allRules.value) {
    const cat = rule.category || 'other'
    if (!cats.has(cat)) {
      cats.set(cat, [])
    }
    cats.get(cat)!.push(rule)
    cats.get('all')!.push(rule)
  }
  return cats
})

// 类别标签映射
const categoryLabels: Record<string, string> = {
  all: '全部',
  cleanup: '清理',
  format: '格式化',
  json: 'JSON',
  lines: '行操作',
  other: '其他',
}

// 当前类别的规则
const filteredRules = computed(() => {
  return categories.value.get(selectedCategory.value) || []
})

// 检查规则是否已置顶
function isPinned(ruleId: string): boolean {
  return props.formData.pinnedRuleIds.includes(ruleId)
}

// 置顶规则操作
async function moveRuleUp(index: number) {
  if (index <= 0) return
  const arr = [...props.formData.pinnedRuleIds]
  ;[arr[index - 1], arr[index]] = [arr[index], arr[index - 1]]
  props.formData.pinnedRuleIds = arr
  await commands.reorderPinnedRules(arr)
}

async function moveRuleDown(index: number) {
  if (index >= props.formData.pinnedRuleIds.length - 1) return
  const arr = [...props.formData.pinnedRuleIds]
  ;[arr[index], arr[index + 1]] = [arr[index + 1], arr[index]]
  props.formData.pinnedRuleIds = arr
  await commands.reorderPinnedRules(arr)
}

async function removeFromPinned(ruleId: string) {
  const newIds = props.formData.pinnedRuleIds.filter(id => id !== ruleId)
  props.formData.pinnedRuleIds = newIds
  await commands.reorderPinnedRules(newIds)
}

async function addToPinned(ruleId: string) {
  if (isPinned(ruleId)) return
  const newIds = [...props.formData.pinnedRuleIds, ruleId]
  props.formData.pinnedRuleIds = newIds
  await commands.reorderPinnedRules(newIds)
}

// 获取规则信息
function getRuleById(ruleId: string): Rule | undefined {
  return allRules.value.find(r => r.id === ruleId)
}

// 删除规则
async function deleteRule(rule: Rule) {
  const confirmed = window.confirm(`确定要删除规则「${rule.name}」吗？此操作不可撤销。`)
  if (!confirmed) return

  try {
    const success = await commands.deleteRule(rule.id)
    if (success) {
      // 从本地列表移除
      allRules.value = allRules.value.filter(r => r.id !== rule.id)
      // 同步更新 formData.customRules（防止保存时覆盖）
      props.formData.customRules = props.formData.customRules.filter(r => r.id !== rule.id)
      // 如果已置顶，也从置顶列表移除
      if (isPinned(rule.id)) {
        props.formData.pinnedRuleIds = props.formData.pinnedRuleIds.filter(id => id !== rule.id)
      }
    }
  } catch (e) {
    console.error('Failed to delete rule:', e)
    window.alert('删除失败，请重试')
  }
}

// 检查规则是否可删除（非内置规则）
function canDelete(rule: Rule): boolean {
  return rule.origin !== 'builtin'
}
</script>

<template>
  <SettingsSectionShell title="快捷操作" :icon="Zap" icon-class="actions" badge="前 3 个显示在主面板">
    <!-- 标签切换 -->
    <div class="tabs">
      <button
        type="button"
        class="tab"
        :class="{ active: activeTab === 'pinned' }"
        @click="activeTab = 'pinned'"
      >
        <Pin :size="14" />
        <span>已置顶 ({{ formData.pinnedRuleIds.length }})</span>
      </button>
      <button
        type="button"
        class="tab"
        :class="{ active: activeTab === 'library' }"
        @click="activeTab = 'library'"
      >
        <Library :size="14" />
        <span>规则库 ({{ allRules.length }})</span>
      </button>
    </div>

    <!-- 置顶规则列表 -->
    <div v-if="activeTab === 'pinned'" class="rules-list">
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
          <span class="rule-name">{{ getRuleById(ruleId)?.name || ruleId }}</span>
          <span v-if="getRuleById(ruleId)?.origin === 'ai'" class="rule-badge ai">AI</span>
          <div class="rule-actions">
            <button type="button" @click="moveRuleUp(idx)" :disabled="idx === 0" class="rule-action-btn" aria-label="上移">
              <ChevronUp :size="14" />
            </button>
            <button type="button" @click="moveRuleDown(idx)" :disabled="idx === formData.pinnedRuleIds.length - 1" class="rule-action-btn" aria-label="下移">
              <ChevronDown :size="14" />
            </button>
            <button type="button" @click="removeFromPinned(ruleId)" class="rule-action-btn delete" aria-label="取消置顶">
              <PinOff :size="14" />
            </button>
          </div>
        </div>
      </TransitionGroup>
      <div v-if="formData.pinnedRuleIds.length === 0" class="rules-empty">
        <Pin :size="20" />
        <p>暂无已置顶的快捷操作</p>
        <p class="hint">从规则库中选择规则进行置顶</p>
      </div>
    </div>

    <!-- 规则库 -->
    <div v-if="activeTab === 'library'" class="library-view">
      <!-- 类别筛选 -->
      <div class="category-tabs">
        <button
          v-for="[cat] in categories"
          :key="cat"
          type="button"
          class="category-tab"
          :class="{ active: selectedCategory === cat }"
          @click="selectedCategory = cat"
        >
          {{ categoryLabels[cat] || cat }}
        </button>
      </div>
      <!-- 规则列表 -->
      <div class="library-list">
        <div
          v-for="rule in filteredRules"
          :key="rule.id"
          class="library-item"
          :class="{ pinned: isPinned(rule.id) }"
        >
          <div class="library-item-info">
            <span class="library-item-name">{{ rule.name }}</span>
            <span v-if="rule.origin === 'ai'" class="rule-badge ai">
              <Sparkles :size="10" /> AI
            </span>
            <span v-else-if="rule.origin === 'user'" class="rule-badge user">自定义</span>
          </div>
          <p class="library-item-desc">{{ rule.description }}</p>
          <div class="library-item-actions">
            <button
              type="button"
              class="library-pin-btn"
              :class="{ active: isPinned(rule.id) }"
              @click="isPinned(rule.id) ? removeFromPinned(rule.id) : addToPinned(rule.id)"
            >
              <Pin v-if="!isPinned(rule.id)" :size="14" />
              <PinOff v-else :size="14" />
              {{ isPinned(rule.id) ? '取消置顶' : '置顶' }}
            </button>
            <button
              v-if="canDelete(rule)"
              type="button"
              class="library-delete-btn"
              @click="deleteRule(rule)"
            >
              <Trash2 :size="14" />
              删除
            </button>
          </div>
        </div>
      </div>
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

/* Tabs */
.tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 12px;
  padding: 4px;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 8px;
}

.dark .tabs {
  background: rgba(255, 255, 255, 0.03);
}

.tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab:hover {
  color: var(--text-primary);
  background: rgba(0, 0, 0, 0.04);
}

.dark .tab:hover {
  background: rgba(255, 255, 255, 0.06);
}

.tab.active {
  color: var(--accent-primary);
  background: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.dark .tab.active {
  background: rgba(255, 255, 255, 0.1);
}

/* Rule Badge */
.rule-badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px;
  font-size: 10px;
  font-weight: 600;
  border-radius: 4px;
}

.rule-badge.ai {
  color: #8b5cf6;
  background: rgba(139, 92, 246, 0.1);
}

.rule-badge.user {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

/* Library View */
.library-view {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.category-tabs {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.category-tab {
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-secondary);
  background: rgba(0, 0, 0, 0.03);
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.dark .category-tab {
  background: rgba(255, 255, 255, 0.05);
}

.category-tab:hover {
  background: rgba(0, 0, 0, 0.06);
}

.dark .category-tab:hover {
  background: rgba(255, 255, 255, 0.08);
}

.category-tab.active {
  color: white;
  background: var(--accent-primary);
}

.library-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 280px;
  overflow-y: auto;
}

.library-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  background: rgba(0, 0, 0, 0.02);
  border: 1px solid rgba(0, 0, 0, 0.04);
  border-radius: 8px;
  transition: all 0.15s ease;
}

.dark .library-item {
  background: rgba(255, 255, 255, 0.02);
  border-color: rgba(255, 255, 255, 0.04);
}

.library-item.pinned {
  background: var(--accent-subtle);
  border-color: rgba(59, 130, 246, 0.15);
}

.library-item-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.library-item-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.library-item-desc {
  font-size: 11px;
  color: var(--text-tertiary);
  margin: 0;
  line-height: 1.4;
}

.library-item-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.library-pin-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  font-size: 11px;
  color: var(--text-secondary);
  background: rgba(0, 0, 0, 0.04);
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.dark .library-pin-btn {
  background: rgba(255, 255, 255, 0.06);
}

.library-pin-btn:hover {
  color: var(--accent-primary);
  background: rgba(59, 130, 246, 0.1);
}

.library-pin-btn.active {
  color: #ef4444;
}

.library-pin-btn.active:hover {
  background: rgba(239, 68, 68, 0.1);
}

.library-delete-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  font-size: 11px;
  color: var(--text-tertiary);
  background: transparent;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.library-delete-btn:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

/* Empty state enhancement */
.rules-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.rules-empty p {
  margin: 0;
}

.rules-empty .hint {
  font-size: 11px;
  color: var(--text-tertiary);
}
</style>
