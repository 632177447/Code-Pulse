<script setup lang="ts">
import { ref } from 'vue';

interface TreeNode {
  name: string;
  fullPath: string;
  absPath: string;
  isDirectory: boolean;
  isExpanded: boolean;
  children: Record<string, TreeNode>;
}

const props = defineProps<{
  node: TreeNode;
}>();

const emit = defineEmits<{
  (e: 'delete', fullPath: string): void;
}>();

const isExpandedLocal = ref(props.node.isExpanded);

function toggle() {
  if (props.node.isDirectory) {
    isExpandedLocal.value = !isExpandedLocal.value;
  }
}

function handleDelete() {
  emit('delete', props.node.fullPath);
}

function bubbleDelete(fullPath: string) {
  emit('delete', fullPath);
}
</script>

<template>
  <div class="pl-2 mt-0.5 select-none"
    :data-drop-path="node.isDirectory ? node.absPath : node.absPath.substring(0, Math.max(node.absPath.lastIndexOf('/'), node.absPath.lastIndexOf('\\')))"
  >
    <div 
      class="flex items-center justify-between group py-1 px-1.5 rounded-md hover:bg-slate-700/50 cursor-pointer transition-colors"
      @click="toggle"
    >
      <div class="flex items-center space-x-1.5 overflow-hidden">
        <span class="text-slate-400 w-4 flex justify-center">
            <template v-if="node.isDirectory">
              <svg v-if="isExpandedLocal" xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
            </template>
            <template v-else>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" /></svg>
            </template>
        </span>
        <span class="truncate text-[13px]" :class="node.isDirectory ? 'font-medium text-slate-200' : 'text-slate-400 font-mono'">{{ node.name }}</span>
      </div>
      <button 
        @click.stop="handleDelete"
        class="opacity-0 group-hover:opacity-100 p-1 mr-1 text-slate-500 hover:text-red-400 hover:bg-slate-600/50 transition-all rounded shadow-sm"
        title="移除此节点 (右侧上下文将自动更新)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
      </button>
    </div>
    
    <div v-show="node.isDirectory && isExpandedLocal" class="border-l border-slate-700/50 ml-1.5">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.fullPath"
        :node="child"
        @delete="bubbleDelete"
      />
    </div>
  </div>
</template>
