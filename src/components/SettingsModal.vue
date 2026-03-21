<script setup lang="ts">
import { ref, watch, reactive } from 'vue';
import DynamicControl from './DynamicControl.vue';

const props = defineProps<{
  show: boolean;
  settings: any;
  groups: any[];
}>();

const emit = defineEmits(['update:show', 'save', 'cancel']);

const localSettings = reactive({ ...props.settings });
const expandedGroups = ref<string[]>([props.groups[0]?.id]);

watch(() => props.show, (newVal) => {
  if (newVal) {
    Object.assign(localSettings, props.settings);
    if (expandedGroups.value.length === 0 && props.groups.length > 0) {
      expandedGroups.value = [props.groups[0].id];
    }
  }
});

const handleSave = () => {
    emit('save', { ...localSettings });
    emit('update:show', false);
};

const handleCancel = () => {
    emit('cancel');
    emit('update:show', false);
};

const toggleGroup = (groupId: string) => {
  const index = expandedGroups.value.indexOf(groupId);
  if (index > -1) {
    expandedGroups.value.splice(index, 1);
  } else {
    expandedGroups.value.push(groupId);
  }
};

const isExpanded = (groupId: string) => expandedGroups.value.includes(groupId);
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-app-text/20 backdrop-blur-md transition-all animate-in fade-in duration-300" @click.self="handleCancel">
    <div class="bg-app-surface border border-app-border rounded-[32px] shadow-app-xl w-full max-w-2xl max-h-[85vh] flex flex-col transform transition-all overflow-hidden animate-in zoom-in-95 duration-500">
      <!-- Header Area -->
      <div class="px-8 py-6 border-b border-app-border flex justify-between items-center bg-app-surface shrink-0">
        <div class="flex flex-col">
            <h3 class="text-xl font-black text-app-text tracking-tight flex items-center">
              设置 <span class="ml-2 font-medium opacity-20 text-sm">SETTINGS</span>
            </h3>
            <span class="text-[10px] text-app-text-mute font-bold uppercase tracking-widest mt-1">定制您的解析工作流</span>
        </div>
        <button @click="handleCancel" class="text-app-text-mute hover:text-app-text transition-all p-2 rounded-2xl hover:bg-app-bg cursor-pointer group">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 group-hover:rotate-90 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Settings Content -->
      <div class="p-8 overflow-y-auto space-y-10 flex-1 custom-scrollbar">
        <template v-for="group in groups" :key="group.id">
          <div class="space-y-4 pt-2">
            <h4 
              class="text-[11px] font-black uppercase tracking-[0.25em] flex items-center justify-between cursor-pointer select-none transition-all hover:opacity-70" 
              :style="{ color: group.color || 'var(--color-app-primary)' }"
              @click="toggleGroup(group.id)"
            >
              <div class="flex items-center">
                  <span class="w-1.5 h-1.5 rounded-full mr-3 opacity-60" :style="{ backgroundColor: group.color || 'var(--color-app-primary)' }"></span>
                  {{ group.title }}
              </div>
              <svg 
                xmlns="http://www.w3.org/2000/svg" 
                class="h-3.5 w-3.5 transition-transform duration-500" 
                :class="isExpanded(group.id) ? 'rotate-180' : ''"
                fill="none" viewBox="0 0 24 24" stroke="currentColor"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M19 9l-7 7-7-7" />
              </svg>
            </h4>
            
            <div v-show="isExpanded(group.id)" class="space-y-8 animate-in slide-in-from-top-2 duration-500">
              <template v-for="item in group.items" :key="item.id">
                <DynamicControl :config="item" v-model="localSettings[item.id]" />
              </template>
            </div>
          </div>
        </template>
      </div>

      <!-- Action Footer -->
      <div class="px-8 py-5 border-t border-app-border flex justify-end shrink-0 gap-3">
        <button 
          @click="handleCancel" 
          class="px-6 py-2 text-app-text-dim hover:text-app-text font-black text-xs uppercase tracking-widest rounded-xl transition-all border border-app-border hover:bg-app-bg cursor-pointer shadow-sm active:scale-95"
        >
          取消
        </button>
        <button 
          @click="handleSave" 
          class="px-8 py-2 bg-app-text text-app-bg hover:bg-app-primary hover:text-white font-black text-xs uppercase tracking-widest rounded-xl shadow-lg transition-all active:scale-95 cursor-pointer"
        >
          保存更改
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { 
    background: var(--color-app-border); 
    border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: var(--color-app-text-mute); }
</style>
