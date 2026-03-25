<script setup lang="ts">
import { ref, onMounted, onUnmounted, reactive, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import AppSettingsModal from "./components/AppSettingsModal.vue";
import AppLayoutGenerateControl from "./components/layout/AppLayoutGenerateControl.vue";
import AppLayoutHeader from "./components/layout/AppLayoutHeader.vue";
import AppLayoutInputSection from "./components/layout/AppLayoutInputSection.vue";
import AppLayoutResultsSection from "./components/layout/AppLayoutResultsSection.vue";
import AppLayoutVersionInfo from "./components/layout/AppLayoutVersionInfo.vue";
import ContextWorker from "./workers/context.worker.ts?worker";
import { normalizePath, isBinaryFile, copyToClipboard } from "./utils";
import pkg from "../package.json";

const version = pkg.version;

const outputContext = ref("");
const fileNodes = ref<{path: string, content: string, abs_path: string, originId?: string}[]>([]);
const isDragging = ref(false);
const isInvalidDrag = ref(false);
let isDraggingInvalidFiles = false; // 非响应式，仅用于跨事件保持状态
const isLoading = ref(false);
const filesList = ref<{id: string, path: string}[]>([]);
const isSettingsOpen = ref(false);
const userPrompt = ref("");
const isEditing = ref(false);
const sidebarWidth = ref(320);
const isResizingSidebar = ref(false);
let sidebarResizeStartX = 0;
let sidebarResizeStartWidth = 0;
let removeFileDebounceTimer: ReturnType<typeof setTimeout> | null = null;
let analysisDebounceTimer: ReturnType<typeof setTimeout> | null = null;
const SIDEBAR_MIN_WIDTH = 260;
const SIDEBAR_MAX_WIDTH = 520;

// Worker 实例：字符串拼接全部在独立线程执行，主线程不卡
const contextWorker = new ContextWorker();
const currentRequestId = ref(0);

contextWorker.onmessage = (e: MessageEvent<{requestId?: number, content: string}>) => {
    const { requestId, content } = e.data;
    // 校验 requestId，如果已被中断或有新任务，则忽略
    if (requestId !== undefined && requestId !== currentRequestId.value) {
        return;
    }
    outputContext.value = content;
    isLoading.value = false;
};
contextWorker.onerror = (e) => {
    console.error('Context worker error:', e);
    isLoading.value = false;
};

const totalCharacters = computed(() => {
  return outputContext.value ? outputContext.value.length : 0;
});

const appConfig = reactive({
  maxDepth: 2,
  includedTypes: ["vue", "ts", "tsx", "js", "py", "json", "css", "scss"],
  ignoreExts: ".git, node_modules, dist, target, build, .vscode, .idea, .next, .nuxt, .output, .vercel, .github, __pycache__, .venv, bin, obj, *.lock, *.log, *.tmp, *.temp, *.png, *.jpg, *.jpeg, *.gif, *.svg, *.ico, *.webp, *.mp4, *.avi, *.mkv, *.mov, *.webm, *.mp3, *.wav, *.flac, *.aac, *.ogg, *.zip, *.tar, *.gz, *.7z, *.rar, *.exe, *.dll, *.so, *.dylib",
  ignoreDeepParse: "package.json, tsconfig.json, vite.config.ts, tauri.conf.json, README.md, Cargo.toml, go.mod, pom.xml, .env, *.test.ts, *.spec.ts",
  customPrompt: "",
  generateTree: true,
  highlightPrimaryFiles: true,
  autoGenerate: true,
  customIncludedTypes: "",
  projectRoots: "",
  enableMinimization: true,
  minimizationThreshold: 8000,
  minimizationDepthThreshold: 1,
});

watch(appConfig, (newVal) => {
  localStorage.setItem("appConfig", JSON.stringify(newVal));
}, { deep: true });

const finalIncludedTypes = computed(() => {
  const customIncludedTypes = appConfig.customIncludedTypes
    .split(/[,\n]/)
    .map(s => s.trim())
    .filter(s => s.length > 0)
    .map(s => s.startsWith('.') ? s.substring(1) : s);
  return Array.from(new Set([...appConfig.includedTypes, ...customIncludedTypes]));
});

// 影响解析核心逻辑的配置项（排除 UI 逻辑项）
const analysisSettingsTrigger = computed(() => {
  return JSON.stringify({
    maxDepth: appConfig.maxDepth,
    includedTypes: [...appConfig.includedTypes].sort(),
    customIncludedTypes: finalIncludedTypes.value.filter(type => !appConfig.includedTypes.includes(type)),
    ignoreExts: appConfig.ignoreExts,
    ignoreDeepParse: appConfig.ignoreDeepParse,
    projectRoots: appConfig.projectRoots
      .split(/[,\n\r]/)
      .map(s => s.trim())
      .filter(s => s.length > 0),
    enableMinimization: appConfig.enableMinimization,
    minimizationThreshold: appConfig.minimizationThreshold,
    minimizationDepthThreshold: appConfig.minimizationDepthThreshold,
  });
});

// 影响前端最终拼接结果的配置项
const uiFormattingTrigger = computed(() => {
  return JSON.stringify({
    customPrompt: appConfig.customPrompt,
    generateTree: appConfig.generateTree,
    highlightPrimaryFiles: appConfig.highlightPrimaryFiles
  });
});

function scheduleProcessPaths(delay = 0, reason: 'analysis' | 'remove' = 'analysis') {
  const timer = reason === 'analysis' ? analysisDebounceTimer : removeFileDebounceTimer;
  if (timer) {
    clearTimeout(timer);
  }

  const nextTimer = setTimeout(() => {
    processPaths(filesList.value.map(f => f.path));
  }, delay);

  if (reason === 'analysis') {
    analysisDebounceTimer = nextTimer;
    return;
  }
  removeFileDebounceTimer = nextTimer;
}

// 当解析参数改变时，防抖后重新触发解析
watch(analysisSettingsTrigger, () => {
  if (appConfig.autoGenerate && filesList.value.length > 0) {
    scheduleProcessPaths(300);
  }
});

// 当格式化参数改变时，仅重新生成输出文本
watch(uiFormattingTrigger, () => {
  if (appConfig.autoGenerate && fileNodes.value.length > 0) {
    updateOutputContext();
  }
});

let unlistenDragDrop: () => void;
let lastHighlightedNode: HTMLElement | null = null;

onMounted(async () => {
  // Load saved config
  const savedConfig = localStorage.getItem("appConfig");
  if (savedConfig) {
    try {
      const parsed = JSON.parse(savedConfig);
      Object.assign(appConfig, parsed);
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }

  const savedSidebarWidth = localStorage.getItem("sidebarWidth");
  if (savedSidebarWidth) {
    const parsedWidth = Number(savedSidebarWidth);
    if (!Number.isNaN(parsedWidth)) {
      sidebarWidth.value = Math.min(SIDEBAR_MAX_WIDTH, Math.max(SIDEBAR_MIN_WIDTH, parsedWidth));
    }
  }

  unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event: any) => {
    const { type, position, paths } = event.payload;

    const dpi = window.devicePixelRatio;
    const lx = position ? position.x / dpi : 0;
    const ly = position ? position.y / dpi : 0;

    if (type === 'over' || type === 'enter') {
      if (position) {
        const el = document.elementFromPoint(lx, ly);
        const dropZone = el?.closest('[data-drop-zone="main"]');
        const nodeZone = el?.closest('[data-drop-path]') as HTMLElement | null;

        isDragging.value = !!dropZone;
        
        // 实时检查是否包含二进制文件 (在 enter 阶段就捕获 paths 并锁定状态)
        if (type === 'enter' && paths && paths.length > 0) {
            isDraggingInvalidFiles = (paths as string[]).some(p => isBinaryFile(p));
        }

        isInvalidDrag.value = isDragging.value && isDraggingInvalidFiles;

        if (nodeZone !== lastHighlightedNode) {
          if (lastHighlightedNode) {
              lastHighlightedNode.classList.remove('drop-node-hover');
              lastHighlightedNode.classList.remove('drop-node-hover-invalid');
          }
          if (nodeZone) {
              nodeZone.classList.add(isInvalidDrag.value ? 'drop-node-hover-invalid' : 'drop-node-hover');
          }
          lastHighlightedNode = nodeZone;
        }
      }
    } else if (type === 'leave') {
      isDragging.value = false;
      isInvalidDrag.value = false;
      isDraggingInvalidFiles = false;
      if (lastHighlightedNode) {
        lastHighlightedNode.classList.remove('drop-node-hover');
        lastHighlightedNode.classList.remove('drop-node-hover-invalid');
        lastHighlightedNode = null;
      }
    } else if (type === 'drop') {
      isDragging.value = false;
      isInvalidDrag.value = false;
      isDraggingInvalidFiles = false;
      if (lastHighlightedNode) {
        lastHighlightedNode.classList.remove('drop-node-hover');
        lastHighlightedNode.classList.remove('drop-node-hover-invalid');
      }
      
      const el = position ? document.elementFromPoint(lx, ly) : null;
      const dropZone = el?.closest('[data-drop-zone="main"]');
      const nodeZone = el?.closest('[data-drop-path]') as HTMLElement | null;

      if (paths && paths.length > 0) {
        // 过滤非文本文件（二进制文件直接拦截，不进入列表）
        const validPaths = (paths as string[]).filter(p => !isBinaryFile(p));
        const hasBlocked = (paths as string[]).length > validPaths.length;

        if (hasBlocked && validPaths.length === 0) {
            // 已自动阻止非文本文件
            return;
        } else if (hasBlocked) {
            console.warn('一些非文本文件已被自动跳过');
        }

        if (dropZone) {
          const newItems = validPaths.map((p: string) => ({
            id: Math.random().toString(36).substring(2, 11),
            path: p
          }));
          filesList.value = newItems;
          if (appConfig.autoGenerate) processPaths(newItems.map((i: {path: string}) => i.path));
        } else if (nodeZone) {
          const destDir = nodeZone.dataset.dropPath;
          if (destDir) handleTreeUploadFiles(validPaths, destDir);
        }
      }
      lastHighlightedNode = null;
    }
  });
});

