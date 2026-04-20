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
    // 阻止默认行为（防止触发其他按钮或表单提交）
    e.preventDefault();
    handleConfirm();
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
    case 'danger': return 'bg-red-500 hover:bg-red-600 focus:ring-red-500';
    case 'warning': return 'bg-amber-500 hover:bg-amber-600 focus:ring-amber-500';
    default: return 'bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-500';
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
      <div v-if="modelValue" class="fixed inset-0 z-9999 overflow-y-auto">
        <!-- 背景遮罩 -->
        <div class="fixed inset-0 bg-black/60 backdrop-blur-sm transition-opacity" @click="handleClose"></div>

        <div class="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
          <Transition
            enter-active-class="ease-out duration-300"
            enter-from-class="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
            enter-to-class="opacity-100 translate-y-0 sm:scale-100"
            leave-active-class="ease-in duration-200"
            leave-from-class="opacity-100 translate-y-0 sm:scale-100"
            leave-to-class="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
          >
            <div
              class="relative transform overflow-hidden rounded-xl bg-slate-900 border border-slate-700/50 text-left shadow-2xl transition-all sm:my-8 sm:w-full sm:max-w-md"
            >
              <div class="p-6">
                <div class="sm:flex sm:items-start">
                  <div class="mt-3 text-center sm:mt-0 sm:text-left w-full">
                    <h3 class="text-xl font-bold leading-6 text-slate-100 mb-4" id="modal-title">
                      {{ title }}
                    </h3>
                    <div class="mt-2">
                      <p class="text-sm text-slate-400 whitespace-pre-wrap leading-relaxed">
                        {{ message }}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
              
              <div class="bg-slate-800/50 px-6 py-4 flex flex-row-reverse gap-3">
                <button
                  type="button"
                  class="inline-flex min-w-[80px] justify-center rounded-lg px-4 py-2 text-sm font-semibold text-white shadow-sm transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-900"
                  :class="getTypeClasses()"
                  @click="handleConfirm"
                >
                  {{ confirmText }}
                </button>
                <button
                  type="button"
                  class="inline-flex min-w-[80px] justify-center rounded-lg bg-slate-700 px-4 py-2 text-sm font-semibold text-slate-200 shadow-sm ring-1 ring-inset ring-slate-600 hover:bg-slate-600 transition-all"
                  @click="handleClose"
                >
                  {{ cancelText }}
                </button>
              </div>
            </div>
          </Transition>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
