<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import AppSettingsModal from "./components/AppSettingsModal.vue";
import LayoutGenerateControl from "./components/layout/LayoutGenerateControl.vue";
import LayoutHeader from "./components/layout/LayoutHeader.vue";
import LayoutInputSection from "./components/layout/LayoutInputSection.vue";
import LayoutResultsSection from "./components/layout/LayoutResultsSection.vue";
import LayoutVersionInfo from "./components/layout/LayoutVersionInfo.vue";
import { APP_CONFIG_STORAGE_KEY, createDefaultAppConfig } from "./config/appSettings";
import { useContextGeneration } from "./composables/useContextGeneration";
import { useFileListManager } from "./composables/useFileListManager";
import { copyToClipboard } from "./utils";
import type { FileListItem } from "./types";
import pkg from "../package.json";

const version = pkg.version;

const filesList = ref<FileListItem[]>([]);
const isSettingsOpen = ref(false);
const userPrompt = ref("");
const isEditing = ref(false);
const sidebarWidth = ref(320);
const isResizingSidebar = ref(false);
let sidebarResizeStartX = 0;
let sidebarResizeStartWidth = 0;
const SIDEBAR_MIN_WIDTH = 260;
const SIDEBAR_MAX_WIDTH = 520;

const appConfig = reactive(createDefaultAppConfig());

watch(appConfig, (newVal) => {
  localStorage.setItem(APP_CONFIG_STORAGE_KEY, JSON.stringify(newVal));
}, { deep: true });

const {
  clearGeneratedContext,
  fileNodes,
  handleNodeDelete,
  isLoading,
  outputContext,
  processPaths,
  scheduleProcessPaths,
  stopProcessing,
  totalCharacters
} = useContextGeneration({
  appConfig,
  filesList,
  userPrompt
});

const {
  handleTreeUploadFiles,
  isDragging,
  isInvalidDrag,
  removeFile,
  triggerDirInput,
  triggerFileInput
} = useFileListManager({
  autoGenerate: computed(() => appConfig.autoGenerate),
  clearGeneratedContext,
  filesList,
  isLoading,
  processPaths,
  scheduleProcessPaths
});

onMounted(() => {
  const savedConfig = localStorage.getItem(APP_CONFIG_STORAGE_KEY);
  if (savedConfig) {
    try {
      const parsed = JSON.parse(savedConfig);
      Object.assign(appConfig, parsed);
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }

  const savedSidebarWidth = localStorage.getItem("sidebarWidth");
  if (savedSidebarWidth) {
    const parsedWidth = Number(savedSidebarWidth);
    if (!Number.isNaN(parsedWidth)) {
      sidebarWidth.value = Math.min(SIDEBAR_MAX_WIDTH, Math.max(SIDEBAR_MIN_WIDTH, parsedWidth));
    }
  }
});

onUnmounted(() => {
  stopResizeSidebar();
});

async function copyToClipboardAll() {
  await copyToClipboard(outputContext.value);
}

function toggleEdit() {
  isEditing.value = !isEditing.value;
}

function startResizeSidebar(e: MouseEvent) {
  isResizingSidebar.value = true;
  sidebarResizeStartX = e.clientX;
  sidebarResizeStartWidth = sidebarWidth.value;
  document.body.classList.add("sidebar-resizing");
  window.addEventListener("mousemove", handleResizeSidebar);
  window.addEventListener("mouseup", stopResizeSidebar);
}

function handleResizeSidebar(e: MouseEvent) {
  if (!isResizingSidebar.value) return;
  const nextWidth = Math.min(SIDEBAR_MAX_WIDTH, Math.max(SIDEBAR_MIN_WIDTH, sidebarResizeStartWidth + e.clientX - sidebarResizeStartX));
  sidebarWidth.value = nextWidth;
}

function stopResizeSidebar() {
  if (!isResizingSidebar.value) return;
  isResizingSidebar.value = false;
  document.body.classList.remove("sidebar-resizing");
  window.removeEventListener("mousemove", handleResizeSidebar);
  window.removeEventListener("mouseup", stopResizeSidebar);
  localStorage.setItem("sidebarWidth", String(sidebarWidth.value));
}
</script>

<template>
  <main class="h-screen flex flex-col items-center p-6 selection:bg-app-primary/10 relative overflow-y-auto">
    <LayoutHeader @open-settings="isSettingsOpen = true" />

    <LayoutInputSection
      v-model:userPrompt="userPrompt"
      :isDragging="isDragging"
      :isInvalidDrag="isInvalidDrag"
      :filesList="filesList"
      @open-file="triggerFileInput"
      @open-dir="triggerDirInput"
      @remove-file="removeFile"
    />

    <LayoutGenerateControl
      :isLoading="isLoading"
      :hasFiles="filesList.length > 0"
      @trigger="isLoading ? stopProcessing() : processPaths(filesList.map(f => f.path))"
    />

    <LayoutResultsSection
      v-model:outputContext="outputContext"
      :fileNodes="fileNodes"
      :sidebarWidth="sidebarWidth"
      :totalCharacters="totalCharacters"
      :isEditing="isEditing"
      :isLoading="isLoading"
      :enableMinimization="appConfig.enableMinimization"
      @delete="(fp, ap, ids) => handleNodeDelete(fp, ap, ids)"
      @toggle-edit="toggleEdit"
      @copy-output="copyToClipboardAll"
      @start-resize-sidebar="startResizeSidebar"
      @upload-files="handleTreeUploadFiles"
    />

    <AppSettingsModal 
      v-model:show="isSettingsOpen" 
      :settings="appConfig"
      @update:settings="val => Object.assign(appConfig, val)"
    />

    <LayoutVersionInfo :version="version" />
  </main>
</template>

<style>
/* 全局外观优化 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
  cursor: default;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--color-app-border);
  border-radius: 10px;
  cursor: default;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--color-app-text-mute);
}

.custom-scrollbar-h {
  scroll-behavior: smooth;
}

.custom-scrollbar-h::-webkit-scrollbar {
  height: 4px;
}

.custom-scrollbar-h::-webkit-scrollbar-track {
  background: transparent;
  margin: 0 40px;
}

.custom-scrollbar-h::-webkit-scrollbar-thumb {
  background: var(--color-app-border);
}

/* 拖拽反馈样式 */
.drop-node-hover {
  background-color: var(--color-app-primary-light) !important;
  box-shadow: 0 0 0 1px var(--color-app-primary) inset;
  border-radius: 12px;
}

.drop-node-hover-invalid {
  background-color: rgba(244, 63, 94, 0.08) !important;
  box-shadow: 0 0 0 1px var(--color-app-rose) inset;
  border-radius: 12px;
}

body.sidebar-resizing {
  cursor: col-resize;
  user-select: none;
}

.shadow-output-text {
  box-sizing: border-box;
  tab-size: 4;
  white-space: pre-wrap;
  overflow-wrap: break-word;
  word-break: break-word;
}
</style>
