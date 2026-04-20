export type { SettingItem, SettingGroup } from './components/common/SettingsModal/types';

export interface FileListItem {
  id: string;
  path: string;
}

export interface FileNode {
  path: string;
  content: string;
  absPath: string;
  depth: number;
  dependencies: string[];
  originId?: string;
}

export interface AppConfig {
  maxDepth: number;
  includedTypes: string[];
  ignoreExts: string;
  ignoreDeepParse: string;
  customPrompt: string;
  generateTree: boolean;
  generateRelationshipText: boolean;
  highlightPrimaryFiles: boolean;
  generateLineNumbers: boolean;
  optimizePathDisplay: boolean;
  omitFileBlocks: boolean;
  autoGenerate: boolean;
  customIncludedTypes: string;
  projectRoots: string;
  enableMinimization: boolean;
  minimizationThreshold: number;
  minimizationDepthThreshold: number;
  apiEnabled: boolean;
  apiPort: number;
  enableCommandOutput: boolean;
  allowedWritePaths: string;
}
