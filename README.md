# FlowPaste (妙贴)

> 剪贴即意图，心流不打断 | Paste at the speed of thought

FlowPaste 是一款隐私优先、AI 驱动的智能剪贴板增强工具，帮助用户快速处理复制的文本内容。

## 功能特性

### 已实现 (v0.1.0)

- **全局热键唤起**: `Ctrl+Shift+V` 快速呼出浮动面板
- **剪贴板读取**: 自动读取并预览剪贴板内容
- **快捷文本处理**:
  - 去空行
  - 去首尾空格
  - 转大写
  - 转小写
- **便捷交互**:
  - ESC 快速关闭
  - 点击窗口外自动隐藏
  - 窗口可拖动
- **透明无边框窗口**: 现代化 UI 设计

### 规划中

- AI 意图识别与推荐操作
- 自然语言指令处理
- 本地模型支持 (Ollama)
- 云端 API 支持 (OpenAI 兼容)
- 隐私盾 (PII 脱敏)

## 技术栈

- **后端**: Rust + Tauri 2.x
- **前端**: Vue 3 + TypeScript + TailwindCSS
- **状态管理**: Pinia
- **构建工具**: Vite

## 开发环境

### 前置要求

- Node.js >= 18
- Rust >= 1.70
- pnpm (推荐) 或 npm

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建发布

```bash
npm run tauri build
```

## 项目结构

```
flow-paste/
├── src/                    # 前端源码
│   ├── components/         # Vue 组件
│   ├── stores/             # Pinia 状态管理
│   └── styles/             # 样式文件
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── lib.rs          # 主入口
│   │   ├── commands.rs     # Tauri 命令
│   │   ├── privacy/        # 隐私模块
│   │   └── ai/             # AI 模块
│   └── capabilities/       # 权限配置
└── docs/                   # 文档
```

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+V` | 呼出/隐藏面板 |
| `ESC` | 关闭面板 |
| `Enter` | 执行指令 |

## 许可证

MIT License
