import type { FileNode } from '../../../types';
import { BASE_PATH_SYMBOL, getDisplayBasePath, normalizePath, replacePathWithSymbol } from '../../../utils';
import type { ContextFormatOptions } from '../types';

export interface ContextRenderableNode {
  path: string;
  absPath?: string;
  content: string;
  depth: number;
  dependencies: string[];
  isPrimary?: boolean;
}

function buildTreeText(paths: string[], basePath: string) {
  let tree = '========================================\n[FILE TREE]\n========================================\n';

  if (basePath) {
    tree += `[BASE PATH]: ${BASE_PATH_SYMBOL} maps to ${basePath}\n`;
  }

  tree += '.\n';
  const sortedPaths = [...paths].sort();
  let prevComponents: string[] = [];

  for (const path of sortedPaths) {
    const components = path.split('/');
    let i = 0;
    while (i < components.length && i < prevComponents.length && components[i] === prevComponents[i]) {
      i++;
    }
    while (i < components.length) {
      const indent = '│   '.repeat(i);
      tree += `${indent}├── ${components[i]}\n`;
      i++;
    }
    prevComponents = components;
  }

  return tree + '\n';
}

function buildRelationshipText(fileNodes: ContextRenderableNode[]) {
  if (fileNodes.length === 0) {
    return '';
  }

  const visiblePaths = new Set(fileNodes.map(node => node.path));
  const incomingMap = new Map<string, Set<string>>();

  fileNodes.forEach(node => {
    incomingMap.set(node.path, new Set<string>());
  });

  fileNodes.forEach(node => {
    node.dependencies.forEach(dependency => {
      if (!visiblePaths.has(dependency)) {
        return;
      }
      if (!incomingMap.has(dependency)) {
        incomingMap.set(dependency, new Set<string>());
      }
      incomingMap.get(dependency)?.add(node.path);
    });
  });

  const sortedNodes = [...fileNodes].sort((a, b) => {
    if (a.depth !== b.depth) {
      return a.depth - b.depth;
    }
    return a.path.localeCompare(b.path);
  });
  const primaryFiles = sortedNodes.filter(node => node.isPrimary).map(node => node.path);
  const maxDepth = sortedNodes.reduce((max, node) => Math.max(max, node.depth), 0);

  const lines = [
    '========================================',
    '[FILE RELATIONSHIPS]',
    '========================================',
    `Summary: total files ${sortedNodes.length}; primary files ${primaryFiles.length > 0 ? primaryFiles.join(', ') : 'none'}; max dependency layer ${maxDepth}.`,
    '',
    'Direct dependency map:',
    ...sortedNodes.map(node => {
      const tags = [`layer ${node.depth}`];
      if (node.isPrimary) {
        tags.unshift('primary');
      }
      const dependencies = node.dependencies.length > 0 ? node.dependencies.join(', ') : 'none';
      const incoming = Array.from(incomingMap.get(node.path) ?? []).sort();
      return `- ${node.path} [${tags.join(', ')}] | depends on: ${dependencies} | used by: ${incoming.length > 0 ? incoming.join(', ') : 'none'}`;
    }),
    ''
  ];

  return lines.join('\n') + '\n';
}

function buildPrimaryTagContent(content: string, isPrimary: boolean | undefined) {
  if (!isPrimary) {
    return content;
  }

  const headerSeparator = '========================================\n';
  const firstSplit = content.indexOf(headerSeparator);
  const secondSplit = content.indexOf(headerSeparator, firstSplit + headerSeparator.length);

  let pathInfo = '';
  let layerInfo = '';
  let actualContent = content;

  if (firstSplit !== -1 && secondSplit !== -1) {
    const headerBlock = content.slice(firstSplit, secondSplit);
    const lines = headerBlock.split('\n');
    pathInfo = lines.find(line => line.startsWith('[FILE PATH]:')) || '';
    layerInfo = lines.find(line => line.includes('(Dependency Layer:')) || '';
    actualContent = content.slice(secondSplit + headerSeparator.length).trimStart();
  }

  return [
    '========================================',
    '[PRIMARY FILE]',
    'This file was directly selected by the user. Use it as the primary reference for this task.',
    pathInfo,
    layerInfo,
    '========================================',
    actualContent
  ].filter(Boolean).join('\n');
}

