# FlowPaste (妙贴)

> 剪贴即意图，心流不打断 | Paste at the speed of thought

FlowPaste 是一款隐私优先、AI 驱动的智能剪贴板增强工具，帮助用户快速处理复制的文本内容。

## 功能特性

### 已实现 (v0.1.0)

#### 核心功能
- **全局热键唤起**: `Ctrl+Shift+V` 快速呼出浮动面板
- **剪贴板读取**: 自动读取并预览剪贴板内容
- **智能粘贴**: 处理后的内容自动粘贴到光标位置

#### AI 增强
- **AI 意图识别**: 自动分析剪贴板内容并推荐操作
- **自然语言指令**: 用自然语言描述格式需求，AI 自动处理
- **本地模型支持**: 集成 Ollama，支持本地 AI 模型（llama3.2 等）
- **云端 API 支持**: 支持 OpenAI 兼容接口

#### 隐私保护
- **隐私盾 (PII 脱敏)**: 自动检测并脱敏敏感信息
  - 手机号、身份证号、邮箱、银行卡号、IP 地址
  - 云端处理时使用占位符，本地还原
  - API Key 安全存储在系统密钥链

#### 快捷处理
- **内置规则引擎**: 5 种快捷文本处理规则
  - 去空行
  - 去首尾空格
  - 转大写 / 转小写
  - 折叠多余空格
- **自定义正则规则**: 支持用户自定义文本处理规则

#### 交互体验
- **便捷交互**:
  - `ESC` 快速关闭
  - 点击窗口外自动隐藏
  - 窗口可拖动
  - 流式输出显示 AI 处理进度
- **现代化 UI**: 透明无边框窗口，支持深色模式

### 规划中

- 剪贴板历史记录
- 自定义 AI Prompt 模板
- 多语言支持 (i18n)
- 快捷键快速选择操作（数字键）

## 快速开始

### 安装与运行

