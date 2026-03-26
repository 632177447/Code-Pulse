<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import DependencyTreeSidebar from "../file-tree/DependencyTreeSidebar.vue";
import { getDisplayBasePath, replacePathWithSymbol } from "../../utils";

const props = defineProps<{
  fileNodes: {path: string, content: string, abs_path: string, originId?: string}[];
  sidebarWidth: number;
  outputContext: string;
  totalCharacters: number;
  isEditing: boolean;
  isLoading: boolean;
  enableMinimization: boolean;
  optimizePathDisplay: boolean;
}>();

const emit = defineEmits<{
  (e: "delete", fullPath: string, absPath: string, originIds?: string[]): void;
  (e: "uploadFiles", files: string[], destDir: string): void;
  (e: "toggleEdit"): void;
  (e: "copyOutput"): void;
  (e: "startResizeSidebar", event: MouseEvent): void;
  (e: "update:outputContext", value: string): void;
}>();

const outputContextModel = computed({
  get: () => props.outputContext,
  set: (val) => emit("update:outputContext", val)
});

const outputAreaRef = ref<HTMLTextAreaElement | null>(null);
const outputShadowMarkerRef = ref<HTMLSpanElement | null>(null);
const outputShadowState = ref<{ beforeText: string; afterText: string } | null>(null);
const displayBasePath = computed(() => {
  if (!props.optimizePathDisplay) {
    return "";
  }
  return getDisplayBasePath(props.fileNodes.map(node => node.path));
});

watch(() => props.isEditing, async (isEditing) => {
  if (!isEditing) return;
  const currentScrollTop = outputAreaRef.value?.scrollTop ?? 0;
  await nextTick();
  requestAnimationFrame(() => {
    if (!outputAreaRef.value) return;
    outputAreaRef.value.focus({ preventScroll: true });
    outputAreaRef.value.scrollTop = currentScrollTop;
  });
});

function clearOutputShadow() {
  outputShadowState.value = null;
}

async function handleNodeSelect(fullPath: string) {
  if (props.isLoading || !props.outputContext) return;

  const anchorPath = props.optimizePathDisplay 
    ? replacePathWithSymbol(fullPath, displayBasePath.value)
    : fullPath;
  const anchor = `[FILE PATH]: ${anchorPath}`;
  const anchorIndex = props.outputContext.indexOf(anchor);
  if (anchorIndex === -1) return;

  outputShadowState.value = {
    beforeText: props.outputContext.slice(0, anchorIndex),
    afterText: props.outputContext.slice(anchorIndex),
  };

  await nextTick();
  const outputArea = outputAreaRef.value;
  const marker = outputShadowMarkerRef.value;
  if (!outputArea || !marker) {
    clearOutputShadow();
    return;
  }

  const maxScrollTop = Math.max(outputArea.scrollHeight - outputArea.clientHeight, 0);
  const targetScrollTop = Math.min(Math.max(marker.offsetTop - 28, 0), maxScrollTop);

  outputArea.scrollTo({
    top: targetScrollTop,
    behavior: "smooth"
  });

  clearOutputShadow();

  if (props.isEditing) {
    requestAnimationFrame(() => {
      outputArea.focus({ preventScroll: true });
      outputArea.setSelectionRange(anchorIndex, anchorIndex);
    });
    return;
  }

  requestAnimationFrame(() => {
    if (!props.isEditing) {
      outputArea.blur();
    }
  });
}
</script>

<template>
  <div class="w-full max-w-6xl flex-1 flex min-h-[420px] gap-5.5 mb-2">
    <div class="h-full shrink-0" :style="{ width: `${sidebarWidth}px` }">
      <DependencyTreeSidebar
        :fileNodes="fileNodes"
        class="w-full"
        @delete="(fp, ap, ids) => emit('delete', fp, ap, ids)"
        @select="handleNodeSelect"
        @upload-files="(fs, dd) => emit('uploadFiles', fs, dd)"
      />
    </div>

    <div
      class="relative shrink-0 w-1.5 -mx-3 cursor-col-resize group"
      @mousedown.prevent="emit('startResizeSidebar', $event)"
    >
      <div
        class="absolute inset-y-0 my-2 left-1/2 -translate-x-1/2 w-1 rounded-full bg-app-border/70 rounded-full transition-colors opacity-0 group-hover:opacity-100"
      ></div>
    </div>

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
            @click="emit('toggleEdit')"
            class="p-1.5 bg-app-surface hover:bg-app-surface-hover text-app-text-dim hover:text-app-primary rounded-lg transition-all border border-app-border cursor-pointer"
            :class="isEditing ? 'bg-app-primary/5 border-app-primary/30 ring-2 ring-app-primary/5 text-app-primary' : ''"
            :title="isEditing ? '应用更改' : '快速编辑'"
          >
            <svg v-if="!isEditing" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
          </button>
          <button
            v-if="outputContext"
            @click="emit('copyOutput')"
            class="p-2 py-1.5 bg-app-text hover:bg-app-text/90 text-app-bg text-[11px] font-black rounded-lg transition-all active:scale-95 flex items-center gap-1.5 shadow-sm cursor-pointer"
            title="复制全部"
          >
            复制
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>
          </button>
        </div>
      </div>
      <div class="relative overflow-hidden w-full h-full flex-1 rounded-b-3xl py-2 px-2">
        <div
          v-show="outputShadowState"
          class="absolute inset-0 pointer-events-none opacity-0 overflow-y-scroll overflow-x-hidden p-4 font-mono text-[13px] leading-relaxed z-[-1] shadow-output-text"
          aria-hidden="true"
        >
          {{ outputShadowState?.beforeText }}<span ref="outputShadowMarkerRef"></span>{{ outputShadowState?.afterText }}
        </div>
        <textarea
          ref="outputAreaRef"
          :readonly="!isEditing"
          v-model="outputContextModel"
          placeholder="所有的上下文内容都在这里准备就绪..."
          class="w-full h-full flex-1 p-2 focus:outline-none font-mono text-[13px] leading-relaxed resize-none transition-all duration-300 z-0 bg-transparent text-app-text placeholder:text-app-text-mute"
          :class="[isEditing ? 'bg-app-primary/5 cursor-text!' : 'cursor-default']"
        ></textarea>
      </div>

      <div v-if="isLoading" class="absolute inset-0 bg-app-surface/60 backdrop-blur-md flex flex-col items-center justify-center z-20 pointer-events-none transition-all">
        <div class="relative w-12 h-12 mb-4">
          <div class="absolute inset-0 rounded-full border-4 border-app-primary/10"></div>
          <div class="absolute inset-0 rounded-full border-4 border-t-app-primary animate-spin"></div>
        </div>
        <p v-if="enableMinimization" class="text-app-primary font-black tracking-widest text-xs uppercase animate-pulse">正在深度解析并压缩代码...</p>
        <p v-else class="text-app-primary font-black tracking-widest text-xs uppercase animate-pulse">正在解析依赖...</p>
      </div>
    </div>
  </div>
</template>
