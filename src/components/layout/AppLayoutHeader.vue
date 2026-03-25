<script setup lang="ts">
const emit = defineEmits<{
  (e: "openSettings"): void;
}>();
</script>

<template>
  <div class="w-full max-w-6xl flex justify-between items-center mb-8 shrink-0">
    <div class="flex flex-col">
      <h1 class="text-3xl font-black text-app-text tracking-tight flex items-center">
        CodePulse <span class="ml-2 font-medium opacity-20 text-xl">文脉</span>
      </h1>
      <p class="text-app-text-dim text-sm mt-1 font-medium italic opacity-70">
        一键解析项目依赖，构建完整提示词上下文
      </p>
    </div>
    <button
      @click="emit('openSettings')"
      class="p-2.5 bg-app-surface hover:bg-app-surface-hover text-app-text-dim hover:text-app-primary rounded-xl transition-all shadow-app-md border border-app-border cursor-pointer group"
      title="设置 (Settings)"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 group-hover:rotate-45 transition-transform duration-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </button>
  </div>
</template>