1. **克隆仓库**
\`\`\`bash
git clone https://github.com/your-username/flow-paste.git
cd flow-paste
\`\`\`

2. **安装依赖**
\`\`\`bash
npm install
\`\`\`

3. **开发模式运行**
\`\`\`bash
npm run tauri:dev
\`\`\`

### 配置说明

#### 使用 Ollama (本地模型，推荐)

1. **安装 Ollama**
   - 访问 [Ollama 官网](https://ollama.com/) 下载并安装
   - 默认监听地址: \`http://localhost:11434\`

2. **下载模型**
\`\`\`bash
ollama pull llama3.2
\`\`\`

3. **配置 FlowPaste**
   - 按 \`Ctrl+,\` 打开设置面板
   - AI 提供商选择: \`Ollama\`
   - 模型名称: \`llama3.2\`（或其他已安装模型）
   - 保存配置

#### 使用 OpenAI API

1. **获取 API Key**
   - 访问 [OpenAI Platform](https://platform.openai.com/) 获取 API Key

2. **配置 FlowPaste**
   - 按 \`Ctrl+,\` 打开设置面板
   - AI 提供商选择: \`OpenAI\`
   - 输入 API Key（自动加密存储在系统密钥链）
   - API 地址: \`https://api.openai.com/v1\`（或兼容接口地址）
   - 模型名称: \`gpt-4o-mini\`（或其他模型）
   - 保存配置

#### 自定义热键

- 按 \`Ctrl+,\` 打开设置面板
- 修改热键组合（默认 \`Ctrl+Shift+V\`）
- 保存后自动重新注册全局快捷键

## 使用指南

### 基础操作

1. **唤起面板**: 按 \`Ctrl+Shift+V\`（或自定义热键）
2. **查看内容**: 面板自动显示剪贴板内容预览
3. **快捷处理**: 点击推荐的操作按钮快速处理
4. **自然语言**: 在输入框输入需求（如 "转成 JSON 格式"）
5. **确认粘贴**: 按 \`Enter\` 将结果粘贴到光标位置

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| \`Ctrl+Shift+V\` | 呼出/隐藏面板 |
| \`Ctrl+,\` | 打开设置面板 |
| \`ESC\` | 关闭面板 / 取消操作 |
| \`Enter\` | 执行指令 / 确认粘贴 |

### 隐私保护模式

FlowPaste 会自动检测剪贴板中的敏感信息:

- 🛡️ **绿色盾牌**: 未检测到敏感信息，可安全使用云端 API
- ⚠️ **黄色盾牌**: 检测到敏感信息，已自动脱敏处理
  - 敏感数据会替换为占位符（如 \`{{PHONE_01}}\`）后发送给 AI
  - AI 返回结果后自动还原为原始数据

**支持检测的敏感信息类型**:
- 中国手机号
- 身份证号
- 邮箱地址
- 银行卡号（支持 Luhn 算法校验）
- IPv4/IPv6 地址

## 技术栈

- **后端**: Rust + Tauri 2.x
- **前端**: Vue 3 + TypeScript + TailwindCSS
- **状态管理**: Pinia
- **构建工具**: Vite
- **数据存储**: SQLite + 系统密钥链 (Keyring)
- **AI 集成**: Ollama / OpenAI 兼容接口

## 开发环境

### 前置要求

- Node.js >= 18
- Rust >= 1.70
- pnpm (推荐) 或 npm

### 安装依赖

\`\`\`bash
npm install
\`\`\`

### 开发模式

\`\`\`bash
npm run tauri:dev
\`\`\`

### 构建发布

\`\`\`bash
npm run tauri:build
\`\`\`

构建产物位于 \`src-tauri/target/release/bundle/\`

## 项目结构

\`\`\`
flow-paste/
├── src/                    # 前端源码
│   ├── components/         # Vue 组件
│   │   ├── FloatingPanel.vue
│   │   ├── SettingsPanel.vue
│   │   └── ...
│   ├── stores/             # Pinia 状态管理
│   ├── composables/        # Vue 组合式函数
│   ├── types/              # TypeScript 类型定义
│   └── styles/             # 样式文件
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── lib.rs          # 主入口
│   │   ├── commands/       # Tauri 命令模块
│   │   ├── clipboard/      # 剪贴板操作
│   │   ├── privacy/        # 隐私保护 (PII 检测)
│   │   ├── ai/             # AI 集成 (Ollama/OpenAI)
│   │   ├── config/         # 配置管理 (SQLite)
│   │   ├── regex/          # 正则规则引擎
│   │   └── hotkey/         # 全局热键管理
│   └── capabilities/       # Tauri 权限配置
└── docs/                   # 文档
    ├── prd.md              # 产品需求文档
    └── ...
\`\`\`

## 常见问题

### Q: Ollama 连接失败?
**A**: 请确保:
1. Ollama 服务已启动 (\`ollama serve\`)
2. 监听地址正确（默认 \`http://localhost:11434\`）
3. 已下载至少一个模型 (\`ollama pull llama3.2\`)

### Q: OpenAI API 报错 401?
**A**: 请检查:
1. API Key 是否正确输入
2. API Key 是否有效（未过期/有余额）
3. API 地址是否正确

### Q: 热键不生效?
**A**: 请检查:
1. 热键组合是否被其他程序占用
2. 尝试更换热键组合（设置面板 \`Ctrl+,\`）
3. 检查程序是否有管理员权限（部分系统需要）

### Q: 粘贴功能不工作?
**A**: FlowPaste 使用键盘模拟实现粘贴，请确保:
1. 目标应用程序可接收键盘输入
2. 目标应用程序未锁定粘贴功能
3. Windows 用户可能需要以管理员身份运行

## 贡献指南

欢迎提交 Issue 和 Pull Request!

### 开发流程
1. Fork 本仓库
2. 创建特性分支 (\`git checkout -b feature/AmazingFeature\`)
3. 提交更改 (\`git commit -m 'Add some AmazingFeature'\`)
4. 推送到分支 (\`git push origin feature/AmazingFeature\`)
5. 提交 Pull Request

## 许可证

MIT License

## 致谢

- [Tauri](https://tauri.app/) - 跨平台应用框架
- [Ollama](https://ollama.com/) - 本地 AI 模型运行时
- [Vue.js](https://vuejs.org/) - 渐进式前端框架
- [TailwindCSS](https://tailwindcss.com/) - 实用优先的 CSS 框架
