<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'

import StatusBadge from '@/components/StatusBadge.vue'
import { useAppStore } from '@/stores/app'
import { shortDeviceId } from '@/utils/format'

const app = useAppStore()
const route = useRoute()

const navItems = [
  { to: '/overview', label: '总览', hint: '状态与网络概况' },
  { to: '/configs', label: '配置', hint: '图形配置与文件配置' },
  { to: '/peers', label: '设备', hint: '节点状态与流量' },
  { to: '/routes', label: '路由', hint: '链路与候选路径' },
]

const serverSummary = computed(() => {
  const servers = app.info.value.server_info
  if (servers.length === 0) {
    return '未配置服务器'
  }

  const connected = servers.filter((item) => item.connected).length
  return `${connected} / ${servers.length} 已连接`
})
</script>

<template>
  <div class="app-shell">
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

    <div class="content-shell">
      <header class="topbar panel">
        <div class="topbar-main">
          <StatusBadge :status="app.info.value.status" />
          <span class="topbar-chip">
            {{ app.info.value.ip ? `${app.info.value.ip}/${app.info.value.prefix_len}` : '未分配虚拟 IP' }}
          </span>
          <span class="topbar-chip" :class="{ online: app.isServerConnected.value }">
            {{ serverSummary }}
          </span>
        </div>

        <div class="topbar-meta">
          <span>{{ app.info.value.name || '未命名设备' }}</span>
          <span class="topbar-chip subtle">{{ shortDeviceId(app.info.value.device_id) }}</span>
        </div>
      </header>

      <main class="page-frame">
        <RouterView />
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: minmax(240px, 280px) minmax(0, 1fr);
  gap: 1.25rem;
  min-height: 100vh;
  padding: 1.25rem;
}

.sidebar {
  display: flex;
  min-height: calc(100vh - 2.5rem);
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
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 1.25rem;
}

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.topbar-main,
.topbar-meta {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  min-width: 0;
}

.topbar-meta {
  justify-content: flex-end;
  color: var(--text-soft);
}

.topbar-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-height: 2.25rem;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  padding: 0.55rem 0.85rem;
  background: rgba(255, 255, 255, 0.03);
  font-size: 0.92rem;
}

.topbar-chip.online {
  color: #8bf1c5;
}

.topbar-chip.subtle {
  color: var(--text-soft);
}

.page-frame {
  min-width: 0;
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

@media (max-width: 720px) {
  .topbar {
    align-items: flex-start;
    flex-direction: column;
  }

  .topbar-main,
  .topbar-meta {
    flex-wrap: wrap;
    justify-content: flex-start;
  }
}
</style>
