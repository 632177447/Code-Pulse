<script setup lang="ts">
import { ref } from 'vue';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

defineProps<{
  version: string;
}>();

const isChecking = ref(false);
const statusText = ref('');

const handleCheckUpdate = async () => {
  if (isChecking.value) return;
  
  isChecking.value = true;
  statusText.value = '正在检查更新...';
  
  try {
    const update = await check();
    if (update) {
      statusText.value = `发现新版本 v${update.version}`;
      if (confirm(`发现新版本 v${update.version}，是否立即更新？\n\n更新说明：\n${update.body || '无'}`)) {
        statusText.value = '正在下载并安装...';
        await update.downloadAndInstall();
        await relaunch();
      }
    } else {
      statusText.value = '已是最新版本';
      setTimeout(() => {
        statusText.value = '';
      }, 3000);
    }
  } catch (err) {
    console.error('Update check failed:', err);
    statusText.value = '检查更新失败';
    setTimeout(() => {
      statusText.value = '';
    }, 3000);
  } finally {
    isChecking.value = false;
  }
};
</script>

<template>
  <div 
    class="fixed bottom-2 right-3 flex items-center gap-2 text-[10px] font-mono select-none z-50 transition-all duration-300"
    :class="[isChecking ? 'opacity-100' : 'opacity-30 hover:opacity-100']"
  >
    <span v-if="statusText" class="text-app-text-accent animate-pulse">{{ statusText }}</span>
    <button 
      @click="handleCheckUpdate"
      class="cursor-pointer hover:underline disabled:cursor-wait text-app-text-mute"
      :disabled="isChecking"
    >
      v{{ version }}
    </button>
  </div>
</template>
