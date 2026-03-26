import type { ApiRequest, ApiResponse } from '../types';

export const handleGetOutline = async (_req: ApiRequest): Promise<ApiResponse> => {
  // 占位逻辑，由于具体业务逻辑将由你后续提供，目前先保证返回正确的状态结构即可
  return {
    status: 200,
    body: JSON.stringify({ message: "Outline generation will be implemented soon." })
  };
};

export const handleGetContext = async (_req: ApiRequest): Promise<ApiResponse> => {
  // 占位逻辑，由于具体业务逻辑将由你后续提供，目前先保证返回正确的状态结构即可
  return {
    status: 200,
    body: JSON.stringify({ message: "Full context generation will be implemented soon." })
  };
};
