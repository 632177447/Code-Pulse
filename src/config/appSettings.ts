import type { AppConfig, SettingGroup } from "../types";

export const APP_CONFIG_STORAGE_KEY = "appConfig";

export const DEFAULT_APP_CONFIG: AppConfig = {
  maxDepth: 2,
  includedTypes: ["vue", "ts", "tsx", "js", "py", "json", "css", "scss"],
  ignoreExts: ".git, node_modules, dist, target, build, .vscode, .idea, .next, .nuxt, .output, .vercel, .github, __pycache__, .venv, bin, obj, *.lock, *.log, *.tmp, *.temp, *.png, *.jpg, *.jpeg, *.gif, *.svg, *.ico, *.webp, *.mp4, *.avi, *.mkv, *.mov, *.webm, *.mp3, *.wav, *.flac, *.aac, *.ogg, *.zip, *.tar, *.gz, *.7z, *.rar, *.exe, *.dll, *.so, *.dylib",
  ignoreDeepParse: "package.json, tsconfig.json, vite.config.ts, tauri.conf.json, README.md, Cargo.toml, go.mod, pom.xml, .env, *.test.ts, *.spec.ts",
  customPrompt: "",
  generateTree: true,
  generateRelationshipText: true,
  highlightPrimaryFiles: true,
  autoGenerate: true,
  customIncludedTypes: "",
  projectRoots: "",
  enableMinimization: true,
  minimizationThreshold: 8000,
  minimizationDepthThreshold: 1,
};

export const createDefaultAppConfig = (): AppConfig => {
  return {
    ...DEFAULT_APP_CONFIG,
    includedTypes: [...DEFAULT_APP_CONFIG.includedTypes]
  };
};

export const APP_SETTINGS_GROUPS: SettingGroup[] = [
  {
    id: "analysis",
    title: "解析范围",
    colorClass: "text-blue-400",
    items: [
      {
        id: "maxDepth",
        type: "slider",
        label: "递归解析深度",
        description: "设置文件解析依赖扫描的层级数。层级越高，包含的相关文件越多。",
        min: 0,
        max: 10
      },
      {
        id: "includedTypes",
        type: "checkbox",
        label: "目标文件类型",
        description: "选择需要提取依赖的目标代码文件格式。",
        layout: "grid",
        columns: 6,
        options: [
          { label: ".vue", value: "vue" },
          { label: ".ts", value: "ts" },
          { label: ".tsx", value: "tsx" },
          { label: ".js", value: "js" },
          { label: ".mjs", value: "mjs" },
          { label: ".rs", value: "rs" },
          { label: ".py", value: "py" },
          { label: ".go", value: "go" },
          { label: ".java", value: "java" },
          { label: ".kt", value: "kt" },
          { label: ".cpp", value: "cpp" },
          { label: ".c", value: "c" },
          { label: ".h", value: "h" },
          { label: ".cs", value: "cs" },
          { label: ".php", value: "php" },
          { label: ".rb", value: "rb" },
          { label: ".json", value: "json" },
          { label: ".md", value: "md" },
          { label: ".html", value: "html" },
          { label: ".css", value: "css" },
          { label: ".scss", value: "scss" },
          { label: ".less", value: "less" }
        ]
      },
      {
        id: "customIncludedTypes",
        type: "input",
        label: "自定义后缀名",
        description: "在此输入其他自定义后缀名，按逗号分隔（如: sh, yaml, xml）。",
        placeholder: "sh, yaml, xml"
      },
      {
        id: "projectRoots",
        type: "textarea",
        label: "自定义项目根目录",
        description: "为空表示自动匹配。可以用逗号或换行分隔多个路径。若手动指定的根目录都不匹配，则回退到自动匹配逻辑。",
        placeholder: "D:/Projects/my-app, D:/Projects/common-lib",
        rows: 2
      },
    ]
  },
  {
    id: "filtering",
    title: "过滤与优化",
    colorClass: "text-purple-400",
    items: [
      {
        id: "enableMinimization",
        type: "switch",
        label: "上下文压缩",
        description: "自动移除间接引用文件的函数实现，仅保留定义，大幅节省上下文空间。"
      },
      {
        id: "minimizationThreshold",
        type: "input",
        inputType: "number",
        label: "压缩触发阈值",
        description: "当生成内容达到该阈值(字符数)时，自动触发深度压缩以节省上下文空间。",
        step: 1000,
        visible: (settings: any) => settings.enableMinimization === true
      },
      {
        id: "minimizationDepthThreshold",
        type: "slider",
        label: "压缩触发层级",
        description: "只有大于等于该层级的依赖文件才会被压缩。默认为 1。",
        min: 0,
        max: 5,
        visible: (settings: any) => settings.enableMinimization === true
      },
      {
        id: "ignoreExts",
        type: "textarea",
        label: "全局忽略项",
        description: "忽略指定的后缀或目录（支持 * 通配符）。通常用于排除依赖库或构建产物。",
        placeholder: ".git, node_modules, dist, target, build",
        rows: 3
      },
      {
        id: "ignoreDeepParse",
        type: "textarea",
        label: "跳过深层解析",
        description: "匹配到的文件将包含内容，但不继续追踪其内部依赖（如配置文件、测试文件）。",
        placeholder: "package.json, tsconfig.json, vite.config.ts, README.md, *.test.ts",
        rows: 3
      },
    ]
  },
  {
    id: "runtime",
    title: "交互与输出",
    colorClass: "text-emerald-400",
    items: [
      {
        id: "autoGenerate",
        type: "switch",
        label: "自动解析",
        description: "开启后，在上传、移除文件或更改相关设置时会自动更新输出结果。"
      },
      {
        id: "generateTree",
        type: "switch",
        label: "包含文件树视图",
        description: "在输出结果的开头生成项目的层级目录树。"
      },
      {
        id: "generateRelationshipText",
        type: "switch",
        label: "包含文件关系说明",
        description: "在输出结果头部生成一段便于 AI 理解的文件依赖关系摘要。"
      },
      {
        id: "highlightPrimaryFiles",
        type: "switch",
        label: "重点文件提示",
        description: "为你直接添加的主要文件插入额外关注标记，提醒 AI 优先理解这些文件。"
      },
      {
        id: "customPrompt",
        type: "textarea",
        label: "全局引导提示词 (Prompt)",
        description: "在生成的上下文头部注入自定义指令，引导 AI 更好地理解代码。",
        placeholder: "例如：你是资深 Rust 工程师，正在为我审查以下代码...",
        rows: 3
      }
    ]
  }
];
