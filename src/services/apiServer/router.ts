import { Hono } from 'hono';
import {
  handleAbortContext,
  handleDeleteCache,
  handleGenerateContext,
  handleGenerateOutline,
  handleGetInfo,
  handleHealthCheck,
  handleRenderContext
} from './handlers';

// 初始化 Hono 应用
const app = new Hono();

// 基础中间件：日志记录（可选）
app.use('*', async (c, next) => {
  const start = Date.now();
  await next();
  const ms = Date.now() - start;
  console.log(`[ApiServer] ${c.req.method} ${c.req.url} - ${c.res.status} (${ms}ms)`);
});

// 系统及通用 API
app.get('/api/v1/health', handleHealthCheck);
app.get('/api/v1/info', handleGetInfo);
app.delete('/api/v1/cache', handleDeleteCache);

// Context (上下文) 资源路由
app.post('/api/v1/contexts/generate', handleGenerateContext);
app.post('/api/v1/contexts/abort', handleAbortContext);
app.post('/api/v1/contexts/render', handleRenderContext);

// Outline (依赖大纲) 资源路由
app.post('/api/v1/outlines/generate', handleGenerateOutline);

// 处理 404
app.notFound((c) => {
  return c.json({ error: 'Not Found' }, 404);
});

// 处理错误
app.onError((err, c) => {
  console.error('[ApiServer] Error:', err);
  return c.json({ error: 'Internal Server Error', details: String(err) }, 500);
});

export { app as apiRouter };
