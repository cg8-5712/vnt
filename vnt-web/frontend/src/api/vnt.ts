import { apiGet, apiSend } from '@/api/http'
import { isAndroidTauriRuntime, tauriInvoke } from '@/utils/tauri'
import type {
  AppInfo,
  ConfigSummary,
  PeerItem,
  RouteItem,
  StartStatusResponse,
} from '@/types/vnt'

export function getInfo() {
  if (isAndroidTauriRuntime()) {
    return tauriInvoke<AppInfo>('android_vpn_info')
  }
  return apiGet<AppInfo>('/api/info')
}

export function getPeers() {
  if (isAndroidTauriRuntime()) {
    return tauriInvoke<PeerItem[]>('android_vpn_peers')
  }
  return apiGet<PeerItem[]>('/api/peers')
}

export function getRoutes() {
  if (isAndroidTauriRuntime()) {
    return tauriInvoke<RouteItem[]>('android_vpn_routes')
  }
  return apiGet<RouteItem[]>('/api/routes')
}

export function getStartStatus() {
  if (isAndroidTauriRuntime()) {
    return tauriInvoke<StartStatusResponse>('android_vpn_start_status')
  }
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
  if (isAndroidTauriRuntime()) {
    return getConfig(fileName).then((configToml) =>
      tauriInvoke<null>('android_vpn_start', {
        fileName,
        configToml,
      }),
    )
  }

  return apiSend<null>('/api/start', {
    method: 'POST',
    body: {
      file_name: fileName,
    },
  })
}

export function stopVnt() {
  if (isAndroidTauriRuntime()) {
    return tauriInvoke<null>('android_vpn_stop')
  }

  return apiSend<null>('/api/stop', {
    method: 'POST',
  })
}

export function restartVnt(fileName: string) {
  if (isAndroidTauriRuntime()) {
    return getConfig(fileName).then((configToml) =>
      tauriInvoke<null>('android_vpn_restart', {
        fileName,
        configToml,
      }),
    )
  }

  return apiSend<null>('/api/restart', {
    method: 'POST',
    body: {
      file_name: fileName,
    },
  })
}
