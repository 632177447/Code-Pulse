use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// 缓存条目，包含原始内容以及可选的压缩结果
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// 原始文件内容，用于依赖提取和未压缩输出
    pub raw_content: String,
    /// 已缓存的压缩结果，按需生成，避免重复压缩同一文件
    pub minimized_content: Option<String>,
}

/// 文件解析缓存管理，用于持久化存储或在同一会话中避免对同一文件的重复解析
/// 键由 (规范化绝对路径, 修改时间) 组成，确保内容的一致性
pub struct FileCache {
    /// 对应的键为 (abs_path, mtime)，值为 CacheEntry
    data: Mutex<HashMap<(PathBuf, SystemTime), CacheEntry>>,
}

impl FileCache {
    /// 创建一个新的空缓存实例
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    /// 根据绝对路径及文件修改时间获取缓存的条目
    ///
    /// # 参数
    /// * `abs_path` - 文件的规范化绝对路径，作为缓存键的主体部分
    /// * `mtime` - 文件的系统修改时间，用于检测内容变更
    pub fn get(&self, abs_path: &Path, mtime: SystemTime) -> Option<CacheEntry> {
        let key = (abs_path.to_path_buf(), mtime);
        self.data.lock().ok()?.get(&key).cloned()
    }

    /// 向缓存中更新或插入一个解析后的文件条目
    ///
    /// # 参数
    /// * `abs_path` - 绝对路径
    /// * `mtime` - 文件的系统修改时间
    /// * `raw_content` - 原始文件内容
    pub fn set(&self, abs_path: PathBuf, mtime: SystemTime, raw_content: String) {
        if let Ok(mut data) = self.data.lock() {
            data.insert((abs_path, mtime), CacheEntry { raw_content, minimized_content: None });
        }
    }

    /// 为已缓存文件补写压缩结果
    pub fn set_minimized(&self, abs_path: &Path, mtime: SystemTime, minimized_content: String) {
        let key = (abs_path.to_path_buf(), mtime);
        if let Ok(mut data) = self.data.lock() {
            if let Some(entry) = data.get_mut(&key) {
                entry.minimized_content = Some(minimized_content);
            }
        }
    }

    /// 清空所有已保存的解析记录
    pub fn clear(&self) {
        if let Ok(mut data) = self.data.lock() {
            data.clear();
        }
    }

    /// 返回当前缓存中的条目数量
    pub fn size(&self) -> usize {
        self.data.lock().map(|data| data.len()).unwrap_or(0)
    }
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}
