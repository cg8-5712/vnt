<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

import StartLogModal from '@/components/StartLogModal.vue'
import AppShell from '@/layouts/AppShell.vue'
import { useAppStore } from '@/stores/app'

const app = useAppStore()

function handleVisibility() {
  app.setPageVisible(!document.hidden)
}

onMounted(() => {
  document.addEventListener('visibilitychange', handleVisibility)
  void app.bootstrap()
})

onUnmounted(() => {
  document.removeEventListener('visibilitychange', handleVisibility)
})
</script>

<template>
  <AppShell />

  <div v-if="app.notice.value" class="toast" :class="`is-${app.notice.value.kind}`">
    {{ app.notice.value.message }}
  </div>

  <StartLogModal />
</template>

<style scoped>
.toast {
  position: fixed;
  right: 1.25rem;
  bottom: 1.25rem;
  z-index: 40;
  min-width: min(26rem, calc(100vw - 2.5rem));
  border-radius: 1rem;
  border: 1px solid rgba(255, 255, 255, 0.08);
  padding: 0.95rem 1rem;
  box-shadow: var(--shadow-soft);
  backdrop-filter: blur(14px);
}

.is-info {
  background: rgba(23, 48, 80, 0.9);
}

.is-success {
  background: rgba(15, 66, 56, 0.92);
}

.is-error {
  background: rgba(87, 20, 20, 0.94);
}
</style>
