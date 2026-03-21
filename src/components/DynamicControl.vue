<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  config: any;
  modelValue: any;
}>();

const emit = defineEmits(['update:modelValue']);

const value = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});
</script>

<template>
  <!-- Slider -->
  <div v-if="config.type === 'slider'" :class="['w-full', config.label ? 'space-y-1.5' : '']">
    <label v-if="config.label" :for="config.id" class="block text-sm font-semibold text-slate-300">{{ config.label }}</label>
    <p v-if="config.description" class="text-xs text-slate-500 pb-1">{{ config.description }}</p>
    <div class="flex items-center space-x-3">
      <input 
        :id="config.id" 
        type="range" 
        v-model.number="value"
        :min="config.min || 0" 
        :max="config.max || 100"
        class="flex-1 w-full h-2 bg-slate-700 rounded-md appearance-none cursor-pointer accent-blue-500"
      />
      <span class="text-lg font-mono font-bold text-blue-400 w-8 text-center">{{ value }}</span>
    </div>
  </div>

  <!-- Input -->
  <div v-else-if="config.type === 'input'" :class="['w-full', config.label ? 'space-y-1.5 mt-3.5' : 'mt-1.5']">
    <label v-if="config.label" :for="config.id" class="block text-sm font-semibold text-slate-300">{{ config.label }}</label>
    <p v-if="config.description" class="text-xs text-slate-500 pb-1">{{ config.description }}</p>
    <input 
      :id="config.id"
      type="text" 
      v-model="value"
      class="w-full bg-slate-900 border border-slate-600 rounded-md px-2.5 py-1.5 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-shadow"
      :placeholder="config.placeholder || ''"
    />
  </div>

  <!-- Textarea -->
  <div v-else-if="config.type === 'textarea'" :class="['w-full', config.label ? 'space-y-1.5 mt-3.5' : 'mt-1.5']">
    <label v-if="config.label" :for="config.id" class="block text-sm font-semibold text-slate-300">{{ config.label }}</label>
    <p v-if="config.description" class="text-xs text-slate-500 pb-1">{{ config.description }}</p>
    <textarea 
      :id="config.id"
      v-model="value"
      :rows="config.rows || 3"
      class="w-full bg-slate-900 border border-slate-600 rounded-md px-2.5 py-1.5 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-shadow resize-y custom-scrollbar"
      :placeholder="config.placeholder || ''"
    ></textarea>
  </div>

  <!-- Switch -->
  <label v-else-if="config.type === 'switch'" :class="['flex items-center justify-between cursor-pointer p-2.5 bg-slate-900/50 rounded-md border border-slate-700/50 hover:bg-slate-900/70 transition-colors', config.label ? 'mt-3.5' : 'mt-1.5']">
    <div class="flex flex-col">
      <span v-if="config.label" class="text-sm font-semibold text-slate-200">{{ config.label }}</span>
      <span v-if="config.description" class="text-xs text-slate-500 mt-0.5">{{ config.description }}</span>
    </div>
    <div class="relative">
      <input type="checkbox" v-model="value" class="sr-only peer">
      <div class="w-11 h-6 bg-slate-600 rounded-full peer peer-checked:bg-blue-500 peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
    </div>
  </label>

  <!-- Radio -->
  <div v-else-if="config.type === 'radio'" :class="['inline-block w-full', config.label ? 'space-y-1.5 mt-3.5' : 'mt-1.5']">
    <label v-if="config.label" class="block text-sm font-semibold text-slate-300">{{ config.label }}</label>
    <p v-if="config.description" class="text-xs text-slate-500 pb-1">{{ config.description }}</p>
    <div class="flex gap-6">
      <label v-for="opt in config.options" :key="opt.value" class="flex items-center space-x-2 cursor-pointer">
        <input type="radio" :value="opt.value" v-model="value" class="w-4 h-4 text-blue-600 bg-slate-700 border-slate-600">
        <span class="text-sm text-slate-300">{{ opt.label }}</span>
      </label>
    </div>
  </div>

  <!-- Checkbox -->
  <div v-else-if="config.type === 'checkbox'" :class="['inline-block w-full', config.label ? 'space-y-1.5 mt-3.5' : 'mt-1.5']">
    <label v-if="config.label" class="block text-sm font-semibold text-slate-300">{{ config.label }}</label>
    <p v-if="config.description" class="text-xs text-slate-500 pb-1">{{ config.description }}</p>
    <div class="flex flex-wrap gap-6">
      <label v-for="opt in config.options" :key="opt.value" class="flex items-center space-x-2 cursor-pointer">
        <input type="checkbox" :value="opt.value" v-model="value" class="w-4 h-4 text-blue-600 bg-slate-700 border-slate-600 rounded">
        <span class="text-sm text-slate-300">{{ opt.label }}</span>
      </label>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #475569;
}
</style>
