import { computed, onUnmounted, ref, watch, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ContextWorker from "../workers/context.worker.ts?worker";
import { normalizePath } from "../utils";
import type { AppConfig, FileListItem, FileNode } from "../types";

type ProcessReason = "analysis" | "remove";

interface UseContextGenerationOptions {
  appConfig: AppConfig;
  filesList: Ref<FileListItem[]>;
  userPrompt: Ref<string>;
}

export function useContextGeneration({ appConfig, filesList, userPrompt }: UseContextGenerationOptions) {
  const outputContext = ref("");
  const fileNodes = ref<FileNode[]>([]);
  const isLoading = ref(false);
  const currentRequestId = ref(0);
  let removeFileDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let analysisDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  const contextWorker = new ContextWorker();

  contextWorker.onmessage = (e: MessageEvent<{ requestId?: number; content: string }>) => {
    const { requestId, content } = e.data;
    if (requestId !== undefined && requestId !== currentRequestId.value) {
      return;
    }
    outputContext.value = content;
    isLoading.value = false;
  };

  contextWorker.onerror = (e) => {
    console.error("Context worker error:", e);
    isLoading.value = false;
  };

  const totalCharacters = computed(() => {
    return outputContext.value ? outputContext.value.length : 0;
  });

  const finalIncludedTypes = computed(() => {
    const customIncludedTypes = appConfig.customIncludedTypes
      .split(/[,\n]/)
      .map(s => s.trim())
      .filter(s => s.length > 0)
      .map(s => s.startsWith(".") ? s.substring(1) : s);

    return Array.from(new Set([...appConfig.includedTypes, ...customIncludedTypes]));
  });

  const analysisSettingsTrigger = computed(() => {
    return JSON.stringify({
      maxDepth: appConfig.maxDepth,
      includedTypes: [...appConfig.includedTypes].sort(),
      customIncludedTypes: finalIncludedTypes.value.filter(type => !appConfig.includedTypes.includes(type)),
      ignoreExts: appConfig.ignoreExts,
      ignoreDeepParse: appConfig.ignoreDeepParse,
      projectRoots: appConfig.projectRoots
        .split(/[,\n\r]/)
        .map(s => s.trim())
        .filter(s => s.length > 0),
      enableMinimization: appConfig.enableMinimization,
      minimizationThreshold: appConfig.minimizationThreshold,
      minimizationDepthThreshold: appConfig.minimizationDepthThreshold,
    });
  });

  const uiFormattingTrigger = computed(() => {
    return JSON.stringify({
      customPrompt: appConfig.customPrompt,
      generateTree: appConfig.generateTree,
      highlightPrimaryFiles: appConfig.highlightPrimaryFiles
    });
  });

  function clearProcessTimers() {
    if (removeFileDebounceTimer) {
      clearTimeout(removeFileDebounceTimer);
      removeFileDebounceTimer = null;
    }
    if (analysisDebounceTimer) {
      clearTimeout(analysisDebounceTimer);
      analysisDebounceTimer = null;
    }
  }

  function scheduleProcessPaths(delay = 0, reason: ProcessReason = "analysis") {
    const timer = reason === "analysis" ? analysisDebounceTimer : removeFileDebounceTimer;
    if (timer) {
      clearTimeout(timer);
    }

    const nextTimer = setTimeout(() => {
      void processPaths(filesList.value.map(file => file.path));
    }, delay);

    if (reason === "analysis") {
      analysisDebounceTimer = nextTimer;
      return;
    }
    removeFileDebounceTimer = nextTimer;
  }

  watch(analysisSettingsTrigger, () => {
    if (appConfig.autoGenerate && filesList.value.length > 0) {
      scheduleProcessPaths(300);
    }
  });

  watch(uiFormattingTrigger, () => {
    if (appConfig.autoGenerate && fileNodes.value.length > 0) {
      updateOutputContext();
    }
  });

  async function processPaths(paths: string[]) {
    if (paths.length === 0) return;

    clearProcessTimers();

    const requestId = ++currentRequestId.value;
    isLoading.value = true;

    try {
      const result = await invoke<Array<{ path: string; content: string; abs_path: string }>>("generate_context", {
        paths: paths,
        maxDepth: appConfig.maxDepth,
        generateTree: appConfig.generateTree,
        ignoreExts: appConfig.ignoreExts,
        ignoreDeepParse: appConfig.ignoreDeepParse,
        includedTypes: finalIncludedTypes.value,
        projectRoots: appConfig.projectRoots,
        enableMinimization: appConfig.enableMinimization,
        minimizationThreshold: appConfig.minimizationThreshold,
        minimizationDepthThreshold: appConfig.minimizationDepthThreshold,
      });

      if (requestId !== currentRequestId.value) return;

      fileNodes.value = result.map(node => {
        const normalizedNodeAbsPath = normalizePath(node.abs_path);
        const origin = filesList.value.find(file => {
          const normalizedFilePath = normalizePath(file.path);
          return normalizedNodeAbsPath === normalizedFilePath || normalizedNodeAbsPath.startsWith(normalizedFilePath + "/");
        });

        return { ...node, originId: origin?.id };
      });

      updateOutputContext(requestId);
    } catch (error) {
      if (requestId !== currentRequestId.value) return;
      console.error("Failed to generate context:", error);
      outputContext.value = `Error: ${error}`;
      isLoading.value = false;
    }
  }

  function stopProcessing() {
    currentRequestId.value++;
    isLoading.value = false;
    invoke("abort_generate_context").catch(e => console.error("Failed to abort:", e));
  }

  function updateOutputContext(requestId?: number) {
    if (fileNodes.value.length === 0) {
      outputContext.value = "";
      return;
    }

    const nextRequestId = requestId ?? ++currentRequestId.value;
    contextWorker.postMessage({
      requestId: nextRequestId,
      fileNodes: fileNodes.value.map(node => ({
        path: node.path,
        content: node.content,
        isPrimary: Boolean(node.originId)
      })),
      generateTree: appConfig.generateTree,
      highlightPrimaryFiles: appConfig.highlightPrimaryFiles,
      customPrompt: appConfig.customPrompt,
      userPrompt: userPrompt.value,
      longContextThreshold: appConfig.minimizationThreshold,
    });
  }

  function handleNodeDelete(fullPath: string, _absPath?: string, originIds?: string[]) {
    fileNodes.value = fileNodes.value.filter(node =>
      !(node.path === fullPath || node.path.startsWith(fullPath + "/"))
    );

    if (originIds && originIds.length > 0) {
      filesList.value = filesList.value.filter(file => !originIds.includes(file.id));
    }

    updateOutputContext();
  }

  function clearGeneratedContext() {
    fileNodes.value = [];
    updateOutputContext();
  }

  onUnmounted(() => {
    clearProcessTimers();
    contextWorker.terminate();
  });

  return {
    clearGeneratedContext,
    fileNodes,
    handleNodeDelete,
    isLoading,
    outputContext,
    processPaths,
    scheduleProcessPaths,
    stopProcessing,
    totalCharacters
  };
}
