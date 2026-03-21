<script setup lang="ts">
import { computed } from 'vue';
import FileTreeNode from './FileTreeNode.vue';

interface TreeNode {
  name: string;
  fullPath: string;
  absPath: string;
  isDirectory: boolean;
  isExpanded: boolean;
  children: Record<string, TreeNode>;
}

const props = defineProps<{
  nodes: {path: string, content: string, abs_path: string}[];
}>();

const emit = defineEmits<{
  (e: 'delete', fullPath: string): void;
  (e: 'uploadFiles', files: string[], destDir: string): void;
  (e: 'updateDropTarget', target: string | null): void;
}>();

// 将 flat paths 构建成树状对象
// 将 flat paths 构建成树状对象
const root = computed(() => {
  const tree: Record<string, TreeNode> = {};
  
  // 确保按字典序或者层级显示比较好，但路径一般已经是排好序的
  const sortedNodes = [...props.nodes].sort((a, b) => a.path.localeCompare(b.path));

  function getDirname(p: string) {
      const lastSlash = Math.max(p.lastIndexOf('/'), p.lastIndexOf('\\'));
      return lastSlash > -1 ? p.substring(0, lastSlash) : p;
  }

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
          isDirectory,
          isExpanded: true, // 默认展开
          children: {}
        };
      } else if (isDirectory && !currentLevel[part].isDirectory) {
          // 在极少数情况下，可能是覆盖导致的异常，确保类型为目录
          currentLevel[part].isDirectory = true;
          currentLevel[part].absPath = currentAbsPath;
      }
      currentLevel = currentLevel[part].children;
    });
  });

  return tree;
});

function handleDelete(fullPath: string) {
  emit('delete', fullPath);
}

function handleUploadFiles(files: string[], destDir: string) {
  emit('uploadFiles', files, destDir);
}
</script>

<template>
  <div class="file-tree w-full h-full overflow-y-auto overflow-x-hidden text-sm text-slate-300 pr-1 pb-2">
    <div v-if="nodes.length === 0" class="text-slate-500 text-xs italic p-4 text-center">
        暂无依赖文件
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
  width: 6px;
}
.file-tree::-webkit-scrollbar-track {
  background: transparent;
}
.file-tree::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 3px;
}
.file-tree::-webkit-scrollbar-thumb:hover {
  background: #475569;
}
</style>