onUnmounted(() => {
  if (removeFileDebounceTimer) {
    clearTimeout(removeFileDebounceTimer);
  }
  if (analysisDebounceTimer) {
    clearTimeout(analysisDebounceTimer);
  }
  stopResizeSidebar();
  if (unlistenDragDrop) unlistenDragDrop();
});

async function processPaths(paths: string[]) {
  if (paths.length === 0) return;
  if (removeFileDebounceTimer) {
    clearTimeout(removeFileDebounceTimer);
    removeFileDebounceTimer = null;
  }
  if (analysisDebounceTimer) {
    clearTimeout(analysisDebounceTimer);
    analysisDebounceTimer = null;
  }
  const requestId = ++currentRequestId.value;
  isLoading.value = true;
  try {
    const result = await invoke<Array<{path: string, content: string, abs_path: string}>>("generate_context", {
      paths: paths,
      maxDepth: appConfig.maxDepth,
      generateTree: appConfig.generateTree,
      ignoreExts: appConfig.ignoreExts,
      ignoreDeepParse: appConfig.ignoreDeepParse,
      includedTypes: finalIncludedTypes.value,
      projectRoots: appConfig.projectRoots,
      enableMinimization: appConfig.enableMinimization,
      minimizationThreshold: appConfig.minimizationThreshold,
      minimizationDepthThreshold: appConfig.minimizationDepthThreshold,
    });
    
    // 中断检查
    if (requestId !== currentRequestId.value) return;

    fileNodes.value = result.map(node => {
        const nNodeAbs = normalizePath(node.abs_path);
        
        const origin = filesList.value.find(f => {
            const nf = normalizePath(f.path);
            return nNodeAbs === nf || nNodeAbs.startsWith(nf + '/');
        });
        
        return { ...node, originId: origin?.id };
    });
    
    // 将耗时的字符串拼接调度到 Worker 线程，主线程立即释放
    updateOutputContext(requestId);
  } catch (error) {
    if (requestId !== currentRequestId.value) return;
    console.error("Failed to generate context:", error);
    outputContext.value = `Error: ${error}`;
    isLoading.value = false;
  }
}

