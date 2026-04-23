import { apiGet, apiSend } from '@/api/http'
import type {
  AppInfo,
  ConfigSummary,
  PeerItem,
  RouteItem,
  StartStatusResponse,
} from '@/types/vnt'

export function getInfo() {
  return apiGet<AppInfo>('/api/info')
}

export function getPeers() {
  return apiGet<PeerItem[]>('/api/peers')
}

export function getRoutes() {
  return apiGet<RouteItem[]>('/api/routes')
}

export function getStartStatus() {
  return apiGet<StartStatusResponse>('/api/start/status')
}

export function listConfigs() {
  return apiGet<ConfigSummary[]>('/api/config/list')
}

export function getConfig(fileName: string) {
  return apiGet<string>(`/api/config?file_name=${encodeURIComponent(fileName)}`)
}

export function saveConfig(fileName: string, config: string) {
  return apiSend<null>('/api/config', {
    method: 'POST',
    body: {
      file_name: fileName,
      config,
    },
  })
}

export function deleteConfig(fileName: string) {
  return apiSend<null>(`/api/config?file_name=${encodeURIComponent(fileName)}`, {
    method: 'DELETE',
  })
}

export function startVnt(fileName: string) {
  return apiSend<null>('/api/start', {
    method: 'POST',
    body: {
      file_name: fileName,
    },
  })
}

export function stopVnt() {
  return apiSend<null>('/api/stop', {
    method: 'POST',
  })
}

export function restartVnt(fileName: string) {
  return apiSend<null>('/api/restart', {
    method: 'POST',
    body: {
      file_name: fileName,
    },
  })
}
