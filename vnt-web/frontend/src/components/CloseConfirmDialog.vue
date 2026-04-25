<script setup lang="ts">
import { useDesktopShellStore } from '@/stores/desktopShell'

const desktopShell = useDesktopShellStore()
</script>

<template>
  <div
    v-if="desktopShell.closePromptVisible.value"
    class="close-dialog-backdrop"
    @click="desktopShell.cancelClosePrompt"
  >
    <section
      class="close-dialog panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="close-dialog-title"
      @click.stop
    >
      <div class="close-dialog-copy">
        <p class="close-dialog-kicker">关闭确认</p>
        <h3 id="close-dialog-title">要关闭 VNT2，还是最小化到托盘？</h3>
        <p>
          选择“最小化到托盘”后，窗口会隐藏到系统托盘，应用和当前网络任务会继续运行。
        </p>
      </div>

      <label class="remember-choice">
        <input v-model="desktopShell.rememberCloseChoice.value" type="checkbox" />
        <span>记住这次选择</span>
      </label>

      <div class="close-dialog-actions">
        <button type="button" class="button ghost" @click="desktopShell.cancelClosePrompt">
          取消
        </button>
        <button
          type="button"
          class="button ghost tray-action"
          @click="desktopShell.resolveClosePrompt('minimize_to_tray')"
        >
          最小化到托盘
        </button>
        <button
          type="button"
          class="button close-action"
          @click="desktopShell.resolveClosePrompt('close')"
        >
          关闭应用
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.close-dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 90;
  display: grid;
  place-items: center;
  padding: 1.25rem;
  background: rgba(3, 8, 14, 0.68);
  backdrop-filter: blur(14px);
}

.close-dialog {
  width: min(30rem, calc(100vw - 2rem));
  padding: 1.4rem;
}

.close-dialog-copy h3,
.close-dialog-copy p {
  margin: 0;
}

.close-dialog-kicker {
  margin-bottom: 0.45rem;
  color: var(--accent);
  font-size: 0.82rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.close-dialog-copy h3 {
  font-size: 1.3rem;
}

.close-dialog-copy p:last-child {
  margin-top: 0.75rem;
  color: var(--text-soft);
}

.remember-choice {
  display: inline-flex;
  align-items: center;
  gap: 0.7rem;
  margin-top: 1.15rem;
  color: var(--text-main);
}

.remember-choice input {
  width: 1rem;
  height: 1rem;
  margin: 0;
  accent-color: var(--accent);
}

.close-dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  margin-top: 1.35rem;
  flex-wrap: wrap;
}

.tray-action {
  border-color: rgba(66, 199, 154, 0.22);
  color: #8bf1c5;
}

.close-action {
  border: 1px solid rgba(255, 123, 123, 0.22);
  background: linear-gradient(135deg, #e86363, #ff8c8c);
  color: #280606;
}

@media (max-width: 640px) {
  .close-dialog-actions {
    flex-direction: column-reverse;
  }

  .close-dialog-actions .button {
    width: 100%;
  }
}
</style>
