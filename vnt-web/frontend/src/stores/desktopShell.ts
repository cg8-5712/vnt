import { readonly, ref } from 'vue'

import { hasTauriRuntime, tauriInvoke, tauriListen } from '@/utils/tauri'

type CloseDecision = 'minimize_to_tray' | 'close'

type DesktopShellInfo = {
  customTitlebar: boolean
}

type CloseRequestOutcome = 'ignored' | 'prompt' | 'minimized_to_tray' | 'closed'

const runtimeAvailable = ref(false)
const customTitlebarEnabled = ref(false)
const closePromptVisible = ref(false)
const rememberCloseChoice = ref(false)
const maximized = ref(false)

let bootstrapped = false
let pendingResizeFrame: number | null = null
let unlistenClosePrompt: (() => void) | null = null

function resetClosePromptState() {
  closePromptVisible.value = false
  rememberCloseChoice.value = false
}

async function syncMaximizedState() {
  if (!runtimeAvailable.value || !customTitlebarEnabled.value) {
    maximized.value = false
    return
  }

  try {
    maximized.value = await tauriInvoke<boolean>('window_is_maximized')
  } catch (error) {
    console.error(error)
  }
}

function handleWindowResize() {
  if (pendingResizeFrame !== null) {
    return
  }

  pendingResizeFrame = window.requestAnimationFrame(() => {
    pendingResizeFrame = null
    void syncMaximizedState()
  })
}

async function bootstrap() {
  if (bootstrapped) {
    return
  }

  runtimeAvailable.value = hasTauriRuntime()
  bootstrapped = true

  if (!runtimeAvailable.value) {
    return
  }

  try {
    const shellInfo = await tauriInvoke<DesktopShellInfo>('desktop_shell_info')
    customTitlebarEnabled.value = shellInfo.customTitlebar
    await syncMaximizedState()

    unlistenClosePrompt = await tauriListen<void>('vnt://confirm-close', () => {
      rememberCloseChoice.value = false
      closePromptVisible.value = true
    })

    window.addEventListener('resize', handleWindowResize)
  } catch (error) {
    console.error(error)
  }
}

function dispose() {
  if (pendingResizeFrame !== null) {
    window.cancelAnimationFrame(pendingResizeFrame)
    pendingResizeFrame = null
  }

  window.removeEventListener('resize', handleWindowResize)
  unlistenClosePrompt?.()
  unlistenClosePrompt = null
  bootstrapped = false
}

async function minimizeWindow() {
  if (!runtimeAvailable.value) {
    return
  }

  await tauriInvoke('window_minimize')
}

async function startDragging() {
  if (!runtimeAvailable.value || !customTitlebarEnabled.value) {
    return
  }

  try {
    await tauriInvoke('window_start_dragging')
  } catch (error) {
    console.error('startDragging failed:', error)
  }
}

async function toggleMaximizeWindow() {
  if (!runtimeAvailable.value || !customTitlebarEnabled.value) {
    return
  }

  maximized.value = await tauriInvoke<boolean>('window_toggle_maximize')
}

async function requestCloseWindow() {
  if (!runtimeAvailable.value) {
    return
  }

  try {
    const outcome = await tauriInvoke<CloseRequestOutcome>('request_close_window')

    if (outcome === 'prompt') {
      rememberCloseChoice.value = false
      closePromptVisible.value = true
    }
  } catch (error) {
    console.error(error)
  }
}

async function cancelClosePrompt() {
  resetClosePromptState()

  if (!runtimeAvailable.value) {
    return
  }

  try {
    await tauriInvoke('dismiss_close_request')
  } catch (error) {
    console.error(error)
  }
}

async function resolveClosePrompt(action: CloseDecision) {
  const remember = rememberCloseChoice.value
  resetClosePromptState()

  if (!runtimeAvailable.value) {
    return
  }

  try {
    await tauriInvoke('resolve_close_request', { action, remember })
  } catch (error) {
    console.error(error)
  }
}

export function useDesktopShellStore() {
  return {
    runtimeAvailable: readonly(runtimeAvailable),
    customTitlebarEnabled: readonly(customTitlebarEnabled),
    closePromptVisible: readonly(closePromptVisible),
    rememberCloseChoice,
    maximized: readonly(maximized),
    bootstrap,
    dispose,
    minimizeWindow,
    startDragging,
    toggleMaximizeWindow,
    requestCloseWindow,
    cancelClosePrompt,
    resolveClosePrompt,
    syncMaximizedState,
  }
}
