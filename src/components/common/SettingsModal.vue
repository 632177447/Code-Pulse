<script setup lang="ts">
import { ref, watch, reactive } from 'vue';
import DynamicControl from './DynamicControl.vue';

const props = defineProps<{
  show: boolean;
  settings: any;
  groups: any[];
}>();

const emit = defineEmits(['update:show', 'update:settings', 'save', 'cancel']);

const STORAGE_KEY = 'settings_expanded_groups';

const localSettings = reactive({ ...props.settings });

// 初始化展开状态：从 localStorage 读取或默认全展开
const getInitialExpandedGroups = () => {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    try {
      return JSON.parse(saved);
    } catch (e) {
      console.error('Failed to parse expanded groups:', e);
    }
  }
  // 默认全部展开
  return props.groups.map(g => g.id);
};

const expandedGroups = ref<string[]>(getInitialExpandedGroups());

// 监听展开状态并持久化
watch(expandedGroups, (newVal) => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(newVal));
}, { deep: true });

watch(() => props.show, (newVal) => {
  if (newVal) {
    Object.assign(localSettings, props.settings);
  }
});

const handleSave = () => {
    emit('update:settings', { ...localSettings });
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

// 根据 item.visible 判断是否渲染该控件
const isItemVisible = (item: any): boolean => {
  if (item.visible === undefined) return true;
  if (typeof item.visible === 'function') return item.visible(localSettings);
  return !!item.visible;
};
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-app-text/20 backdrop-blur-md transition-all animate-in fade-in duration-300" @click.self="handleCancel">
    <div class="bg-app-surface border border-app-border rounded-[32px] shadow-app-xl w-full max-w-2xl max-h-[85vh] flex flex-col transform transition-all overflow-hidden animate-in zoom-in-95 duration-500">
      <!-- Header Area -->
      <div class="px-6 py-3 border-b border-app-border flex justify-between items-center bg-app-surface shrink-0">
        <div class="flex flex-col">
            <h3 class="text-xl font-black text-app-text tracking-tight flex items-center">
              设置 <span class="ml-2 font-medium opacity-20 text-sm">SETTINGS</span>
            </h3>
        </div>
        <button @click="handleCancel" class="text-app-text-mute hover:text-app-text transition-all p-2 rounded-2xl hover:bg-app-bg cursor-pointer group">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 group-hover:rotate-90 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Settings Content -->
      <div class="p-6 overflow-y-auto space-y-10 flex-1 custom-scrollbar">
        <template v-for="group in groups" :key="group.id">
          <div class="space-y-2 mb-1">
            <h4 
              class="text-[14px] font-black uppercase tracking-[0.25em] flex items-center justify-between py-2 mb-2 cursor-pointer select-none transition-all hover:opacity-70" 
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
            
            <div v-show="isExpanded(group.id)" class="space-y-6 animate-in slide-in-from-top-2 duration-500">
              <template v-for="item in group.items" :key="item.id">
                <DynamicControl v-if="isItemVisible(item)" :config="item" v-model="localSettings[item.id]" />
              </template>
            </div>
          </div>
        </template>
      </div>

      <!-- Action Footer -->
      <div class="px-8 py-4 border-t border-app-border flex justify-end shrink-0 gap-3">
        <button 
          @click="handleCancel" 
          class="px-8 py-2.5 text-app-text-dim hover:text-app-text font-black text-xs uppercase tracking-widest rounded-xl transition-all border border-app-border hover:bg-app-bg cursor-pointer shadow-sm active:scale-95"
        >
          取消
        </button>
        <button 
          @click="handleSave" 
          class="px-10 py-2.5 bg-app-text text-app-bg hover:bg-app-primary hover:text-white font-black text-xs uppercase tracking-widest rounded-xl shadow-lg transition-all active:scale-95 cursor-pointer"
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