function stopProcessing() {
    currentRequestId.value++;
    isLoading.value = false;
    invoke("abort_generate_context").catch(e => console.error("Failed to abort:", e));
}

function updateOutputContext(requestId?: number) {
    if (fileNodes.value.length === 0) {
        outputContext.value = "";
        return;
    }
    const rid = requestId ?? ++currentRequestId.value;
    // 将所有数据发给 Worker，主线程立即返回，字符串拼接在后台线程执行
    // isLoading 的 false 由 worker.onmessage 回调负责关闭
    contextWorker.postMessage({
        requestId: rid,
        fileNodes: fileNodes.value.map(n => ({ path: n.path, content: n.content, isPrimary: Boolean(n.originId) })),
        generateTree: appConfig.generateTree,
        highlightPrimaryFiles: appConfig.highlightPrimaryFiles,
        customPrompt: appConfig.customPrompt,
        userPrompt: userPrompt.value,
        longContextThreshold: appConfig.minimizationThreshold,
    });
}

function handleNodeDelete(fullPath: string, _absPath?: string, originIds?: string[]) {
    fileNodes.value = fileNodes.value.filter(node => 
        !(node.path === fullPath || node.path.startsWith(fullPath + '/'))
    );
    if (originIds && originIds.length > 0) {
        filesList.value = filesList.value.filter(f => !originIds.includes(f.id));
    }
    updateOutputContext();
}

