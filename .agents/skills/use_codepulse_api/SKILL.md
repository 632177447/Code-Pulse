---
name: Use CodePulse API
description: "通过 CodePulse 提供的本地 RESTful API 服务，快速获取目标文件及其依赖链的依赖大纲结构或代码上下文完整文本，避免人工遍历。"
---

# CodePulse API 交互指南

CodePulse 是一个代码上下文收集工具。它通过内置的本地 HTTP 服务提供 API 接口，AI 助手可以直接通过请求 API 获取跨文件的完整代码上下文关联数据。当面对复杂的代码阅读或重构任务时，你可以把 CodePulse 的 API 当作你的**进阶上下文视窗**来使用。

## 📍 核心前提
1. **基础路径与端口**: API 请求的基础路径为 `http://localhost:<端口>`。未提供端口时，默认使用 `13535`。
2. **可用性确认**: 确保 CodePulse API 处于运行状态（通过 `GET /api/v1/health` 检查）。
3. **绝对路径原则**: CodePulse API 中的所有 `path` 或 `paths` 参数必须是**操作系统的完整绝对路径** (例如: `D:/Projects/my-project/src/App.vue`)。

## 📚 关键接口与用法

交互式接口文档始终位于: `http://localhost:<运行端口>/api/v1/ui`

### 1. 生成文件依赖大纲结构 (最常用)
当不需要实际代码逻辑，仅需了解有哪些文件关联和模块架构时调用该接口，速度极快。

- **URL**: `POST /api/v1/outlines/generate`
- **Payload 示例**:
  ```json
  {
    "paths": ["<绝对文件或目录路径>"],
    "maxDepth": 3
  }
  ```
- **特征**: 返回该文件依赖的具体路径与嵌套层级关系。

### 2. 生成并获取完整代码上下文
如果你需要读取某个文件，以及它 import 的相关依赖文件内容，请调用此接口。它会返回合并好、排版清洗过的文本，甚至还能附带文件依赖目录树。

- **URL**: `POST /api/v1/contexts/generate`
- **Payload 示例**:
  ```json
  {
    "paths": ["<绝对路径1>", "<绝对路径2>"],
    "maxDepth": 2,                      // 依赖挖掘的深度 (0 为仅本身，默认向下递归解析)
    "format": "text",                   // 返回专为大模型阅读优化的包含提供文件的所有依赖的完整代码上下文
    "generateTree": true,               // 在顶部附带依赖结构树图
    "enableMinimization": true,         // 自动压缩较远深度的文件(只传声明、忽略实现，省Token)
    "minimizationDepthThreshold": 2,    // 从第 2 层依赖开始执行压缩
  }
  ```
- **典型返回**:
  ```json
  {
    "text": "==== 项目依赖树 ===\n...\n==== 文件正文 ===\n...\n=== 依赖文件1 ===\n...\n=== 依赖文件2 ===\n...",
    "meta": {
      "count": 6, 
      "length": 8432
    }
  }
  ```

### 3. 清除内部解析缓存
CodePulse 默认具有内部文件缓存以加速处理。在此项目的代码经历过明显重构或变更后，建议立即调用清除缓存，以免拿到旧版代码数据。

- **URL**: `DELETE /api/v1/cache`

---

## 🛠️ AI 助手标准操作流程 (SOP)

当被要求“分析某个文件树或重构涉及依赖的组件”时：
1. **端口探查**: 从用户环境/会话确认当前 CodePulse 运行端口。在未提供端口时默认使用 `13535`。
2. **触发分析**: 组合 `curl -X POST ...` 或等效本地 HTTP 请求，使用 `http://localhost:13535/api/v1/contexts/generate` (设为 `format="text"`) 对目标入口文件下达请求。
3. **读取结果**: 将返回的 `text` 内容直接采纳为当前的上下文记忆，然后实施你的修改或分析计划。
4. **清理状态**: 必要时执行 `DELETE /api/v1/cache`。
