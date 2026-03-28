import { z } from '@hono/zod-openapi';

const FileNodeSchema = z.object({
  path: z.string().openapi({ example: 'src/main.ts', description: '文件相对路径' }),
  content: z.string().openapi({ description: '文件全文内容' }),
  absPath: z.string().openapi({ example: '/Users/dev/project/src/main.ts', description: '文件绝对路径' }),
  depth: z.number().int().min(0).openapi({ example: 1, description: '文件在源码树中的深度' }),
  dependencies: z.array(z.string()).openapi({ description: '该文件依赖的文件路径列表' }),
  originId: z.string().optional().openapi({ description: '原始标识符' })
}).openapi('FileNode');

export const GenerateContextBodySchema = z.object({
  path: z.string().optional().openapi({ example: 'D:/Projects/my-project/src/App.vue', description: '单个目标路径 (兼容)' }),
  paths: z.array(z.string()).optional().openapi({ example: ['src/App.vue'], description: '待解析的目标路径列表' }),
  maxDepth: z.number().int().min(0).optional().openapi({ example: 3, description: '依赖解析的最大深度' }),
  ignoreExts: z.string().optional().openapi({ example: 'spec.ts,test.ts', description: '忽略的文件后缀，逗号分隔' }),
  ignoreDeepParse: z.string().optional().openapi({ description: '忽略深度解析的目录/文件后缀' }),
  includedTypes: z.array(z.string()).optional().openapi({ example: ['ts', 'vue'], description: '包含的文件类型' }),
  projectRoots: z.string().optional().openapi({ description: '项目根目录配置' }),
  enableMinimization: z.boolean().optional().openapi({ example: true, description: '是否启用内容最小化（去除空行/多余空格）' }),
  minimizationThreshold: z.number().int().min(0).optional().openapi({ description: '最小化阈值' }),
  minimizationDepthThreshold: z.number().int().min(0).optional().openapi({ description: '最小化深度阈值' }),

  // 格式化相关可选项
  generateTree: z.boolean().optional().openapi({ description: '是否在文本中生成源码树结构' }),
  generateRelationshipText: z.boolean().optional().openapi({ description: '是否在文本中生成关联描述' }),
  highlightPrimaryFiles: z.boolean().optional().openapi({ description: '是否高亮主文件' }),
  optimizePathDisplay: z.boolean().optional().openapi({ description: '是否优化路径显示（简化公共前缀）' }),
  customPrompt: z.string().optional().openapi({ description: '自定义提示词' }),
  userPrompt: z.string().optional().openapi({ description: '用户特定提示词' }),
  longContextThreshold: z.number().int().min(0).optional().openapi({ description: '长上下文判定阈值' }),

  // 输出格式要求，默认返回 json 节点结构。
  format: z.enum(['json', 'text']).optional().default('json').openapi({ example: 'text', description: '输出格式：json 原生节点或 text 格式化文本' })
}).openapi('GenerateContextRequest');

export const GenerateOutlineBodySchema = z.object({
  path: z.string().optional().openapi({ example: 'src/App.vue' }),
  paths: z.array(z.string()).optional().openapi({ example: ['src/App.vue'] }),
  maxDepth: z.number().int().min(0).optional().openapi({ example: 1 }),
  ignoreExts: z.string().optional(),
  ignoreDeepParse: z.string().optional(),
  includedTypes: z.array(z.string()).optional(),
  projectRoots: z.string().optional(),
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
  engine: z.string().openapi({ example: 'frontend' }),
  timestamp: z.string().optional().openapi({ example: '1711612740' }),
  count: z.number().optional().openapi({ example: 5 }),
  length: z.number().optional().openapi({ example: 1024 })
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
  status: z.string(),
  meta: CommonMetaSchema
}).openapi('SimpleStatusResponse');
