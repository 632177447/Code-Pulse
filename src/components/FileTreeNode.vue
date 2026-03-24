<script setup lang="ts">
import { ref } from 'vue';

interface TreeNode {
  name: string;
  fullPath: string;
  absPath: string;
  originIds: string[];
  isDirectory: boolean;
  isExpanded: boolean;
  children: Record<string, TreeNode>;
}

const props = defineProps<{
  node: TreeNode;
}>();

const emit = defineEmits<{
  (e: 'delete', fullPath: string, absPath: string, originIds?: string[]): void;
}>();

const isExpandedLocal = ref(props.node.isExpanded);

function toggle() {
  if (props.node.isDirectory) {
    isExpandedLocal.value = !isExpandedLocal.value;
  }
}

function handleDelete() {
  emit('delete', props.node.fullPath, props.node.absPath, props.node.originIds);
}

function bubbleDelete(fullPath: string, absPath: string, originIds?: string[]) {
  emit('delete', fullPath, absPath, originIds);
}
</script>

<template>
  <div class="pl-0"
    :data-drop-path="node.isDirectory ? node.absPath : node.absPath.substring(0, Math.max(node.absPath.lastIndexOf('/'), node.absPath.lastIndexOf('\\')))"
  >
    <div 
      class="flex items-center justify-between group py-1 px-2 rounded-xl hover:bg-app-surface-hover transition-all duration-300"
      :class="node.isDirectory ? 'mb-0.5 cursor-pointer' : 'cursor-default'"
      :title="node.name"
      @click="toggle"
    >
      <div class="flex items-center space-x-2.5 overflow-hidden">
        <span class="text-app-text-mute w-4 flex justify-center group-hover:text-app-primary transition-colors">
            <template v-if="node.isDirectory">
              <svg v-if="isExpandedLocal" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
            </template>
            <template v-else>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 opacity-60" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" /></svg>
            </template>
        </span>
        <span
          class="truncate text-[13px] tracking-tight transition-colors"
          :class="node.isDirectory ? 'font-bold text-app-text italic pr-0.5' : 'text-app-text-dim font-medium group-hover:text-app-text'"
        >{{ node.name }}</span>
      </div>
      <button 
        @click.stop="handleDelete"
        class="opacity-0 group-hover:opacity-100 p-1.5 text-app-text-mute hover:text-app-rose hover:bg-app-rose/10 transition-all rounded-lg cursor-pointer"
        title="移除"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" /></svg>
      </button>
    </div>
    
    <div v-show="node.isDirectory && isExpandedLocal" class="border-l border-app-border ml-3.5 transition-all">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.fullPath"
        :node="child"
        @delete="bubbleDelete"
      />
    </div>
  </div>
</template>
