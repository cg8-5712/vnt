import { parse, stringify } from 'smol-toml'

import type { VisualConfig, VisualInputRule } from '@/types/config'

const DEFAULT_CONFIG: VisualConfig = {
  config_name: 'default',
  server: ['quic://YOUR_SERVER_IP:29872'],
  cert_mode: 'skip',
  network_code: 'default',
  network_secret: '',
  device_id: '',
  device_name: '',
  tun_name: '',
  ip: '',
  password: '',
  no_punch: false,
  compress: true,
  rtx: true,
  fec: false,
  input: [],
  output: [],
  no_nat: false,
  no_tun: false,
  mtu: '',
  port_mapping: [],
  allow_mapping: false,
  udp_stun: ['stun.miwifi.com:3478'],
  tcp_stun: ['stun.miwifi.com:3478'],
  tunnel_port: '',
}

function shouldPreferNoTun() {
  return false
}

function trimString(value: unknown) {
  return typeof value === 'string' ? value.trim() : ''
}

function stringArray(value: unknown) {
  if (!Array.isArray(value)) {
    return [] as string[]
  }

  return value
    .filter((item): item is string => typeof item === 'string')
    .map((item) => item.trim())
    .filter(Boolean)
}

function booleanValue(value: unknown, fallback = false) {
  return typeof value === 'boolean' ? value : fallback
}

function optionalNumberString(value: unknown) {
  return typeof value === 'number' ? String(value) : ''
}

function cleanInputRule(rule: VisualInputRule): VisualInputRule {
  return {
    net: rule.net.trim(),
    target_ip: rule.target_ip.trim(),
  }
}

function cleanInputList(value: VisualInputRule[]) {
  return value
    .map(cleanInputRule)
    .filter((item) => item.net.length > 0 || item.target_ip.length > 0)
}

export function splitLines(value: string) {
  return value
    .split(/\r?\n/)
    .map((item) => item.trim())
    .filter(Boolean)
}

export function createDefaultVisualConfig(): VisualConfig {
  return {
    ...DEFAULT_CONFIG,
    server: [...DEFAULT_CONFIG.server],
    input: [],
    output: [...DEFAULT_CONFIG.output],
    port_mapping: [...DEFAULT_CONFIG.port_mapping],
    udp_stun: [...DEFAULT_CONFIG.udp_stun],
    tcp_stun: [...DEFAULT_CONFIG.tcp_stun],
    no_tun: shouldPreferNoTun(),
  }
}

export function normalizeVisualConfig(config: VisualConfig): VisualConfig {
  return {
    config_name: config.config_name.trim(),
    server: config.server.map((item) => item.trim()).filter(Boolean),
    cert_mode: config.cert_mode.trim(),
    network_code: config.network_code.trim(),
    network_secret: config.network_secret.trim(),
    device_id: config.device_id.trim(),
    device_name: config.device_name.trim(),
    tun_name: config.tun_name.trim(),
    ip: config.ip.trim(),
    password: config.password.trim(),
    no_punch: config.no_punch,
    compress: config.compress,
    rtx: config.rtx,
    fec: config.fec,
    input: cleanInputList(config.input),
    output: config.output.map((item) => item.trim()).filter(Boolean),
    no_nat: config.no_nat,
    no_tun: config.no_tun,
    mtu: config.mtu.trim(),
    port_mapping: config.port_mapping.map((item) => item.trim()).filter(Boolean),
    allow_mapping: config.allow_mapping,
    udp_stun: config.udp_stun.map((item) => item.trim()).filter(Boolean),
    tcp_stun: config.tcp_stun.map((item) => item.trim()).filter(Boolean),
    tunnel_port: config.tunnel_port.trim(),
  }
}

export function visualConfigFingerprint(config: VisualConfig) {
  return JSON.stringify(normalizeVisualConfig(config))
}

