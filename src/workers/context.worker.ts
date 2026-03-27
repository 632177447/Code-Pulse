import { formatContextContent, type ContextRenderableNode } from '../services/apiServer/core/contextFormatter';
import type { ContextFormatOptions } from '../services/apiServer/types';

export interface WorkerInput extends ContextFormatOptions {
  requestId?: number;
  fileNodes: ContextRenderableNode[];
}

// 所有耗时的字符串拼接全部在 Worker 线程执行，主线程不受影响
self.onmessage = (e: MessageEvent<WorkerInput>) => {
  const { requestId, fileNodes, ...options } = e.data;

  self.postMessage({
    requestId,
    content: formatContextContent(fileNodes, options)
  });
};
