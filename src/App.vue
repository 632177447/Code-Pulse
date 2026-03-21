<script setup lang="ts">
import { ref, onMounted, onUnmounted, reactive, computed, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import AppSettingsModal from "./components/AppSettingsModal.vue";
import DependencyTreeSidebar from "./components/DependencyTreeSidebar.vue";

const outputContext = ref("");
const fileNodes = ref<{path: string, content: string, abs_path: string, originId?: string}[]>([]);
const isDragging = ref(false);
const isLoading = ref(false);
const filesList = ref<{id: string, path: string}[]>([]);
const isSettingsOpen = ref(false);
const userPrompt = ref("");
const isEditing = ref(false);
const outputAreaRef = ref<HTMLTextAreaElement | null>(null);

const totalCharacters = computed(() => {
  return outputContext.value ? outputContext.value.length : 0;
});


const appConfig = reactive({
  maxDepth: 2,
  includedTypes: ["vue", "ts", "js", "py", "json", "css", "scss"],
  ignoreExts: ".git, node_modules, dist, target, build, .vscode, .idea, .next, .nuxt, .output, .vercel, .github, __pycache__, .venv, bin, obj, *.lock, *.log, *.tmp, *.temp, *.png, *.jpg, *.jpeg, *.gif, *.svg, *.ico, *.webp, *.mp4, *.avi, *.mkv, *.mov, *.webm, *.mp3, *.wav, *.flac, *.aac, *.ogg, *.zip, *.tar, *.gz, *.7z, *.rar, *.exe, *.dll, *.so, *.dylib",
  ignoreDeepParse: "package.json, tsconfig.json, vite.config.ts, tauri.conf.json, README.md, Cargo.toml, go.mod, pom.xml, .env, *.test.ts, *.spec.ts",
  customPrompt: "",
  generateTree: true,
  autoGenerate: true,
  customIncludedTypes: "",
});
let unlistenDragDrop: () => void;
let lastHighlightedNode: HTMLElement | null = null;

onMounted(async () => {
  unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event: any) => {
    const { type, position, paths } = event.payload;

    // 关键修正：Tauri 的坐标是物理像素，需要除以缩放比例转换为逻辑像素
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
        if (dropZone) {
          const newItems = (paths as string[]).map((p: string) => ({
            id: Math.random().toString(36).substring(2, 11),
            path: p
          }));
          // 如果 autoGenerate 为 true，且原本列表为空，则直接替换；否则添加（类似追加行为，但根据原有逻辑 filesList.value = paths 是替换）
          filesList.value = newItems;
          if (appConfig.autoGenerate) processPaths(newItems.map((i: {path: string}) => i.path));
        } else if (nodeZone) {
          const destDir = nodeZone.dataset.dropPath;
          if (destDir) handleTreeUploadFiles(paths, destDir);
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
    });
    
    fileNodes.value = result.map(node => {
        // 在前端为每个解析出的文件节点注入来源上传项的 ID
        const normalize = (p: string) => p.replace(/\\/g, '/').toLowerCase().trim().replace(/^\\\\?\\/, '').replace(/^\/\/\?\//, '').replace(/\/+$/, '');
        const nNodeAbs = normalize(node.abs_path);
        
        const origin = filesList.value.find(f => {
            const nf = normalize(f.path);
            return nNodeAbs === nf || nNodeAbs.startsWith(nf + '/');
        });
        
        return { ...node, originId: origin?.id };
    });
    updateOutputContext();
} catch (error) {
    console.error("Failed to generate context:", error);
    outputContext.value = `Error: ${error}`;
  } finally {
    isLoading.value = false;
  }
}

function updateOutputContext() {
    if (fileNodes.value.length === 0) {
        outputContext.value = "";
        return;
    }

    let finalContext = "";

    if (appConfig.generateTree) {
        const paths = fileNodes.value.map(n => n.path);
        let tree = "========================================\n[FILE TREE]\n========================================\n.\n";
        const sortedPaths = [...paths].sort();
        let prevComponents: string[] = [];
        for (const path of sortedPaths) {
            const components = path.split('/');
            let i = 0;
            while (i < components.length && i < prevComponents.length && components[i] === prevComponents[i]) {
                i++;
            }
            while (i < components.length) {
                const indent = "│   ".repeat(i);
                tree += `${indent}├── ${components[i]}\n`;
                i++;
            }
            prevComponents = components;
        }
        finalContext += tree + "\n";
    }

    if (appConfig.customPrompt.trim()) {
      finalContext += "========================================\n";
      finalContext += "[SYSTEM SETTINGS]\n";
      finalContext += "========================================\n";
      finalContext += appConfig.customPrompt.trim() + "\n\n";
    }

    const PENDING_USER_PROMPT = userPrompt.value.trim();
    const LONG_CONTEXT_THRESHOLD = 8000;

    const blocksContent = fileNodes.value.map(n => n.content).join("\n\n");

    if (PENDING_USER_PROMPT && blocksContent.length <= LONG_CONTEXT_THRESHOLD) {
      finalContext += "========================================\n";
      finalContext += "[USER REQUIREMENTS]\n";
      finalContext += "========================================\n";
      finalContext += PENDING_USER_PROMPT + "\n\n";
    }

    finalContext += blocksContent;

    if (PENDING_USER_PROMPT && blocksContent.length > LONG_CONTEXT_THRESHOLD) {
      finalContext += "\n\n========================================\n";
      finalContext += "[USER REQUIREMENTS]\n";
      finalContext += "========================================\n";
      finalContext += PENDING_USER_PROMPT;
    }
    
    outputContext.value = finalContext;
}

function handleNodeDelete(fullPath: string, absPath?: string, originIds?: string[]) {
    // 1. 同步过滤 fileNodes (树里的文件)
    fileNodes.value = fileNodes.value.filter(node => 
        !(node.path === fullPath || node.path.startsWith(fullPath + '/'))
    );

    // 2. 联动删除 filesList (用户上传列表)
    // 根据 ID 列表进行精准过滤：如果 originIds 中包含了某个初始项的 ID，则该项消失
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
            // 将新路径添加进 filesList
            for (const p of newPaths) {
                if (!filesList.value.find(f => f.path === p)) {
                    filesList.value.push({
                        id: Math.random().toString(36).substring(2, 11),
                        path: p
                    });
                }
            }
            // 无论 autoGenerate 是否开启，都需要更新结果
            await processPaths(filesList.value.map(f => f.path));
        }
    } catch (e) {
        console.error("Upload failed:", e);
        alert(`上传失败: ${e}`);
    } finally {
        isLoading.value = false;
    }
}

