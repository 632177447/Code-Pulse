import type { Context } from 'hono';
import { CodeService } from '../core/code.service';

/**
 * 前端引擎健康检查
 */
export const handleHealthCheck = async (c: Context) => {
  return c.json(CodeService.getHealth());
};

/**
 * 获取前端引擎服务信息
 */
export const handleGetInfo = async (c: Context) => {
  return c.json(CodeService.getInfo());
};
