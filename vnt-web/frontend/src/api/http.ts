import type { ApiResponse } from '@/types/vnt'
import { hasTauriRuntime } from '@/utils/tauri'

const apiBase = hasTauriRuntime()
  ? 'http://localhost:19099'
  : (import.meta.env.VITE_API_BASE ?? '').replace(/\/$/, '')

async function parseResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`)
  }

  const payload = (await response.json()) as ApiResponse<T>
  if (payload.code !== 0) {
    throw new Error(payload.msg || 'Request failed')
  }

  return payload.data as T
}

export async function apiGet<T>(path: string): Promise<T> {
  const response = await fetch(`${apiBase}${path}`)
  return parseResponse<T>(response)
}

export async function apiSend<T>(
  path: string,
  init: Omit<RequestInit, 'body'> & { body?: unknown } = {},
): Promise<T> {
  const headers = new Headers(init.headers)
  let body: BodyInit | undefined

  if (init.body !== undefined) {
    headers.set('Content-Type', 'application/json')
    body = JSON.stringify(init.body)
  }

  const response = await fetch(`${apiBase}${path}`, {
    ...init,
    headers,
    body,
  })

  return parseResponse<T>(response)
}
