<script setup lang="ts">
import { computed } from "vue";
import SettingsModal from "./SettingsModal.vue";

const props = defineProps<{
  show: boolean;
  settings: Record<string, any>;
}>();

const emit = defineEmits(["update:show", "update:settings"]);

const isSettingsOpen = computed({
  get: () => props.show,
  set: (val) => emit("update:show", val)
});

const appConfig = computed({
  get: () => props.settings,
  set: (val) => emit("update:settings", val)
});

const settingsGroups = [
  {
    id: 'basic',
    title: '基础设置 (Basic Options)',
    colorClass: 'text-blue-400',
    items: [
      {
        id: 'maxDepth',
        type: 'slider',
        label: '递归解析深度 (Recursive Parsing Depth)',
        description: '设置文件解析依赖扫描的层级数。设置得越高，包含的相关文件越多。',
        min: 0,
        max: 10
      },
      {
        id: 'ignoreExts',
        type: 'input',
        label: '忽略后缀或目录 (Ignore Patterns)',
        description: '通过英文逗号分隔，匹配的目录或文件将不被解析。',
        placeholder: '.git, node_modules, dist'
      },
      {
        id: 'customPrompt',
        type: 'textarea',
        label: '自定义提示词首部 (Custom Prompt Header)',
        description: '可以在生成的上下文前面插入所需的引导信息。',
        placeholder: '请输入自定义提示词...',
        rows: 3
      }
    ]
  },
  {
    id: 'advanced',
    title: '解析选项 (Parse Options)',
    colorClass: 'text-purple-400',
    items: [
      {
        id: 'generateTree',
        type: 'switch',
        label: '顶部生成文件树结构',
        description: '结果中最开头将包含解析目录的层级树状图。'
      },
      {
        id: 'autoGenerate',
        type: 'switch',
        label: '选择文件后立即解析',
        description: '如果关闭，在拖拽或选择路径后需要手动点击“生成”按钮。'
      },
      {
        id: 'parseMode',
        type: 'radio',
        label: '默认解析模式 (Parse Mode)',
        options: [
          { label: '普通模式', value: 'normal' },
          { label: '严格模式', value: 'strict' },
          { label: '智能过滤', value: 'smart' }
        ]
      },
      {
        id: 'includedTypes',
        type: 'checkbox',
        label: '特殊偏好的文件类型 (Included Prefs)',
        options: [
          { label: '.vue', value: 'vue' },
          { label: '.ts', value: 'ts' },
          { label: '.rs', value: 'rs' },
          { label: '.json', value: 'json' }
        ]
      }
    ]
  }
];
</script>

<template>
  <SettingsModal 
    v-model:show="isSettingsOpen" 
    v-model:settings="appConfig"
    :groups="settingsGroups" 
  />
</template>