function addLineNumbers(content: string) {
  if (!content) {
    return content;
  }

  const hasTrailingNewline = content.endsWith('\n');
  const contentWithoutTrailingNewline = hasTrailingNewline ? content.slice(0, -1) : content;
  const lines = contentWithoutTrailingNewline.split('\n');

  if (lines.length === 1 && lines[0] === '') {
    return content;
  }

  const lineNumberWidth = Math.max(String(lines.length).length, 4);
  const numberedContent = lines
    .map((line, index) => `L${String(index + 1).padStart(lineNumberWidth, '0')}|${line}`)
    .join('\n');

  return hasTrailingNewline ? `${numberedContent}\n` : numberedContent;
}

function buildLineNumberedContent(content: string) {
  const contentStartMarker = '[CONTENT START]\n';
  const contentEndMarker = '\n[CONTENT END]';
  const contentStartIndex = content.indexOf(contentStartMarker);
  const contentEndIndex = content.lastIndexOf(contentEndMarker);

  if (contentStartIndex === -1 || contentEndIndex === -1) {
    return content;
  }

  const actualContentStartIndex = contentStartIndex + contentStartMarker.length;
  const rawContent = content.slice(actualContentStartIndex, contentEndIndex);

  if (!rawContent) {
    return content;
  }

  return [
    content.slice(0, actualContentStartIndex),
    '[LINE NUMBERS]: 1-based, format "L0001|code"\n',
    addLineNumbers(rawContent),
    content.slice(contentEndIndex)
  ].join('');
}

function buildCommandOutputPrompt() {
  return [
    '========================================',
    '[OUTPUT FORMAT: PulseCommand JSON]',
    '========================================',
    'You MUST provide your solution as a JSON array of commands (PulseCommand) inside a ```json_commands``` markdown block to enable automated execution.',
    '',
    '## Command Data Structure',
    '[',
    '  { "action": "write", "path": "absolute/path/to/file.ts", "content": "..." },',
    '  { "action": "patch", "path": "absolute/path/to/file.ts", "search": "old snippet", "replace": "new snippet" },',
    '  { "action": "delete", "path": "absolute/path/to/file.ts" },',
    '  { "action": "move", "path": "absolute/path/to/old.ts", "target": "absolute/path/to/new.ts" }',
    ']',
    '',
    'Ensure your JSON is valid and the "search" text in "patch" actions is unique and exists in the target file.',
    'CRITICAL WARNING: All "path" and "target" fields MUST be strictly ABSOLUTE PATHS (e.g., D:/project/src/file.ts).',
    'DO NOT use relative paths starting with src/ or ./ under any circumstances. Refer to [BASE PATH], [PROJECT ROOT], and [FILE PATH] for the correct absolute prefix.',
    ''
  ].join('\n');
}

function isSelectedPath(absPath: string, selectedPaths: string[]) {
  const normalizedAbsPath = normalizePath(absPath);

  return selectedPaths.some(selectedPath => {
    const normalizedSelectedPath = normalizePath(selectedPath);
    return normalizedAbsPath === normalizedSelectedPath || normalizedAbsPath.startsWith(`${normalizedSelectedPath}/`);
  });
}

export function createRenderableContextNodes(fileNodes: FileNode[], selectedPaths: string[]) {
  return fileNodes.map(node => ({
    path: node.path,
    absPath: node.absPath,
    content: node.content,
    depth: node.depth,
    dependencies: [...node.dependencies],
    isPrimary: Boolean(node.originId) || isSelectedPath(node.absPath || node.path, selectedPaths)
  }));
}

