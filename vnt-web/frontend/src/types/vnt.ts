export type VntStatus = 'stopped' | 'starting' | 'running'

export interface ApiResponse<T> {
  code: number
  msg: string
  data: T | null
}

export interface ServerInfo {
  server: string
  connected: boolean
  server_rtt: number | null
  server_version?: string | null
}

export interface AppInfo {
  name: string
  version: string
  ip: string | null
  prefix_len: number | null
  gateway: string | null
  device_id: string
  status: VntStatus
  current_config_name: string | null
  current_config_file: string | null
  online_client_num: number
  offline_client_num: number
  direct_client_num: number
  server_info: ServerInfo[]
  nat_type: string | null
  public_ipv6: string | null
  public_ipv4s: string[]
  network_code: string | null
  mtu: number | null
  fec: boolean | null
  compress: boolean | null
  encrypt: boolean | null
  rtx: boolean | null
}

export interface ConfigSummary {
  file_name: string
  config_name: string
}

export interface StartStatusResponse {
  status: VntStatus
  logs: string[]
}

export interface PeerNatInfo {
  nat_type: string
  public_ips: string[]
  ipv6: string | null
}

export interface PacketLoss {
  sent: number
  received: number
  loss_rate: number
}

export interface TrafficInfo {
  tx_bytes: number
  rx_bytes: number
}

export interface RouteDetail {
  addr: string
  protocol: string
  metric: number
  rtt: number
  loss_rate: number
}

export interface PeerItem {
  ip: string
  name: string | null
  online: boolean
  route: RouteDetail | null
  version: string
  last_connected_time: number
  key_equal: number
  nat_info: PeerNatInfo | null
  packet_loss: PacketLoss | null
  traffic: TrafficInfo | null
}

export interface RouteItem {
  ip: string
  routes: RouteDetail[]
}

export type NoticeKind = 'info' | 'success' | 'error'

export interface Notice {
  kind: NoticeKind
  message: string
}
