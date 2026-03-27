import type { Context } from 'hono';
import type { ZodError, ZodType } from 'zod';
import { ApiValidationError, CodeService } from '../core/code.service';
import {
  ContextRequestBodySchema,
  ContextRequestQuerySchema,
  ContextTextRequestBodySchema,
  ContextTextRequestQuerySchema,
  RenderContextRequestBodySchema
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
 * 获取原始上下文节点
 */
export const handleGetContext = async (c: Context) => {
  const query = ContextRequestQuerySchema.safeParse(c.req.query());

  if (!query.success) {
    return createSchemaErrorResponse(c, query.error);
  }

  try {
    return c.json(await CodeService.getContext(query.data));
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};

/**
 * 通过请求体获取原始上下文节点
 */
export const handlePostContext = async (c: Context) => {
  const parsedBody = await parseJsonBody(c, ContextRequestBodySchema);

  if (parsedBody.response) {
    return parsedBody.response;
  }

  try {
    return c.json(await CodeService.getContext(parsedBody.data!));
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};

/**
 * 获取格式化后的完整上下文文本
 */
export const handleGetContextText = async (c: Context) => {
  const query = ContextTextRequestQuerySchema.safeParse(c.req.query());

  if (!query.success) {
    return createSchemaErrorResponse(c, query.error);
  }

  try {
    return c.json(await CodeService.getContextText(query.data));
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};

/**
 * 通过请求体获取格式化后的完整上下文文本
 */
export const handlePostContextText = async (c: Context) => {
  const parsedBody = await parseJsonBody(c, ContextTextRequestBodySchema);

  if (parsedBody.response) {
    return parsedBody.response;
  }

  try {
    return c.json(await CodeService.getContextText(parsedBody.data!));
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};

/**
 * 直接渲染已有节点为上下文文本
 */
export const handleRenderContextText = async (c: Context) => {
  const parsedBody = await parseJsonBody(c, RenderContextRequestBodySchema);

  if (parsedBody.response) {
    return parsedBody.response;
  }

  return c.json(await CodeService.renderContextText(parsedBody.data!));
};

/**
 * 获取依赖大纲
 */
export const handleGetOutline = async (c: Context) => {
  const query = ContextRequestQuerySchema.safeParse(c.req.query());

  if (!query.success) {
    return createSchemaErrorResponse(c, query.error);
  }

  try {
    return c.json(await CodeService.getOutline(query.data));
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};

/**
 * 通过请求体获取依赖大纲
 */
export const handlePostOutline = async (c: Context) => {
  const parsedBody = await parseJsonBody(c, ContextRequestBodySchema);

  if (parsedBody.response) {
    return parsedBody.response;
  }

  try {
    return c.json(await CodeService.getOutline(parsedBody.data!));
  } catch (error) {
    return createServiceErrorResponse(c, error);
  }
};
