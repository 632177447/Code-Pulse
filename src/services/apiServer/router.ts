import { Hono } from 'hono';
import {
  handleAbortContext,
  handleDeleteCache,
  handleGetContext,
  handleGetContextText,
  handleGetInfo,
  handleGetOutline,
  handleHealthCheck,
  handlePostContext,
  handlePostContextText,
  handlePostOutline,
  handleRenderContextText
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

// 基础功能路由
app.get('/api/health', handleHealthCheck);
app.get('/api/info', handleGetInfo);
app.delete('/api/cache', handleDeleteCache);

// 代码上下文相关路由
app.get('/api/context', handleGetContext);
app.post('/api/context', handlePostContext);
app.post('/api/context/abort', handleAbortContext);
app.get('/api/outline', handleGetOutline);
app.post('/api/outline', handlePostOutline);

// 前端增强业务路由 (处理格式化)
app.get('/api/context/text', handleGetContextText);
app.post('/api/context/text', handlePostContextText);
app.post('/api/context/render', handleRenderContextText);

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
