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
        id: 'includedTypes',
        type: 'checkbox',
        label: '目标文件格式',
        description: '选择你需要包含和提取的目标代码文件格式。',
        options: [
          { label: '.vue', value: 'vue' },
          { label: '.ts', value: 'ts' },
          { label: '.js', value: 'js' },
          { label: '.rs', value: 'rs' },
          { label: '.py', value: 'py' },
          { label: '.go', value: 'go' },
          { label: '.java', value: 'java' },
          { label: '.kt', value: 'kt' },
          { label: '.cpp', value: 'cpp' },
          { label: '.c', value: 'c' },
          { label: '.h', value: 'h' },
          { label: '.cs', value: 'cs' },
          { label: '.php', value: 'php' },
          { label: '.rb', value: 'rb' },
          { label: '.json', value: 'json' },
          { label: '.md', value: 'md' },
          { label: '.html', value: 'html' },
          { label: '.css', value: 'css' }
        ]
      },
      {
        id: 'customIncludedTypes',
        type: 'input',
        placeholder: '在此输入其他自定义后缀名 (例如: sh, yaml, xml)'
      },
      {
        id: 'customPrompt',
        type: 'textarea',
        label: '自定义提示词',
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
        id: 'ignoreExts',
        type: 'textarea',
        label: '忽略后缀或目录',
        description: '可通过英文逗号或换行分隔，支持使用 * 通配符匹配（如 *.log, test-* 等）。',
        placeholder: '.git, node_modules, dist, target, build',
        rows: 4
      },
      {
        id: 'ignoreDeepParse',
        type: 'textarea',
        label: '不进行深层解析的忽略配置',
        description: '可通过英文逗号或换行分隔，支持使用 *。匹配到的文件将作为终端上下文，但不继续解析依赖。',
        placeholder: "package.json, tsconfig.json, vite.config.ts, README.md, *.test.ts",
        rows: 3
      },
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
