<script setup lang="ts">
interface Props {
  icon?: string;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'danger';
  title?: string;
}

withDefaults(defineProps<Props>(), {
  icon: '✕',
  size: 'md',
  variant: 'default',
  title: ''
});

const emit = defineEmits(['click']);
</script>

<template>
  <button 
    class="icon-button" 
    :class="[size, variant]" 
    :title="title"
    @click.stop="emit('click')"
  >
    <span class="icon-content">{{ icon }}</span>
  </button>
</template>

<style scoped>
.icon-button {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border: none;
  color: #666;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  padding: 0;
  flex-shrink: 0;
}

/* Sizes */
.sm {
  width: 28px;
  height: 28px;
  font-size: 0.8rem;
  border-radius: 6px;
}

.md {
  width: 36px;
  height: 36px;
  font-size: 1.1rem;
  border-radius: 50%;
}

.lg {
  width: 44px;
  height: 44px;
  font-size: 1.4rem;
  border-radius: 50%;
}

/* Variants */
.default:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
  transform: rotate(90deg);
}

.danger:hover {
  background: rgba(255, 77, 77, 0.2);
  color: #ff4d4d;
  transform: scale(1.1);
}

.icon-content {
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.icon-button:active {
  transform: scale(0.9);
}
</style>
