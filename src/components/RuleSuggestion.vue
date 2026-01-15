<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAppStore } from '@/stores/app'
import { Sparkles, X, Check, ChevronDown, ChevronUp, AlertTriangle, Loader2 } from 'lucide-vue-next'
import type { TransformationType } from '@/types'

const store = useAppStore()
const showDetails = ref(false)

// 转换类型映射为可读标签
const transformTypeLabels: Record<TransformationType, string> = {
  'regex_replace': '正则替换',
  'json_format': 'JSON 格式化',
  'json_minify': 'JSON 压缩',
  'sort_lines': '行排序',
  'dedupe_lines': '行去重',
  'to_uppercase': '转大写',
  'to_lowercase': '转小写',
}

// 置信度颜色
const confidenceClass = computed(() => {
  if (!store.ruleSuggestion) return ''
  const conf = store.ruleSuggestion.confidence
  if (conf >= 0.9) return 'high'
  if (conf >= 0.8) return 'medium'
  return 'low'
})

const confidencePercent = computed(() => {
  if (!store.ruleSuggestion) return '0%'
  return Math.round(store.ruleSuggestion.confidence * 100) + '%'
})

const transformTypeLabel = computed(() => {
  if (!store.ruleSuggestion) return ''
  return transformTypeLabels[store.ruleSuggestion.transformationType] || store.ruleSuggestion.transformationType
})

async function handleSave() {
  await store.saveRuleSuggestion()
}
</script>

<template>
  <div
    v-if="store.ruleSuggestion"
    class="rule-suggestion"
  >
    <!-- 主行：名称 + 置信度徽章 + 操作按钮 -->
    <div class="suggestion-header">
      <Sparkles class="w-4 h-4 text-amber-500 shrink-0" />
      <div class="suggestion-info">
        <p class="suggestion-title">
          可保存为快捷规则: <span class="font-medium">{{ store.ruleSuggestion.name }}</span>
        </p>
        <div class="suggestion-meta">
          <span class="confidence-badge" :class="confidenceClass">
            {{ confidencePercent }}
          </span>
          <span class="type-label">{{ transformTypeLabel }}</span>
        </div>
      </div>
      <div class="suggestion-actions">
        <button
          @click="showDetails = !showDetails"
          class="action-btn toggle"
          :title="showDetails ? '收起详情' : '展开详情'"
        >
          <ChevronUp v-if="showDetails" class="w-3.5 h-3.5" />
          <ChevronDown v-else class="w-3.5 h-3.5" />
        </button>
        <button
          @click="handleSave"
          class="action-btn save"
          :class="{ disabled: store.isSavingRule }"
          :disabled="store.isSavingRule"
          title="保存规则"
        >
          <Loader2 v-if="store.isSavingRule" class="w-3.5 h-3.5 animate-spin" />
          <Check v-else class="w-3.5 h-3.5" />
        </button>
        <button
          @click="store.dismissRuleSuggestion()"
          class="action-btn dismiss"
          title="忽略"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- 详情区（可折叠） -->
    <div v-if="showDetails" class="suggestion-details">
      <div class="detail-item">
        <span class="detail-label">匹配模式:</span>
        <code class="detail-pattern">{{ store.ruleSuggestion.pattern }}</code>
      </div>
      <div v-if="store.ruleSuggestion.replacement" class="detail-item">
        <span class="detail-label">替换为:</span>
        <code class="detail-pattern">{{ store.ruleSuggestion.replacement }}</code>
      </div>
    </div>

    <!-- 后端验证错误展示 -->
    <div
      v-if="store.ruleValidationError && !store.ruleValidationError.valid"
      class="validation-errors"
      role="alert"
      aria-live="polite"
    >
      <AlertTriangle class="w-3 h-3 shrink-0" />
      <div class="error-list">
        <span v-for="error in store.ruleValidationError.errors" :key="error">{{ error }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.rule-suggestion {
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.08), rgba(245, 158, 11, 0.05));
  border: 1px solid rgba(251, 191, 36, 0.25);
  border-radius: 10px;
  overflow: hidden;
}

.dark .rule-suggestion {
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.1), rgba(245, 158, 11, 0.05));
  border-color: rgba(251, 191, 36, 0.2);
}

.suggestion-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
}

.suggestion-info {
  flex: 1;
  min-width: 0;
}

.suggestion-title {
  font-size: 12px;
  color: var(--text-primary);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.suggestion-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.confidence-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
}

.confidence-badge.high {
  background: rgba(34, 197, 94, 0.15);
  color: #16a34a;
}

.confidence-badge.medium {
  background: rgba(234, 179, 8, 0.15);
  color: #ca8a04;
}

.confidence-badge.low {
  background: rgba(239, 68, 68, 0.15);
  color: #dc2626;
}

.dark .confidence-badge.high {
  background: rgba(34, 197, 94, 0.2);
  color: #4ade80;
}

.dark .confidence-badge.medium {
  background: rgba(234, 179, 8, 0.2);
  color: #facc15;
}

.dark .confidence-badge.low {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.type-label {
  font-size: 10px;
  color: var(--text-tertiary);
}

.suggestion-actions {
  display: flex;
  gap: 4px;
  shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn.toggle {
  background: transparent;
  color: var(--text-secondary);
}

.action-btn.toggle:hover {
  background: rgba(0, 0, 0, 0.05);
  color: var(--text-primary);
}

.dark .action-btn.toggle:hover {
  background: rgba(255, 255, 255, 0.1);
}

.action-btn.save {
  background: #f59e0b;
  color: white;
}

.action-btn.save:hover:not(.disabled) {
  background: #d97706;
}

.action-btn.save.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.dismiss {
  background: transparent;
  color: var(--text-secondary);
}

.action-btn.dismiss:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* 详情区 */
.suggestion-details {
  padding: 8px 12px 10px;
  border-top: 1px solid rgba(251, 191, 36, 0.15);
  background: rgba(0, 0, 0, 0.02);
}

.dark .suggestion-details {
  background: rgba(0, 0, 0, 0.1);
}

.detail-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 6px;
}

.detail-item:last-child {
  margin-bottom: 0;
}

.detail-label {
  font-size: 10px;
  color: var(--text-tertiary);
  white-space: nowrap;
  padding-top: 2px;
}

.detail-pattern {
  font-size: 11px;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
  background: rgba(0, 0, 0, 0.05);
  padding: 3px 6px;
  border-radius: 4px;
  color: var(--text-secondary);
  word-break: break-all;
  max-height: 60px;
  overflow-y: auto;
}

.dark .detail-pattern {
  background: rgba(255, 255, 255, 0.08);
}

.pattern-error {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  font-size: 10px;
  color: #ef4444;
}

.validation-errors {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border-top: 1px solid rgba(239, 68, 68, 0.2);
  color: #ef4444;
  font-size: 11px;
}

.dark .validation-errors {
  background: rgba(239, 68, 68, 0.12);
}

.error-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
