/// 所有支持解析并包含在分析结果中的文件扩展名
pub const ALL_SUPPORTED_EXTS: &[&str] = &[
    "js", "mjs", "jsx", "ts", "tsx", "vue", "svelte", "py", "rs", "go",
    "java", "kt", "c", "cpp", "h", "hpp", "cs", "php", "rb", "css", "scss", "less"
];

/// JavaScript/TypeScript 及其相关框架（用于依赖分析）
pub const JS_TS_FAMILY: &[&str] = &["js", "mjs", "jsx", "ts", "tsx", "vue", "svelte"];

/// 使用 C 风格注释 (//, /* */) 的扩展名
pub const C_STYLE_COMMENT_EXTS: &[&str] = &[
    "js", "mjs", "jsx", "ts", "tsx", "rs", "go", "java", "kt", "c", "cpp", "h", "hpp", "cs", "php", "css", "scss", "less"
];

/// 使用 # 号注释的扩展名 (Python, Ruby)
pub const HASH_STYLE_COMMENT_EXTS: &[&str] = &["py", "rb"];

/// 混合注释风格 (HTML + C) 的扩展名 (Vue, Svelte)
pub const MIXED_STYLE_COMMENT_EXTS: &[&str] = &["vue", "svelte"];

/// C/C++ 家族（用于路径解析）
pub const C_CPP_FAMILY: &[&str] = &["cpp", "c", "h", "hpp"];

/// Java/Kotlin 家族 (用于路径解析)
pub const JAVA_KT_FAMILY: &[&str] = &["java", "kt"];

/// 样式文件系列 (用于风格定义与解析)
pub const STYLE_FAMILY: &[&str] = &["css", "scss", "less"];

/// HTML 关联解析扩展名
pub const HTML_RESOLVE_EXTS: &[&str] = &["js", "css", "html"];

/// Markdown 关联解析扩展名 (多用于文档链接校验)
pub const MD_RESOLVE_EXTS: &[&str] = &["md", "png", "jpg", "jpeg", "svg"];

/// 项目根目录标识文件
pub const PROJECT_ROOT_MARKERS: &[&str] = &[
    "package.json", "Cargo.toml", ".git", "go.mod", "go.work",
    "pyproject.toml", "requirements.txt", "pom.xml", 
    "build.gradle", "composer.json", "Gemfile", "Makefile",
    "pnpm-workspace.yaml", "lerna.json", "nx.json", "deno.json",
    "tsconfig.json", "jsconfig.json"
];

/// 默认忽略的文件/目录模式
pub const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    "node_modules", ".git", "dist", "target", "build", ".vscode", ".idea", 
    ".next", ".nuxt", ".output", ".vercel", ".github", 
    "__pycache__", ".venv", ".pytest_cache", ".gradle", ".m2", "bin", "obj",
    "*.lock", "*-lock.json", "package-lock.json", "yarn.lock", "pnpm-lock.yaml",
    ".jpg", ".jpeg", ".png", ".gif", ".svg", ".ico", ".webp",
    ".mp4", ".avi", ".mkv", ".mov", ".webm",
    ".mp3", ".wav", ".flac", ".aac", ".ogg",
    ".zip", ".tar", ".gz", ".7z", ".rar",
    ".exe", ".dll", ".so", ".dylib",
    ".log", ".tmp", ".temp", ".swp", ".DS_Store"
];

/// 已知组件库的 kebab-case 前缀（自动引入扫描时跳过这些标签）
pub const COMPONENT_LIB_PREFIXES: &[&str] = &[
    "el-",    // Element Plus
    "n-",     // Naive UI
    "a-",     // Ant Design Vue
    "van-",   // Vant
    "q-",     // Quasar
    "nut-",   // NutUI
    "wd-",    // Wot Design
    "t-",     // TDesign
    "v-",     // Vuetify
    "md-",    // Material Design
    "b-",     // Bootstrap Vue
    "arco-",  // ArcoDesign
    "vxe-",   // VXE-Table
    "dp-",    // DatePicker
    "naive-", // Naive UI alias
    "arco-",  // ArcoDesign
    "vxe-",   // VXE-Table
];

/// Rust 标准库与内置 Crate
pub const RS_STD_LIBS: &[&str] = &["std", "core", "alloc", "proc_macro", "test", "panic", "unwind"];

/// Python 常用内置模块
pub const PY_STD_LIBS: &[&str] = &[
    "os", "sys", "time", "pathlib", "json", "math", "re", "datetime", "itertools", "collections", 
    "functools", "typing", "enum", "threading", "multiprocessing", "subprocess", "shutil", "urllib", 
    "http", "unittest", "logging", "argparse", "pickle", "uuid", "abc", "contextlib", "glob"
];

/// Go 常用内置模块
pub const GO_STD_LIBS: &[&str] = &[
    "fmt", "os", "io", "time", "net", "http", "sync", "reflect", "runtime", "encoding", 
    "context", "bytes", "math", "regexp", "container", "errors", "flag", "log", "sort", "strings"
];

/// Java/Kotlin 系统包前缀
pub const JAVA_STD_LIBS: &[&str] = &["java", "javax", "sun", "oracle", "kotlin", "android"];

/// Node.js 内置模块
pub const NODE_STD_LIBS: &[&str] = &[
    "fs", "path", "http", "https", "crypto", "os", "stream", "util", "events", "child_process", 
    "cluster", "dns", "net", "readline", "url", "vm", "zlib", "v8", "perf_hooks", "worker_threads"
];
