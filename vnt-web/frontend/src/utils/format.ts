import type { RouteDetail, VntStatus } from '@/types/vnt'

const statusLabels: Record<VntStatus, string> = {
  stopped: '未启动',
  starting: '启动中',
  running: '运行中',
}

export function statusLabel(status: VntStatus) {
  return statusLabels[status]
}

export function formatBytes(value?: number | null) {
  if (value === undefined || value === null) {
    return '--'
  }

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = value
  let unitIndex = 0

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex += 1
  }

  const digits = unitIndex === 0 ? 0 : size >= 10 ? 1 : 2
  return `${size.toFixed(digits)} ${units[unitIndex]}`
}

export function formatSpeed(value?: number | null) {
  if (value === undefined || value === null) {
    return '--'
  }

  return `${formatBytes(value)}/s`
}

export function formatLossRate(value?: number | null) {
  if (value === undefined || value === null) {
    return '--'
  }

  const normalized = value > 1 ? value / 100 : value
  return `${(normalized * 100).toFixed(1)}%`
}

export function formatTime(value?: number | null) {
  if (!value) {
    return '--'
  }

  const ts = value > 1_000_000_000_000 ? value : value * 1000
  const date = new Date(ts)
  if (Number.isNaN(date.getTime())) {
    return '--'
  }

  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(date)
}

export function formatBoolean(value?: boolean | null) {
  if (value === undefined || value === null) {
    return '--'
  }

  return value ? '开启' : '关闭'
}

export function routeModeLabel(route: RouteDetail | null) {
  if (!route) {
    return '--'
  }

  const isDirect = route.metric === 1
  const isTcp = route.protocol.includes('Tcp')

  if (isDirect) {
    return isTcp ? 'TCP 直连' : 'UDP 直连'
  }

  return isTcp ? 'TCP 中继' : 'UDP 中继'
}

export function shortDeviceId(value: string) {
  if (!value) {
    return '--'
  }

  return value.length <= 12 ? value : `${value.slice(0, 8)}...${value.slice(-4)}`
}
