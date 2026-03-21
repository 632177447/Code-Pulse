export interface WorkerInput {
  fileNodes: { path: string; content: string }[];
  generateTree: boolean;
  customPrompt: string;
  userPrompt: string;
  longContextThreshold: number;
}

// 所有耗时的字符串拼接全部在 Worker 线程执行，主线程不受影响
self.onmessage = (e: MessageEvent<WorkerInput>) => {
  const { fileNodes, generateTree, customPrompt, userPrompt, longContextThreshold } = e.data;

  if (fileNodes.length === 0) {
    self.postMessage('');
    return;
  }

  let finalContext = '';

  if (generateTree) {
    const paths = fileNodes.map(n => n.path);
    let tree = '========================================\n[FILE TREE]\n========================================\n.\n';
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
    finalContext += tree + '\n';
  }

  if (customPrompt.trim()) {
    finalContext += '========================================\n';
    finalContext += '[SYSTEM SETTINGS]\n';
    finalContext += '========================================\n';
    finalContext += customPrompt.trim() + '\n\n';
  }

  const PENDING_USER_PROMPT = userPrompt.trim();
  // 使用数组 join 代替逐次 += 以减少中间字符串对象的生成
  const blocksContent = fileNodes.map(n => n.content).join('\n\n');

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

  self.postMessage(finalContext);
};
