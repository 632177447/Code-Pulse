<script setup lang="ts">
defineProps<{
  isLoading: boolean;
  hasFiles: boolean;
}>();

const emit = defineEmits<{
  (e: "trigger"): void;
}>();
</script>

<template>
  <div class="w-full max-w-6xl mb-8 flex justify-center shrink-0">
    <button
      @click="emit('trigger')"
      :disabled="!hasFiles && !isLoading"
      class="h-14 px-10 group/btn bg-app-text hover:bg-app-primary text-app-bg font-black rounded-2xl shadow-xl shadow-app-text/10 transition-all active:scale-95 disabled:opacity-20 disabled:cursor-not-allowed flex items-center gap-3 cursor-pointer"
      :class="isLoading ? 'hover:bg-app-rose!' : ''"
    >
      <span v-if="isLoading" class="flex items-center gap-4">
        <svg class="animate-spin h-5 w-5" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
        构建中 (点击中断)...
      </span>
      <span v-else class="flex items-center gap-4 select-none">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 transition-transform group-hover/btn:translate-x-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M13 5l7 7-7 7M5 5l7 7-7 7" /></svg>
        即刻生成完整上下文
      </span>
    </button>
  </div>
</template>
