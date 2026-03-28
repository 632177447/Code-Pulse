import { ApiValidationError, CodeService } from '../core/code.service';

function createServiceErrorResponse(c: any, error: unknown) {
  if (error instanceof ApiValidationError) {
    return c.json({
      error: {
        message: error.message,
        details: error.details
      }
    }, 400);
  }

  throw error;
}

/**
 * 清空 Rust 解析缓存
 */
export const handleDeleteCache = async (c: any) => {
  return c.json(await CodeService.clearCache(), 200);
};

/**
 * 中断当前上下文生成
 */
export const handleAbortContext = async (c: any) => {
  return c.json(await CodeService.abortContext(), 200);
};

/**
 * 生成上下文 (支持 json 或 text 格式)
 */
export const handleGenerateContext = async (c: any) => {
  try {
    const data = c.req.valid('json');
    
    // 决定返回格式:
    // 1. 如果请求体明确指定了 format，优先使用
    // 2. 如果体未指定，但客户端 Accept Header 包含 text/plain，则按 text 返回
    let format = data.format;
    if (!format || format === 'json') {
      const acceptHeader = c.req.header('Accept');
      if (acceptHeader && acceptHeader.includes('text/plain')) {
        format = 'text';
      }
    }

    if (format === 'text') {
      return c.json(await CodeService.getContextText(data), 200);
    } else {
      return c.json(await CodeService.getContext(data), 200);
    }
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};

/**
 * 直接渲染已有节点为上下文文本
 */
export const handleRenderContext = async (c: any) => {
  const data = c.req.valid('json');
  return c.json(await CodeService.renderContextText(data), 200);
};

/**
 * 获取依赖大纲
 */
export const handleGenerateOutline = async (c: any) => {
  try {
    const data = c.req.valid('json');
    return c.json(await CodeService.getOutline(data), 200);
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};
