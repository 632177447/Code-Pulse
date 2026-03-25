<script setup lang="ts">
import { computed } from "vue";
import SettingsModal from "../common/SettingsModal.vue";
import { APP_SETTINGS_GROUPS } from "../../config/appSettings";
import type { AppConfig } from "../../types";

const props = defineProps<{
  show: boolean;
  settings: AppConfig;
}>();

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
  (e: "update:settings", value: AppConfig): void;
}>();

const isSettingsOpen = computed({
  get: () => props.show,
  set: (val) => emit("update:show", val)
});

const appConfig = computed({
  get: () => props.settings,
  set: (val) => emit("update:settings", val)
});
</script>

<template>
  <SettingsModal 
    v-model:show="isSettingsOpen" 
    v-model:settings="appConfig"
    :groups="APP_SETTINGS_GROUPS" 
  />
</template>
