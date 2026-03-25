<script setup lang="ts">
import { computed, ref } from "vue";
import { handleWheelHorizontal } from "../../utils";

const props = defineProps<{
  isDragging: boolean;
  isInvalidDrag: boolean;
  filesList: {id: string, path: string}[];
  userPrompt: string;
}>();

const emit = defineEmits<{
  (e: "update:userPrompt", value: string): void;
  (e: "openFile"): void;
  (e: "openDir"): void;
  (e: "removeFile", index: number): void;
}>();

const userPromptModel = computed({
  get: () => props.userPrompt,
  set: (val) => emit("update:userPrompt", val)
});

const fileListContainer = ref<HTMLElement | null>(null);

function handleWheel(e: WheelEvent) {
  handleWheelHorizontal(e, fileListContainer.value);
}
</script>

<template>
  <div class="w-full max-w-6xl grid grid-cols-1 md:grid-cols-2 gap-6 mb-8 shrink-0">
    <div
      data-drop-zone="main"
      class="h-56 rounded-3xl border-2 border-dashed flex flex-col items-center justify-center transition-all duration-500 relative overflow-hidden group shadow-app-md bg-app-surface/50 backdrop-blur-sm"
      :class="isDragging ? (isInvalidDrag ? 'border-app-rose bg-app-rose/5 ring-4 ring-app-rose/5' : 'border-app-primary bg-app-primary-light ring-4 ring-app-primary/5') : 'border-app-border hover:border-app-primary/40 hover:bg-app-surface'"
    >
      <div class="flex flex-col items-center space-y-3 z-10 w-full px-6">
        <div class="w-12 h-12 flex items-center justify-center bg-app-surface rounded-2xl shadow-sm transition-all duration-500 pointer-events-none">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-app-text-mute transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
        </div>
        <p class="text-base font-bold text-app-text-dim group-hover:text-app-text transition-colors tracking-tight pointer-events-none" :class="isInvalidDrag ? 'text-app-rose' : ''">
          {{ isDragging ? (isInvalidDrag ? '包含不受支持的二进制文件' : '松开即刻解析...') : '拖入代码文件或功能块目录' }}
        </p>
        <div v-if="!isDragging" class="flex gap-2 pt-1">
          <button @click="emit('openFile')" class="px-3 py-1 bg-app-bg text-[11px] font-bold text-app-text-dim hover:text-app-text border border-app-border rounded-lg hover:border-app-primary/50 transition-all cursor-pointer shadow-sm">添加文件</button>
          <button @click="emit('openDir')" class="px-3 py-1 bg-app-bg text-[11px] font-bold text-app-text-dim hover:text-app-text border border-app-border rounded-lg hover:border-app-primary/50 transition-all cursor-pointer shadow-sm">添加目录</button>
        </div>
      </div>

      <div
        v-if="filesList.length > 0 && !isDragging"
        ref="fileListContainer"
        @wheel="handleWheel"
        class="flex items-center gap-2 w-full overflow-x-auto px-6 mt-4 z-10 custom-scrollbar-h pb-2"
      >
        <div
          v-for="(file, idx) in filesList"
          :key="idx"
          @click="emit('removeFile', idx)"
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

    <div class="h-56 rounded-3xl bg-app-surface shadow-app-md border border-app-border overflow-hidden flex flex-col group focus-within:ring-4 focus-within:ring-app-primary/5 transition-all">
      <div class="px-5 py-2.5 bg-app-bg/30 border-b border-app-border flex items-center justify-between">
        <span class="text-[11px] font-black uppercase tracking-widest text-app-text-mute group-focus-within:text-app-primary transition-colors">附加提示词 / 需求</span>
        <div class="flex gap-1">
          <div class="w-1.5 h-1.5 rounded-full bg-app-border"></div>
          <div class="w-1.5 h-1.5 rounded-full bg-app-border"></div>
        </div>
      </div>
      <textarea
        v-model="userPromptModel"
        placeholder="例如：请作为核心开发者对这些逻辑做 Code Review；或指定特定功能模块的重构需求..."
        class="w-full flex-1 px-5 py-4 resize-none text-app-text placeholder:text-app-text-mute bg-transparent focus:outline-none font-sans text-sm leading-relaxed"
      ></textarea>
    </div>
  </div>
</template>
