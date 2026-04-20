<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  show: boolean;
  allowedWritePaths: string;
}>();

const emit = defineEmits(['update:show']);

const commandsJson = ref('');
const isLoading = ref(false);
const error = ref('');
const successMessage = ref('');

const handleClose = () => {
  emit('update:show', false);
  commandsJson.value = '';
  error.value = '';
  successMessage.value = '';
};

const runCommands = async () => {
  if (!commandsJson.value.trim()) return;
  
  isLoading.value = true;
  error.value = '';
  successMessage.value = '';
  
  try {
    let jsonToParse = commandsJson.value.trim();
    // 提取 json_commands 块内容
    const match = jsonToParse.match(/```json_commands\s*([\s\S]*?)```/);
    if (match) {
      jsonToParse = match[1].trim();
    } else {
      const matchJson = jsonToParse.match(/```json\s*([\s\S]*?)```/);
      if (matchJson) jsonToParse = matchJson[1].trim();
    }

    // 仅使用 allowedWritePaths，如果为空则没有任何目录权限
    const rawRoots = props.allowedWritePaths.trim();
    const roots = rawRoots
      .split(/[,\n\r]/)
      .map(s => s.trim())
      .filter(s => s.length > 0);

    await invoke('execute_ai_commands', {
      commandsJson: jsonToParse,
      projectRoots: roots
    });
    
    successMessage.value = '命令执行成功！';
    commandsJson.value = '';
    setTimeout(() => {
        handleClose();
    }, 1500);
  } catch (e: any) {
    error.value = e.toString();
  } finally {
    isLoading.value = false;
  }
};
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-60 flex items-center justify-center p-4 bg-app-text/20 backdrop-blur-md transition-all animate-in fade-in duration-300" @click.self="handleClose">
    <div class="bg-app-surface border border-app-border rounded-[32px] shadow-app-xl w-full max-w-4xl h-[680px] max-h-[90vh] flex flex-col transform transition-all overflow-hidden animate-in zoom-in-95 duration-500">
      <!-- Header -->
      <div class="px-7 py-4 border-b border-app-border flex justify-between items-center bg-app-surface shrink-0">
        <div class="flex flex-col">
            <h3 class="text-2xl font-black text-app-text tracking-tight flex items-center">
              自动化执行 <span class="ml-3 font-medium opacity-20 text-sm tracking-[0.2em]">COMMANDS</span>
            </h3>
        </div>
        <button @click="handleClose" class="text-app-text-mute hover:text-app-text transition-all p-2 rounded-2xl hover:bg-app-bg cursor-pointer group">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 group-hover:rotate-90 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="p-8 flex-1 flex flex-col min-h-0 space-y-6 overflow-hidden">
        <div class="flex-1 flex flex-col min-h-0">
          <label class="block text-[12px] font-black uppercase tracking-[0.2em] opacity-40 mb-3 ml-1 shrink-0">粘贴 AI 返回的 JSON 指令</label>
          <textarea
            v-model="commandsJson"
            placeholder='[
  {
    "action": "patch",
    "path": "absolute/path/to/file.ts",
    "search": "old string",
    "replace": "new string"
  }
]'
            class="w-full flex-1 bg-app-bg border border-app-border rounded-2xl p-6 text-app-text font-mono text-sm focus:ring-2 focus:ring-app-primary/20 focus:border-app-primary outline-none transition-all resize-none custom-scrollbar"
          ></textarea>
        </div>

        <div v-if="error" class="p-4 bg-red-500/10 border border-red-500/20 rounded-2xl text-red-500 text-sm font-medium animate-in fade-in slide-in-from-top-2 shrink-0">
          {{ error }}
        </div>
        
        <div v-if="successMessage" class="p-4 bg-green-500/10 border border-green-500/20 rounded-2xl text-green-500 text-sm font-medium animate-in fade-in slide-in-from-top-2 shrink-0">
          {{ successMessage }}
        </div>
      </div>

      <!-- Footer -->
      <div class="px-9 py-5 border-t border-app-border flex justify-end shrink-0 gap-4 bg-app-surface">
        <button 
          @click="handleClose" 
          class="px-8 py-3 text-app-text-dim hover:text-app-text font-black text-xs uppercase tracking-[0.2em] rounded-2xl transition-all border border-app-border hover:bg-app-bg cursor-pointer shadow-sm active:scale-95"
        >
          取消
        </button>
        <button 
          @click="runCommands" 
          :disabled="isLoading || !commandsJson.trim()"
          class="px-11 py-3 bg-app-text text-app-bg hover:bg-app-primary hover:text-white font-black text-xs uppercase tracking-[0.2em] rounded-2xl shadow-xl shadow-app-primary/10 transition-all active:scale-95 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          <svg v-if="isLoading" class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ isLoading ? '执行中...' : '运行指令' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar { width: 5px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { 
    background: var(--color-app-border); 
    border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: var(--color-app-text-mute); }

.animate-in {
  animation-fill-mode: both;
}

@keyframes slide-in-from-top {
  from {
    transform: translateY(-8px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.slide-in-from-top-2 {
  animation: slide-in-from-top 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
</style>
