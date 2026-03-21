<script setup lang="ts">
import FileTree from './FileTree.vue';

defineProps<{
    fileNodes: {path: string, content: string, abs_path: string, originId?: string}[];
}>();

const emit = defineEmits<{
    (e: 'delete', fullPath: string, absPath: string, originIds?: string[]): void;
    (e: 'uploadFiles', files: string[], destDir: string): void;
}>();
</script>

<template>
    <div class="h-full flex flex-col bg-app-surface/60 backdrop-blur-md border border-app-border rounded-3xl overflow-hidden shadow-app-md shrink-0 transition-all font-sans">
        <div class="px-5 py-3 border-b border-app-border flex items-center justify-between shrink-0 bg-app-surface">
            <span class="text-xs font-black uppercase tracking-[0.2em] text-app-text-mute">
                <span class="inline-block w-2.5 h-2.5 rounded-full bg-app-primary mr-2 opacity-60"></span>
                依赖文件树
            </span>
            <span v-if="fileNodes.length > 0" class="text-[9px] bg-app-primary-light text-app-primary font-black px-2 py-0.5 rounded-full border border-app-primary/10">{{ fileNodes.length }} 项</span>
        </div>
        <div class="flex-1 overflow-y-auto p-3 relative custom-scrollbar">
            <FileTree 
                :nodes="fileNodes" 
                @delete="(fp, ap, ids) => emit('delete', fp, ap, ids)" 
                @upload-files="(fs, dd) => emit('uploadFiles', fs, dd)"
            />
        </div>
    </div>
</template>
