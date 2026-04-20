<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';

interface Props {
  modelValue: boolean;
  title?: string;
  message?: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'primary' | 'danger' | 'warning';
}

const props = withDefaults(defineProps<Props>(), {
  title: '温馨提示',
  message: '确定要执行此操作吗？',
  confirmText: '确定',
  cancelText: '取消',
  type: 'primary',
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'confirm'): void;
  (e: 'cancel'): void;
}>();

const handleClose = () => {
  emit('update:modelValue', false);
  emit('cancel');
};

const handleConfirm = () => {
  emit('update:modelValue', false);
  emit('confirm');
};

const handleKeydown = (e: KeyboardEvent) => {
  if (props.modelValue && e.key === 'Enter') {
    e.preventDefault();
    handleConfirm();
  } else if (props.modelValue && e.key === 'Escape') {
    e.preventDefault();
    handleClose();
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

const getTypeClasses = () => {
  switch (props.type) {
    case 'danger': return 'bg-app-rose text-white hover:opacity-90 shadow-app-rose/20';
    case 'warning': return 'bg-amber-500 text-white hover:opacity-90 shadow-amber-500/20';
    default: return 'bg-app-text text-app-bg hover:bg-app-primary hover:text-white shadow-app-primary/10';
  }
};
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="ease-out duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="ease-in duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="modelValue" class="fixed inset-0 z-9999 flex items-center justify-center p-4 sm:p-0">
        
        <div class="absolute inset-0 bg-app-text/20 backdrop-blur-md transition-opacity" @click="handleClose"></div>

        <Transition
          enter-active-class="ease-out duration-300"
          enter-from-class="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
          enter-to-class="opacity-100 translate-y-0 sm:scale-100"
          leave-active-class="ease-in duration-200"
          leave-from-class="opacity-100 translate-y-0 sm:scale-100"
          leave-to-class="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
          appear
        >
          <div
            v-if="modelValue"
            class="relative bg-app-surface border border-app-border rounded-[32px] shadow-app-xl w-full max-w-md flex flex-col transform transition-all overflow-hidden sm:my-8"
          >
            
            <div class="px-7 py-5 border-b border-app-border flex justify-between items-center bg-app-surface shrink-0">
              <h3 class="text-xl font-black text-app-text tracking-tight flex items-center" id="modal-title">
                {{ title }}
              </h3>
              <button @click="handleClose" class="text-app-text-mute hover:text-app-text transition-all p-2 rounded-2xl hover:bg-app-bg cursor-pointer group -mr-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 group-hover:rotate-90 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            
            <div class="p-7 flex-1">
              <p class="text-sm text-app-text-mute whitespace-pre-wrap leading-relaxed font-medium">
                {{ message }}
              </p>
            </div>
            
            
            <div class="px-7 py-5 border-t border-app-border flex justify-end shrink-0 gap-3 bg-app-surface">
              <button
                type="button"
                class="px-6 py-2.5 text-app-text-dim hover:text-app-text font-black text-xs uppercase tracking-[0.2em] rounded-2xl transition-all border border-app-border hover:bg-app-bg cursor-pointer shadow-sm active:scale-95"
                @click="handleClose"
              >
                {{ cancelText }}
              </button>
              <button
                type="button"
                class="px-8 py-2.5 font-black text-xs uppercase tracking-[0.2em] rounded-2xl transition-all active:scale-95 cursor-pointer shadow-xl"
                :class="getTypeClasses()"
                @click="handleConfirm"
              >
                {{ confirmText }}
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
