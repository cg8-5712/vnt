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

function handleTitlebarMouseDown(event: MouseEvent) {
  // 只在左键点击且不是按钮时触发拖动
  if (event.button !== 0) return

  const target = event.target as HTMLElement
  // 如果点击的是按钮或按钮内的元素，不触发拖动
  if (target.closest('.titlebar-actions')) return

  desktopShell.startDragging()
}
</script>

<template>
  <div class="app-shell">
    <header
      class="window-header"
      :class="{ 'is-desktop-shell': desktopShell.customTitlebarEnabled.value }"
    >
      <div
        v-if="desktopShell.customTitlebarEnabled.value"
        class="titlebar"
        @mousedown="handleTitlebarMouseDown"
      >
        <div class="titlebar-left">
          <div class="app-icon">V</div>
          <div class="app-title">
            <span class="title-main">VNT Console</span>
            <span class="title-sub">{{ activeNavItem.label }}</span>
          </div>
        </div>

        <div class="titlebar-center">
          <div>
            <StatusBadge :status="app.info.value.status" />
          </div>
          <span class="status-chip">
            {{ app.info.value.ip ? `${app.info.value.ip}/${app.info.value.prefix_len}` : '未分配IP' }}
          </span>
          <span class="status-chip" :class="{ connected: app.isServerConnected.value }">
            {{ serverSummary }} 服务器
          </span>
        </div>

        <div class="titlebar-actions">
          <button
            type="button"
            class="titlebar-btn minimize"
            aria-label="最小化"
            @click="desktopShell.minimizeWindow"
          >
            <svg viewBox="0 0 12 12">
              <line x1="2" y1="6" x2="10" y2="6" />
            </svg>
          </button>

          <button
            type="button"
            class="titlebar-btn maximize"
            :aria-label="desktopShell.maximized.value ? '还原' : '最大化'"
            @click="desktopShell.toggleMaximizeWindow"
          >
            <svg v-if="desktopShell.maximized.value" viewBox="0 0 12 12">
              <rect x="3" y="2" width="7" height="7" />
              <polyline points="2,3 2,10 9,10" />
            </svg>
            <svg v-else viewBox="0 0 12 12">
              <rect x="2.5" y="2.5" width="7" height="7" />
            </svg>
          </button>

          <button
            type="button"
            class="titlebar-btn close"
            aria-label="关闭"
            @click="desktopShell.requestCloseWindow"
          >
            <svg viewBox="0 0 12 12">
              <line x1="3" y1="3" x2="9" y2="9" />
              <line x1="9" y1="3" x2="3" y2="9" />
            </svg>
          </button>
        </div>
      </div>

      <div v-else class="header-content panel">
        <div class="header-left">
          <div class="window-mark">V</div>
          <div class="window-copy">
            <strong>{{ activeNavItem.label }}</strong>
            <span>{{ activeNavItem.hint }}</span>
          </div>
        </div>

        <div class="header-status">
          <StatusBadge :status="app.info.value.status" />
          <span class="topbar-chip">
            {{ app.info.value.ip ? `${app.info.value.ip}/${app.info.value.prefix_len}` : '未分配虚拟 IP' }}
          </span>
          <span class="topbar-chip" :class="{ online: app.isServerConnected.value }">
            服务器连接 {{ serverSummary }}
          </span>
        </div>

        <div class="header-meta">
          <span>{{ app.info.value.name || '未命名设备' }}</span>
          <span class="topbar-chip subtle">{{ shortDeviceId(app.info.value.device_id) }}</span>
        </div>
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
  padding: 0 1.25rem 1.25rem;
}

.window-header {
  grid-column: 1 / -1;
}

.window-header.is-desktop-shell {
  margin: 0 -1.25rem;
  padding: 0;
}

/* Custom Titlebar Styles */
.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 2.5rem;
  background: linear-gradient(180deg, rgba(8, 18, 32, 0.95), rgba(5, 12, 22, 0.98));
  border-bottom: 1px solid rgba(124, 155, 255, 0.12);
  backdrop-filter: blur(12px);
  user-select: none;
  cursor: default;
}

.titlebar:active {
  cursor: move;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  padding-left: 0.75rem;
  min-width: 0;
  flex: 1;
}

.app-icon {
  display: grid;
  place-items: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: 0.45rem;
  background: linear-gradient(135deg, #42c79a, #7c9bff);
  color: #04121d;
  font-size: 0.85rem;
  font-weight: 900;
  flex-shrink: 0;
}

.app-title {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  min-width: 0;
  font-size: 0.8rem;
}

.title-main {
  color: #e8f0ff;
  font-weight: 600;
  white-space: nowrap;
}

.title-sub {
  color: rgba(255, 255, 255, 0.45);
  font-weight: 400;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.titlebar-center {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0 1rem;
  min-width: 0;
  flex: 0 1 auto;
}

.status-chip {
  display: inline-flex;
  align-items: center;
  height: 1.65rem;
  padding: 0 0.6rem;
  border-radius: 0.4rem;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.65);
  font-size: 0.75rem;
  white-space: nowrap;
}

.status-chip.connected {
  background: rgba(66, 199, 154, 0.08);
  border-color: rgba(66, 199, 154, 0.2);
  color: #8bf1c5;
}

.titlebar-actions {
  display: flex;
  align-items: stretch;
  height: 100%;
  flex-shrink: 0;
}

.titlebar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3rem;
  height: 100%;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.15s ease;
}

.titlebar-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}

.titlebar-btn:active {
  background: rgba(255, 255, 255, 0.12);
}

.titlebar-btn.close:hover {
  background: #e81123;
  color: #fff;
}

.titlebar-btn.close:active {
  background: #c50b1c;
}

.titlebar-btn svg {
  width: 0.75rem;
  height: 0.75rem;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

/* Non-desktop header styles */
.header-content {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 1rem;
  padding: 0.82rem 1rem;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.95rem;
  min-width: 0;
}

.window-mark {
  display: grid;
  width: 2.65rem;
  height: 2.65rem;
  flex: none;
  place-items: center;
  border-radius: 0.98rem;
  background: linear-gradient(140deg, rgba(66, 199, 154, 0.92), rgba(124, 155, 255, 0.95));
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

.header-status,
.header-meta {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
}

.header-status {
  overflow: auto hidden;
  padding-bottom: 0.1rem;
}

.header-meta {
  justify-content: flex-end;
  color: var(--text-soft);
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

.sidebar {
  display: flex;
  min-height: calc(100vh - 6.4rem);
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

@media (max-width: 1180px) {
  .titlebar-center {
    display: none;
  }

  .header-content {
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .header-meta {
    grid-column: 1 / 2;
    justify-content: flex-start;
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
    padding: 0 0.95rem 1rem;
  }

  .window-header.is-desktop-shell {
    margin: 0 -0.95rem;
  }

  .titlebar-left {
    flex: 0 1 auto;
  }

  .title-sub {
    display: none;
  }

  .header-content {
    grid-template-columns: 1fr;
    padding-right: 1rem;
  }

  .header-left,
  .header-meta {
    flex-wrap: wrap;
  }
}

@media (max-width: 640px) {
  .header-status {
    width: 100%;
  }

  .window-copy {
    width: calc(100% - 3.6rem);
  }

  .titlebar-btn {
    width: 2.75rem;
  }
}
</style>
