<script setup lang="ts">
import { computed } from 'vue'

import MetricCard from '@/components/MetricCard.vue'
import SectionCard from '@/components/SectionCard.vue'
import { useAppStore } from '@/stores/app'
import { formatBoolean } from '@/utils/format'

const app = useAppStore()

const serverConnection = computed(() => {
  const servers = app.info.value.server_info
  const connected = servers.filter((server) => server.connected).length
  return `${connected} / ${servers.length}`
})

const featureItems = computed(() => [
  { label: '压缩', value: formatBoolean(app.info.value.compress) },
  { label: '重传', value: formatBoolean(app.info.value.rtx) },
  { label: 'FEC', value: formatBoolean(app.info.value.fec) },
  { label: '加密', value: formatBoolean(app.info.value.encrypt) },
])
</script>

<template>
  <div class="page-grid">
    <section class="metrics-grid">
      <MetricCard label="在线设备" :value="app.info.value.online_client_num" hint="当前在线节点数量" />
      <MetricCard label="离线设备" :value="app.info.value.offline_client_num" hint="最近一次拉取结果" />
      <MetricCard label="P2P 直连" :value="app.info.value.direct_client_num" hint="具备直连路径的节点" />
      <MetricCard
        label="网络代码"
        :value="app.info.value.network_code || '--'"
        :hint="app.info.value.current_config_name || '当前未加载配置'"
      />
    </section>

    <SectionCard title="本机信息" subtitle="集中展示当前设备、隧道和公网信息。">
      <div class="kv-grid">
        <div class="kv-item">
          <span>设备名称</span>
          <strong>{{ app.info.value.name || '--' }}</strong>
        </div>
        <div class="kv-item">
          <span>设备 ID</span>
          <strong>{{ app.info.value.device_id || '--' }}</strong>
        </div>
        <div class="kv-item">
          <span>本机虚拟 IP</span>
          <strong>{{ app.info.value.ip ? `${app.info.value.ip}/${app.info.value.prefix_len}` : '--' }}</strong>
        </div>
        <div class="kv-item">
          <span>网关</span>
          <strong>{{ app.info.value.gateway || '--' }}</strong>
        </div>
        <div class="kv-item">
          <span>NAT 类型</span>
          <strong>{{ app.info.value.nat_type || '--' }}</strong>
        </div>
        <div class="kv-item">
          <span>服务器连接</span>
          <strong>{{ serverConnection }}</strong>
        </div>
        <div class="kv-item">
          <span>公网 IPv4</span>
          <strong>{{ app.info.value.public_ipv4s.join(', ') || '--' }}</strong>
        </div>
        <div class="kv-item">
          <span>公网 IPv6</span>
          <strong>{{ app.info.value.public_ipv6 || '--' }}</strong>
        </div>
        <div class="kv-item">
          <span>MTU</span>
          <strong>{{ app.info.value.mtu ?? '--' }}</strong>
        </div>
        <div class="kv-item">
          <span>当前配置文件</span>
          <strong>{{ app.info.value.current_config_file || '--' }}</strong>
        </div>
      </div>
    </SectionCard>

    <SectionCard title="服务器链路" subtitle="控制节点的连接状态、版本和延迟。">
      <div v-if="app.info.value.server_info.length === 0" class="empty-state">
        还没有服务器信息。先到“配置”页保存并启动一个节点。
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

    <SectionCard title="能力开关" subtitle="这些字段直接映射当前生效配置。">
      <div class="feature-grid">
        <article v-for="feature in featureItems" :key="feature.label" class="feature-chip">
          <span>{{ feature.label }}</span>
          <strong>{{ feature.value }}</strong>
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
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.kv-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.kv-item {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.025);
  padding: 1rem;
}

.kv-item span {
  color: var(--text-soft);
  font-size: 0.9rem;
}

.kv-item strong {
  font-size: 1rem;
  word-break: break-all;
}

.table-wrap {
  overflow: auto;
}

.feature-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.feature-chip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  border-radius: 1rem;
  border: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(255, 255, 255, 0.025);
  padding: 0.95rem 1rem;
}

.feature-chip span {
  color: var(--text-soft);
}

@media (max-width: 1080px) {
  .metrics-grid,
  .feature-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 720px) {
  .metrics-grid,
  .kv-grid,
  .feature-grid {
    grid-template-columns: 1fr;
  }
}
</style>
