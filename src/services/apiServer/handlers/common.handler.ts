import type { Context } from 'hono';

/**
 * 健康检查
 */
export const handleHealthCheck = async (c: Context) => {
  return c.json({ status: 'ok', timestamp: new Date().toISOString() });
};

/**
 * 获取服务信息
 */
export const handleGetInfo = async (c: Context) => {
  return c.json({
    name: 'CodePulse API Service',
    version: '1.0.0',
    description: 'Local code analysis and context service'
  });
};
