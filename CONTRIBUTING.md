# 贡献指南 | Contributing Guide

感谢你对 FlowPaste 的关注！我们非常欢迎社区贡献，无论是代码、文档、翻译还是功能建议。

## 行为准则

参与本项目即表示你同意遵守我们的 [行为准则](CODE_OF_CONDUCT.md)。

## 如何贡献

### 报告 Bug

如果你发现了 Bug，请：

1. 先在 [Issues](https://github.com/your-username/flow-paste/issues) 中搜索是否已有相关报告
2. 如果没有，创建新 Issue 并提供：
   - 清晰的标题和描述
   - 复现步骤
   - 预期行为 vs 实际行为
   - 系统环境（操作系统、版本号等）
   - 相关日志或截图

### 提出功能建议

我们欢迎新功能建议！请：

1. 在 [Discussions](https://github.com/your-username/flow-paste/discussions) 中讨论你的想法
2. 说明功能的使用场景和价值
3. 如果可能，提供设计草图或示例

### 贡献代码

#### 开发环境搭建

1. **Fork 仓库**
   ```bash
   # 在 GitHub 上 Fork 本仓库
   git clone https://github.com/your-username/flow-paste.git
   cd flow-paste
   ```

2. **安装依赖**
   ```bash
   # 前端依赖
   npm install

   # Rust 依赖（自动处理）
   # 确保已安装 Rust >= 1.70
   ```

3. **运行开发环境**
   ```bash
   npm run tauri:dev
   ```

4. **配置 AI 模型（可选）**
   ```bash
   # 安装 Ollama
   # 参考: https://ollama.com/

   # 下载模型
   ollama pull llama3.2
   ```

#### 代码规范

**TypeScript/Vue**:
- 使用 ESLint 和 Prettier 格式化代码
- 遵循 Vue 3 组合式 API 风格
- 组件命名使用 PascalCase
- 函数和变量使用 camelCase
- 常量使用 SCREAMING_SNAKE_CASE

**Rust**:
- 运行 `cargo fmt` 格式化代码
- 运行 `cargo clippy` 检查代码质量
- 避免使用 `unwrap()`，使用 `?` 或 `match` 处理错误
- 为公共 API 添加文档注释

#### 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type 类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式调整（不影响功能）
- `refactor`: 重构（不是新功能也不是 Bug 修复）
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建工具或辅助工具的变动

**示例**:
```bash
git commit -m "feat(ai): add support for Anthropic Claude API"
git commit -m "fix(clipboard): resolve paste failure on Windows"
git commit -m "docs: update installation guide for macOS"
```

#### Pull Request 流程

1. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或
   git checkout -b fix/your-bug-fix
   ```

2. **开发并提交**
   ```bash
   # 进行开发
   git add .
   git commit -m "feat: your feature description"
   ```

3. **推送到 Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

4. **创建 Pull Request**
   - 在 GitHub 上创建 PR
   - 填写 PR 模板
   - 关联相关 Issue（如果有）
   - 等待 Review

5. **代码审查**
   - 维护者会审查你的代码
   - 根据反馈进行修改
   - 保持 PR 更新

6. **合并**
   - PR 通过审查后会被合并
   - 你的贡献会出现在 Contributors 列表中

### 贡献文档

文档同样重要！你可以：

- 修复文档中的错误
- 改进文档的清晰度
- 添加使用示例
- 翻译文档到其他语言

文档位于：
- `README.md` - 项目说明
- `docs/` - 详细文档
- `CLAUDE.md` - AI 上下文文档

### 贡献翻译

我们欢迎多语言支持！如果你想贡献翻译：

1. 在 [Discussions](https://github.com/your-username/flow-paste/discussions) 中提出
2. 等待 i18n 系统建立后开始翻译
3. 遵循翻译指南（即将发布）

## 寻找贡献机会

### Good First Issues

我们为新贡献者标记了 `good first issue` 标签的 Issue，这些是：
- 相对简单的任务
- 有清晰的描述和指导
- 适合熟悉项目的第一步

查看 [Good First Issues](https://github.com/your-username/flow-paste/labels/good%20first%20issue)

### Help Wanted

标记为 `help wanted` 的 Issue 表示我们特别需要社区帮助。

## 开发技巧

### 调试

**前端调试**:
- 使用浏览器开发者工具（在 Tauri 窗口中按 F12）
- 查看 Console 日志
- 使用 Vue DevTools

**后端调试**:
- 使用 `println!` 或 `dbg!` 宏
- 查看终端输出
- 使用 Rust 调试器（如 rust-lldb）

### 测试

```bash
# 前端测试（待建立）
npm run test

# 后端测试
cd src-tauri
cargo test
```

### 构建

```bash
# 开发构建
npm run tauri:dev

# 生产构建
npm run tauri:build
```

## 社区

- **GitHub Discussions**: 功能讨论、问答
- **Issues**: Bug 报告、功能请求
- **Discord**: 实时交流（即将开放）

## 许可证

贡献代码即表示你同意将代码以 MIT 协议授权。

## 致谢

感谢所有贡献者！你的贡献让 FlowPaste 变得更好。

---

如有任何问题，欢迎在 [Discussions](https://github.com/your-username/flow-paste/discussions) 中提问。
