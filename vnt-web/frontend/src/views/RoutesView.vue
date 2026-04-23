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
    </section>

    <SectionCard title="路由表" subtitle="按目标设备分组展示，便于后续直接接 Tauri 卡片或详情抽屉。">
      <div v-if="routes.length === 0" class="empty-state">
        当前没有路由数据。只有在节点已运行时才会持续拉取。
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
  grid-template-columns: repeat(2, minmax(0, 1fr));
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

@media (max-width: 720px) {
  .metrics-grid {
    grid-template-columns: 1fr;
  }
}
</style>
