import type { AppConfig, SettingGroup } from "../types";
import { extractDefaultSettings } from "../components/common/SettingsModal/utils";

export const APP_CONFIG_STORAGE_KEY = "appConfig";

export const APP_SETTINGS_GROUPS: SettingGroup[] = [
  {
    id: "runtime",
    title: "内容输出",
    colorClass: "text-emerald-400",
    items: [
      {
        id: "autoGenerate",
        type: "switch",
        label: "自动生成",
        description: "当文件列表发生变化时，是否自动触发上下文生成。",
        defaultValue: true,
      },
      {
        id: "generateTree",
        type: "switch",
        label: "包含文件树视图",
        description: "在输出结果的开头生成项目的层级目录树。",
        defaultValue: true
      },
      {
        id: "generateRelationshipText",
        type: "switch",
        label: "包含文件关系说明",
        description: "在输出结果头部生成一段便于 AI 理解的文件依赖关系摘要。",
        defaultValue: true
      },
      {
        id: "highlightPrimaryFiles",
        type: "switch",
        label: "重点文件提示",
        description: "为你直接添加的主要文件插入额外关注标记，提醒 AI 优先理解这些文件。",
        defaultValue: true
      },
      {
        id: "generateLineNumbers",
        type: "switch",
        label: "为内容添加行序号",
        description: "为每个文件内容区的每一行添加 1-based 行序号，便于 AI 精确引用与理解。",
        defaultValue: false
      },
      {
        id: "optimizePathDisplay",
        type: "switch",
        label: "优化路径显示",
        description: "提取公共基础路径，在文件树顶部单独展示，并在完整输出结果中移除重复路径前缀。",
        defaultValue: false
      },
      {
        id: "omitFileBlocks",
        type: "switch",
        label: "仅输出结构大纲",
        description: "开启后，输出结果将不包含具体文件块及文件内容，仅保留文件树视图和依赖关系说明。",
        defaultValue: false
      },
      {
        id: "enableCommandOutput",
        type: "switch",
        label: "AI 自动化指令",
        description: "引导 AI 输出修复方案的 PulseCommand 指令，该指令可粘贴到自动化控制台中执行。",
        defaultValue: true
      },
      {
        id: "customPrompt",
        type: "textarea",
        label: "全局引导提示词",
        description: "在生成的上下文头部注入自定义指令，引导 AI 更好地理解代码。",
        placeholder: "例如：你是资深 Rust 工程师，正在为我审查以下代码...",
        rows: 3,
        defaultValue: ""
      }
    ]
  },
  {
    id: "analysis",
    title: "依赖解析",
    colorClass: "text-blue-400",
    items: [
      {
        id: "maxDepth",
        type: "slider",
        label: "递归解析深度",
        description: "设置文件解析依赖扫描的层级数。层级越高，包含的相关文件越多。",
        min: 0,
        max: 10,
        defaultValue: 2
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
        ],
        defaultValue: ["vue", "ts", "tsx", "js", "mjs", "rs", "py", "go", "java", "kt", "cpp", "c", "h", "cs", "php", "rb", "json", "md", "html", "css", "scss", "less"]
      },
      {
        id: "customIncludedTypes",
        type: "input",
        label: "自定义后缀名",
        description: "在此输入其他自定义后缀名，按逗号分隔（如: sh, yaml, xml），自定义后缀名的文件会输出到结果中，但不会解析依赖。",
        placeholder: "sh, yaml, xml",
        defaultValue: ""
      },
      {
        id: "projectRoots",
        type: "textarea",
        label: "自定义项目根目录",
        description: "为空表示自动匹配。可以用逗号或换行分隔多个路径。若手动指定的根目录都不匹配，则回退到自动匹配逻辑。",
        placeholder: "D:/Projects/my-app, D:/Projects/common-lib",
        rows: 2,
        defaultValue: ""
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
        description: "移除代码中的函数实现和注释，仅保留定义，大幅节省上下文空间。",
        defaultValue: true
      },
      {
        id: "minimizationThreshold",
        type: "input",
        inputType: "number",
        label: "压缩触发阈值",
        description: "当生成内容达到该阈值(字符数)时，自动触发深度压缩以节省上下文空间。",
        step: 1000,
        visible: (settings: any) => settings.enableMinimization === true,
        defaultValue: 8000
      },
      {
        id: "minimizationDepthThreshold",
        type: "slider",
        label: "压缩触发层级",
        description: "大于等于该层级的依赖文件将会被压缩。默认为 2。",
        min: 0,
        max: 5,
        visible: (settings: any) => settings.enableMinimization === true,
        defaultValue: 2
      },
      {
        id: "ignoreExts",
        type: "textarea",
        label: "全局忽略项",
        description: "忽略指定的后缀或目录（支持 * 通配符）。通常用于排除依赖库或构建产物。",
        placeholder: ".git, node_modules, dist, target, build",
        rows: 3,
        defaultValue: ".git, node_modules, dist, target, build, .vscode, .idea, .next, .nuxt, .output, .vercel, .github, __pycache__, .venv, bin, obj, *.lock, *.log, *.tmp, *.temp, *.png, *.jpg, *.jpeg, *.gif, *.svg, *.ico, *.webp, *.mp4, *.avi, *.mkv, *.mov, *.webm, *.mp3, *.wav, *.flac, *.aac, *.ogg, *.zip, *.tar, *.gz, *.7z, *.rar, *.exe, *.dll, *.so, *.dylib"
      },
      {
        id: "ignoreDeepParse",
        type: "textarea",
        label: "跳过深层解析",
        description: "匹配到的文件将包含内容，但不继续追踪其内部依赖（如配置文件、测试文件）。",
        placeholder: "package.json, tsconfig.json, vite.config.ts, README.md, *.test.ts",
        rows: 3,
        defaultValue: "package.json, tsconfig.json, vite.config.ts, tauri.conf.json, README.md, Cargo.toml, go.mod, pom.xml, .env, *.test.ts, *.spec.ts"
      },
    ]
  },
  {
    id: "apiService",
    title: "API 服务",
    colorClass: "text-orange-400",
    items: [
      {
        id: "apiEnabled",
        type: "switch",
        label: "启用核心 API 服务",
        description: "开启后，您可以通过 API 调用获取指定项目文件的依赖关系大纲或完整解析上下文。",
        defaultValue: false
      },
      {
        id: "apiPort",
        type: "input",
        inputType: "number",
        label: "API 监听端口",
        description: "运行服务绑定的本地端口号，建议不冲突的情况下使用。",
        placeholder: "13535",
        defaultValue: 13535,
        visible: (settings: any) => settings.apiEnabled === true
      },
      {
        id: "startApiAction",
        type: "button",
        label: "服务运行状态",
        description: "点击重启 API 服务，使最新的端口配置生效。",
        buttonText: "重启服务",
        visible: (settings: any) => settings.apiEnabled === true,
        onClick: async (settings: any) => {
          const port = settings?.apiPort || 13535;
          console.log("Restarting API Service at port:", port);
          const { apiServerManager } = await import("../services/apiServer");
          await apiServerManager.start(port);
        }
      },
      {
        id: "apiDocInfo",
        type: "info-text",
        label: "接口文档说明",
        text: "API 服务启动后，您可以通过访问 [API文档](http://localhost:{{apiPort}}/docs) 查看详细的 OpenAPI 规格文档和接口测试界面。",
        visible: (settings: any) => settings.apiEnabled === true
      }
    ]
  },
  {
    id: "security",
    title: "权限安全",
    colorClass: "text-red-400",
    items: [
      {
        id: "allowedWritePaths",
        type: "textarea",
        label: "自动化指令写入权限",
        description: "设置 PulseCommand 指令允许写入或修改文件的绝对路径白名单。若为空，所有自动化指令都将被拦截并报错。",
        placeholder: "D:/Projects/my-project",
        rows: 3,
        defaultValue: ""
      }
    ]
  },
];

export const DEFAULT_APP_CONFIG: AppConfig = extractDefaultSettings(APP_SETTINGS_GROUPS) as AppConfig;

export const createDefaultAppConfig = (): AppConfig => {
  return {
    ...DEFAULT_APP_CONFIG,
    includedTypes: [...DEFAULT_APP_CONFIG.includedTypes]
  };
};
