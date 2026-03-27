import { invoke } from '@tauri-apps/api/core';
import pkg from '../../../../package.json';
import type { FileNode } from '../../../types';
import { createRenderableContextNodes, formatContextContent } from './contextFormatter';
import {
  DEFAULT_CONTEXT_FORMAT_OPTIONS,
  DEFAULT_CONTEXT_REQUEST,
  type ContextFormatOptions,
  type ContextRequest,
  type ContextRequestInput,
  type ContextTextRequestInput,
  type OutlineNode,
  type RenderContextRequest
} from '../types';

const FRONTEND_API_ROUTES = [
  '/api/context/text',
  '/api/context/render'
];

const GENERATE_CONTEXT_COMMAND = 'generate_context';
const ABORT_CONTEXT_COMMAND = 'abort_generate_context';
const CLEAR_CACHE_COMMAND = 'clear_cache';
const FRONTEND_ENGINE = 'frontend';

export class ApiValidationError extends Error {
  details?: string;

  constructor(message: string, details?: string) {
    super(message);
    this.name = 'ApiValidationError';
    this.details = details;
  }
}

function splitTextList(value: string) {
  return value
    .split(/[\n\r,]/)
    .map(item => item.trim())
    .filter(Boolean);
}

function normalizePaths(input?: string[] | string, fallbackPath?: string) {
  const result: string[] = [];

  if (typeof fallbackPath === 'string' && fallbackPath.trim()) {
    result.push(fallbackPath.trim());
  }

  if (Array.isArray(input)) {
    input.forEach(item => {
      if (typeof item === 'string' && item.trim()) {
        result.push(item.trim());
      }
    });
  } else if (typeof input === 'string') {
    result.push(...splitTextList(input));
  }

  return Array.from(new Set(result));
}

function normalizeIncludedTypes(input?: string[] | string) {
  if (Array.isArray(input)) {
    const normalized = input
      .map(item => item.trim().replace(/^\./, '').toLowerCase())
      .filter(Boolean);

    return normalized.length > 0 ? Array.from(new Set(normalized)) : [...DEFAULT_CONTEXT_REQUEST.includedTypes];
  }

  if (typeof input === 'string' && input.trim()) {
    const normalized = splitTextList(input)
      .map(item => item.replace(/^\./, '').toLowerCase())
      .filter(Boolean);

    return normalized.length > 0 ? Array.from(new Set(normalized)) : [...DEFAULT_CONTEXT_REQUEST.includedTypes];
  }

  return [...DEFAULT_CONTEXT_REQUEST.includedTypes];
}

function createContextRequest(input: ContextRequestInput = {}): ContextRequest {
  return {
    paths: normalizePaths(input.paths, input.path),
    maxDepth: input.maxDepth ?? DEFAULT_CONTEXT_REQUEST.maxDepth,
    ignoreExts: input.ignoreExts ?? DEFAULT_CONTEXT_REQUEST.ignoreExts,
    ignoreDeepParse: input.ignoreDeepParse ?? DEFAULT_CONTEXT_REQUEST.ignoreDeepParse,
    includedTypes: normalizeIncludedTypes(input.includedTypes),
    projectRoots: input.projectRoots ?? DEFAULT_CONTEXT_REQUEST.projectRoots,
    enableMinimization: input.enableMinimization ?? DEFAULT_CONTEXT_REQUEST.enableMinimization,
    minimizationThreshold: input.minimizationThreshold ?? DEFAULT_CONTEXT_REQUEST.minimizationThreshold,
    minimizationDepthThreshold: input.minimizationDepthThreshold ?? DEFAULT_CONTEXT_REQUEST.minimizationDepthThreshold
  };
}

