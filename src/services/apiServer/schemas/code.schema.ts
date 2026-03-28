import { z } from 'zod';

const FileNodeSchema = z.object({
  path: z.string(),
  content: z.string(),
  absPath: z.string(),
  depth: z.number().int().min(0),
  dependencies: z.array(z.string()),
  originId: z.string().optional()
});

export const GenerateContextBodySchema = z.object({
  path: z.string().optional(),
  paths: z.array(z.string()).optional(),
  maxDepth: z.number().int().min(0).optional(),
  ignoreExts: z.string().optional(),
  ignoreDeepParse: z.string().optional(),
  includedTypes: z.array(z.string()).optional(),
  projectRoots: z.string().optional(),
  enableMinimization: z.boolean().optional(),
  minimizationThreshold: z.number().int().min(0).optional(),
  minimizationDepthThreshold: z.number().int().min(0).optional(),

  // 格式化相关可选项
  generateTree: z.boolean().optional(),
  generateRelationshipText: z.boolean().optional(),
  highlightPrimaryFiles: z.boolean().optional(),
  optimizePathDisplay: z.boolean().optional(),
  customPrompt: z.string().optional(),
  userPrompt: z.string().optional(),
  longContextThreshold: z.number().int().min(0).optional(),

  // 输出格式要求，默认返回 json 节点结构。
  format: z.enum(['json', 'text']).optional().default('json')
});

export const GenerateOutlineBodySchema = z.object({
  path: z.string().optional(),
  paths: z.array(z.string()).optional(),
  maxDepth: z.number().int().min(0).optional(),
  ignoreExts: z.string().optional(),
  ignoreDeepParse: z.string().optional(),
  includedTypes: z.array(z.string()).optional(),
  projectRoots: z.string().optional(),
});

export const RenderContextBodySchema = z.object({
  fileNodes: z.array(FileNodeSchema).min(1),
  selectedPaths: z.array(z.string()).optional(),
  
  // 格式化相关可选项
  generateTree: z.boolean().optional(),
  generateRelationshipText: z.boolean().optional(),
  highlightPrimaryFiles: z.boolean().optional(),
  optimizePathDisplay: z.boolean().optional(),
  customPrompt: z.string().optional(),
  userPrompt: z.string().optional(),
  longContextThreshold: z.number().int().min(0).optional(),
});
