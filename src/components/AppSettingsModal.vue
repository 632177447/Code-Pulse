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
    title: '基础设置',
    colorClass: 'text-blue-400',
    items: [
      {
        id: 'maxDepth',
        type: 'slider',
        label: '递归解析深度',
        description: '设置文件解析依赖扫描的层级数。设置得越高，包含的相关文件越多。',
        min: 0,
        max: 10
      },
      {
        id: 'ignoreExts',
        type: 'input',
        label: '忽略后缀或目录',
        description: '通过英文逗号分隔，匹配的目录或文件将不被解析。',
        placeholder: '.git, node_modules, dist'
      },
      {
        id: 'ignoreDeepParse',
        type: 'input',
        label: '不进行深层解析的忽略后缀或目录',
        description: '通过英文逗号分隔，匹配的目录或文件将不进行深层递归解析。',
        placeholder: ''
      },
      {
        id: 'customPrompt',
        type: 'textarea',
        label: '自定义提示词首部',
        description: '可以在生成的上下文前面插入所需的引导信息。',
        placeholder: '请输入自定义提示词...',
        rows: 3
      }
    ]
  },
  {
    id: 'advanced',
    title: '解析选项',
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
        label: '默认解析模式',
        options: [
          { label: '普通模式', value: 'normal' },
          { label: '严格模式', value: 'strict' },
          { label: '智能过滤', value: 'smart' }
        ]
      },
      {
        id: 'includedTypes',
        type: 'checkbox',
        label: '目标文件格式',
        description: '选择你需要包含和提取的目标代码文件格式。',
        options: [
          { label: '.vue', value: 'vue' },
          { label: '.ts', value: 'ts' },
          { label: '.js', value: 'js' },
          { label: '.rs', value: 'rs' },
          { label: '.json', value: 'json' },
          { label: '.md', value: 'md' },
          { label: '.html', value: 'html' },
          { label: '.css', value: 'css' }
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
