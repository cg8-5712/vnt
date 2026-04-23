<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { getRoutes } from '@/api/vnt'
import MetricCard from '@/components/MetricCard.vue'
import SectionCard from '@/components/SectionCard.vue'
import { useAppStore } from '@/stores/app'
import type { RouteItem } from '@/types/vnt'
import { formatLossRate, routeModeLabel } from '@/utils/format'

const app = useAppStore()
const routes = ref<RouteItem[]>([])

let timer: number | null = null

const totalRoutes = computed(() =>
  routes.value.reduce((total, item) => total + item.routes.length, 0),
)

const connectedServerCount = computed(
  () => app.info.value.server_info.filter((server) => server.connected).length,
)

const activeSummary = computed(() => [
  {
    label: '本机虚拟 IP',
    value: app.info.value.ip
      ? `${app.info.value.ip}/${app.info.value.prefix_len}`
      : '--',
  },
  {
    label: '网关',
    value: app.info.value.gateway || '--',
  },
  {
    label: 'NAT 类型',
    value: app.info.value.nat_type || '--',
  },
  {
    label: 'MTU',
    value: app.info.value.mtu ?? '--',
  },
  {
    label: '网络代码',
    value: app.info.value.network_code || '--',
  },
  {
    label: '服务器连接',
    value: `${connectedServerCount.value} / ${app.info.value.server_info.length}`,
  },
])

async function fetchRouteData() {
  if (app.info.value.status !== 'running') {
    routes.value = []
    return
  }

  try {
    routes.value = await getRoutes()
  } catch (error) {
    console.error(error)
  }
}

function startPolling() {
  if (timer !== null) {
    return
  }

  timer = window.setInterval(() => {
    if (app.pageVisible.value) {
      void fetchRouteData()
    }
  }, 3000)
}

function stopPolling() {
  if (timer !== null) {
    window.clearInterval(timer)
    timer = null
  }
}

watch(
  () => app.info.value.status,
  (status) => {
    if (status === 'running') {
      void fetchRouteData()
      return
    }

    routes.value = []
  },
  { immediate: true },
)

onMounted(() => {
  startPolling()
  void fetchRouteData()
})

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="page-grid">
    <section class="metrics-grid">
      <MetricCard label="目标节点" :value="routes.length" hint="存在路由信息的目的 IP 数量" />
      <MetricCard label="总路由数" :value="totalRoutes" hint="所有候选链路总和" />
      <MetricCard
        label="服务器连接"
        :value="`${connectedServerCount} / ${app.info.value.server_info.length}`"
        hint="已连接 / 已配置"
      />
    </section>

    <SectionCard title="当前活动路由 / 服务器链路" subtitle="显示当前节点的虚拟网络参数和控制服务器链路。">
      <div class="summary-grid">
        <div v-for="item in activeSummary" :key="item.label" class="summary-item">
          <span>{{ item.label }}</span>
          <strong>{{ item.value }}</strong>
        </div>
      </div>

      <div v-if="app.info.value.server_info.length === 0" class="empty-state">
        当前没有服务器链路信息。先完成配置并启动节点。
      </div>
      <div v-else class="table-wrap">
        <table class="table">
          <thead>
            <tr>
              <th>Server</th>
              <th>Status</th>
              <th>RTT</th>
              <th>Version</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="server in app.info.value.server_info" :key="server.server">
              <td>{{ server.server }}</td>
              <td>
                <span class="inline-pill" :class="{ ok: server.connected }">
                  {{ server.connected ? '已连接' : '未连接' }}
                </span>
              </td>
              <td>{{ server.server_rtt ?? '--' }} ms</td>
              <td>{{ server.server_version || '--' }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </SectionCard>

    <SectionCard title="对端节点路由表" subtitle="按目标设备分组展示候选路径，便于后续扩展详情抽屉。">
      <div v-if="routes.length === 0" class="empty-state">
        当前没有其他节点路由数据。只有连入其他设备后，这里才会出现候选路径。
      </div>

      <div v-else class="route-stack">
        <article v-for="item in routes" :key="item.ip" class="route-card">
          <header>
            <strong>{{ item.ip }}</strong>
            <span>{{ item.routes.length }} 条候选路径</span>
          </header>

          <div class="table-wrap">
            <table class="table">
              <thead>
                <tr>
                  <th>Mode</th>
                  <th>Address</th>
                  <th>Protocol</th>
                  <th>Metric</th>
                  <th>RTT</th>
                  <th>Loss</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="route in item.routes" :key="`${item.ip}-${route.addr}-${route.metric}`">
                  <td>{{ routeModeLabel(route) }}</td>
                  <td>{{ route.addr }}</td>
                  <td>{{ route.protocol }}</td>
                  <td>{{ route.metric }}</td>
                  <td>{{ route.rtt }} ms</td>
                  <td>{{ formatLossRate(route.loss_rate) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>
      </div>
    </SectionCard>
  </div>
</template>

<style scoped>
.page-grid {
  display: grid;
  gap: 1rem;
}

.metrics-grid {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.summary-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.summary-item {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.025);
  padding: 1rem;
}

.summary-item span {
  color: var(--text-soft);
  font-size: 0.9rem;
}

.summary-item strong {
  font-size: 1rem;
  word-break: break-all;
}

.route-stack {
  display: grid;
  gap: 1rem;
}

.route-card {
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 1.2rem;
  background: rgba(255, 255, 255, 0.025);
  padding: 1rem;
}

.route-card header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 0.9rem;
}

.route-card header span {
  color: var(--text-soft);
  font-size: 0.9rem;
}

.table-wrap {
  overflow: auto;
}

@media (max-width: 960px) {
  .metrics-grid,
  .summary-grid {
    grid-template-columns: 1fr;
  }
}
</style>
