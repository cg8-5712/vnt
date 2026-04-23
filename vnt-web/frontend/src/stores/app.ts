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
      setNotice('error', '启动失败，请检查配置和网络状态。', 5200)
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
