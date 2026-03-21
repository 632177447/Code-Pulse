<script setup lang="ts">
import DynamicControl from "./DynamicControl.vue";

const props = defineProps<{
  show: boolean;
  settings: Record<string, any>;
  groups: any[];
}>();

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
  (e: "update:settings", value: Record<string, any>): void;
}>();

function close() {
  emit("update:show", false);
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm">
    <div class="bg-slate-800 border border-slate-700 rounded-2xl shadow-xl w-full max-w-2xl max-h-[90vh] flex flex-col transform transition-all">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-700 flex justify-between items-center bg-slate-800/50 shrink-0">
        <h3 class="text-xl font-bold text-slate-100 flex items-center">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 mr-2 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          偏好设置 (Settings)
        </h3>
        <button @click="close" class="text-slate-400 hover:text-slate-200 transition-colors p-1 rounded-md hover:bg-slate-700">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 overflow-y-auto space-y-8 flex-1 custom-scrollbar">
        <template v-for="group in groups" :key="group.id">
          <div class="space-y-4 pt-2">
            <h4 class="text-sm font-semibold uppercase tracking-widest border-b border-slate-700/50 pb-2" :class="group.colorClass || 'text-blue-400'">
              {{ group.title }}
            </h4>
            
            <template v-for="item in group.items" :key="item.id">
              <DynamicControl :config="item" v-model="settings[item.id]" />
            </template>
          </div>
        </template>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-700 flex justify-end shrink-0">
        <button 
          @click="close" 
          class="px-5 py-2 bg-blue-600 hover:bg-blue-500 text-white font-semibold rounded-lg shadow-md transition-colors"
        >
          完成 (Done)
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #475569;
}
</style>
