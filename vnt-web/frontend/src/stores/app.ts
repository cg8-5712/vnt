import { computed, readonly, ref } from 'vue'

import {
  getInfo,
  getStartStatus,
  listConfigs,
  restartVnt,
  startVnt,
  stopVnt,
} from '@/api/vnt'
import type {
  AppInfo,
  ConfigSummary,
  Notice,
  NoticeKind,
  VntStatus,
} from '@/types/vnt'

const info = ref<AppInfo>({
  name: '',
  version: '',
  ip: null,
  prefix_len: null,
  gateway: null,
  device_id: '',
  status: 'stopped',
  current_config_name: null,
  current_config_file: null,
  online_client_num: 0,
  offline_client_num: 0,
  direct_client_num: 0,
  server_info: [],
  nat_type: null,
  public_ipv6: null,
  public_ipv4s: [],
  network_code: null,
  mtu: null,
  fec: null,
  compress: null,
  encrypt: null,
  rtx: null,
})

const configList = ref<ConfigSummary[]>([])
const loading = ref(false)
const showStartLog = ref(false)
const startStatus = ref<VntStatus>('stopped')
const startLogs = ref<string[]>([])
const notice = ref<Notice | null>(null)
const pageVisible = ref(typeof document === 'undefined' ? true : !document.hidden)

let bootstrapped = false
let infoTimer: number | null = null
let startTimer: number | null = null
let noticeTimer: number | null = null
let autoCloseTimer: number | null = null

function setNotice(kind: NoticeKind, message: string, timeout = 3600) {
  notice.value = { kind, message }
  if (noticeTimer !== null) {
    window.clearTimeout(noticeTimer)
  }
  noticeTimer = window.setTimeout(() => {
    notice.value = null
  }, timeout)
}

function extractErrorMessage(log: string): string {
  // 提取最后一条日志中的关键错误信息
  if (log.includes('Token错误') || log.includes('token')) {
    return '入网Token错误，请检查配置'
  }
  if (log.includes('密码错误') || log.includes('password')) {
    return '加密密码错误，请检查配置'
  }
  if (log.includes('密钥错误') || log.includes('secret')) {
    return '网络密钥错误，请检查配置'
  }
  if (log.includes('网络代码错误')) {
    return '网络代码错误，请检查配置'
  }
  if (log.includes('连接失败') || log.includes('连接超时')) {
    return '无法连接到服务器，请检查网络和服务器地址'
  }
  if (log.includes('连接被拒绝')) {
    return '服务器拒绝连接，请检查服务器地址和端口'
  }
  if (log.includes('域名解析失败')) {
    return '域名解析失败，请检查服务器地址'
  }
  if (log.includes('IP地址冲突') || log.includes('IP') && log.includes('无效')) {
    return 'IP地址冲突或无效，请更换虚拟IP'
  }
  if (log.includes('版本不兼容')) {
    return '客户端与服务器版本不兼容，请升级'
  }
  if (log.includes('TUN') || log.includes('虚拟网卡')) {
    return 'TUN设备创建失败，请确认有管理员权限'
  }
  if (log.includes('端口') && (log.includes('占用') || log.includes('绑定失败'))) {
    return '端口已被占用，请更换端口或关闭占用程序'
  }
  if (log.includes('配置文件格式错误') || log.includes('TOML')) {
    return '配置文件格式错误，请检查TOML语法'
  }
  if (log.includes('无权限') || log.includes('权限')) {
    return '权限不足，请以管理员身份运行'
  }
  if (log.includes('启动失败')) {
    return log
  }
  return '启动失败，请查看日志了解详情'
}

async function fetchInfo() {
  try {
    info.value = await getInfo()
  } catch (error) {
    console.error(error)
  }
}

async function fetchConfigList() {
  try {
    configList.value = await listConfigs()
  } catch (error) {
    console.error(error)
    setNotice('error', (error as Error).message)
  }
}

function stopStartPolling() {
  if (startTimer !== null) {
    window.clearInterval(startTimer)
    startTimer = null
  }
}

function scheduleLogAutoClose() {
  if (autoCloseTimer !== null) {
    window.clearTimeout(autoCloseTimer)
  }

  autoCloseTimer = window.setTimeout(() => {
    showStartLog.value = false
  }, 900)
}

async function pollStartStatus() {
  try {
    const status = await getStartStatus()
    startStatus.value = status.status
    startLogs.value = status.logs

    if (status.status === 'running') {
      stopStartPolling()
      await fetchInfo()
      setNotice('success', '组网已启动。')
      scheduleLogAutoClose()
      return
    }

    if (status.status === 'stopped' && status.logs.length > 0) {
      stopStartPolling()
      await fetchInfo()
      const lastLog = status.logs[status.logs.length - 1]
      const errorMsg = extractErrorMessage(lastLog)
      setNotice('error', errorMsg, 6000)
    }
  } catch (error) {
    console.error(error)
  }
}

function openStartLog() {
  showStartLog.value = true
  if (startTimer === null) {
    startTimer = window.setInterval(() => {
      void pollStartStatus()
    }, 1000)
  }
  void pollStartStatus()
}

function closeStartLog() {
  showStartLog.value = false
}

function ensureInfoPolling() {
  if (infoTimer !== null) {
    return
  }

  infoTimer = window.setInterval(() => {
    if (info.value.status === 'running' && pageVisible.value) {
      void fetchInfo()
    }
  }, 3000)
}

async function start(fileName: string) {
  loading.value = true
  try {
    await startVnt(fileName)
    startStatus.value = 'starting'
    startLogs.value = []
    openStartLog()
    setNotice('info', `正在启动 ${fileName}`)
  } catch (error) {
    setNotice('error', (error as Error).message, 4800)
  } finally {
    loading.value = false
  }
}

async function stop({ keepLog = false }: { keepLog?: boolean } = {}) {
  loading.value = true
  try {
    await stopVnt()
    stopStartPolling()
    if (!keepLog) {
      showStartLog.value = false
    }
    await fetchInfo()
    setNotice('info', '组网已停止。')
  } catch (error) {
    setNotice('error', (error as Error).message, 4800)
  } finally {
    loading.value = false
  }
}

async function restart(fileName: string) {
  loading.value = true
  try {
    await restartVnt(fileName)
    startStatus.value = 'starting'
    startLogs.value = []
    openStartLog()
    setNotice('info', `正在重启 ${fileName}`)
  } catch (error) {
    setNotice('error', (error as Error).message, 4800)
  } finally {
    loading.value = false
  }
}

async function cancelStart() {
  await stop({ keepLog: true })
  startStatus.value = 'stopped'
  startLogs.value = [...startLogs.value, '启动已取消。']
}

function setPageVisible(visible: boolean) {
  pageVisible.value = visible
}

async function bootstrap() {
  if (bootstrapped) {
    return
  }

  bootstrapped = true
  ensureInfoPolling()
  await Promise.all([fetchInfo(), fetchConfigList()])

  if (info.value.status === 'starting') {
    openStartLog()
  }
}

const isServerConnected = computed(() =>
  info.value.server_info.some((server) => server.connected),
)

export function useAppStore() {
  return {
    info: readonly(info),
    configList: readonly(configList),
    loading: readonly(loading),
    showStartLog: readonly(showStartLog),
    startStatus: readonly(startStatus),
    startLogs: readonly(startLogs),
    notice: readonly(notice),
    pageVisible: readonly(pageVisible),
    isServerConnected,
    setNotice,
    bootstrap,
    fetchInfo,
    fetchConfigList,
    start,
    stop,
    restart,
    cancelStart,
    openStartLog,
    closeStartLog,
    setPageVisible,
  }
}
