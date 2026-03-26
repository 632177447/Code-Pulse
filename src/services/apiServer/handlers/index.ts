import { z } from 'zod';
import type { Context } from 'hono';

// 定义请求校验 Schema (示例)
const QuerySchema = z.object({
  path: z.string().optional(),
});

export const handleGetOutline = async (c: Context) => {
  // 使用 Zod 校验查询参数
  const query = QuerySchema.safeParse(c.req.query());
  
  if (!query.success) {
    return c.json({ error: 'Invalid query parameters', details: query.error.format() }, 400);
  }

  return c.json({ 
    message: "Outline generation will be implemented soon.",
    params: query.data 
  });
};

export const handleGetContext = async (c: Context) => {
  return c.json({ 
    message: "Full context generation will be implemented soon." 
  });
};