export function formatContextContent(fileNodes: ContextRenderableNode[], options: ContextFormatOptions) {
  if (fileNodes.length === 0) {
    return '';
  }

  const {
    customPrompt,
    generateRelationshipText,
    generateTree,
    highlightPrimaryFiles,
    generateLineNumbers,
    longContextThreshold,
    optimizePathDisplay,
    omitFileBlocks,
    userPrompt,
    enableCommandOutput
  } = options;

  const basePath = getDisplayBasePath(fileNodes.map(node => node.path));
  const absBasePath = getDisplayBasePath(fileNodes.map(node => node.absPath || node.path));

  const displayFileNodes: ContextRenderableNode[] = (optimizePathDisplay && basePath)
    ? fileNodes.map(node => ({
        ...node,
        path: replacePathWithSymbol(node.path, basePath),
        dependencies: node.dependencies.map(dependency => replacePathWithSymbol(dependency, basePath))
      }))
    : fileNodes;

  let finalContext = '';
  
  // 自动从文件节点推导实际的项目根目录，避免直接显示设置中无关的路径
  const actualRoots = new Set<string>();
  fileNodes.forEach(node => {
    if (node.absPath && node.path) {
      const normAbs = node.absPath.replace(/\\/g, '/');
      const normRel = node.path.replace(/\\/g, '/');
      if (normAbs.endsWith(normRel)) {
        let root = normAbs.substring(0, normAbs.length - normRel.length);
        root = root.replace(/^\/\/(\?|\\)\//, '').replace(/\/+$/, '');
        if (root) actualRoots.add(root);
      }
    }
  });

  const projectPathInfo = Array.from(actualRoots).sort().join(', ');
  if (projectPathInfo) {
    finalContext += '========================================\n';
    finalContext += '[PROJECT ROOT]\n';
    finalContext += '========================================\n';
    finalContext += projectPathInfo + '\n\n';
  }

  if (generateTree) {
    finalContext += buildTreeText(displayFileNodes.map(node => node.path), basePath);
  } else if (basePath) {
    finalContext += '========================================\n';
    finalContext += '[BASE PATH]\n';
    finalContext += '========================================\n';
    finalContext += `All paths starting with "${BASE_PATH_SYMBOL}" are relative to: ${basePath}\n\n`;
  }

  if (generateRelationshipText) {
    finalContext += buildRelationshipText(displayFileNodes);
  }

  if (customPrompt.trim()) {
    finalContext += '========================================\n';
    finalContext += '[SYSTEM SETTINGS]\n';
    finalContext += '========================================\n';
    finalContext += customPrompt.trim() + '\n\n';
  }

  if (enableCommandOutput) {
    finalContext += buildCommandOutputPrompt() + '\n';
  }

  const pendingUserPrompt = userPrompt.trim();
  
  if (omitFileBlocks) {
    if (pendingUserPrompt) {
      finalContext += '========================================\n';
      finalContext += '[USER REQUIREMENTS]\n';
      finalContext += '========================================\n';
      finalContext += pendingUserPrompt + '\n\n';
    }
    return finalContext;
  }

  const blocksContent = fileNodes.map(node => {
    let displayContent = node.content;

    if (basePath) {
      const filePathPrefix = '[FILE PATH]: ';
      const prefixIndex = node.content.indexOf(filePathPrefix);
      if (prefixIndex !== -1) {
        const lineContentStart = prefixIndex + filePathPrefix.length;
        const lineEndIndex = node.content.indexOf('\n', lineContentStart);
        if (lineEndIndex !== -1) {
          const fullPath = node.content.slice(lineContentStart, lineEndIndex).trim();
          const displayPath = replacePathWithSymbol(fullPath, absBasePath || basePath);
          if (displayPath !== fullPath) {
            displayContent = node.content.slice(0, lineContentStart) + displayPath + node.content.slice(lineEndIndex);
          }
        }
      }
    }

    if (generateLineNumbers) {
      displayContent = buildLineNumberedContent(displayContent);
    }

    return buildPrimaryTagContent(displayContent, highlightPrimaryFiles ? node.isPrimary : false);
  }).join('\n\n');

  if (pendingUserPrompt && blocksContent.length <= longContextThreshold) {
    finalContext += '========================================\n';
    finalContext += '[USER REQUIREMENTS]\n';
    finalContext += '========================================\n';
    finalContext += pendingUserPrompt + '\n\n';
  }

  finalContext += blocksContent;

  if (pendingUserPrompt && blocksContent.length > longContextThreshold) {
    finalContext += '\n\n========================================\n';
    finalContext += '[USER REQUIREMENTS]\n';
    finalContext += '========================================\n';
    finalContext += pendingUserPrompt;
  }

  return finalContext;
}
