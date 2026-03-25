import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { onMounted, onUnmounted, ref, type Ref } from "vue";
import { createFileList, createFileListItem, isBinaryFile } from "../utils";
import type { FileListItem } from "../types";

interface UseFileListManagerOptions {
  autoGenerate: Ref<boolean>;
  filesList: Ref<FileListItem[]>;
  isLoading: Ref<boolean>;
  processPaths: (paths: string[]) => Promise<void>;
  scheduleProcessPaths: (delay?: number, reason?: "analysis" | "remove") => void;
  clearGeneratedContext: () => void;
}

export function useFileListManager({
  autoGenerate,
  filesList,
  isLoading,
  processPaths,
  scheduleProcessPaths,
  clearGeneratedContext
}: UseFileListManagerOptions) {
  const isDragging = ref(false);
  const isInvalidDrag = ref(false);
  let isDraggingInvalidFiles = false;
  let lastHighlightedNode: HTMLElement | null = null;
  let unlistenDragDrop: (() => void) | undefined;

  function clearHighlightedNode() {
    if (!lastHighlightedNode) return;
    lastHighlightedNode.classList.remove("drop-node-hover");
    lastHighlightedNode.classList.remove("drop-node-hover-invalid");
    lastHighlightedNode = null;
  }

  function setFiles(paths: string[]) {
    filesList.value = createFileList(paths);
  }

  function normalizeSelectedPaths(selected: string | string[] | null): string[] {
    if (!selected) {
      return [];
    }
    return Array.isArray(selected) ? selected : [selected];
  }

  async function applySelectedPaths(paths: string[]) {
    if (paths.length === 0) return;
    setFiles(paths);
    if (autoGenerate.value) {
      await processPaths(paths);
    }
  }

  async function triggerFileInput() {
    const selected = await open({
      multiple: true,
      directory: false,
    });

    await applySelectedPaths(normalizeSelectedPaths(selected));
  }

  async function triggerDirInput() {
    const selected = await open({
      multiple: true,
      directory: true,
    });

    await applySelectedPaths(normalizeSelectedPaths(selected));
  }

  async function handleTreeUploadFiles(files: string[], destDir: string) {
    try {
      isLoading.value = true;
      const newPaths = await invoke<string[]>("copy_files_to_dest", {
        sources: files,
        destDir: destDir
      });

      if (newPaths && newPaths.length > 0) {
        for (const path of newPaths) {
          if (!filesList.value.find(file => file.path === path)) {
            filesList.value.push(createFileListItem(path));
          }
        }

        await processPaths(filesList.value.map(file => file.path));
        return;
      }

      isLoading.value = false;
    } catch (e) {
      console.error("Upload failed:", e);
      alert(`上传失败: ${e}`);
      isLoading.value = false;
    }
  }

  function removeFile(index: number) {
    filesList.value.splice(index, 1);

    if (!autoGenerate.value) return;

    if (filesList.value.length === 0) {
      clearGeneratedContext();
      return;
    }

    scheduleProcessPaths(300, "remove");
  }

  onMounted(async () => {
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event: any) => {
      const { type, position, paths } = event.payload;

      const dpi = window.devicePixelRatio;
      const localX = position ? position.x / dpi : 0;
      const localY = position ? position.y / dpi : 0;

      if (type === "over" || type === "enter") {
        if (position) {
          const element = document.elementFromPoint(localX, localY);
          const dropZone = element?.closest("[data-drop-zone=\"main\"]");
          const nodeZone = element?.closest("[data-drop-path]") as HTMLElement | null;

          isDragging.value = !!dropZone;

          if (type === "enter" && paths && paths.length > 0) {
            isDraggingInvalidFiles = (paths as string[]).some(path => isBinaryFile(path));
          }

          isInvalidDrag.value = isDragging.value && isDraggingInvalidFiles;

          if (nodeZone !== lastHighlightedNode) {
            clearHighlightedNode();
            if (nodeZone) {
              nodeZone.classList.add(isInvalidDrag.value ? "drop-node-hover-invalid" : "drop-node-hover");
            }
            lastHighlightedNode = nodeZone;
          }
        }
      } else if (type === "leave") {
        isDragging.value = false;
        isInvalidDrag.value = false;
        isDraggingInvalidFiles = false;
        clearHighlightedNode();
      } else if (type === "drop") {
        isDragging.value = false;
        isInvalidDrag.value = false;
        isDraggingInvalidFiles = false;
        clearHighlightedNode();

        const element = position ? document.elementFromPoint(localX, localY) : null;
        const dropZone = element?.closest("[data-drop-zone=\"main\"]");
        const nodeZone = element?.closest("[data-drop-path]") as HTMLElement | null;

        if (paths && paths.length > 0) {
          const validPaths = (paths as string[]).filter(path => !isBinaryFile(path));
          const hasBlocked = (paths as string[]).length > validPaths.length;

          if (hasBlocked && validPaths.length === 0) {
            return;
          } else if (hasBlocked) {
            console.warn("一些非文本文件已被自动跳过");
          }

          if (dropZone) {
            setFiles(validPaths);
            if (autoGenerate.value) {
              void processPaths(validPaths);
            }
          } else if (nodeZone) {
            const destDir = nodeZone.dataset.dropPath;
            if (destDir) {
              void handleTreeUploadFiles(validPaths, destDir);
            }
          }
        }
      }
    });
  });

  onUnmounted(() => {
    clearHighlightedNode();
    if (unlistenDragDrop) {
      unlistenDragDrop();
    }
  });

  return {
    handleTreeUploadFiles,
    isDragging,
    isInvalidDrag,
    removeFile,
    triggerDirInput,
    triggerFileInput
  };
}
