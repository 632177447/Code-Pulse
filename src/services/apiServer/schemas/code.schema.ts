import { z } from 'zod';

const QueryNumberSchema = z.preprocess((value) => {
  if (value === undefined || value === null || value === '') {
    return undefined;
  }

  if (typeof value === 'number') {
    return value;
  }

  if (typeof value === 'string') {
    const parsed = Number(value);
    return Number.isNaN(parsed) ? value : parsed;
  }

  return value;
}, z.number().int().min(0).optional());

const QueryBooleanSchema = z.preprocess((value) => {
  if (value === undefined || value === null || value === '') {
    return undefined;
  }

  if (typeof value === 'boolean') {
    return value;
  }

  if (typeof value === 'string') {
    const normalized = value.trim().toLowerCase();
    if (normalized === 'true') {
      return true;
    }
    if (normalized === 'false') {
      return false;
    }
  }

  return value;
}, z.boolean().optional());

const FileNodeSchema = z.object({
  path: z.string(),
  content: z.string(),
  absPath: z.string(),
  depth: z.number().int().min(0),
  dependencies: z.array(z.string()),
  originId: z.string().optional()
});

export const ContextRequestQuerySchema = z.object({
  path: z.string().optional(),
  paths: z.string().optional(),
  maxDepth: QueryNumberSchema,
  ignoreExts: z.string().optional(),
  ignoreDeepParse: z.string().optional(),
  includedTypes: z.string().optional(),
  projectRoots: z.string().optional(),
  enableMinimization: QueryBooleanSchema,
  minimizationThreshold: QueryNumberSchema,
  minimizationDepthThreshold: QueryNumberSchema
});

export const ContextRequestBodySchema = z.object({
  path: z.string().optional(),
  paths: z.array(z.string()).optional(),
  maxDepth: z.number().int().min(0).optional(),
  ignoreExts: z.string().optional(),
  ignoreDeepParse: z.string().optional(),
  includedTypes: z.array(z.string()).optional(),
  projectRoots: z.string().optional(),
  enableMinimization: z.boolean().optional(),
  minimizationThreshold: z.number().int().min(0).optional(),
  minimizationDepthThreshold: z.number().int().min(0).optional()
});

export const ContextFormatQuerySchema = z.object({
  generateTree: QueryBooleanSchema,
  generateRelationshipText: QueryBooleanSchema,
  highlightPrimaryFiles: QueryBooleanSchema,
  optimizePathDisplay: QueryBooleanSchema,
  customPrompt: z.string().optional(),
  userPrompt: z.string().optional(),
  longContextThreshold: QueryNumberSchema
});

export const ContextFormatBodySchema = z.object({
  generateTree: z.boolean().optional(),
  generateRelationshipText: z.boolean().optional(),
  highlightPrimaryFiles: z.boolean().optional(),
  optimizePathDisplay: z.boolean().optional(),
  customPrompt: z.string().optional(),
  userPrompt: z.string().optional(),
  longContextThreshold: z.number().int().min(0).optional()
});

export const ContextTextRequestQuerySchema = ContextRequestQuerySchema.extend({
  ...ContextFormatQuerySchema.shape
});

export const ContextTextRequestBodySchema = ContextRequestBodySchema.extend({
  ...ContextFormatBodySchema.shape
});

export const RenderContextRequestBodySchema = ContextFormatBodySchema.extend({
  fileNodes: z.array(FileNodeSchema).min(1),
  selectedPaths: z.array(z.string()).optional()
});
