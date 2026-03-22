<script setup lang="ts">
import { ref, onMounted, onUnmounted, reactive, computed, nextTick, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import AppSettingsModal from "./components/AppSettingsModal.vue";
import DependencyTreeSidebar from "./components/DependencyTreeSidebar.vue";
import ContextWorker from "./workers/context.worker.ts?worker";
import { normalizePath, isBinaryFile, copyToClipboard, handleWheelHorizontal } from "./utils";

const outputContext = ref("");
const fileNodes = ref<{path: string, content: string, abs_path: string, originId?: string}[]>([]);
const isDragging = ref(false);
const isLoading = ref(false);
const filesList = ref<{id: string, path: string}[]>([]);
const isSettingsOpen = ref(false);
const userPrompt = ref("");
const isEditing = ref(false);
const outputAreaRef = ref<HTMLTextAreaElement | null>(null);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

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
  autoGenerate: true,
  customIncludedTypes: "",
  projectRoots: "",
  enableMinimization: true,
});

watch(appConfig, (newVal) => {
  localStorage.setItem("appConfig", JSON.stringify(newVal));
}, { deep: true });

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

        if (nodeZone !== lastHighlightedNode) {
          if (lastHighlightedNode) lastHighlightedNode.classList.remove('drop-node-hover');
          if (nodeZone) nodeZone.classList.add('drop-node-hover');
          lastHighlightedNode = nodeZone;
        }
      }
    } else if (type === 'leave') {
      isDragging.value = false;
      if (lastHighlightedNode) {
        lastHighlightedNode.classList.remove('drop-node-hover');
        lastHighlightedNode = null;
      }
    } else if (type === 'drop') {
      isDragging.value = false;
      if (lastHighlightedNode) {
        lastHighlightedNode.classList.remove('drop-node-hover');
      }
      
      const el = position ? document.elementFromPoint(lx, ly) : null;
      const dropZone = el?.closest('[data-drop-zone="main"]');
      const nodeZone = el?.closest('[data-drop-path]') as HTMLElement | null;

      if (paths && paths.length > 0) {
        // 过滤非文本文件（二进制文件直接拦截，不进入列表）
        const validPaths = (paths as string[]).filter(p => !isBinaryFile(p));
        const hasBlocked = (paths as string[]).length > validPaths.length;

        if (hasBlocked && validPaths.length === 0) {
            alert('禁止拖入非文本内容（如图片、视频、压缩包等二进制文件）');
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
  if (unlistenDragDrop) unlistenDragDrop();
});

async function processPaths(paths: string[]) {
  if (paths.length === 0) return;
  const requestId = ++currentRequestId.value;
  isLoading.value = true;
  try {
    const customTypesArray = appConfig.customIncludedTypes
      .split(/[,\n]/)
      .map(s => s.trim())
      .filter(s => s.length > 0)
      .map(s => s.startsWith('.') ? s.substring(1) : s);
    const finalIncludedTypes = Array.from(new Set([...appConfig.includedTypes, ...customTypesArray]));

    const result = await invoke<Array<{path: string, content: string, abs_path: string}>>("generate_context", {
      paths: paths,
      maxDepth: appConfig.maxDepth,
      generateTree: appConfig.generateTree,
      ignoreExts: appConfig.ignoreExts,
      ignoreDeepParse: appConfig.ignoreDeepParse,
      includedTypes: finalIncludedTypes,
      projectRoots: appConfig.projectRoots,
      enableMinimization: appConfig.enableMinimization,
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
        fileNodes: fileNodes.value.map(n => ({ path: n.path, content: n.content })),
        generateTree: appConfig.generateTree,
        customPrompt: appConfig.customPrompt,
        userPrompt: userPrompt.value,
        longContextThreshold: 8000,
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

async function toggleEdit() {
    isEditing.value = !isEditing.value;
    if (isEditing.value) {
        await nextTick();
        outputAreaRef.value?.focus();
    }
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

    if (debounceTimer) {
        clearTimeout(debounceTimer);
    }

    if (filesList.value.length === 0) {
        fileNodes.value = [];
        updateOutputContext();
        return;
    }

    debounceTimer = setTimeout(() => {
        processPaths(filesList.value.map((f: {path: string}) => f.path));
    }, 400);
}

const fileListContainer = ref<HTMLElement | null>(null);
function handleWheel(e: WheelEvent) {
    handleWheelHorizontal(e, fileListContainer.value);
}
</script>

<template>
  <main class="h-screen flex flex-col items-center p-6 selection:bg-app-primary/10 relative overflow-y-auto">
    <!-- Header Area -->
    <div class="w-full max-w-6xl flex justify-between items-center mb-8 shrink-0">
      <div class="flex flex-col">
          <h1 class="text-3xl font-black text-app-text tracking-tight flex items-center">
            CodePulse <span class="ml-2 font-medium opacity-20 text-xl">文脉</span>
          </h1>
          <p class="text-app-text-dim text-sm mt-1 font-medium italic opacity-70">
            一键解析项目依赖，构建完整提示词上下文
          </p>
      </div>
      <button 
        @click="isSettingsOpen = true"
        class="p-2.5 bg-app-surface hover:bg-app-surface-hover text-app-text-dim hover:text-app-primary rounded-xl transition-all shadow-app-md border border-app-border cursor-pointer group"
        title="设置 (Settings)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 group-hover:rotate-45 transition-transform duration-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </button>
    </div>

    <!-- Top Section: Drop Zone & User Prompt -->
    <div class="w-full max-w-6xl grid grid-cols-1 md:grid-cols-2 gap-6 mb-8 shrink-0">
      <!-- Left: Drop Zone Card -->
      <div 
        data-drop-zone="main"
        class="h-56 rounded-3xl border-2 border-dashed flex flex-col items-center justify-center transition-all duration-500 relative overflow-hidden group shadow-app-md bg-app-surface/50 backdrop-blur-sm"
        :class="isDragging ? 'border-app-primary bg-app-primary-light ring-4 ring-app-primary/5' : 'border-app-border hover:border-app-primary/40 hover:bg-app-surface'"
      >
        <div class="flex flex-col items-center space-y-3 z-10 w-full px-6">
          <div class="w-12 h-12 flex items-center justify-center bg-app-surface rounded-2xl shadow-sm transition-all duration-500 pointer-events-none">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-app-text-mute transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
          </div>
          <p class="text-base font-bold text-app-text-dim group-hover:text-app-text transition-colors tracking-tight pointer-events-none">
            {{ isDragging ? '松开即刻解析...' : '拖入代码文件或功能块目录' }}
          </p>
          <div v-if="!isDragging" class="flex gap-2 pt-1">
              <button @click="triggerFileInput" class="px-3 py-1 bg-app-bg text-[11px] font-bold text-app-text-dim hover:text-app-text border border-app-border rounded-lg hover:border-app-primary/50 transition-all cursor-pointer shadow-sm">添加文件</button>
              <button @click="triggerDirInput" class="px-3 py-1 bg-app-bg text-[11px] font-bold text-app-text-dim hover:text-app-text border border-app-border rounded-lg hover:border-app-primary/50 transition-all cursor-pointer shadow-sm">添加目录</button>
          </div>
        </div>

        <!-- Files List Overlay -->
        <div v-if="filesList.length > 0 && !isDragging" 
            ref="fileListContainer"
            @wheel="handleWheel"
            class="flex items-center gap-2 w-full overflow-x-auto px-6 mt-4 z-10 custom-scrollbar-h pb-2"
        >
            <div 
              v-for="(file, idx) in filesList" 
              :key="idx" 
              @click="removeFile(idx)"
              class="group/item flex items-center shrink-0 text-[10px] bg-app-surface px-3 py-1.5 rounded-xl border border-app-border hover:border-app-rose/40 hover:bg-app-rose/5 transition-all cursor-pointer text-app-text-dim font-mono select-none shadow-sm"
              title="点击移除"
            >
                <span class="truncate max-w-32 group-hover/item:text-app-rose">
                    {{ file.path.split('/').pop()?.split('\\').pop() }}
                </span>
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 ml-2 text-app-text-mute group-hover/item:text-app-rose transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </div>
        </div>
      </div>

      <!-- Right: User Prompt Card -->
      <div class="h-56 rounded-3xl bg-app-surface shadow-app-md border border-app-border overflow-hidden flex flex-col group focus-within:ring-4 focus-within:ring-app-primary/5 transition-all">
        <div class="px-5 py-2.5 bg-app-bg/30 border-b border-app-border flex items-center justify-between">
            <span class="text-[11px] font-black uppercase tracking-widest text-app-text-mute group-focus-within:text-app-primary transition-colors">附加提示词 / 需求</span>
            <div class="flex gap-1">
                <div class="w-1.5 h-1.5 rounded-full bg-app-border"></div>
                <div class="w-1.5 h-1.5 rounded-full bg-app-border"></div>
            </div>
        </div>
        <textarea 
          v-model="userPrompt"
          placeholder="例如：请作为核心开发者对这些逻辑做 Code Review；或指定特定功能模块的重构需求..."
          class="w-full flex-1 px-5 py-4 resize-none text-app-text placeholder:text-app-text-mute bg-transparent focus:outline-none font-sans text-sm leading-relaxed"
        ></textarea>
      </div>
    </div>

    <!-- Generate Control Area -->
    <div class="w-full max-w-6xl mb-8 flex justify-center shrink-0">
        <button 
          @click="isLoading ? stopProcessing() : processPaths(filesList.map(f => f.path))"
          :disabled="filesList.length === 0 && !isLoading"
          class="h-14 px-10 group/btn bg-app-text hover:bg-app-primary text-app-bg font-black rounded-2xl shadow-xl shadow-app-text/10 transition-all active:scale-95 disabled:opacity-20 disabled:cursor-not-allowed flex items-center gap-3 cursor-pointer"
          :class="isLoading ? 'hover:bg-app-rose!' : ''"
        >
          <span v-if="isLoading" class="flex items-center gap-4">
              <svg class="animate-spin h-5 w-5" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
              构建中 (点击中断)...
          </span>
          <span v-else class="flex items-center gap-4">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 transition-transform group-hover/btn:translate-x-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M13 5l7 7-7 7M5 5l7 7-7 7" /></svg>
              即刻生成完整上下文
          </span>
        </button>
    </div>

    <!-- Bottom: Results Area -->
    <div class="w-full max-w-6xl flex-1 flex min-h-[420px] gap-6 mb-2">
      <!-- Left: Sidebar Tree -->
      <DependencyTreeSidebar 
        :fileNodes="fileNodes" 
        class="w-80"
        @delete="(fp, ap, ids) => handleNodeDelete(fp, ap, ids)" 
        @upload-files="handleTreeUploadFiles"
      />

      <!-- Right: Main Context Output -->
      <div class="flex-1 flex flex-col bg-app-surface border border-app-border rounded-3xl shadow-app-md relative overflow-hidden">
        <div class="px-5 py-2 bg-app-surface/80 border-b border-app-border rounded-t-3xl flex justify-between items-center backdrop-blur-xl shrink-0 z-10">
          <div class="flex flex-col">
              <span class="text-sm font-black text-app-text flex items-center gap-4">
                  输出上下文
                  <span v-if="outputContext" class="text-[9px] bg-app-bg text-app-text px-3 py-0.5 rounded-lg border border-app-text/10 font-black">
                    {{ totalCharacters.toLocaleString() }} 字
                  </span>
              </span>
          </div>
          <div class="flex items-center space-x-2">
            <button 
              v-if="outputContext"
              @click="toggleEdit"
              class="p-1.5 bg-app-surface hover:bg-app-surface-hover text-app-text-dim hover:text-app-primary rounded-lg transition-all border border-app-border cursor-pointer"
              :class="isEditing ? 'bg-app-primary/5 border-app-primary/30 ring-2 ring-app-primary/5 text-app-primary' : ''"
              :title="isEditing ? '应用更改' : '快速编辑'"
            >
              <svg v-if="!isEditing" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            </button>
            <button 
              v-if="outputContext"
              @click="copyToClipboardAll"
              class="p-2 py-1.5 bg-app-text hover:bg-app-text/90 text-app-bg text-[11px] font-black rounded-lg transition-all active:scale-95 flex items-center gap-1.5 shadow-sm cursor-pointer"
              title="复制全部"
            >
              复制
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>
            </button>
          </div>
        </div>
        <div class="overflow-hidden w-full h-full flex-1 rounded-b-3xl py-2 px-2">
          <textarea 
            ref="outputAreaRef"
            :readonly="!isEditing"
            v-model="outputContext"
            placeholder="所有的上下文内容都在这里准备就绪..."
            class="w-full h-full flex-1 p-2 focus:outline-none font-mono text-[13px] leading-relaxed resize-none transition-all duration-300 z-0 bg-transparent text-app-text placeholder:text-app-text-mute"
            :class="[isEditing ? 'bg-app-primary/5 cursor-text!' : 'cursor-default']"
          ></textarea>
        </div>
        
        <!-- Loading Overlay -->
        <div v-if="isLoading" class="absolute inset-0 bg-app-surface/60 backdrop-blur-md flex flex-col items-center justify-center z-20 pointer-events-none transition-all">
          <div class="relative w-12 h-12 mb-4">
             <div class="absolute inset-0 rounded-full border-4 border-app-primary/10"></div>
             <div class="absolute inset-0 rounded-full border-4 border-t-app-primary animate-spin"></div>
          </div>
          <p v-if="appConfig.enableMinimization" class="text-app-primary font-black tracking-widest text-xs uppercase animate-pulse">正在深度解析并压缩代码...</p>
          <p v-else class="text-app-primary font-black tracking-widest text-xs uppercase animate-pulse">正在解析依赖...</p>
        </div>
      </div>
    </div>

    <!-- Settings Modal -->
    <AppSettingsModal 
      v-model:show="isSettingsOpen" 
      :settings="appConfig"
      @update:settings="val => Object.assign(appConfig, val)"
    />
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
</style>