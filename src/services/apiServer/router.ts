import { Hono } from 'hono';
import {
  handleGetContextText,
  handlePostContextText,
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

// 注册前端增强业务路由（未在 Rust 段定义的逻辑）
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
