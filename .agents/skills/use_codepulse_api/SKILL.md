---
name: Use CodePulse API
description: "当需要跨文件分析代码依赖关系、理解模块架构、或在重构/阅读时获取带递归依赖链的完整代码上下文时，调用 CodePulse 本地 API 一次性获取目标文件及其依赖的大纲结构或含所有依赖的完整结构化代码。"
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
| **路径格式** | 所有 `path` / `paths` 参数必须使用**操作系统绝对路径**（如 `H:\Projects\my-project\src\App.vue`） |
| **⚠️ 路径转义** | 在 JSON Body 中，Windows 反斜杠必须双重转义（`H:\\Projects\\...`）；**推荐使用 GET 方式**以避免此问题 |
| **⚠️ Windows 环境** | PowerShell 中 `curl` 是 `Invoke-WebRequest` 的别名，行为不同。必须使用 `Invoke-RestMethod` |

---

## 📚 API 接口

### 1. 获取依赖大纲结构（轻量，仅含文件路径和依赖列表）

> 只需了解文件之间的依赖关系和层级，不需要读取实际代码时使用。

**`GET /api/v1/outline?paths=<绝对路径>`**（推荐，无转义问题）

```powershell
Invoke-RestMethod -Uri "http://localhost:13535/api/v1/outline?paths=H:\Projects\aibuild\CodePulse\src\App.vue" -Method Get | ConvertTo-Json -Depth 10
```

**`POST /api/v1/outline`**（需要指定更多参数时使用）

```powershell
$body = '{"paths":["H:\\Projects\\aibuild\\CodePulse\\src\\App.vue"],"maxDepth":3}'
Invoke-RestMethod -Uri "http://localhost:13535/api/v1/outline" -Method Post -ContentType "application/json" -Body $body | ConvertTo-Json -Depth 10
```

**返回结构：**
```json
{
  "data": [
    {
      "path": "src/App.vue",
      "absPath": "H:\\Projects\\...\\src\\App.vue",
      "depth": 0,
      "dependencies": ["src/components/Sidebar.vue", "src/composables/useTheme.ts"]
    },
    {
      "path": "src/components/Sidebar.vue",
      "absPath": "H:\\Projects\\...\\src\\components\\Sidebar.vue",
      "depth": 1,
      "dependencies": ["src/types.ts"]
    }
  ],
  "meta": { "count": 12, "engine": "rust" }
}
```

- `data[].path`：相对于项目根目录的路径
- `data[].absPath`：文件绝对路径
- `data[].depth`：在依赖树中的深度（0 = 目标文件本身）
- `data[].dependencies`：该文件直接依赖的文件相对路径列表

---

### 2. 获取完整代码上下文（含所有依赖文件的实际代码）

> 需要读取目标文件及其依赖文件的完整代码内容时使用。

**`GET /api/v1/context?paths=<绝对路径>`**（推荐，无转义问题）

```powershell
Invoke-RestMethod -Uri "http://localhost:13535/api/v1/context?paths=H:\Projects\aibuild\CodePulse\src\App.vue" -Method Get | ConvertTo-Json -Depth 10
```

**`POST /api/v1/context`**（需要指定更多参数时使用）

```powershell
$body = '{"paths":["H:\\Projects\\aibuild\\CodePulse\\src\\App.vue"],"maxDepth":2,"enableMinimization":true}'
Invoke-RestMethod -Uri "http://localhost:13535/api/v1/context" -Method Post -ContentType "application/json" -Body $body | ConvertTo-Json -Depth 10
```

**请求体参数（POST，所有字段均有默认值，可省略）：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `paths` | `string[]` | — | **必填**，入口文件绝对路径列表 |
| `maxDepth` | `number` | `2` | 依赖递归深度，0 = 仅目标文件本身 |
| `enableMinimization` | `boolean` | `true` | 压缩较深层依赖，只保留函数/类型声明以节省 Token |
| `minimizationDepthThreshold` | `number` | `2` | 从第几层依赖开始压缩 |
| `minimizationThreshold` | `number` | `8000` | 文件超过多少字节时执行压缩 |
| `includedTypes` | `string[]` | `["vue","ts","tsx","js","py","json","css","scss"]` | 纳入分析的文件扩展名 |
| `projectRoots` | `string` | `""` | 项目根目录路径（留空时自动推断） |

**返回结构：**
```json
{
  "data": [
    {
      "path": "src/App.vue",
      "absPath": "H:\\Projects\\...\\src\\App.vue", 
      "depth": 0,
      "dependencies": ["src/components/Sidebar.vue"],
      "content": "<template>...</template>\n<script setup>...</script>"
    }
  ],
  "meta": { "count": 12, "engine": "rust" }
}
```

- `data[].content`：文件的完整代码内容（或经过压缩后的精简版本）

---

## 🛠️ 标准操作流程

```
1. 获取数据  → 仅需依赖结构：GET /api/v1/outline?paths=<绝对路径>
              需要完整代码：GET /api/v1/context?paths=<绝对路径>

2. 使用数据  → outline：读取 data[].dependencies 了解依赖树
              context：读取 data[].content 获取每个文件的代码
```