async function handleTreeUploadFiles(files: string[], destDir: string) {
    try {
        isLoading.value = true;
        const newPaths = await invoke<string[]>("copy_files_to_dest", {
            sources: files,
            destDir: destDir
        });
        if (newPaths && newPaths.length > 0) {
            for (const p of newPaths) {
                if (!filesList.value.find(f => f.path === p)) {
                    filesList.value.push({
                        id: Math.random().toString(36).substring(2, 11),
                        path: p
                    });
                }
            }
            await processPaths(filesList.value.map(f => f.path));
            // isLoading 由 worker.onmessage 关闭，这里不再 finally 关闭
        } else {
            isLoading.value = false;
        }
    } catch (e) {
        console.error("Upload failed:", e);
        alert(`上传失败: ${e}`);
        isLoading.value = false;
    }
}

async function copyToClipboardAll() {
  await copyToClipboard(outputContext.value);
}

function toggleEdit() {
    isEditing.value = !isEditing.value;
}

async function triggerFileInput() {
    const selected = await open({
        multiple: true,
        directory: false,
    });
    if (selected && Array.isArray(selected)) {
        filesList.value = (selected as string[]).map((p: string) => ({ id: Math.random().toString(36).substring(2, 11), path: p }));
        if (appConfig.autoGenerate) processPaths(selected as string[]);
    } else if (selected && typeof selected === 'string') {
        filesList.value = [{ id: Math.random().toString(36).substring(2, 11), path: selected as string }];
        if (appConfig.autoGenerate) processPaths([selected as string]);
    }
}

async function triggerDirInput() {
    const selected = await open({
        multiple: true,
        directory: true,
    });
    if (selected && Array.isArray(selected)) {
        filesList.value = (selected as string[]).map((p: string) => ({ id: Math.random().toString(36).substring(2, 11), path: p }));
        if (appConfig.autoGenerate) processPaths(selected as string[]);
    } else if (selected && typeof selected === 'string') {
        filesList.value = [{ id: Math.random().toString(36).substring(2, 11), path: selected as string }];
        if (appConfig.autoGenerate) processPaths([selected as string]);
    }
}

function removeFile(index: number) {
    filesList.value.splice(index, 1);
    if (!appConfig.autoGenerate) return;

    if (filesList.value.length === 0) {
        fileNodes.value = [];
        updateOutputContext();
        return;
    }

    scheduleProcessPaths(300, 'remove');
}

