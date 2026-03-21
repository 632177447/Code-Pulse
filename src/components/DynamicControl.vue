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
  <div v-if="config.type === 'slider'" :class="['w-full', config.label ? 'space-y-2' : '']">
    <div class="flex items-center justify-between">
        <label v-if="config.label" :for="config.id" class="block text-[11px] font-black uppercase tracking-widest text-app-text-dim">{{ config.label }}</label>
        <span class="text-xs font-black font-mono text-app-primary bg-app-primary-light px-2 py-0.5 rounded-lg border border-app-primary/10 tracking-widest">{{ value }}</span>
    </div>
    <p v-if="config.description" class="text-[11px] text-app-text-mute pb-1 leading-relaxed italic opacity-80">{{ config.description }}</p>
    <div class="flex items-center space-x-4">
      <input 
        :id="config.id" 
        type="range" 
        v-model.number="value"
        :min="config.min || 0" 
        :max="config.max || 100"
        class="flex-1 w-full h-1 bg-app-border rounded-full appearance-none cursor-pointer accent-app-primary"
      />
    </div>
  </div>

  <!-- Input -->
  <div v-else-if="config.type === 'input'" :class="['w-full', config.label ? 'space-y-2' : '']">
    <label v-if="config.label" :for="config.id" class="block text-[11px] font-black uppercase tracking-widest text-app-text-dim">{{ config.label }}</label>
    <p v-if="config.description" class="text-[11px] text-app-text-mute pb-1 leading-relaxed italic opacity-80">{{ config.description }}</p>
    <div class="relative group">
        <input 
          :id="config.id"
          type="text" 
          v-model="value"
          class="w-full bg-app-surface border border-app-border rounded-xl px-4 py-2.5 text-[13px] text-app-text font-medium placeholder:text-app-text-mute focus:outline-none focus:border-app-primary/50 focus:ring-4 focus:ring-app-primary/5 transition-all shadow-app-sm"
          :placeholder="config.placeholder || ''"
        />
        <div class="absolute inset-y-0 right-4 flex items-center opacity-0 group-focus-within:opacity-30 transition-opacity">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" /></svg>
        </div>
    </div>
  </div>

  <!-- Textarea -->
  <div v-else-if="config.type === 'textarea'" :class="['w-full', config.label ? 'space-y-2' : '']">
    <label v-if="config.label" :for="config.id" class="block text-[11px] font-black uppercase tracking-widest text-app-text-dim">{{ config.label }}</label>
    <p v-if="config.description" class="text-[11px] text-app-text-mute pb-1 leading-relaxed italic opacity-80">{{ config.description }}</p>
    <textarea 
      :id="config.id"
      v-model="value"
      :rows="config.rows || 3"
      class="w-full bg-app-surface border border-app-border rounded-2xl px-4 py-2.5 text-[13px] text-app-text font-medium placeholder:text-app-text-mute focus:outline-none focus:border-app-primary/50 focus:ring-4 focus:ring-app-primary/5 transition-all shadow-app-sm resize-y custom-scrollbar leading-relaxed"
      :placeholder="config.placeholder || ''"
    ></textarea>
  </div>

  <!-- Switch -->
  <label v-else-if="config.type === 'switch'" class="flex items-center justify-between cursor-pointer group py-3 px-4 bg-app-bg rounded-2xl border border-app-border/40 hover:border-app-primary/30 transition-all duration-500">
    <div class="flex flex-col pr-6">
      <span v-if="config.label" class="text-[11px] font-black uppercase tracking-widest text-app-text-dim group-hover:text-app-text transition-colors">{{ config.label }}</span>
      <span v-if="config.description" class="text-[11px] text-app-text-mute italic mt-1 leading-relaxed opacity-80">{{ config.description }}</span>
    </div>
    <div class="relative shrink-0">
      <input type="checkbox" v-model="value" class="sr-only peer">
      <div class="w-10 h-5.5 bg-app-border rounded-full peer peer-checked:bg-app-primary transition-all after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:rounded-full after:h-4.5 after:w-4.5 after:transition-all after:shadow-app-sm peer-checked:after:translate-x-4.5"></div>
    </div>
  </label>

  <!-- Radio -->
  <div v-else-if="config.type === 'radio'" :class="[config.label ? 'space-y-2' : '']">
    <label v-if="config.label" class="block text-[11px] font-black uppercase tracking-widest text-app-text-dim">{{ config.label }}</label>
    <p v-if="config.description" class="text-[11px] text-app-text-mute pb-1 leading-relaxed italic opacity-80">{{ config.description }}</p>
    <div class="flex flex-wrap gap-4">
      <label v-for="opt in config.options" :key="opt.value" class="flex items-center space-x-2.5 cursor-pointer group">
        <div class="relative w-4.5 h-4.5 flex items-center justify-center">
            <input type="radio" :value="opt.value" v-model="value" class="w-full h-full accent-app-primary bg-app-surface border-app-border transition-all">
        </div>
        <span class="text-xs font-black uppercase tracking-widest text-app-text-mute group-hover:text-app-text transition-colors">{{ opt.label }}</span>
      </label>
    </div>
  </div>

  <!-- Checkbox -->
  <div v-else-if="config.type === 'checkbox'" :class="[config.label ? 'space-y-3' : '']">
    <label v-if="config.label" class="block text-[11px] font-black uppercase tracking-widest text-app-text-dim">{{ config.label }}</label>
    <p v-if="config.description" class="text-[11px] text-app-text-mute pb-1 leading-relaxed italic opacity-80">{{ config.description }}</p>
    
    <!-- Grid Layout Mode -->
    <div v-if="config.layout === 'grid'" 
      class="grid gap-2"
      :style="{ gridTemplateColumns: `repeat(${config.columns || 4}, minmax(0, 1fr))` }"
    >
      <label 
        v-for="opt in config.options" 
        :key="opt.value" 
        class="flex items-center space-x-2 px-3 py-2 rounded-xl border transition-all cursor-pointer group shadow-app-sm"
        :class="Array.isArray(value) && value.includes(opt.value) 
          ? 'bg-app-primary-light border-app-primary/40 ring-2 ring-app-primary/5' 
          : 'bg-app-surface border-app-border hover:border-app-border-focus hover:bg-app-bg'"
        :title="opt.label"
      >
        <input type="checkbox" :value="opt.value" v-model="value" class="w-3.5 h-3.5 accent-app-primary bg-app-surface border-app-border rounded">
        <span class="text-[10px] uppercase font-black tracking-widest text-app-text-mute group-hover:text-app-text truncate transition-colors">{{ opt.label }}</span>
      </label>
    </div>

    <!-- Default Multi-line wrap mode -->
    <div v-else class="flex flex-wrap gap-6">
      <label v-for="opt in config.options" :key="opt.value" class="flex items-center space-x-2.5 cursor-pointer group">
        <input type="checkbox" :value="opt.value" v-model="value" class="w-4 h-4 accent-app-primary bg-app-surface border-app-border rounded transition-all">
        <span class="text-[11px] font-black uppercase tracking-widest text-app-text-mute group-hover:text-app-text transition-colors">{{ opt.label }}</span>
      </label>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar { width: 3px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { 
    background: var(--color-app-border); 
    border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: var(--color-app-text-mute); }
</style>
