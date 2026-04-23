export interface VisualInputRule {
  net: string
  target_ip: string
}

export interface VisualConfig {
  config_name: string
  server: string[]
  cert_mode: string
  network_code: string
  network_secret: string
  device_id: string
  device_name: string
  tun_name: string
  ip: string
  password: string
  no_punch: boolean
  compress: boolean
  rtx: boolean
  fec: boolean
  input: VisualInputRule[]
  output: string[]
  no_nat: boolean
  no_tun: boolean
  mtu: string
  port_mapping: string[]
  allow_mapping: boolean
  udp_stun: string[]
  tcp_stun: string[]
  tunnel_port: string
}
