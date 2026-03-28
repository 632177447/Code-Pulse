import { z } from '@hono/zod-openapi';

const FileNodeSchema = z.object({
  path: z.string().openapi({ example: 'src/main.ts', description: '文件相对路径（相对于项目根目录的显示路径）' }),
  content: z.string().openapi({ description: '文件全文内容或格式化后的文本视图' }),
  absPath: z.string().openapi({ example: '/Users/dev/project/src/main.ts', description: '文件的物理绝对路径' }),
  depth: z.number().int().min(0).openapi({ example: 1, description: '文件在源码依赖树中的层级深度 (0 为主文件)' }),
  dependencies: z.array(z.string()).openapi({ description: '该文件依赖的其它文件路径列表（显示路径）' }),
  originId: z.string().optional().openapi({ description: '文件原始标识符，用于追踪节点来源' })
}).openapi('FileNode');

export const GenerateContextBodySchema = z.object({
  path: z.string().optional().openapi({ 
    example: 'D:/Projects/my-project/src/App.vue', 
    description: '单个目标绝对路径。API 仅接受物理绝对路径以确保解析准确性。' 
  }),
  paths: z.array(z.string()).optional().openapi({ 
    example: ['D:/Projects/my-project/src/App.vue'], 
    description: '待解析的目标绝对路径列表。所有路径必须为物理绝对路径。' 
  }),
  maxDepth: z.number().int().min(0).optional().openapi({ example: 2, description: '依赖解析的最大递归深度' }),
  ignoreExts: z.string().optional().openapi({ 
    example: '.git,node_modules,dist', 
    description: '需要忽略的文件后缀或目录名，多个以逗号分隔' 
  }),
  ignoreDeepParse: z.string().optional().openapi({ 
    example: 'package.json,*.test.ts',
    description: '需要忽略深度依赖解析（仅保留内容但不追踪其 import）的文件或后缀' 
  }),
  includedTypes: z.array(z.string()).optional().openapi({ 
    example: ['ts', 'vue', 'js'], 
    description: '包含在解析范围内的文件类型后缀列表' 
  }),
  projectRoots: z.string().optional().openapi({ 
    example: 'D:/Projects/my-project',
    description: '可选的多个项目根目录配置（逗号分隔），用于辅助绝对路径的显示转换' 
  }),
  enableMinimization: z.boolean().optional().openapi({ 
    example: true, 
    description: '是否启用代码压缩（对于深层依赖文件，压缩代码中的函数具体实现）' 
  }),
  minimizationThreshold: z.number().int().min(0).optional().openapi({ 
    example: 8000,
    description: '触发代码压缩的文件字符长度阈值' 
  }),
  minimizationDepthThreshold: z.number().int().min(0).optional().openapi({ 
    example: 2,
    description: '触发代码压缩的依赖层级深度阈值（深度大于等于该值的文件才会被代码压缩）' 
  }),

  // 格式化相关可选项
  generateTree: z.boolean().optional().openapi({ description: '输出文本时，是否在头部生成源码树结构图' }),
  generateRelationshipText: z.boolean().optional().openapi({ description: '输出文本时，是否生成文件间的依赖关系摘要' }),
  highlightPrimaryFiles: z.boolean().optional().openapi({ description: '渲染时是否使用 [PRIMARY FILE] 标签标记用户直接选中的文件' }),
  optimizePathDisplay: z.boolean().optional().openapi({ description: '是否通过提取公共前缀（<BASE_PATH>）来优化长路径的显示' }),
  customPrompt: z.string().optional().openapi({ description: '注入到上下文底部的系统级自定义提示词' }),
  userPrompt: z.string().optional().openapi({ description: '注入到上下文顶部的用户特定指令或需求描述' }),
  longContextThreshold: z.number().int().min(0).optional().openapi({ 
    description: '当内容长度超过此阈值时，自动调整 userPrompt 的布局位置' 
  }),

  // 输出格式要求
  format: z.enum(['json', 'text']).optional().default('json').openapi({ 
    example: 'text', 
    description: '期望的数据返回格式：json (结构化节点) 或 text (适合 LLM 的格式化长文本)' 
  })
}).openapi('GenerateContextRequest');

export const GenerateOutlineBodySchema = z.object({
  path: z.string().optional().openapi({ 
    example: 'D:/Projects/my-project/main.ts',
    description: '单个目标绝对路径' 
  }),
  paths: z.array(z.string()).optional().openapi({ 
    example: ['D:/Projects/my-project/main.ts'],
    description: '待生成大纲的绝对路径列表' 
  }),
  maxDepth: z.number().int().min(0).optional().openapi({ example: 1, description: '依赖展开的最大深度' }),
  ignoreExts: z.string().optional().openapi({ description: '忽略的文件后缀' }),
  ignoreDeepParse: z.string().optional().openapi({ description: '忽略深度解析的文件后缀' }),
  includedTypes: z.array(z.string()).optional().openapi({ description: '包含的文件后缀扩展名' }),
  projectRoots: z.string().optional().openapi({ description: '项目根目录配置，用于修正显示路径' }),
}).openapi('GenerateOutlineRequest');

export const RenderContextBodySchema = z.object({
  fileNodes: z.array(FileNodeSchema).min(1).openapi({ description: '待渲染的文件节点列表' }),
  selectedPaths: z.array(z.string()).optional().openapi({ description: '被选中高亮的文件路径' }),
  
  // 格式化相关可选项
  generateTree: z.boolean().optional(),
  generateRelationshipText: z.boolean().optional(),
  highlightPrimaryFiles: z.boolean().optional(),
  optimizePathDisplay: z.boolean().optional(),
  customPrompt: z.string().optional(),
  userPrompt: z.string().optional(),
  longContextThreshold: z.number().int().min(0).optional(),
}).openapi('RenderContextRequest');

export const CommonMetaSchema = z.object({
  timestamp: z.string().optional().openapi({ example: '1711612740', description: '操作完成时的 UNIX 时间戳' }),
  count: z.number().optional().openapi({ example: 5, description: '处理涉及到的文件总数' }),
  length: z.number().optional().openapi({ example: 1024, description: '生成内容的字符总长度（针对 text 模式）' })
}).openapi('CommonMeta');

export const ErrorResponseSchema = z.object({
  error: z.object({
    message: z.string().openapi({ example: 'Invalid request' }),
    details: z.any().optional()
  })
}).openapi('ErrorResponse');

export const ContextResponseSchema = z.object({
  data: z.array(FileNodeSchema).optional(),
  text: z.string().optional(),
  meta: CommonMetaSchema
}).openapi('ContextResponse');

export const OutlineResponseSchema = z.object({
  data: z.array(z.object({
    path: z.string(),
    absPath: z.string(),
    depth: z.number(),
    dependencies: z.array(z.string())
  })).openapi({ description: '大纲节点列表' }),
  meta: CommonMetaSchema
}).openapi('OutlineResponse');

export const HealthResponseSchema = z.object({
  status: z.string().openapi({ example: 'ok' }),
  meta: CommonMetaSchema
}).openapi('HealthResponse');

export const InfoResponseSchema = z.object({
  data: z.object({
    name: z.string(),
    version: z.string(),
    description: z.string(),
    routes: z.array(z.string())
  }),
  meta: CommonMetaSchema
}).openapi('SimpleMessageResponse');

export const SimpleStatusResponseSchema = z.object({
  status: z.string().openapi({ example: 'ok', description: '操作执行状态标识' }),
  meta: CommonMetaSchema
}).openapi('SimpleStatusResponse');
