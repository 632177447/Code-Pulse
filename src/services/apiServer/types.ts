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

export const DEFAULT_CONTEXT_REQUEST: ContextRequest = {
  paths: [],
  maxDepth: 2,
  ignoreExts: '.git, node_modules, dist, target, build, .vscode, .idea, .next, .nuxt, .output, .vercel, .github, __pycache__, .venv, bin, obj, *.lock, *.log, *.tmp, *.temp, *.png, *.jpg, *.jpeg, *.gif, *.svg, *.ico, *.webp, *.mp4, *.avi, *.mkv, *.mov, *.webm, *.mp3, *.wav, *.flac, *.aac, *.ogg, *.zip, *.tar, *.gz, *.7z, *.rar, *.exe, *.dll, *.so, *.dylib',
  ignoreDeepParse: 'package.json, tsconfig.json, vite.config.ts, tauri.conf.json, README.md, Cargo.toml, go.mod, pom.xml, .env, *.test.ts, *.spec.ts',
  includedTypes: ['vue', 'ts', 'tsx', 'js', 'py', 'json', 'css', 'scss'],
  projectRoots: '',
  enableMinimization: true,
  minimizationThreshold: 8000,
  minimizationDepthThreshold: 2
};

export const DEFAULT_CONTEXT_FORMAT_OPTIONS: ContextFormatOptions = {
  generateTree: true,
  generateRelationshipText: true,
  highlightPrimaryFiles: true,
  optimizePathDisplay: false,
  customPrompt: '',
  userPrompt: '',
  longContextThreshold: DEFAULT_CONTEXT_REQUEST.minimizationThreshold
};