async function copyToClipboard() {
  if (!outputContext.value) return;
  try {
    await navigator.clipboard.writeText(outputContext.value);
    alert("上下文已成功复制到剪贴板！");
  } catch (e) {
    console.error(e);
  }
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
    if (appConfig.autoGenerate) {
        if (filesList.value.length === 0) {
            fileNodes.value = [];
            updateOutputContext();
        } else {
            processPaths(filesList.value.map((f: {path: string}) => f.path));
        }
    }
}

const fileListContainer = ref<HTMLElement | null>(null);
function handleWheel(e: WheelEvent) {
    if (fileListContainer.value) {
        e.preventDefault();
        fileListContainer.value.scrollLeft += e.deltaY;
    }
}
</script>

<template>
  <main class="h-screen bg-slate-900 text-slate-100 p-6 flex flex-col items-center font-sans antialiased selection:bg-blue-500/30 relative overflow-y-auto">
    <button 
      @click="isSettingsOpen = true"
      class="absolute top-6 right-6 p-2 bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white rounded-lg transition-colors shadow-lg border border-slate-700"
      title="设置 (Settings)"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </button>
    <h1 class="text-4xl font-extrabold mb-2 bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent drop-shadow-sm">
      CodePulse 码脉
    </h1>
    <p class="text-slate-400 mb-8 max-w-lg text-center font-medium">
      通过代码文件或项目目录，一键解析依赖，生成供 AI 阅读的完整代码上下文。
    </p>

    <!-- Top Section: Drop Zone & User Prompt -->
    <div class="w-full max-w-6xl flex gap-6 mb-6">
      <!-- Left: Drop Zone -->
      <div 
        data-drop-zone="main"
        class="flex-1 h-48 rounded-2xl border-2 border-dashed flex flex-col items-center justify-center transition-all duration-300 relative overflow-hidden group shadow-sm bg-slate-800/30"
        :class="isDragging ? 'border-blue-400 bg-blue-900/10 scale-[1.01] shadow-blue-500/10' : 'border-slate-600 hover:border-slate-400 hover:bg-slate-800/50'"
      >
        <div class="pointer-events-none flex flex-col items-center space-y-3 z-10 w-full px-4">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-slate-400 group-hover:text-blue-400 transition-colors drop-shadow-md" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          <p class="text-lg font-medium text-slate-300 group-hover:text-blue-300 transition-colors tracking-wide">
            {{ isDragging ? '松开以解析文件...' : '拖拽 文件 或 目录' }}
          </p>
        </div>
        <!-- Clickable Actions -->
        <div v-if="!isDragging" class="flex mt-4 space-x-4 z-20">
          <button 
            @click="triggerFileInput"
            class="px-4 py-1.5 bg-slate-700/80 hover:bg-slate-600 text-sm text-slate-200 font-medium rounded-md shadow-sm border border-slate-600 transition-colors"
          >
            📄 文件
          </button>
          <button 
            @click="triggerDirInput"
            class="px-4 py-1.5 bg-slate-700/80 hover:bg-slate-600 text-sm text-slate-200 font-medium rounded-md shadow-sm border border-slate-600 transition-colors"
          >
            📁 目录
          </button>
        </div>

        <div v-if="filesList.length > 0 && !isDragging" 
            ref="fileListContainer"
            @wheel="handleWheel"
            class="flex items-center gap-2 w-full overflow-x-auto px-6 mt-3 z-10 opacity-90 custom-scrollbar-h pb-2"
        >
            <div 
              v-for="(file, idx) in filesList" 
              :key="idx" 
              @click="removeFile(idx)"
              class="group/item flex items-center shrink-0 text-xs bg-slate-700/80 px-2.5 py-1.5 rounded-lg border border-slate-600 hover:border-red-500/50 hover:bg-slate-700 transition-all cursor-pointer text-slate-300 font-mono select-none"
              title="点击移除此文件/目录"
            >
                <span class="truncate max-w-[180px] group-hover/item:text-red-400">
                    {{ file.path.split('/').pop()?.split('\\').pop() }}
                </span>
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 ml-1.5 text-slate-500 group-hover/item:text-red-500 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </div>
        </div>
      </div>

      <!-- Right: User Prompt Textarea -->
      <div class="flex-1 h-48 flex flex-col">
        <textarea 
          v-model="userPrompt"
          placeholder="在此输入您的自定义需求、提示词...（将自动添加至最终生成的代码上下文合适位置）"
          class="w-full h-full p-4 bg-slate-800/30 border-2 border-slate-600 rounded-2xl resize-none text-slate-200 placeholder-slate-400 focus:outline-none focus:border-blue-400 focus:bg-slate-800/50 transition-all font-sans text-sm shadow-sm"
        ></textarea>
      </div>
    </div>

    <!-- Controls (Generate Button) -->
    <div class="w-full max-w-6xl flex justify-center mb-6">
      <button 
        @click="processPaths(filesList.map((f: {path: string}) => f.path))"
        :disabled="filesList.length === 0 || isLoading"
        class="h-[52px] px-8 w-full sm:w-auto min-w-50 flex items-center justify-center bg-blue-600 hover:bg-blue-500 disabled:bg-slate-700 disabled:text-slate-500 text-white font-semibold rounded-xl shadow-lg hover:shadow-blue-500/20 transition-all active:scale-95"
      >
        <span v-if="isLoading" class="flex items-center text-lg">
            <svg class="animate-spin -ml-1 mr-2 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            解析构建中...
        </span>
        <span v-else class="text-lg">重新生成上下文</span>
      </button>
    </div>

    <!-- Output Area -->
    <div class="w-full max-w-6xl relative flex-1 flex min-h-[380px] gap-4 mb-4">
      
      <!-- Tree Component Sidebar -->
      <DependencyTreeSidebar 
        :fileNodes="fileNodes" 
        @delete="(fp, ap, ids) => handleNodeDelete(fp, ap, ids)" 
        @upload-files="handleTreeUploadFiles"
      />

      <!-- Context Textarea -->
      <div class="flex-1 flex flex-col relative w-full overflow-hidden">
        <div class="flex justify-between items-center bg-slate-800/80 backdrop-blur-md px-4 py-2.5 border-t border-x border-slate-700 rounded-t-xl z-10">
          <span class="text-sm font-semibold text-slate-400 flex items-center">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" /></svg>
              输出上下文
              <span v-if="outputContext" class="ml-3 text-[10px] bg-slate-700/50 px-2 py-0.5 rounded-full border border-slate-600/50 text-slate-400 font-mono">
                {{ totalCharacters.toLocaleString() }} 字
              </span>
          </span>
          <div class="flex items-center space-x-2">
            <button 
              v-if="outputContext"
              @click="toggleEdit"
              class="p-2 bg-slate-700 hover:bg-slate-600 rounded-md transition-colors"
              :class="isEditing ? 'bg-blue-600/60 ring-1 ring-blue-500/50' : ''"
              :title="isEditing ? '保存并退出编辑' : '编辑内容'"
            >
              <svg v-if="!isEditing" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </button>
            <button 
              @click="copyToClipboard"
              class="p-2 bg-slate-700 hover:bg-slate-600 rounded-md transition-colors active:bg-slate-800"
              title="一键复制"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>
            </button>
          </div>
        </div>
        <textarea 
          ref="outputAreaRef"
          :readonly="!isEditing"
          v-model="outputContext"
          placeholder="解析后的所有代码将合并展示在这里..."
          class="w-full flex-1 p-4 border rounded-b-xl focus:outline-none font-mono text-sm leading-relaxed resize-none shadow-inner z-0 transition-all duration-300"
          :class="isEditing ? 'bg-[#161b22] border-blue-500/50 text-slate-100 ring-1 ring-blue-500/20' : 'bg-[#0d1117] border-slate-700 text-green-400'"
        ></textarea>
        
        <!-- Loading Overlay -->
        <div v-if="isLoading" class="absolute inset-0 top-[45px] flex flex-col items-center justify-center bg-slate-900/60 backdrop-blur-sm rounded-b-xl z-20">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mb-4"></div>
          <p class="text-blue-300 font-medium tracking-widest animate-pulse">深度分析中，请耐心等待...</p>
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
/* 全局滚动条美化 - 极简现代风格 */
::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(100, 116, 139, 0.4); /* slate-500 */
  border-radius: 20px;
  border: 3px solid transparent; /* 通过透明边框实现内边距效果 */
  background-clip: content-box;
  cursor: pointer;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(148, 163, 184, 0.6); /* slate-400 */
  border-width: 2px; /* 悬停时稍微变胖一点，暗示可交互 */
  cursor: pointer;
}

/* 针对特定深色背景容器的微调 */
textarea::-webkit-scrollbar-track, 
.custom-scrollbar::-webkit-scrollbar-track {
  background: #0d1117; 
  border-bottom-right-radius: 0.75rem;
}

textarea::-webkit-scrollbar-thumb,
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(100, 116, 139, 0.3);
}

/* 水平滚动条美化 */
.custom-scrollbar-h::-webkit-scrollbar {
  height: 4px;
}

.custom-scrollbar-h::-webkit-scrollbar-track {
  background: transparent;
  margin: 0 20px;
}

.custom-scrollbar-h::-webkit-scrollbar-thumb {
  background: rgba(100, 116, 139, 0.3);
  border-radius: 10px;
}

.custom-scrollbar-h::-webkit-scrollbar-thumb:hover {
  background: rgba(148, 163, 184, 0.5);
}

/* 拖拽到树节点的高亮样式 */
.drop-node-hover {
  background-color: rgba(59, 130, 246, 0.3) !important;
  box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.5);
  border-radius: 6px;
}
</style>