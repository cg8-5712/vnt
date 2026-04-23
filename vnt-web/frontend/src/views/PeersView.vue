<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { getPeers } from '@/api/vnt'
import MetricCard from '@/components/MetricCard.vue'
import SectionCard from '@/components/SectionCard.vue'
import { useAppStore } from '@/stores/app'
import type { PeerItem } from '@/types/vnt'
import {
  formatBytes,
  formatLossRate,
  formatSpeed,
  formatTime,
  routeModeLabel,
} from '@/utils/format'

type DeviceRow = PeerItem & {
  isSelf: boolean
  deviceId: string | null
}

const app = useAppStore()

const peers = ref<PeerItem[]>([])
const expanded = ref<string[]>([])
const speedMap = ref<Record<string, { tx: number; rx: number }>>({})

let timer: number | null = null
let lastAt = 0
let lastTraffic: Record<string, { tx: number; rx: number }> = {}

const selfDevice = computed<DeviceRow | null>(() => {
  if (app.info.value.status !== 'running') {
    return null
  }

  return {
    ip: app.info.value.ip || 'local',
    name: app.info.value.name || '当前设备',
    online: true,
    route: null,
    version: app.info.value.version || '',
    last_connected_time: 0,
    key_equal: 1,
    nat_info: {
      nat_type: app.info.value.nat_type || '--',
      public_ips: app.info.value.public_ipv4s,
      ipv6: app.info.value.public_ipv6,
    },
    packet_loss: null,
    traffic: null,
    isSelf: true,
    deviceId: app.info.value.device_id || null,
  }
})

const deviceRows = computed<DeviceRow[]>(() => {
  const self = selfDevice.value
  const selfIp = self?.ip
  const peerRows = peers.value
    .filter((peer) => peer.ip !== selfIp)
    .map((peer) => ({
      ...peer,
      isSelf: false,
      deviceId: null,
    }))

  return self ? [self, ...peerRows] : peerRows
})

const onlinePeers = computed(() => deviceRows.value.filter((peer) => peer.online))
const directPeers = computed(() =>
  deviceRows.value.filter((peer) => !peer.isSelf && peer.route?.metric === 1),
)

const sortedPeers = computed(() =>
  [...deviceRows.value].sort((left, right) => {
    if (left.isSelf !== right.isSelf) {
      return Number(right.isSelf) - Number(left.isSelf)
    }

    if (left.online !== right.online) {
      return Number(right.online) - Number(left.online)
    }

    return left.ip.localeCompare(right.ip)
  }),
)

function isExpanded(ip: string) {
  return expanded.value.includes(ip)
}

function toggleExpand(ip: string) {
  expanded.value = isExpanded(ip)
    ? expanded.value.filter((item) => item !== ip)
    : [...expanded.value, ip]
}

function deviceStatusLabel(peer: DeviceRow) {
  if (peer.isSelf) {
    return '本机'
  }

  return peer.online ? '在线' : '离线'
}

function deviceModeLabel(peer: DeviceRow) {
  if (peer.isSelf) {
    return '当前设备'
  }

  return routeModeLabel(peer.route)
}

async function fetchPeerData() {
  if (app.info.value.status !== 'running') {
    peers.value = []
    speedMap.value = {}
    lastTraffic = {}
    lastAt = 0
    return
  }

  try {
    const items = await getPeers()
    const now = Date.now()
    const nextSpeedMap: Record<string, { tx: number; rx: number }> = {}
    const elapsed = lastAt > 0 ? (now - lastAt) / 1000 : 0

    for (const peer of items) {
      if (!peer.traffic) {
        continue
      }

      const prev = lastTraffic[peer.ip]
      if (prev && elapsed > 0) {
        nextSpeedMap[peer.ip] = {
          tx: Math.max(0, Math.round((peer.traffic.tx_bytes - prev.tx) / elapsed)),
          rx: Math.max(0, Math.round((peer.traffic.rx_bytes - prev.rx) / elapsed)),
        }
      } else {
        nextSpeedMap[peer.ip] = { tx: 0, rx: 0 }
      }
    }

    lastTraffic = Object.fromEntries(
      items
        .filter((peer) => peer.traffic)
        .map((peer) => [
          peer.ip,
          {
            tx: peer.traffic!.tx_bytes,
            rx: peer.traffic!.rx_bytes,
          },
        ]),
    )

    lastAt = now
    peers.value = items
    speedMap.value = nextSpeedMap
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
      void fetchPeerData()
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
      void fetchPeerData()
      return
    }

    peers.value = []
  },
  { immediate: true },
)

