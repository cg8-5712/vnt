<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'

import StatusBadge from '@/components/StatusBadge.vue'
import { useAppStore } from '@/stores/app'
import { useDesktopShellStore } from '@/stores/desktopShell'
import { shortDeviceId } from '@/utils/format'

const app = useAppStore()
const desktopShell = useDesktopShellStore()
const route = useRoute()

const navItems = [
  { to: '/overview', label: '总览', hint: '状态与网络概况' },
  { to: '/configs', label: '配置', hint: '图形配置与文件配置' },
  { to: '/peers', label: '设备', hint: '本机与节点状态' },
  { to: '/routes', label: '路由', hint: '活动路由与服务器链路' },
]

const serverSummary = computed(() => {
  const servers = app.info.value.server_info
  const connected = servers.filter((item) => item.connected).length
  return `${connected} / ${servers.length}`
})

const activeNavItem = computed(
  () => navItems.find((item) => item.to === route.path) ?? navItems[0],
)
</script>

<template>
  <div class="app-shell">
    <header
      class="window-header panel"
      :class="{ 'is-desktop-shell': desktopShell.customTitlebarEnabled.value }"
      data-tauri-drag-region
    >
      <div
        class="window-drag-zone"
        data-tauri-drag-region
        @dblclick="desktopShell.toggleMaximizeWindow"
      >
        <div class="window-mark">V</div>

        <div class="window-copy">
          <strong>{{ activeNavItem.label }}</strong>
          <span>{{ activeNavItem.hint }}</span>
        </div>

        <div class="window-status">
          <StatusBadge :status="app.info.value.status" />
          <span class="topbar-chip">
            {{ app.info.value.ip ? `${app.info.value.ip}/${app.info.value.prefix_len}` : '未分配虚拟 IP' }}
          </span>
          <span class="topbar-chip" :class="{ online: app.isServerConnected.value }">
            服务器连接 {{ serverSummary }}
          </span>
        </div>
      </div>

      <div class="window-meta">
        <span>{{ app.info.value.name || '未命名设备' }}</span>
        <span class="topbar-chip subtle">{{ shortDeviceId(app.info.value.device_id) }}</span>
      </div>

      <div v-if="desktopShell.customTitlebarEnabled.value" class="window-actions">
        <button
          type="button"
          class="window-action"
          aria-label="最小化"
          title="最小化"
          @click="desktopShell.minimizeWindow"
        >
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path d="M3 8.5h10" />
          </svg>
        </button>

        <button
          type="button"
          class="window-action"
          :aria-label="desktopShell.maximized.value ? '还原' : '最大化'"
          :title="desktopShell.maximized.value ? '还原' : '最大化'"
          @click="desktopShell.toggleMaximizeWindow"
        >
          <svg v-if="desktopShell.maximized.value" viewBox="0 0 16 16" aria-hidden="true">
            <path d="M5 3.5h7v7" />
            <path d="M4 5.5h7v7H4z" />
          </svg>
          <svg v-else viewBox="0 0 16 16" aria-hidden="true">
            <path d="M4 4.5h8v8H4z" />
          </svg>
        </button>

        <button
          type="button"
          class="window-action close"
          aria-label="关闭"
          title="关闭"
          @click="desktopShell.requestCloseWindow"
        >
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path d="M4 4l8 8" />
            <path d="M12 4L4 12" />
          </svg>
        </button>
      </div>
    </header>

    <aside class="sidebar panel">
      <div class="brand">
        <div class="brand-mark">V</div>
        <div>
          <p>VNT Console</p>
          <span>新的 Web 控制台基座</span>
        </div>
      </div>

      <nav class="nav-list">
        <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          class="nav-item"
          :class="{ active: route.path === item.to }"
        >
          <strong>{{ item.label }}</strong>
          <span>{{ item.hint }}</span>
        </RouterLink>
      </nav>

      <div class="sidebar-foot">
        <span>Version {{ app.info.value.version || '--' }}</span>
        <span>{{ app.info.value.current_config_name || '当前没有活动配置' }}</span>
      </div>
    </aside>

    <main class="content-shell">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: minmax(240px, 280px) minmax(0, 1fr);
  grid-template-rows: auto 1fr;
  gap: 1.1rem 1.25rem;
  min-height: 100vh;
  padding: 0.85rem 1.25rem 1.25rem;
}

.window-header {
  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 1rem;
  padding: 0.8rem 0.8rem 0.8rem 1rem;
}

.window-header.is-desktop-shell {
  border-color: rgba(124, 155, 255, 0.18);
}

.window-drag-zone {
  display: flex;
  align-items: center;
  gap: 0.95rem;
  min-width: 0;
  width: 100%;
}

.window-header.is-desktop-shell .window-drag-zone {
  cursor: grab;
}

