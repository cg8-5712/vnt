<script setup lang="ts">
import { computed } from 'vue'

import { useAppStore } from '@/stores/app'
import { statusLabel } from '@/utils/format'

const app = useAppStore()

const statusClass = computed(() => `modal-status is-${app.startStatus.value}`)
</script>

<template>
  <div v-if="app.showStartLog.value" class="modal-mask">
    <div class="modal-card">
      <header class="modal-head">
        <div>
          <p class="eyebrow">Startup Trace</p>
          <h2>组网启动日志</h2>
        </div>
        <span :class="statusClass">{{ statusLabel(app.startStatus.value) }}</span>
      </header>

      <div class="log-window">
        <p v-if="app.startLogs.value.length === 0" class="log-empty">
          正在等待后续输出...
        </p>
        <p v-for="(item, index) in app.startLogs.value" :key="`${index}-${item}`" class="log-line">
          {{ item }}
        </p>
      </div>

      <footer class="modal-actions">
        <button
          v-if="app.startStatus.value === 'starting'"
          class="button ghost"
          @click="app.cancelStart()"
        >
          取消启动
        </button>
        <button
          v-else
          class="button primary"
          @click="app.closeStartLog()"
        >
          关闭
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.modal-mask {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: grid;
  place-items: center;
  padding: 1.25rem;
  background: rgba(3, 8, 15, 0.72);
  backdrop-filter: blur(16px);
}

.modal-card {
  width: min(900px, 100%);
  border-radius: 1.5rem;
  border: 1px solid var(--border-strong);
  background: linear-gradient(180deg, rgba(10, 22, 39, 0.96), rgba(5, 13, 24, 0.98));
  box-shadow: var(--shadow-soft);
}

.modal-head,
.modal-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1.35rem 1.5rem;
}

.eyebrow {
  margin: 0 0 0.3rem;
  color: var(--text-soft);
  font-size: 0.82rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.modal-head h2 {
  margin: 0;
}

.modal-status {
  border-radius: 999px;
  padding: 0.45rem 0.85rem;
  font-size: 0.9rem;
  font-weight: 600;
}

.is-running {
  background: rgba(66, 199, 154, 0.14);
  color: #8bf1c5;
}

.is-starting {
  background: rgba(246, 185, 56, 0.15);
  color: #ffd678;
}

.is-stopped {
  background: rgba(239, 90, 90, 0.15);
  color: #ff9c9c;
}

.log-window {
  min-height: 20rem;
  max-height: 30rem;
  overflow: auto;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  padding: 1rem 1.5rem 1.25rem;
  background: rgba(0, 0, 0, 0.18);
}

.log-empty {
  color: var(--text-soft);
}

.log-line {
  margin: 0;
  padding: 0.55rem 0;
  color: #cfe3ff;
  font-family: 'IBM Plex Mono', 'Cascadia Code', 'Consolas', monospace;
  font-size: 0.93rem;
  border-bottom: 1px dashed rgba(255, 255, 255, 0.06);
  word-break: break-all;
}

@media (max-width: 640px) {
  .modal-head,
  .modal-actions {
    align-items: flex-start;
    flex-direction: column;
  }

  .modal-actions .button {
    width: 100%;
  }
}
</style>