onMounted(() => {
  startPolling()
  void fetchPeerData()
})

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="page-grid">
    <section class="metrics-grid">
      <MetricCard label="设备总数" :value="sortedPeers.length" hint="包含当前本机与其他节点" />
      <MetricCard label="在线设备" :value="onlinePeers.length" hint="本机与在线节点合计" />
      <MetricCard label="直连节点" :value="directPeers.length" hint="不包含当前本机，metric = 1" />
    </section>

    <SectionCard title="设备与链路" subtitle="设备列表包含当前本机，点击行可以展开 NAT、流量和路径细节。">
      <div v-if="sortedPeers.length === 0" class="empty-state">
        当前没有设备数据。确认 VNT 已启动并且至少完成一次状态拉取。
      </div>

      <div v-else class="table-wrap">
        <table class="table peer-table">
          <thead>
            <tr>
              <th>Device</th>
              <th>IP</th>
              <th>Status</th>
              <th>Mode</th>
              <th>RTT</th>
              <th>Loss</th>
              <th>Traffic</th>
            </tr>
          </thead>
          <tbody>
            <template v-for="peer in sortedPeers" :key="peer.isSelf ? `self-${peer.ip}` : peer.ip">
              <tr class="click-row" @click="toggleExpand(peer.ip)">
                <td>
                  <div class="name-cell">
                    <strong>{{ peer.name || (peer.isSelf ? '当前设备' : 'Unknown') }}</strong>
                    <span>{{ peer.version || '--' }}</span>
                  </div>
                </td>
                <td>{{ peer.ip }}</td>
                <td>
                  <span class="inline-pill" :class="{ ok: peer.online }">
                    {{ deviceStatusLabel(peer) }}
                  </span>
                </td>
                <td>{{ deviceModeLabel(peer) }}</td>
                <td>{{ peer.route?.rtt ?? '--' }} ms</td>
                <td>{{ formatLossRate(peer.packet_loss?.loss_rate) }}</td>
                <td>
                  <div class="traffic-cell">
                    <span>RX {{ formatSpeed(speedMap[peer.ip]?.rx) }}</span>
                    <span>TX {{ formatSpeed(speedMap[peer.ip]?.tx) }}</span>
                  </div>
                </td>
              </tr>
              <tr v-if="isExpanded(peer.ip)" class="detail-row">
                <td colspan="7">
                  <div class="detail-grid">
                    <div>
                      <span>{{ peer.isSelf ? '设备 ID' : '最后连接时间' }}</span>
                      <strong>{{ peer.isSelf ? (peer.deviceId || '--') : formatTime(peer.last_connected_time) }}</strong>
                    </div>
                    <div>
                      <span>NAT</span>
                      <strong>{{ peer.nat_info?.nat_type || '--' }}</strong>
                    </div>
                    <div>
                      <span>公网地址</span>
                      <strong>{{ peer.nat_info?.public_ips.join(', ') || '--' }}</strong>
                    </div>
                    <div>
                      <span>IPv6</span>
                      <strong>{{ peer.nat_info?.ipv6 || '--' }}</strong>
                    </div>
                    <div>
                      <span>总接收</span>
                      <strong>{{ formatBytes(peer.traffic?.rx_bytes) }}</strong>
                    </div>
                    <div>
                      <span>总发送</span>
                      <strong>{{ formatBytes(peer.traffic?.tx_bytes) }}</strong>
                    </div>
                    <div>
                      <span>{{ peer.isSelf ? '网关' : '路径' }}</span>
                      <strong>{{ peer.isSelf ? (app.info.value.gateway || '--') : (peer.route?.addr || '--') }}</strong>
                    </div>
                    <div>
                      <span>{{ peer.isSelf ? '网络代码' : '协议' }}</span>
                      <strong>
                        {{ peer.isSelf ? (app.info.value.network_code || '--') : (peer.route?.protocol || '--') }}
                      </strong>
                    </div>
                  </div>
                </td>
              </tr>
            </template>
          </tbody>
        </table>
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

.peer-table .name-cell,
.traffic-cell {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.peer-table .name-cell span,
.traffic-cell span {
  color: var(--text-soft);
  font-size: 0.88rem;
}

.click-row {
  cursor: pointer;
}

.detail-row td {
  background: rgba(255, 255, 255, 0.03);
}

.detail-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  padding: 0.5rem 0;
}

.detail-grid div {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  border-radius: 0.85rem;
  background: rgba(255, 255, 255, 0.025);
  padding: 0.9rem;
}

.detail-grid span {
  color: var(--text-soft);
  font-size: 0.84rem;
}

@media (max-width: 960px) {
  .metrics-grid,
  .detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>
