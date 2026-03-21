<script setup lang="ts">
import { ref, onMounted, onUnmounted, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import AppSettingsModal from "./components/AppSettingsModal.vue";

const outputContext = ref("");
const isDragging = ref(false);
const isLoading = ref(false);
const filesList = ref<string[]>([]);
const isSettingsOpen = ref(false);

const appConfig = reactive({
  maxDepth: 3,
  ignoreExts: ".git, node_modules, dist",
  customPrompt: "",
  generateTree: true,
  autoGenerate: true,
  parseMode: "normal",
  includedTypes: ["vue", "ts"],
});

let unlistenDragDrop: () => void;

onMounted(async () => {
  unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === 'over' || event.payload.type === 'enter') {
      isDragging.value = true;
    } else if (event.payload.type === 'leave') {
      isDragging.value = false;
    } else if (event.payload.type === 'drop') {
      isDragging.value = false;
      const paths = event.payload.paths;
      if (paths && paths.length > 0) {
        filesList.value = paths;
        if (appConfig.autoGenerate) {
          processPaths(paths);
        }
      }
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
    const result = await invoke<string>("generate_context", {
      paths: paths,
      maxDepth: appConfig.maxDepth,
      generateTree: appConfig.generateTree,
    });
    outputContext.value = result;
  } catch (error) {
    console.error("Failed to generate context:", error);
    outputContext.value = `Error: ${error}`;
  } finally {
    isLoading.value = false;
  }
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  isDragging.value = false;

  if (event.dataTransfer?.files) {
    const paths: string[] = [];
    for (let i = 0; i < event.dataTransfer.files.length; i++) {
        const file = event.dataTransfer.files[i] as any;
        // Tauri 运行时常常会在 File 对象中注入真实绝对路径 path
        if (file.path) {
            paths.push(file.path);
        } else {
            paths.push(file.name);
        }
    }
    if (paths.length > 0) {
      filesList.value = paths;
      if (appConfig.autoGenerate) {
        processPaths(paths);
      }
    }
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

async function triggerFileInput() {
    const selected = await open({
        multiple: true,
        directory: false,
    });
    if (selected && Array.isArray(selected)) {
        filesList.value = selected;
        if (appConfig.autoGenerate) processPaths(selected);
    } else if (selected && typeof selected === 'string') {
        filesList.value = [selected];
        if (appConfig.autoGenerate) processPaths([selected]);
    }
}

async function triggerDirInput() {
    const selected = await open({
        multiple: true,
        directory: true,
    });
    if (selected && Array.isArray(selected)) {
        filesList.value = selected;
        if (appConfig.autoGenerate) processPaths(selected);
    } else if (selected && typeof selected === 'string') {
        filesList.value = [selected];
        if (appConfig.autoGenerate) processPaths([selected]);
    }
}
</script>

<template>
  <main class="min-h-screen bg-slate-900 text-slate-100 p-6 flex flex-col items-center font-sans antialiased selection:bg-blue-500/30 relative">
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
      拖拽代码文件或项目目录，一键深度递归解析依赖，自动生成供大语言模型阅读的完整代码上下文。
    </p>

    <!-- Drop Zone -->
    <div 
      class="w-full max-w-4xl h-48 rounded-2xl border-2 border-dashed flex flex-col items-center justify-center transition-all duration-300 mb-6 relative overflow-hidden group shadow-sm bg-slate-800/30"
      :class="isDragging ? 'border-blue-400 bg-blue-900/10 scale-[1.01] shadow-blue-500/10' : 'border-slate-600 hover:border-slate-400 hover:bg-slate-800/50'"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop="handleDrop"
    >
      <div class="pointer-events-none flex flex-col items-center space-y-3 z-10 w-full px-4">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-slate-400 group-hover:text-blue-400 transition-colors drop-shadow-md" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
        <p class="text-lg font-medium text-slate-300 group-hover:text-blue-300 transition-colors tracking-wide">
          {{ isDragging ? '松开以解析文件...' : '拖拽 文件 或 项目目录 到此处' }}
        </p>
      </div>
      <!-- Clickable Actions -->
      <div v-if="!isDragging" class="flex mt-4 space-x-4 z-20">
        <button 
          @click="triggerFileInput"
          class="px-4 py-1.5 bg-slate-700/80 hover:bg-slate-600 text-sm text-slate-200 font-medium rounded-md shadow-sm border border-slate-600 transition-colors"
        >
          📄 选择文件
        </button>
        <button 
          @click="triggerDirInput"
          class="px-4 py-1.5 bg-slate-700/80 hover:bg-slate-600 text-sm text-slate-200 font-medium rounded-md shadow-sm border border-slate-600 transition-colors"
        >
          📁 选择目录
        </button>
      </div>

      <div v-if="filesList.length > 0 && !isDragging" class="flex flex-wrap gap-2 justify-center max-w-full overflow-hidden mt-3 z-10 opacity-75 hover:opacity-100 transition-opacity">
          <span v-for="(file, idx) in filesList" :key="idx" class="text-xs bg-slate-700/80 px-2 py-1 rounded border border-slate-600 truncate max-w-[200px] text-slate-300 font-mono">
              {{ file.split('/').pop()?.split('\\').pop() }}
          </span>
      </div>
    </div>

    <!-- Controls (Generate Button) -->
    <div class="w-full max-w-4xl flex justify-center mb-6">
      <button 
        @click="processPaths(filesList)"
        :disabled="filesList.length === 0 || isLoading"
        class="px-8 py-3 w-full sm:w-auto min-w-50 flex justify-center bg-blue-600 hover:bg-blue-500 disabled:bg-slate-700 disabled:text-slate-500 text-white font-semibold rounded-xl shadow-lg hover:shadow-blue-500/20 transition-all active:scale-95"
      >
        <span v-if="isLoading" class="flex items-center">
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
    <div class="w-full max-w-4xl relative flex-1 flex flex-col min-h-[400px]">
      <div class="flex justify-between items-center bg-slate-800/80 backdrop-blur-md px-4 py-2.5 border-t border-x border-slate-700 rounded-t-xl">
        <span class="text-sm font-semibold text-slate-400 flex items-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" /></svg>
            输出上下文 (Generated Context)
        </span>
        <button 
          @click="copyToClipboard"
          class="flex items-center space-x-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-sm font-medium rounded-md transition-colors active:bg-slate-800"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>
          <span class="text-slate-200">一键复制</span>
        </button>
      </div>
      <textarea 
        readonly
        v-model="outputContext"
        placeholder="解析后的所有代码将合并展示在这里..."
        class="w-full flex-1 p-4 bg-[#0d1117] border border-slate-700 rounded-b-xl focus:outline-none focus:ring-1 focus:ring-blue-500 font-mono text-sm text-green-400 leading-relaxed resize-none shadow-inner"
      ></textarea>
      
      <!-- Loading Overlay -->
      <div v-if="isLoading" class="absolute inset-0 top-[45px] flex flex-col items-center justify-center bg-slate-900/60 backdrop-blur-sm rounded-b-xl z-20">
         <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mb-4"></div>
         <p class="text-blue-300 font-medium tracking-widest animate-pulse">深度分析中，请耐心等待...</p>
      </div>
    </div>

    <!-- Settings Modal -->
    <AppSettingsModal 
      v-model:show="isSettingsOpen" 
      v-model:settings="appConfig"
    />
  </main>
</template>

<style>
/* Custom Scrollbar for Textarea */
textarea::-webkit-scrollbar {
  width: 8px;
}
textarea::-webkit-scrollbar-track {
  background: #0d1117;
  border-bottom-right-radius: 0.75rem;
}
textarea::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 4px;
}
textarea::-webkit-scrollbar-thumb:hover {
  background: #475569;
}
</style>