// 工具函数
export const BASE_PATH_SYMBOL = '<BASE_PATH>';


/**
 * 标准化路径，处理正反斜杠、大小写转换、前缀消除等
 * @param p 路径字符串
 * @returns 标准化后的路径
 */
export const normalizePath = (p: string) => {
  return p.replace(/\\/g, '/')
          .toLowerCase()
          .trim()
          .replace(/^\\\\?\\/, '')
          .replace(/^\/\/\?\//, '')
          .replace(/\/+$/, '');
};

export const getDisplayBasePath = (paths: string[]) => {
  const normalizedPaths = paths
    .map(path => path.replace(/\\/g, '/').trim().replace(/\/+$/, ''))
    .filter(path => path.length > 0);

  if (normalizedPaths.length === 0) {
    return '';
  }

  const directorySegmentsList = normalizedPaths.map(path => path.split('/').filter(Boolean).slice(0, -1));
  let sharedCount = directorySegmentsList[0].length;

  for (let i = 1; i < directorySegmentsList.length; i++) {
    sharedCount = Math.min(sharedCount, directorySegmentsList[i].length);
    let cursor = 0;
    while (cursor < sharedCount && directorySegmentsList[0][cursor] === directorySegmentsList[i][cursor]) {
      cursor++;
    }
    sharedCount = cursor;
    if (sharedCount === 0) {
      break;
    }
  }

  if (sharedCount <= 1) {
    return '';
  }

  return directorySegmentsList[0].slice(0, sharedCount - 1).join('/') + '/';
};

export const stripDisplayBasePath = (path: string, basePath: string) => {
  if (!basePath || !path.startsWith(basePath)) {
    return path;
  }
  return path.slice(basePath.length);
};

export const replacePathWithSymbol = (path: string, basePath: string, symbol: string = BASE_PATH_SYMBOL) => {
  if (!basePath || !path.startsWith(basePath)) {
    return path;
  }
  return symbol + path.slice(basePath.length);
};


/**
 * 获取路径的父目录
 * @param p 路径字符串
 * @returns 父目录路径
 */
export const getDirname = (p: string) => {
  const lastSlash = Math.max(p.lastIndexOf('/'), p.lastIndexOf('\\'));
  return lastSlash > -1 ? p.substring(0, lastSlash) : p;
};

/**
 * 判断是否为二进制文件（非文本类型）
 * 基于常见后缀名进行识别
 * @param path 路径或文件名
 * @returns 是否为二进制文件
 */
export const isBinaryFile = (path: string) => {
  const binaryExts = [
    'png', 'jpg', 'jpeg', 'gif', 'svg', 'ico', 'webp',
    'mp4', 'avi', 'mkv', 'mov', 'webm',
    'mp3', 'wav', 'flac', 'aac', 'ogg',
    'zip', 'tar', 'gz', '7z', 'rar',
    'exe', 'dll', 'so', 'dylib',
    'bin', 'obj', 'o', 'a', 'lib', 'pdb', 'pyc', 'pyo', 'lock'
  ];
  // 匹配路径结尾的后缀名
  return binaryExts.some(b => path.toLowerCase().endsWith('.' + b));
};

/**
 * 复制文本到剪贴板
 * @param text 要复制的文本
 */
export const copyToClipboard = async (text: string) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
  } catch (e) {
    console.error('Failed to copy text:', e);
  }
};

/**
 * 处理横向滚动容器的滚轮事件（将纵向滚动转为横向）
 * @param e 滚轮事件
 * @param container 容器元素
 */
export const handleWheelHorizontal = (e: WheelEvent, container: HTMLElement | null) => {
  if (container) {
    e.preventDefault();
    container.scrollBy({
      left: e.deltaY,
      behavior: 'smooth'
    });
  }
};

export const createFileListItem = (path: string) => {
  return {
    id: Math.random().toString(36).substring(2, 11),
    path
  };
};

export const createFileList = (paths: string[]) => {
  return paths.map(path => createFileListItem(path));
};