export function parseTomlConfig(content: string): VisualConfig {
  const document = parse(content) as Record<string, unknown>

  return {
    config_name: trimString(document.config_name) || DEFAULT_CONFIG.config_name,
    server: stringArray(document.server).length > 0 ? stringArray(document.server) : [...DEFAULT_CONFIG.server],
    cert_mode: trimString(document.cert_mode) || DEFAULT_CONFIG.cert_mode,
    network_code: trimString(document.network_code) || DEFAULT_CONFIG.network_code,
    network_secret: trimString(document.network_secret),
    device_id: trimString(document.device_id),
    device_name: trimString(document.device_name),
    tun_name: trimString(document.tun_name),
    ip: trimString(document.ip),
    password: trimString(document.password),
    no_punch: booleanValue(document.no_punch),
    compress: booleanValue(document.compress, DEFAULT_CONFIG.compress),
    rtx: booleanValue(document.rtx, DEFAULT_CONFIG.rtx),
    fec: booleanValue(document.fec),
    input: stringArray(document.input).map((item) => {
      const [net = '', target_ip = ''] = item.split(',').map((part) => part.trim())
      return { net, target_ip }
    }),
    output: stringArray(document.output),
    no_nat: booleanValue(document.no_nat),
    no_tun: booleanValue(document.no_tun, shouldPreferNoTun()),
    mtu: optionalNumberString(document.mtu),
    port_mapping: stringArray(document.port_mapping),
    allow_mapping: booleanValue(document.allow_mapping),
    udp_stun: stringArray(document.udp_stun),
    tcp_stun: stringArray(document.tcp_stun),
    tunnel_port: optionalNumberString(document.tunnel_port),
  }
}

export function validateVisualConfig(config: VisualConfig) {
  const normalized = normalizeVisualConfig(config)

  if (normalized.server.length === 0) {
    return '至少填写一个控制服务器地址。'
  }

  if (!normalized.network_code) {
    return 'network_code 不能为空。'
  }

  if (
    config.input.some(
      (item) =>
        (item.net.trim().length > 0 && item.target_ip.trim().length === 0) ||
        (item.net.trim().length === 0 && item.target_ip.trim().length > 0),
    )
  ) {
    return '子网转发规则需要同时填写网段和目标 IP。'
  }

  if (normalized.mtu && !/^\d+$/.test(normalized.mtu)) {
    return 'MTU 需要是正整数。'
  }

  if (normalized.tunnel_port && !/^\d+$/.test(normalized.tunnel_port)) {
    return '隧道端口需要是非负整数。'
  }

  return ''
}

export function stringifyVisualConfig(config: VisualConfig) {
  const normalized = normalizeVisualConfig(config)
  const document: Record<string, unknown> = {
    config_name: normalized.config_name || DEFAULT_CONFIG.config_name,
    server: normalized.server,
    network_code: normalized.network_code,
    no_punch: normalized.no_punch,
    compress: normalized.compress,
    rtx: normalized.rtx,
    fec: normalized.fec,
    no_nat: normalized.no_nat,
    no_tun: normalized.no_tun,
    allow_mapping: normalized.allow_mapping,
  }

  if (normalized.cert_mode) {
    document.cert_mode = normalized.cert_mode
  }

  if (normalized.network_secret) {
    document.network_secret = normalized.network_secret
  }

  if (normalized.device_id) {
    document.device_id = normalized.device_id
  }

  if (normalized.device_name) {
    document.device_name = normalized.device_name
  }

  if (normalized.tun_name) {
    document.tun_name = normalized.tun_name
  }

  if (normalized.ip) {
    document.ip = normalized.ip
  }

  if (normalized.password) {
    document.password = normalized.password
  }

  if (normalized.input.length > 0) {
    document.input = normalized.input.map((item) => `${item.net},${item.target_ip}`)
  }

  if (normalized.output.length > 0) {
    document.output = normalized.output
  }

  if (normalized.mtu) {
    document.mtu = Number.parseInt(normalized.mtu, 10)
  }

  if (normalized.port_mapping.length > 0) {
    document.port_mapping = normalized.port_mapping
  }

  if (normalized.udp_stun.length > 0) {
    document.udp_stun = normalized.udp_stun
  }

  if (normalized.tcp_stun.length > 0) {
    document.tcp_stun = normalized.tcp_stun
  }

  if (normalized.tunnel_port) {
    document.tunnel_port = Number.parseInt(normalized.tunnel_port, 10)
  }

  return `${stringify(document).trim()}\n`
}
