import { CodeService } from '../core/code.service';

/**
 * 前端引擎健康检查
 */
export const handleHealthCheck = async (c: any) => {
  return c.json(CodeService.getHealth(), 200);
};

/**
 * 获取前端引擎服务信息
 */
export const handleGetInfo = async (c: any) => {
  return c.json(CodeService.getInfo(), 200);
};
