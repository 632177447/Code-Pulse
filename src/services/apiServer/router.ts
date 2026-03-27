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

// 注册前端桥接路由
app.get('/api/frontend/health', handleHealthCheck);
app.get('/api/frontend/info', handleGetInfo);
app.delete('/api/frontend/cache', handleDeleteCache);
app.get('/api/frontend/context', handleGetContext);
app.post('/api/frontend/context', handlePostContext);
app.post('/api/frontend/context/abort', handleAbortContext);
app.get('/api/frontend/context/text', handleGetContextText);
app.post('/api/frontend/context/text', handlePostContextText);
app.post('/api/frontend/context/render', handleRenderContextText);
app.get('/api/frontend/outline', handleGetOutline);
app.post('/api/frontend/outline', handlePostOutline);

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
