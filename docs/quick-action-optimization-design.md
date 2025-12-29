# FlowPaste 快捷操作优化 - 设计方案

## 1. 设计目标

将快捷操作区域从"AI 依赖"转变为"本地优先 + AI 学习"模式：
- 快捷按钮全部为本地规则，零延迟
- 用户可自定义显示哪些按钮（最多 3 个直接显示）
- AI 操作后智能评估，可将常用操作固化为本地规则

## 2. 核心设计决策

### 2.1 主面板快捷按钮

```
┌─────────────────────────────────────────────────────────────┐
│  [预览区域]                                                   │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ 剪贴板内容预览...                                         │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  [快捷按钮] (用户自选的 Top 3，全部为本地规则)                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────┐              │
│  │去首尾空格[1]│ │CJK间距[2]│ │格式化JSON[3]│ │ ⋯ │              │
│  └──────────┘ └──────────┘ └──────────┘ └────┘              │
│                                           ↑ 更多菜单          │
└─────────────────────────────────────────────────────────────┘
```

- **按钮数量**：固定显示 3 个 + 更多菜单
- **按钮类型**：全部为 LocalRule（本地执行，无 AI 延迟）
- **快捷键**：1, 2, 3 对应三个按钮

### 2.2 更多菜单设计

```
点击 [⋯] 展开 Popover：
┌─────────────────────────┐
│ 最近使用                 │
│ ├─ 每行加逗号            │
│ ├─ 去空行               │
│                         │
│ 自定义 / AI 学习         │
│ ├─ 每行加逗号      [AI]  │
│                         │
│ 内置规则                 │
│ ├─ 去空行               │
│ ├─ 合并空格             │
│ ├─ 压缩 JSON            │
│ └─ ...                  │
│ ─────────────────────── │
│ ⚙️ 管理快捷操作...        │
└─────────────────────────┘
```

- **排序逻辑**：最近使用优先
- **分组**：两类 - "自定义/AI学习" 与 "内置"
- **底部入口**：跳转设置页管理

### 2.3 设置页 - 快捷操作管理

```
┌─────────────────────────────────────────────────────────────┐
│ 快捷操作管理                                                  │
├─────────────────────────────────────────────────────────────┤
│ 拖拽排序，前 3 个显示在主面板                                   │
│                                                              │
│ ☰ ⭐ 去首尾空格              [内置]                           │
│ ☰ ⭐ CJK/英文间距            [内置]                           │
│ ☰ ⭐ 格式化 JSON             [内置]                           │
│ ─────────────── 以上显示在主面板 ───────────────              │
│ ☰    去空行                  [内置]                           │
│ ☰    合并空格                [内置]                           │
│ ☰    压缩 JSON               [内置]                           │
│ ☰    行排序                  [内置]                           │
│ ☰    行去重                  [内置]                           │
│ ☰    每行加逗号              [AI学习] 🗑️                      │
│                                                              │
│ [+ 添加自定义规则]                                            │
│                                                              │
│ ─────────────────────────────────────────────────────────── │
│ □ 启用 AI 规则学习（AI 操作后自动评估是否可保存为本地规则）       │
└─────────────────────────────────────────────────────────────┘
```

- **拖拽排序**：用户拖动调整顺序
- **前 3 固定**：前 3 个自动显示在主面板，带 ⭐ 标记
- **来源标签**：[内置] / [AI学习] / [自定义]
- **删除**：仅 AI 学习和自定义规则可删除
- **开关**：可关闭 AI 规则学习功能

### 2.4 AI 规则学习机制

**触发条件**：
- 用户执行 AI 格式化操作
- 设置中"AI 规则学习"开关已启用

**评估流程**：
```
用户输入 AI 指令 → AI 执行格式化 → 返回结果 + 规则评估
                                         ↓
                         confidence ≥ 0.8 且可本地执行？
                                         ↓ 是
                              显示保存提示（inline）
```

**保存提示 UI**：
```
┌─────────────────────────────────────────────────────────────┐
│ 💡 此操作可保存为快捷规则，下次直接本地执行                      │
│                                                              │
│ 规则名称: [每行末尾加逗号________]                             │
│ 预览效果: "a\nb" → "a,\nb,"                                   │
│                                                              │
│                              [忽略]  [保存到快捷操作]          │
└─────────────────────────────────────────────────────────────┘
```

