export interface SettingItem {
  id: string;
  type: 'slider' | 'input' | 'textarea' | 'switch' | 'radio' | 'checkbox';
  inputType?: 'text' | 'number' | 'password';
  label?: string;
  description?: string;
  placeholder?: string;
  min?: number;
  max?: number;
  step?: number;
  rows?: number;
  layout?: 'grid' | 'flex';
  columns?: number;
  options?: Array<{ label: string; value: any }>;
  visible?: (settings: any) => boolean;
}

export interface SettingGroup {
  id: string;
  title: string;
  colorClass?: string;
  color?: string;
  items: SettingItem[];
}

export interface FileListItem {
  id: string;
  path: string;
}

export interface FileNode {
  path: string;
  content: string;
  abs_path: string;
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
  autoGenerate: boolean;
  customIncludedTypes: string;
  projectRoots: string;
  enableMinimization: boolean;
  minimizationThreshold: number;
  minimizationDepthThreshold: number;
}
