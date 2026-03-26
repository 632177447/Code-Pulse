import { z } from 'zod';

// 通用查询参数
export const CommonQuerySchema = z.object({
  path: z.string().optional(),
});

// 未来可扩展更多 Schema...
