import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { apiRouter } from './router';
import type { ApiRequest, ApiResponse } from './types';

class ApiServerManager {
  private unlistenRequestEvent: UnlistenFn | null = null;
  private isRunning: boolean = false;

  constructor() {}

  // 根据当前最新的全局设置，决定如何重启或停止 API Server 
  public async syncState(apiEnabled: boolean, apiPort: number): Promise<void> {
    if (apiEnabled) {
      await this.start(apiPort);
    } else {
      await this.stop();
    }
  }

  // 触发 Rust 端的监听器启动事件，并挂载 Webview 请求响应处理器
  public async start(port: number): Promise<void> {
    if (this.isRunning) {
      await this.stop();
    }

    try {
      await invoke('start_api_server', { port });
      
      this.unlistenRequestEvent = await listen<ApiRequest>('api-request', async (event) => {
        const req = event.payload;
        const response = await this.dispatchToHono(req);
        
        await invoke('api_response', { id: req.id, response });
      });

      this.isRunning = true;
      console.log(`[ApiServerManager] Service successfully bound to port ${port}`);
    } catch (error) {
      console.error('[ApiServerManager] Failed to start service on rust backend:', error);
    }
  }

  /**
   * 将自定义的 ApiRequest 适配并转发给 Hono 处理
   */
  private async dispatchToHono(req: ApiRequest): Promise<ApiResponse> {
    try {
      // 构造标准 Request 对象。使用 http://localhost 作为基准域
      const url = req.url.startsWith('http') ? req.url : `http://localhost${req.url}`;
      
      const standardReq = new Request(url, {
        method: req.method,
        headers: req.headers,
        body: (req.method !== 'GET' && req.method !== 'HEAD') ? req.body : undefined
      });

      // 调用 Hono 应用
      const res = await apiRouter.fetch(standardReq);

      // 将标准 Response 转换回 ApiResponse
      const body = await res.text();
      const headers: Record<string, string> = {};
      res.headers.forEach((value, key) => {
        headers[key] = value;
      });

      return {
        status: res.status,
        headers,
        body
      };
    } catch (error) {
      console.error('[ApiServerManager] Adapter Error:', error);
      return {
        status: 500,
        body: JSON.stringify({ error: 'Adapter Error', details: String(error) })
      };
    }
  }

  // 解除 Webview 事件绑定并告知 Rust 关闭服务监听
  public async stop(): Promise<void> {
    if (!this.isRunning) return;

    try {
      if (this.unlistenRequestEvent) {
        this.unlistenRequestEvent();
        this.unlistenRequestEvent = null;
      }
      
      await invoke('stop_api_server');
      this.isRunning = false;
      console.log('[ApiServerManager] Service stopped successfully');
    } catch (error) {
      console.error('[ApiServerManager] Failed to shut down service on rust backend:', error);
    }
  }
}

export const apiServerManager = new ApiServerManager();