.window-mark {
  display: grid;
  width: 2.6rem;
  height: 2.6rem;
  flex: none;
  place-items: center;
  border-radius: 0.95rem;
  background: linear-gradient(140deg, rgba(66, 199, 154, 0.9), rgba(124, 155, 255, 0.92));
  color: #04121d;
  font-size: 1.08rem;
  font-weight: 900;
  box-shadow: 0 1rem 2rem rgba(66, 199, 154, 0.18);
}

.window-copy {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.12rem;
}

.window-copy strong,
.window-copy span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.window-copy strong {
  font-size: 1rem;
}

.window-copy span {
  color: var(--text-soft);
  font-size: 0.88rem;
}

.window-status,
.window-meta {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
}

.window-status {
  overflow: auto hidden;
  padding-bottom: 0.1rem;
}

.window-meta {
  justify-content: flex-end;
  color: var(--text-soft);
}

.window-header.is-desktop-shell .window-mark,
.window-header.is-desktop-shell .window-copy,
.window-header.is-desktop-shell .window-copy *,
.window-header.is-desktop-shell .window-status,
.window-header.is-desktop-shell .window-status *,
.window-header.is-desktop-shell .window-meta,
.window-header.is-desktop-shell .window-meta * {
  pointer-events: none;
}

.window-actions {
  display: flex;
  align-items: center;
  gap: 0.2rem;
}

.window-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.3rem;
  height: 2.1rem;
  border: 0;
  border-radius: 0.8rem;
  background: transparent;
  color: var(--text-main);
  transition:
    background 160ms ease,
    color 160ms ease,
    transform 160ms ease;
}

.window-action:hover {
  background: rgba(255, 255, 255, 0.07);
  transform: translateY(-1px);
}

.window-action.close:hover {
  background: rgba(239, 90, 90, 0.18);
  color: #ffb3b3;
}

.window-action svg {
  width: 1rem;
  height: 1rem;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.5;
}

.sidebar {
  display: flex;
  min-height: calc(100vh - 6.35rem);
  flex-direction: column;
  gap: 1.5rem;
}

.brand {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.brand-mark {
  display: grid;
  width: 3rem;
  height: 3rem;
  place-items: center;
  border-radius: 1rem;
  background: linear-gradient(140deg, var(--accent), var(--accent-strong));
  color: #04121d;
  font-weight: 900;
  font-size: 1.3rem;
  box-shadow: 0 1rem 2rem rgba(66, 199, 154, 0.2);
}

.brand p,
.brand span {
  margin: 0;
}

.brand p {
  font-size: 1.1rem;
  font-weight: 700;
}

.brand span {
  color: var(--text-soft);
  font-size: 0.9rem;
}

.nav-list {
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}

.nav-item {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  border-radius: 1rem;
  border: 1px solid transparent;
  padding: 0.95rem 1rem;
  text-decoration: none;
  color: var(--text-main);
  transition:
    transform 180ms ease,
    border-color 180ms ease,
    background 180ms ease;
}

.nav-item span {
  color: var(--text-soft);
  font-size: 0.9rem;
}

.nav-item:hover,
.nav-item.active {
  transform: translateY(-1px);
  border-color: rgba(124, 155, 255, 0.22);
  background: linear-gradient(180deg, rgba(14, 33, 57, 0.88), rgba(11, 22, 36, 0.7));
}

.sidebar-foot {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  color: var(--text-soft);
  font-size: 0.88rem;
}

.content-shell {
  min-width: 0;
}

.topbar-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-height: 2.15rem;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  padding: 0.52rem 0.82rem;
  background: rgba(255, 255, 255, 0.03);
  font-size: 0.9rem;
  white-space: nowrap;
}

.topbar-chip.online {
  color: #8bf1c5;
}

.topbar-chip.subtle {
  color: var(--text-soft);
}

@media (max-width: 1180px) {
  .window-header {
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .window-meta {
    grid-column: 1 / 2;
    justify-content: flex-start;
  }

  .window-actions {
    grid-row: 1 / span 2;
    grid-column: 2 / 3;
    align-self: stretch;
  }
}

@media (max-width: 1080px) {
  .app-shell {
    grid-template-columns: 1fr;
  }

  .sidebar {
    min-height: unset;
  }

  .nav-list {
    overflow: auto;
    flex-direction: row;
  }

  .nav-item {
    min-width: 11rem;
  }
}

@media (max-width: 760px) {
  .app-shell {
    padding: 0.75rem 0.95rem 1rem;
  }

  .window-header {
    grid-template-columns: 1fr;
    padding-right: 1rem;
  }

  .window-drag-zone,
  .window-meta {
    flex-wrap: wrap;
  }

  .window-actions {
    justify-content: flex-end;
  }
}

@media (max-width: 640px) {
  .window-status {
    width: 100%;
  }

  .window-copy {
    width: calc(100% - 3.55rem);
  }
}
</style>
