import type { Context } from 'hono';
import type { ZodError, ZodType } from 'zod';
import { ApiValidationError, CodeService } from '../core/code.service';
import {
  GenerateContextBodySchema,
  GenerateOutlineBodySchema,
  RenderContextBodySchema
} from '../schemas/code.schema';

function createSchemaErrorResponse(c: Context, error: ZodError) {
  return c.json({
    error: {
      message: 'Invalid request parameters',
      details: error.format()
    }
  }, 400);
}

function createServiceErrorResponse(c: Context, error: unknown) {
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

async function parseJsonBody<T>(c: Context, schema: ZodType<T>) {
  try {
    const body = await c.req.json();
    const result = schema.safeParse(body);

    if (!result.success) {
      return { response: createSchemaErrorResponse(c, result.error) };
    }

    return { data: result.data };
  } catch (error) {
    return {
      response: c.json({
        error: {
          message: 'Invalid JSON body',
          details: String(error)
        }
      }, 400)
    };
  }
}

/**
 * 清空 Rust 解析缓存
 */
export const handleDeleteCache = async (c: Context) => {
  return c.json(await CodeService.clearCache());
};

/**
 * 中断当前上下文生成
 */
export const handleAbortContext = async (c: Context) => {
  return c.json(await CodeService.abortContext());
};

/**
 * 生成上下文 (支持 json 或 text 格式)
 */
export const handleGenerateContext = async (c: Context) => {
  const parsedBody = await parseJsonBody(c, GenerateContextBodySchema);

  if (parsedBody.response) {
    return parsedBody.response;
  }

  try {
    const data = parsedBody.data!;
    
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
      return c.json(await CodeService.getContextText(data));
    } else {
      return c.json(await CodeService.getContext(data));
    }
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};

/**
 * 直接渲染已有节点为上下文文本
 */
export const handleRenderContext = async (c: Context) => {
  const parsedBody = await parseJsonBody(c, RenderContextBodySchema);

  if (parsedBody.response) {
    return parsedBody.response;
  }

  return c.json(await CodeService.renderContextText(parsedBody.data!));
};

/**
 * 获取依赖大纲
 */
export const handleGenerateOutline = async (c: Context) => {
  const parsedBody = await parseJsonBody(c, GenerateOutlineBodySchema);

  if (parsedBody.response) {
    return parsedBody.response;
  }

  try {
    return c.json(await CodeService.getOutline(parsedBody.data!));
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};
