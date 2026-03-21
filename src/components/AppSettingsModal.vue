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
          { label: '.tsx', value: 'tsx' },
          { label: '.js', value: 'js' },
          { label: '.mjs', value: 'mjs' },
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
          { label: '.css', value: 'css' },
          { label: '.scss', value: 'scss' },
          { label: '.less', value: 'less' }
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
        label: '自动解析',
        description: '开启后，在上传、移除文件或更改相关设置时会自动更新输出结果。'
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
      {
        id: 'enableMinimization',
        type: 'switch',
        label: '开启依赖代码精简',
        description: '仅对间接依赖文件生效。自动移除函数实现体，仅保留函数签名定义，从而大幅减少上下文长度。'
      },
      {
        id: 'projectRoots',
        type: 'textarea',
        label: '指定根目录',
        description: '为空表示自动匹配。可以用逗号或换行分隔多个路径。若手动指定的根目录都不匹配，则回退到自动匹配逻辑。',
        placeholder: "D:/Projects/my-app, D:/Projects/common-lib",
        rows: 2
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