**AI 返回结构**：
```json
{
  "ruleCandidate": {
    "isRuleCandidate": true,
    "confidence": 0.9,
    "transformationType": "regex_replace",
    "pattern": "(.+)$",
    "replacement": "$1,",
    "flags": "gm",
    "suggestedName": "每行末尾加逗号",
    "examples": [{ "before": "a\nb", "after": "a,\nb," }]
  }
}
```

## 3. 数据结构设计

### 3.1 CustomRule 扩展

```typescript
interface CustomRule {
  id: string
  name: string
  description?: string

  // 规则定义
  transformationType: 'regex_replace' | 'json_format' | 'json_minify' | 'sort_lines' | 'dedupe_lines'
  pattern?: string
  replacement?: string
  flags?: string

  // 元数据
  isBuiltin: boolean
  createdBy: 'builtin' | 'user' | 'ai'
  sourcePrompt?: string
  examples?: { before: string; after: string }[]

  // 统计
  usageCount: number
  lastUsedAt?: number
  createdAt: number
}
```

### 3.2 AppConfig 扩展

```typescript
interface AppConfig {
  // ... 现有字段

  // 快捷操作配置
  pinnedRuleIds: string[]      // 固定的规则 ID（前 3 个显示在主面板）
  customRules: CustomRule[]    // 用户自定义 + AI 学习的规则
  enableAIRuleLearning: boolean // AI 规则学习开关，默认 true
}
```

## 4. 内置规则列表

| ID | 名称 | 类型 | 说明 |
|----|------|------|------|
| remove_spaces | 去除空格 | regex_replace | 移除所有空格和制表符 |
| remove_newlines | 去除换行 | regex_replace | 移除所有换行符 |
| remove_empty_lines | 去空行 | regex_replace | 移除连续空行 |
| to_plain_text | 转纯文本 | regex_replace | 移除 Markdown 标记 |
| format_json | 格式化 JSON | json_format | 美化 JSON 缩进 |
| sort_lines | 行排序 | sort_lines | 按字典序排序 |
| dedupe_lines | 行去重 | dedupe_lines | 去除重复行 |

**默认置顶（新用户）**：
1. remove_spaces
2. remove_empty_lines
3. format_json

## 5. 系统提示词追加

在 AI 执行格式化时，追加以下指令：

```
在完成用户请求的格式化后，请评估此操作是否可被本地规则稳定复现。

判定标准：
- 仅接受确定性转换（相同输入必定产生相同输出）
- 拒绝需要语义理解的任务（摘要、翻译、改写、解释）

在响应末尾追加 JSON（用 ```json 包裹）：

如果可规则化：
{"ruleCandidate":{"isRuleCandidate":true,"confidence":0.9,"transformationType":"regex_replace","pattern":"正则","replacement":"替换","flags":"gm","suggestedName":"名称","examples":[{"before":"输入","after":"输出"}]}}

如果不可规则化：
{"ruleCandidate":{"isRuleCandidate":false,"reason":"原因"}}
```

## 6. 技术实现要点

### 6.1 前端
- `ActionChips.vue`：限制显示 3 个 + 更多按钮
- `OverflowMenu.vue`：新组件，Popover 菜单
- `SettingsPanel.vue`：新增快捷操作管理 Tab
- `RuleSavePrompt.vue`：新组件，AI 规则保存提示

### 6.2 后端 (Rust)
- `src-tauri/src/regex/mod.rs`：新增 format_json, minify_json, sort_lines, dedupe_lines
- `src-tauri/src/ai/intent.rs`：移除 AI 依赖的 ActionChip 生成逻辑
- `src-tauri/src/config/mod.rs`：扩展配置结构

### 6.3 Store
- `src/stores/app.ts`：新增规则管理方法（pin/unpin/save/delete）

## 7. 不在本次范围

- 内容类型检测优化（保持现有逻辑）
- AI 提供商切换
- 规则导入/导出
