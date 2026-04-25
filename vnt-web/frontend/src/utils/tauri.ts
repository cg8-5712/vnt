type TauriEvent<T> = {
  payload: T
}

type TauriUnlisten = () => void

type TauriWindowHandle = {
  listen: <T>(
    eventName: string,
    handler: (event: TauriEvent<T>) => void,
  ) => Promise<TauriUnlisten>
}

type TauriGlobal = {
  core: {
    invoke: <T>(command: string, args?: Record<string, unknown>) => Promise<T>
  }
  event: {
    listen: <T>(
      eventName: string,
      handler: (event: TauriEvent<T>) => void,
    ) => Promise<TauriUnlisten>
  }
  window?: {
    getCurrentWindow: () => TauriWindowHandle
  }
}

declare global {
  interface Window {
    __TAURI__?: TauriGlobal
  }
}

export function hasTauriRuntime() {
  return typeof window !== 'undefined' && !!window.__TAURI__?.core?.invoke
}

export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
) {
  const tauri = window.__TAURI__

  if (!tauri?.core?.invoke) {
    throw new Error('Tauri runtime unavailable')
  }

  return tauri.core.invoke<T>(command, args)
}

export async function tauriListen<T>(
  eventName: string,
  handler: (payload: T) => void,
) {
  const tauri = window.__TAURI__

  const currentWindow = tauri?.window?.getCurrentWindow?.()

  if (currentWindow?.listen) {
    return currentWindow.listen<T>(eventName, (event) => {
      handler(event.payload)
    })
  }

  if (!tauri?.event?.listen) {
    throw new Error('Tauri event API unavailable')
  }

  return tauri.event.listen<T>(eventName, (event) => {
    handler(event.payload)
  })
}
