<script setup lang="ts">
import FileTree from './FileTree.vue';

defineProps<{
    fileNodes: { path: string, content: string, abs_path: string, originId?: string }[];
}>();

const emit = defineEmits<{
    (e: 'delete', fullPath: string, absPath: string, originIds?: string[]): void;
    (e: 'uploadFiles', files: string[], destDir: string): void;
    (e: 'updateDropTarget', target: string | null): void;
}>();

function handleNodeDelete(fullPath: string, absPath: string, originIds?: string[]) {
    emit('delete', fullPath, absPath, originIds);
}

function handleUploadFiles(files: string[], destDir: string) {
    emit('uploadFiles', files, destDir);
}
</script>

<template>
    <div class="w-1/3 min-w-[250px] flex flex-col bg-[#0d1117] border border-slate-700 rounded-xl overflow-hidden shadow-inner shrink-0 min-h-full">
        <div class="px-4 py-2.5 bg-slate-800/80 backdrop-blur-md border-b border-slate-700 flex items-center justify-between z-10">
            <span class="text-sm font-semibold text-slate-400">依赖文件树</span>
            <span v-if="fileNodes.length > 0" class="text-[10px] bg-slate-700 w-fit text-slate-300 font-mono px-2 py-0.5 rounded-full">{{ fileNodes.length }} 项</span>
        </div>
        <div class="flex-1 overflow-y-auto p-2 relative custom-scrollbar">
            <FileTree 
                :nodes="fileNodes" 
                @delete="handleNodeDelete" 
                @upload-files="handleUploadFiles" 
                @update-drop-target="(target: string | null) => emit('updateDropTarget', target)"
            />
        </div>
    </div>
</template>