function createFormatOptions(input: Partial<ContextFormatOptions> = {}): ContextFormatOptions {
  return {
    generateTree: input.generateTree ?? DEFAULT_CONTEXT_FORMAT_OPTIONS.generateTree,
    generateRelationshipText: input.generateRelationshipText ?? DEFAULT_CONTEXT_FORMAT_OPTIONS.generateRelationshipText,
    highlightPrimaryFiles: input.highlightPrimaryFiles ?? DEFAULT_CONTEXT_FORMAT_OPTIONS.highlightPrimaryFiles,
    optimizePathDisplay: input.optimizePathDisplay ?? DEFAULT_CONTEXT_FORMAT_OPTIONS.optimizePathDisplay,
    customPrompt: input.customPrompt ?? DEFAULT_CONTEXT_FORMAT_OPTIONS.customPrompt,
    userPrompt: input.userPrompt ?? DEFAULT_CONTEXT_FORMAT_OPTIONS.userPrompt,
    longContextThreshold: input.longContextThreshold ?? DEFAULT_CONTEXT_FORMAT_OPTIONS.longContextThreshold
  };
}

function ensurePaths(paths: string[], routeDescription: string) {
  if (paths.length === 0) {
    throw new ApiValidationError('Missing required field: paths', routeDescription);
  }
}

async function generateFileNodes(request: ContextRequest) {
  ensurePaths(request.paths, 'Use query parameter path/paths or JSON body with paths.');

  return invoke<FileNode[]>(GENERATE_CONTEXT_COMMAND, {
    paths: request.paths,
    maxDepth: request.maxDepth,
    ignoreExts: request.ignoreExts,
    ignoreDeepParse: request.ignoreDeepParse,
    includedTypes: request.includedTypes,
    projectRoots: request.projectRoots,
    enableMinimization: request.enableMinimization,
    minimizationThreshold: request.minimizationThreshold,
    minimizationDepthThreshold: request.minimizationDepthThreshold
  });
}

function toOutline(nodes: FileNode[]): OutlineNode[] {
  return nodes.map(node => ({
    path: node.path,
    absPath: node.absPath,
    depth: node.depth,
    dependencies: [...node.dependencies]
  }));
}

export const CodeService = {
  getHealth() {
    return {
      status: 'ok',
      meta: {
        engine: FRONTEND_ENGINE,
        timestamp: Math.floor(Date.now() / 1000).toString()
      }
    };
  },

  getInfo() {
    return {
      data: {
        name: 'CodePulse Frontend API Service',
        version: pkg.version,
        description: 'Frontend-powered context rendering and analysis gateway',
        routes: FRONTEND_API_ROUTES
      },
      meta: {
        engine: FRONTEND_ENGINE
      }
    };
  },

  async clearCache() {
    await invoke(CLEAR_CACHE_COMMAND);

    return {
      status: 'ok',
      meta: {
        engine: FRONTEND_ENGINE
      }
    };
  },

  async abortContext() {
    await invoke(ABORT_CONTEXT_COMMAND);

    return {
      status: 'aborting',
      meta: {
        engine: FRONTEND_ENGINE
      }
    };
  },

  async getContext(input: ContextRequestInput = {}) {
    const request = createContextRequest(input);
    const nodes = await generateFileNodes(request);

    return {
      data: nodes,
      meta: {
        count: nodes.length,
        engine: FRONTEND_ENGINE
      }
    };
  },

  async getOutline(input: ContextRequestInput = {}) {
    const request = createContextRequest(input);
    const outline = toOutline(await generateFileNodes(request));

    return {
      data: outline,
      meta: {
        count: outline.length,
        engine: FRONTEND_ENGINE
      }
    };
  },

  async getContextText(input: ContextTextRequestInput = {}) {
    const request = createContextRequest(input);
    const formatOptions = createFormatOptions(input);
    const nodes = await generateFileNodes(request);
    const text = formatContextContent(createRenderableContextNodes(nodes, request.paths), formatOptions);

    return {
      text,
      meta: {
        count: nodes.length,
        length: text.length,
        engine: FRONTEND_ENGINE
      }
    };
  },

  async renderContextText(input: RenderContextRequest) {
    const formatOptions = createFormatOptions(input);
    const selectedPaths = input.selectedPaths ?? [];
    const text = formatContextContent(createRenderableContextNodes(input.fileNodes, selectedPaths), formatOptions);

    return {
      text,
      meta: {
        count: input.fileNodes.length,
        length: text.length,
        engine: FRONTEND_ENGINE
      }
    };
  }
};
