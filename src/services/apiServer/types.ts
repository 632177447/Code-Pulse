export interface ApiRequest {
  id: string; // 用于关联并匹配具体的请求和响应
  url: string;
  method: string;
  headers: Record<string, string>;
  query: Record<string, string>;
  body?: string;
}

export interface ApiResponse {
  status: number;
  headers?: Record<string, string>;
  body: string;
}

export type RouteHandler = (req: ApiRequest) => Promise<ApiResponse> | ApiResponse;
