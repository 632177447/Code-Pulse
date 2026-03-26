import { BASE_PATH_SYMBOL, getDisplayBasePath, replacePathWithSymbol } from "../utils";

export interface WorkerInput {
  requestId?: number;
  fileNodes: { path: string; content: string; depth: number; dependencies: string[]; isPrimary?: boolean }[];
  generateTree: boolean;
  generateRelationshipText: boolean;
  highlightPrimaryFiles?: boolean;
  optimizePathDisplay?: boolean;
  customPrompt: string;
  userPrompt: string;
  longContextThreshold: number;
}

type WorkerFileNode = WorkerInput["fileNodes"][number];

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


function buildRelationshipText(fileNodes: WorkerInput["fileNodes"]) {
  if (fileNodes.length === 0) {
    return "";
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
    "========================================",
    "[FILE RELATIONSHIPS]",
    "========================================",
    `Summary: total files ${sortedNodes.length}; primary files ${primaryFiles.length > 0 ? primaryFiles.join(", ") : "none"}; max dependency layer ${maxDepth}.`,
    "",
    "Direct dependency map:",
    ...sortedNodes.map(node => {
      const tags = [`layer ${node.depth}`];
      if (node.isPrimary) {
        tags.unshift("primary");
      }
      const dependencies = node.dependencies.length > 0 ? node.dependencies.join(", ") : "none";
      const incoming = Array.from(incomingMap.get(node.path) ?? []).sort();
      return `- ${node.path} [${tags.join(", ")}] | depends on: ${dependencies} | used by: ${incoming.length > 0 ? incoming.join(", ") : "none"}`;
    }),
    ""
  ];

  return lines.join("\n") + "\n";
}

// 所有耗时的字符串拼接全部在 Worker 线程执行，主线程不受影响
self.onmessage = (e: MessageEvent<WorkerInput>) => {
  const { requestId, fileNodes, generateTree, generateRelationshipText, highlightPrimaryFiles, optimizePathDisplay, customPrompt, userPrompt, longContextThreshold } = e.data;

  if (fileNodes.length === 0) {
    self.postMessage({ requestId, content: '' });
    return;
  }

  const basePath = optimizePathDisplay ? getDisplayBasePath(fileNodes.map(node => node.path)) : "";
  const displayFileNodes: WorkerFileNode[] = basePath
    ? fileNodes.map(node => ({
        ...node,
        path: replacePathWithSymbol(node.path, basePath),
        dependencies: node.dependencies.map(dependency => replacePathWithSymbol(dependency, basePath))
      }))
    : fileNodes;

  let finalContext = '';

  if (generateTree) {
    finalContext += buildTreeText(displayFileNodes.map(node => node.path), basePath);
  } else if (basePath) {
    // 如果没有显示文件树, 则需要单独再添加一个基础路径的说明
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

  const PENDING_USER_PROMPT = userPrompt.trim();
  // 使用数组 join 代替逐次 += 以减少中间字符串对象的生成
  const blocksContent = fileNodes.map(n => {
    let displayContent = n.content;

    if (basePath) {
      const filePathPrefix = '[FILE PATH]: ';
      const prefixIndex = n.content.indexOf(filePathPrefix);
      if (prefixIndex !== -1) {
        const lineContentStart = prefixIndex + filePathPrefix.length;
        const lineEndIndex = n.content.indexOf('\n', lineContentStart);
        if (lineEndIndex !== -1) {
          const fullPath = n.content.slice(lineContentStart, lineEndIndex).trim();
          const displayPath = replacePathWithSymbol(fullPath, basePath);
          if (displayPath !== fullPath) {
            displayContent = n.content.slice(0, lineContentStart) + displayPath + n.content.slice(lineEndIndex);
          }
        }
      }
    }

    if (!highlightPrimaryFiles || !n.isPrimary) {
      return displayContent;
    }
    return [
      '========================================',
      '[PRIMARY FILE]',
      'This file was directly selected by the user. Use it as the primary reference for this task.',
      '========================================',
      displayContent
    ].join('\n');
  }).join('\n\n');

  if (PENDING_USER_PROMPT && blocksContent.length <= longContextThreshold) {
    finalContext += '========================================\n';
    finalContext += '[USER REQUIREMENTS]\n';
    finalContext += '========================================\n';
    finalContext += PENDING_USER_PROMPT + '\n\n';
  }

  finalContext += blocksContent;

  if (PENDING_USER_PROMPT && blocksContent.length > longContextThreshold) {
    finalContext += '\n\n========================================\n';
    finalContext += '[USER REQUIREMENTS]\n';
    finalContext += '========================================\n';
    finalContext += PENDING_USER_PROMPT;
  }
  self.postMessage({ requestId, content: finalContext });
};
