<script setup lang="ts">
const props = defineProps<{ message: string; storageKey: string }>()
const visible = ref(false)

function acknowledge() {
  visible.value = false
  try {
    localStorage.setItem(props.storageKey, '1')
  } catch {
    // Private browsing may reject persistence; the hint still dismisses now.
  }
}

onMounted(() => {
  try {
    visible.value = localStorage.getItem(props.storageKey) !== '1'
  } catch {
    visible.value = true
  }
})
</script>

<template>
  <span
    class="onboarding-tooltip"
    @pointerenter="acknowledge"
    @focusin="acknowledge"
    @click="acknowledge"
  >
    <slot />
    <Transition name="onboarding-hint">
      <span v-if="visible" class="onboarding-tooltip-message" role="tooltip">
        {{ message }}
      </span>
    </Transition>
  </span>
</template>

<style scoped>
.onboarding-tooltip {
  position: relative;
  display: inline-flex;
  align-items: center;
}
.onboarding-tooltip-message {
  position: absolute;
  top: calc(100% + 16px);
  left: 50%;
  z-index: 30;
  width: max-content;
  max-width: min(230px, calc(100vw - 32px));
  padding: 9px 12px;
  border: 1px solid var(--accent);
  border-radius: 7px;
  color: var(--ink);
  background: var(--surface);
  box-shadow: 0 8px 24px rgb(15 23 42 / 12%);
  font-size: 11px;
  font-weight: 600;
  line-height: 1.35;
  text-align: center;
  transform: translateX(-50%);
  animation: onboarding-hint-pulse 2.4s ease-in-out infinite;
}
.onboarding-tooltip-message::before {
  position: absolute;
  bottom: 100%;
  left: 50%;
  width: 9px;
  height: 9px;
  border-top: 1px solid var(--accent);
  border-left: 1px solid var(--accent);
  content: '';
  background: var(--surface);
  transform: translate(-50%, 5px) rotate(45deg);
}
.onboarding-hint-enter-active,
.onboarding-hint-leave-active {
  transition:
    opacity 0.16s,
    transform 0.16s;
}
.onboarding-hint-enter-from,
.onboarding-hint-leave-to {
  opacity: 0;
  transform: translate(-50%, -4px);
}
@keyframes onboarding-hint-pulse {
  50% {
    box-shadow: 0 8px 28px color-mix(in srgb, var(--accent) 22%, transparent);
  }
}
@media (prefers-reduced-motion: reduce) {
  .onboarding-tooltip-message {
    animation: none;
  }
}
</style>
