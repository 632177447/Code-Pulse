# CodePulse (文脉)

![CodePulse Preview](./docs/assets/pic1.png)

**CodePulse (文脉)** 是一款专为大语言模型 (LLM) 设计的高效代码上下文生成工具。它能跨越 20+ 种编程语言，深度递归地解析文件内所有的 `import`、`require`、`use` 等依赖，将分散在项目各处的逻辑自动化地汇聚成一份层级分明、结构化的文本块。

它是开发者向 AI 发送代码 Prompt 的最佳拍档，通过极简的交互提升 AI 对复杂工程的理解能力。

[**立即下载最新版本**](https://github.com/632177447/Code-Pulse/releases) 🚀

---

## ✨ 核心特性

- **🔍 深度依赖追踪 (Multi-language Support)**: 自动解析 20 余种主流编程语言（包括 TS/JS, Rust, Python, Go, Java, C++, Ruby, PHP 等）的依赖引用，递归获取所有关联源代码。
- **🛡️ 智能过滤与忽略**: 内置 50+ 项过滤规则（如 `.git`, `node_modules`, `target`, `build`, `bin`, `obj` 等），并支持自定义 Glob 模式和后缀过滤。
- **⚡ 极致上下文压缩 (Context Minimization)**: **核心黑科技** —— 对非直接选区的引用文件，支持自动移除函数/类成员实现而仅保留定义。该功能可大幅优化 LLM 上下文空间，将关键 Token 留给核心业务代码。
- **🌳 结构化文件树**: 自动在输出框顶部生成清爽的 `FILE TREE`，使 AI 在接收代码前先掌握项目的整体文件拓扑。
- **✍️ 实时预览与精修**: 结果生成后支持直接在输出框内进行二次编辑，并在边缘提供字数统计，确保 Prompt 完美符合预期。
- **🎨 现代交互体验**: 基于 **Tauri 2.x + Vue 3 + Tailwind CSS 4.x** 构建，界面采用极简 Apple/Linear 风格，支持全局拖放、横向滚动管理和毛玻璃视觉效果。

---

## 💡 为何选择 CodePulse？

在 AI 开发时代，**上下文的完整性** 决定了 AI 回答的上限。

当 **AI Agent 的使用额度受限** 或 **无法自动访问项目全貌** 时，CodePulse 为你提供了一种“平替”方案：通过精准解析代码间的“文脉”联系，它能将复杂的工程逻辑压缩并汇聚。即便在普通的 Chat 窗口（如 ChatGPT-5, Claude 4.5）中直接粘贴生成的结果，你也能让 AI 获得**接近 Agent 级别的全局感知力**，从而完成高难度的重构或代码审查任务。

---

## 📥 下载与安装

您可以直接前往 [GitHub Releases](https://github.com/632177447/Code-Pulse/releases) 下载适用于 Windows、macOS 或 Linux 的最新预编译程序。

---

## 🏗️ 快速开始

### 开发环境配置

确保系统中已安装 [Rust](https://www.rust-lang.org/) (rustc 1.77+) 环境。

1. **克隆项目**:
   ```bash
   git clone https://github.com/your-repo/CodePulse.git
   cd CodePulse
   ```

2. **安装前端依赖**:
   ```bash
   npm install
   ```

3. **启动开发服务器**:
   ```bash
   npm run tauri dev
   ```

### 构建正式版本

```bash
npm run tauri build
```

---

## 📖 使用指南

1. **核心配置**: 在“设置 (Settings)”中配置递归深度、包含的文件类型或指定自定义的项目根路径。
   
   <details>
   <summary>📸 点击查看设置页预览 (Settings Previews)</summary>
   <br />
   <p align="center">
     <img src="./docs/assets/pic2.png" width="45%" />
     <img src="./docs/assets/pic3.png" width="45%" />
   </p>
   </details>

2. **导入代码**: 将特定的代码文件、目录或整个功能模块拖入顶部的“解析区”。
3. **附加需求**: 在右侧的“附加提示词”框内输入你的针对性指令（如“请基于这些文件进行代码审查”）。
4. **即刻解析**: 点击中央的“生成完整上下文”，CodePulse 会开始深度扫描依赖并压缩逻辑体。
5. **一键就绪**: 点击“复制”，将整理好的上下文直接粘贴到 ChatGPT / Claude / DeepSeek。

---

## 🛠️ 技术底座

- **前端框架**: [Vue 3](https://vuejs.org/) (Composition API)
- **桌面引擎**: [Tauri 2.0](https://tauri.app/) (Rust 驱动，极小体积与极致安全)
- **样式系统**: [Tailwind CSS 4.x](https://tailwindcss.com/)
- **解析内核**: 基于 Rust 的 Regex 驱动的高性能依赖嗅探器。

---

## 🤝 贡献与反馈

如果你有任何建议、发现 Bug 或想贡献代码，欢迎提交 Issue 或 Pull Request！我们期待你的声音！

---

## 📄 开源协议

MIT License | Copyright (c) 2024 CodePulse Team
