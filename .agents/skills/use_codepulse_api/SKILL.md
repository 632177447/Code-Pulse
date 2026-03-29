---
name: Use CodePulse API
description: "当需要跨文件分析代码依赖关系、理解模块架构、或在重构/阅读时获取带递归依赖链的完整代码上下文时，调用 CodePulse 本地 API 一次性获取目标文件及其依赖的大纲结构或含所有依赖的完整结构化代码文本。"
---

# CodePulse API — AI 代码上下文获取工具

CodePulse 在本地运行 HTTP 服务，暴露 API 接口，让你能够**一次请求即获取跨文件的完整代码上下文**——包括目标文件本身和它递归 import 的所有依赖。

## ✅ 何时使用此 Skill

满足以下**任一**条件时，优先使用 CodePulse API 而非逐文件 `view_file`：

- **跨文件分析**：任务涉及理解一个文件和它 import 的依赖链（如组件→composable→util→types 多层引用）
- **模块架构探索**：需要快速了解某个入口文件的依赖树结构，而不需要读取每个文件的具体代码
- **重构前上下文收集**：重构前需要一次性掌握所有受影响文件的代码内容
- **依赖影响范围评估**：需要知道修改某个文件会影响哪些下游文件

## ⚠️ 何时不使用

- 只需要读取**单个文件**且不关心其依赖 → 直接用 `view_file`
- 需要搜索代码中的特定文本 → 用 `grep_search`

---

## 📍 核心约束

| 项目 | 说明 |
|------|------|
| **基础 URL** | `http://localhost:<port>`，未指定端口时默认 `13535` |
| **健康检查** | `GET /api/v1/health` — 首次调用前确认服务可用 |
| **路径格式** | 所有 `path` / `paths` 参数必须使用**操作系统完整绝对路径**（如 `D:/Projects/my-project/src/App.vue`） |
| **交互式文档** | `http://localhost:<port>/api/v1/ui` |

---

## 📚 API 接口

### 1. 生成依赖大纲结构（轻量快速）

> 只需了解文件之间的依赖关系和模块层级，不需要读取实际代码时使用。

**`POST /api/v1/outlines/generate`**

```json
{
  "paths": ["<绝对文件或目录路径>"],
  "maxDepth": 3
}
```

| 参数 | 类型 | 说明 |
|------|------|------|
| `paths` | `string[]` | 要分析的入口文件或目录的绝对路径列表 |
| `maxDepth` | `number` | 依赖递归解析的最大深度（默认向下递归） |

**返回**：目标文件依赖的文件路径清单及嵌套层级关系。

---

### 2. 生成完整代码上下文（核心接口）

> 需要读取目标文件及其依赖文件的完整代码内容时使用。返回已合并、排版清洗过的文本，可直接作为上下文。

**`POST /api/v1/contexts/generate`**

```json
{
  "paths": ["<绝对路径1>", "<绝对路径2>"],
  "maxDepth": 2,
  "format": "text",
  "generateTree": true,
  "enableMinimization": true,
  "minimizationDepthThreshold": 2
}
```

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `paths` | `string[]` | — | 入口文件的绝对路径列表 |
| `maxDepth` | `number` | 递归 | 依赖挖掘深度。`0` = 仅文件本身 |
| `format` | `string` | `"text"` | 设为 `"text"` 以获取为大模型阅读优化的纯文本 |
| `generateTree` | `boolean` | `false` | 在输出顶部附带依赖结构树图 |
| `enableMinimization` | `boolean` | `false` | 对较远层级的依赖自动压缩（只保留声明，省 Token） |
| `minimizationDepthThreshold` | `number` | `2` | 从第 N 层依赖开始执行压缩 |

**返回示例**：
```json
{
  "text": "==== 项目依赖树 ===\n...\n==== 文件正文 ===\n...\n=== 依赖文件1 ===\n...",
  "meta": { "count": 6, "length": 8432 }
}
```

- `text`：直接采纳为上下文记忆的完整代码文本
- `meta.count`：包含的文件总数
- `meta.length`：文本总字符数

---

### 3. 清除解析缓存

> 当目标项目的代码刚发生过重构或大量变更时调用，避免获取到旧版数据。

**`DELETE /api/v1/cache`**

---

## 🛠️ 标准操作流程

```
1. 确认端口 → 默认 13535，或从用户会话中获取
2. 健康检查 → GET http://localhost:13535/api/v1/health
3. 获取上下文 → POST /api/v1/contexts/generate（常规场景）
                或 POST /api/v1/outlines/generate（仅需结构）
4. 使用返回的 text 作为当前任务的代码上下文
5. 如果项目代码刚经历过大改动 → DELETE /api/v1/cache
```

**curl 快速示例**：
```bash
curl -X POST http://localhost:13535/api/v1/contexts/generate \
  -H "Content-Type: application/json" \
  -d '{"paths":["D:/Projects/my-app/src/App.vue"],"maxDepth":2,"format":"text","generateTree":true}'
```
