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
| **基础 URL** | `http://localhost:<port>`，默认端口 `13535` |
| **路径格式** | 所有 `path` / `paths` 参数必须使用**操作系统绝对路径**（如 `H:\Projects\...\App.vue`） |
| **请求方式** | 核心接口均使用 **POST** 请求，数据通过 JSON Body 传输 |
| **⚠️ 路径转义** | 在 JSON Body 中，Windows 反斜杠必须双重转义（`H:\\Projects\\...`） |
| **⚠️ Windows 环境** | PowerShell 中必须使用 `Invoke-RestMethod` 发起请求 |

---

## 📚 API 接口

### 1. 生成依赖大纲（轻量，仅含文件路径和依赖列表）

> 只需了解文件之间的依赖关系和层级，不需要读取实际代码时使用。

**`POST /api/v1/outlines/generate`**

```powershell
# 使用 PowerShell 发起请求示例
$body = '{"paths":["H:\\Projects\\aibuild\\CodePulse\\src\\App.vue"],"maxDepth":3}'
Invoke-RestMethod -Uri "http://localhost:13535/api/v1/outlines/generate" -Method Post -ContentType "application/json" -Body $body | ConvertTo-Json -Depth 10
```

**请求参数 (JSON)：**
- `paths`: `string[]` 目标文件绝对路径列表（推荐使用）
- `path`: `string` 单个目标文件绝对路径（可选）
- `maxDepth`: `number` 依赖递归深度（默认值视后端配置而定）

**返回结构：**
```json
{
  "data": [
    {
      "path": "src/App.vue",
      "absPath": "H:\\Projects\\...\\src\\App.vue",
      "depth": 0,
      "dependencies": ["src/components/Sidebar.vue", "src/composables/useTheme.ts"]
    }
  ],
  "meta": { "count": 1, "timestamp": "..." }
}
```

---

### 2. 生成完整代码上下文（含源码内容）

> 需要读取目标文件及其依赖文件的完整代码内容时使用。支持返回结构化数据或 LLM 友好的长文本。

**`POST /api/v1/contexts/generate`**

```powershell
# 获取 LLM 友好的格式化文本 (format: "text")
$body = '{"paths":["H:\\Projects\\aibuild\\CodePulse\\src\\App.vue"],"format":"text"}'
Invoke-RestMethod -Uri "http://localhost:13535/api/v1/contexts/generate" -Method Post -ContentType "application/json" -Body $body | ConvertTo-Json -Depth 10
```

**关键请求参数 (JSON)：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `paths` | `string[]` | — | **核心**，入口文件绝对路径列表 |
| `format` | `"json" \| "text"` | `"json"` | `json` 返回节点列表；`text` 返回拼接好的完整长文本 |
| `maxDepth` | `number` | `2` | 依赖递归深度 |
| `enableMinimization` | `boolean` | `true` | 是否压缩深层依赖（保留定义，隐藏实现） |
| `generateTree` | `boolean` | `false` | (仅 text) 是否在头部生成源码树结构图 |

**返回结构 (format: "json")：**
```json
{
  "data": [
    {
      "path": "src/App.vue",
      "content": "<template>...</template>...",
      "absPath": "H:\\...",
      "depth": 0,
      "dependencies": [...]
    }
  ],
  "meta": { "count": 5, "timestamp": "..." }
}
```

**返回结构 (format: "text")：**
```json
{
  "text": "File: src/App.vue\n---\n<content>\n...",
  "meta": { "count": 5, "length": 12040, "timestamp": "..." }
}
```

---

### 3. 系统接口

- **健康检查**: `GET /api/v1/health`
- **清空缓存**: `DELETE /api/v1/cache` (当解析结果不符合预期时尝试)

---

## 🛠️ 标准操作流程

```
1. 确定范围 → 仅需依赖树：POST /api/v1/outlines/generate
              需获取代码：POST /api/v1/contexts/generate (推荐 format: "text")

2. 准备路径 → 获取当前工程目标文件的绝对路径（如 H:\Projects\...）

3. 发起请求 → 使用 PowerShell 执行 Invoke-RestMethod

4. 处理结果 → 解析 data 数组或直接阅读 text 文本内容
```