function startResizeSidebar(e: MouseEvent) {
  isResizingSidebar.value = true;
  sidebarResizeStartX = e.clientX;
  sidebarResizeStartWidth = sidebarWidth.value;
  document.body.classList.add('sidebar-resizing');
  window.addEventListener('mousemove', handleResizeSidebar);
  window.addEventListener('mouseup', stopResizeSidebar);
}

function handleResizeSidebar(e: MouseEvent) {
  if (!isResizingSidebar.value) return;
  const nextWidth = Math.min(SIDEBAR_MAX_WIDTH, Math.max(SIDEBAR_MIN_WIDTH, sidebarResizeStartWidth + e.clientX - sidebarResizeStartX));
  sidebarWidth.value = nextWidth;
}

function stopResizeSidebar() {
  if (!isResizingSidebar.value) return;
  isResizingSidebar.value = false;
  document.body.classList.remove('sidebar-resizing');
  window.removeEventListener('mousemove', handleResizeSidebar);
  window.removeEventListener('mouseup', stopResizeSidebar);
  localStorage.setItem("sidebarWidth", String(sidebarWidth.value));
}
</script>

<template>
  <main class="h-screen flex flex-col items-center p-6 selection:bg-app-primary/10 relative overflow-y-auto">
    <AppLayoutHeader @open-settings="isSettingsOpen = true" />

    <AppLayoutInputSection
      v-model:userPrompt="userPrompt"
      :isDragging="isDragging"
      :isInvalidDrag="isInvalidDrag"
      :filesList="filesList"
      @open-file="triggerFileInput"
      @open-dir="triggerDirInput"
      @remove-file="removeFile"
    />

    <AppLayoutGenerateControl
      :isLoading="isLoading"
      :hasFiles="filesList.length > 0"
      @trigger="isLoading ? stopProcessing() : processPaths(filesList.map(f => f.path))"
    />

    <AppLayoutResultsSection
      v-model:outputContext="outputContext"
      :fileNodes="fileNodes"
      :sidebarWidth="sidebarWidth"
      :totalCharacters="totalCharacters"
      :isEditing="isEditing"
      :isLoading="isLoading"
      :enableMinimization="appConfig.enableMinimization"
      @delete="(fp, ap, ids) => handleNodeDelete(fp, ap, ids)"
      @toggle-edit="toggleEdit"
      @copy-output="copyToClipboardAll"
      @start-resize-sidebar="startResizeSidebar"
      @upload-files="handleTreeUploadFiles"
    />

    <AppSettingsModal 
      v-model:show="isSettingsOpen" 
      :settings="appConfig"
      @update:settings="val => Object.assign(appConfig, val)"
    />

    <AppLayoutVersionInfo :version="version" />
  </main>
</template>

<style>
/* 全局外观优化 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
  cursor: default;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--color-app-border);
  border-radius: 10px;
  cursor: default;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--color-app-text-mute);
}

.custom-scrollbar-h {
  scroll-behavior: smooth;
}

.custom-scrollbar-h::-webkit-scrollbar {
  height: 4px;
}

.custom-scrollbar-h::-webkit-scrollbar-track {
  background: transparent;
  margin: 0 40px;
}

.custom-scrollbar-h::-webkit-scrollbar-thumb {
  background: var(--color-app-border);
}

/* 拖拽反馈样式 */
.drop-node-hover {
  background-color: var(--color-app-primary-light) !important;
  box-shadow: 0 0 0 1px var(--color-app-primary) inset;
  border-radius: 12px;
}

.drop-node-hover-invalid {
  background-color: rgba(244, 63, 94, 0.08) !important;
  box-shadow: 0 0 0 1px var(--color-app-rose) inset;
  border-radius: 12px;
}

body.sidebar-resizing {
  cursor: col-resize;
  user-select: none;
}

.shadow-output-text {
  box-sizing: border-box;
  tab-size: 4;
  white-space: pre-wrap;
  overflow-wrap: break-word;
  word-break: break-word;
}
</style>
