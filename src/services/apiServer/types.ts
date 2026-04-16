import type { FileNode } from '../../types';

export interface ApiRequest {
  id: string; // 用于关联并匹配具体的请求和响应
  url: string;
  method: string;
  headers: Record<string, string>;
  query: Record<string, string>;
  body?: string;
}

export interface ApiResponse {
  status: number;
  headers?: Record<string, string>;
  body: string;
}

export interface ContextRequest {
  paths: string[];
  maxDepth: number;
  ignoreExts: string;
  ignoreDeepParse: string;
  includedTypes: string[];
  projectRoots: string;
  enableMinimization: boolean;
  minimizationThreshold: number;
  minimizationDepthThreshold: number;
}

export interface ContextFormatOptions {
  generateTree: boolean;
  generateRelationshipText: boolean;
  highlightPrimaryFiles: boolean;
  generateLineNumbers: boolean;
  optimizePathDisplay: boolean;
  customPrompt: string;
  userPrompt: string;
  longContextThreshold: number;
}

export interface OutlineNode {
  path: string;
  absPath: string;
  depth: number;
  dependencies: string[];
}

export interface RenderContextRequest extends Partial<ContextFormatOptions> {
  fileNodes: FileNode[];
  selectedPaths?: string[];
}

export type ContextRequestInput = Omit<Partial<ContextRequest>, 'paths' | 'includedTypes'> & {
  path?: string;
  paths?: string[] | string;
  includedTypes?: string[] | string;
};

export type ContextTextRequestInput = ContextRequestInput & Partial<ContextFormatOptions>;

import { DEFAULT_APP_CONFIG } from '../../config/appSettings';

export const DEFAULT_CONTEXT_REQUEST: ContextRequest = {
  paths: [],
  maxDepth: DEFAULT_APP_CONFIG.maxDepth,
  ignoreExts: DEFAULT_APP_CONFIG.ignoreExts,
  ignoreDeepParse: DEFAULT_APP_CONFIG.ignoreDeepParse,
  includedTypes: [...DEFAULT_APP_CONFIG.includedTypes],
  projectRoots: DEFAULT_APP_CONFIG.projectRoots,
  enableMinimization: DEFAULT_APP_CONFIG.enableMinimization,
  minimizationThreshold: DEFAULT_APP_CONFIG.minimizationThreshold,
  minimizationDepthThreshold: DEFAULT_APP_CONFIG.minimizationDepthThreshold
};

export const DEFAULT_CONTEXT_FORMAT_OPTIONS: ContextFormatOptions = {
  generateTree: DEFAULT_APP_CONFIG.generateTree,
  generateRelationshipText: DEFAULT_APP_CONFIG.generateRelationshipText,
  highlightPrimaryFiles: DEFAULT_APP_CONFIG.highlightPrimaryFiles,
  generateLineNumbers: DEFAULT_APP_CONFIG.generateLineNumbers,
  optimizePathDisplay: DEFAULT_APP_CONFIG.optimizePathDisplay,
  customPrompt: DEFAULT_APP_CONFIG.customPrompt,
  userPrompt: '',
  longContextThreshold: DEFAULT_APP_CONFIG.minimizationThreshold
};
