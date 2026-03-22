<script setup lang="ts">
import { computed } from 'vue';
import FileTreeNode from './FileTreeNode.vue';
import { getDirname } from '../utils';

const props = defineProps<{
  nodes: {path: string, content: string, abs_path: string, originId?: string}[];
}>();

const emit = defineEmits<{
  (e: 'delete', fullPath: string, absPath: string, originIds?: string[]): void;
  (e: 'uploadFiles', files: string[], destDir: string): void;
  (e: 'updateDropTarget', target: string | null): void;
}>();

const root = computed(() => {
  const tree: Record<string, any> = {};
  const sortedNodes = [...props.nodes].sort((a, b) => a.path.localeCompare(b.path));

  sortedNodes.forEach(node => {
    const parts = node.path.split('/');
    let currentLevel = tree;
    let currentPath = '';

    const absPaths: string[] = new Array(parts.length);
    let curr = node.abs_path;
    for (let i = parts.length - 1; i >= 0; i--) {
        absPaths[i] = curr;
        curr = getDirname(curr);
    }

    parts.forEach((part, index) => {
      currentPath = currentPath ? `${currentPath}/${part}` : part;
      const isDirectory = index < parts.length - 1;
      const currentAbsPath = absPaths[index];
      
      if (!currentLevel[part]) {
        currentLevel[part] = {
          name: part,
          fullPath: currentPath,
          absPath: currentAbsPath,
          originIds: [],
          isDirectory,
          isExpanded: true,
          children: {}
        };
      }
      
      if (node.originId && !currentLevel[part].originIds.includes(node.originId)) {
        currentLevel[part].originIds.push(node.originId);
      }
      
      currentLevel = currentLevel[part].children;
    });
  });

  return tree;
});

function handleDelete(fullPath: string, absPath: string, originIds?: string[]) {
  emit('delete', fullPath, absPath, originIds);
}

function handleUploadFiles(files: string[], destDir: string) {
  emit('uploadFiles', files, destDir);
}
</script>

<template>
  <div class="file-tree w-full h-full overflow-y-auto overflow-x-hidden text-sm text-app-text pr-1 pb-4">
    <div v-if="nodes.length === 0" class="flex flex-col items-center justify-center p-8 space-y-4">
        <div class="w-12 h-12 bg-app-bg rounded-2xl flex items-center justify-center border border-app-border opacity-30 shadow-sm">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>
        </div>
        <p class="text-xs text-app-text-mute font-bold italic">未检测到依赖</p>
    </div>
    <FileTreeNode
      v-for="node in root"
      :key="node.fullPath"
      :node="node"
      @delete="handleDelete"
      @upload-files="handleUploadFiles"
      @update-drop-target="(target: string | null) => emit('updateDropTarget', target)"
    />
  </div>
</template>

<style scoped>
.file-tree::-webkit-scrollbar {
  width: 4px;
}
.file-tree::-webkit-scrollbar-track {
  background: transparent;
}
.file-tree::-webkit-scrollbar-thumb {
  background: var(--color-app-border);
  border-radius: 10px;
}
.file-tree::-webkit-scrollbar-thumb:hover {
  background: var(--color-app-text-mute);
}
</style>
