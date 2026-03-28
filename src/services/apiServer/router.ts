import { OpenAPIHono, createRoute } from '@hono/zod-openapi';
import { swaggerUI } from '@hono/swagger-ui';
import {
  handleAbortContext,
  handleDeleteCache,
  handleGenerateContext,
  handleGenerateOutline,
  handleGetInfo,
  handleHealthCheck,
  handleRenderContext
} from './handlers';
import {
  GenerateContextBodySchema,
  GenerateOutlineBodySchema,
  RenderContextBodySchema,
  ContextResponseSchema,
  OutlineResponseSchema,
  HealthResponseSchema,
  InfoResponseSchema,
  SimpleStatusResponseSchema,
  ErrorResponseSchema
} from './schemas/code.schema';

// 初始化 OpenAPIHono 应用
const app = new OpenAPIHono();

// 基础中间件：日志记录
app.use('*', async (c, next) => {
  const start = Date.now();
  await next();
  const ms = Date.now() - start;
  console.log(`[ApiServer] ${c.req.method} ${c.req.url} - ${c.res.status} (${ms}ms)`);
});

// --- 系统及通用 API ---

app.openapi(
  createRoute({
    method: 'get',
    path: '/api/v1/health',
    summary: '健康检查',
    description: '检查 API 服务及其依赖系统的当前运行状态',
    responses: {
      200: { content: { 'application/json': { schema: HealthResponseSchema } }, description: '服务运行正常' }
    }
  }),
  handleHealthCheck
);

app.openapi(
  createRoute({
    method: 'get',
    path: '/api/v1/info',
    summary: '获取服务信息',
    description: '获取服务元数据、版本信息及可用路由列表',
    responses: {
      200: { content: { 'application/json': { schema: InfoResponseSchema } }, description: '返回服务详情' }
    }
  }),
  handleGetInfo
);

app.openapi(
  createRoute({
    method: 'delete',
    path: '/api/v1/cache',
    summary: '清空缓存',
    description: '清除 Rust 后端解析引擎的所有内部缓存',
    responses: {
      200: { content: { 'application/json': { schema: SimpleStatusResponseSchema } }, description: '缓存已清空' }
    }
  }),
  handleDeleteCache
);

// --- Context 资源路由 ---

app.openapi(
  createRoute({
    method: 'post',
    path: '/api/v1/contexts/generate',
    summary: '生成上下文',
    description: '基于文件路径解析源码并生成上下文数据。支持返回原始 JSON 或格式化文本。',
    request: { body: { content: { 'application/json': { schema: GenerateContextBodySchema } }, required: true } },
    responses: {
      200: { content: { 'application/json': { schema: ContextResponseSchema } }, description: '解析成功' },
      400: { content: { 'application/json': { schema: ErrorResponseSchema } }, description: '参数错误' }
    }
  }),
  handleGenerateContext
);

app.openapi(
  createRoute({
    method: 'post',
    path: '/api/v1/contexts/abort',
    summary: '中断生成',
    description: '停止当前正在进行的上下文生成任务（全局生效）',
    responses: {
      200: { content: { 'application/json': { schema: SimpleStatusResponseSchema } }, description: '已发起中断信号' }
    }
  }),
  handleAbortContext
);

app.openapi(
  createRoute({
    method: 'post',
    path: '/api/v1/contexts/render',
    summary: '渲染已有节点',
    description: '将内存中已存在的文件节点数据快速转换为格式化文本。该操作纯前端执行，不涉及磁盘读取。',
    request: { body: { content: { 'application/json': { schema: RenderContextBodySchema } }, required: true } },
    responses: {
      200: { content: { 'application/json': { schema: ContextResponseSchema } }, description: '渲染成功' }
    }
  }),
  handleRenderContext
);

// --- Outline 资源路由 ---

app.openapi(
  createRoute({
    method: 'post',
    path: '/api/v1/outlines/generate',
    summary: '生成依赖大纲',
    description: '仅解析文件间的依赖树结构（不包含文件具体内容），用于生成轻量级的逻辑结构图。',
    request: { body: { content: { 'application/json': { schema: GenerateOutlineBodySchema } }, required: true } },
    responses: {
      200: { content: { 'application/json': { schema: OutlineResponseSchema } }, description: '大纲生成成功' }
    }
  }),
  handleGenerateOutline
);

// --- 文档及交互界面 ---

// 提供 OpenAPI JSON 规格
app.doc('/api/v1/doc', {
  openapi: '3.0.0',
  info: {
    title: 'CodePulse API',
    version: '1.0.0',
    description: '文件依赖解析和上下文管理服务接口文档'
  }
});

// 提供 Swagger UI 界面
app.get('/api/v1/ui', swaggerUI({ url: '/api/v1/doc' }));

// 处理 404
app.notFound((c) => c.json({ error: 'Not Found' }, 404));

// 处理错误
app.onError((err, c) => {
  console.error('[ApiServer] Error:', err);
  return c.json({ error: 'Internal Server Error', details: String(err) }, 500);
});

export { app as apiRouter };
