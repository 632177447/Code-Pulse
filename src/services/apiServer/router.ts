import type { ApiRequest, ApiResponse, RouteHandler } from './types';

export class ApiRouter {
  private routes: Map<string, RouteHandler> = new Map();

  // 注册 GET 路由
  public get(path: string, handler: RouteHandler): void {
    this.routes.set(`GET:${path}`, handler);
  }

  // 注册 POST 路由
  public post(path: string, handler: RouteHandler): void {
    this.routes.set(`POST:${path}`, handler);
  }

  // 统一分发和处理到达的 API 请求
  public async handle(req: ApiRequest): Promise<ApiResponse> {
    try {
      // 剔除 URL 中的 Query 字符串，以确保能够精确匹配到注册的路径
      const path = req.url.split('?')[0];
      const routeKey = `${req.method.toUpperCase()}:${path}`;
      const handler = this.routes.get(routeKey);

      if (!handler) {
        return {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ error: 'Not Found' })
        };
      }

      const response = await handler(req);
      
      // 补充缺失的 Headers，默认视为 JSON 结构
      if (!response.headers) {
        response.headers = { 'Content-Type': 'application/json' };
      }
      
      return response;
    } catch (error) {
      console.error('[ApiRouter] Error handling request:', error);
      return {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ error: 'Internal Server Error', details: String(error) })
      };
    }
  }
}
