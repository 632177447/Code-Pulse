import type { Context } from 'hono';
import { CommonQuerySchema } from '../schemas/common.schema';
import { CodeService } from '../core/code.service';

/**
 * 获取代码大纲
 */
export const handleGetOutline = async (c: Context) => {
  const query = CommonQuerySchema.safeParse(c.req.query());
  
  if (!query.success) {
    return c.json({ 
      error: 'Invalid query parameters', 
      details: query.error.format() 
    }, 400);
  }

  const result = await CodeService.getOutline(query.data.path);
  return c.json(result);
};

/**
 * 获取代码上下文
 */
export const handleGetContext = async (c: Context) => {
  const result = await CodeService.getFullContext();
  return c.json(result);
};
