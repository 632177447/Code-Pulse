import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { ApiRouter } from './router';
import { handleGetOutline, handleGetContext } from './handlers';
import type { ApiRequest } from './types';

class ApiServerManager {
  private router: ApiRouter;
  private unlistenRequestEvent: UnlistenFn | null = null;
  private isRunning: boolean = false;

  constructor() {
    this.router = new ApiRouter();
    this.registerRoutes();
  }

  // 集中维护所有暴露外部的路由路径
  private registerRoutes(): void {
    this.router.get('/api/outline', handleGetOutline);
    this.router.get('/api/context', handleGetContext);
  }

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
    // 强制先清理并关闭，以确保如果端口变更能正常重启
    if (this.isRunning) {
      await this.stop();
    }

    try {
      // 开启后端 HTTPServer 监听指定端口
      await invoke('start_api_server', { port });
      
      // 接受由 Rust 传递过来的真实 HTTP HttpRequest 封装数据
      this.unlistenRequestEvent = await listen<ApiRequest>('api-request', async (event) => {
        const req = event.payload;
        const response = await this.router.handle(req);
        
        // 带着匹配的请求 Id 把处理结果原路丢回 Rust，供它回复给该请求
        await invoke('api_response', { id: req.id, response });
      });

      this.isRunning = true;
      console.log(`[ApiServerManager] Service successfully bound to port ${port}`);
    } catch (error) {
      console.error('[ApiServerManager] Failed to start service on rust backend:', error);
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
