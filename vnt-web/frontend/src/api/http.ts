import type { ApiResponse } from '@/types/vnt'
import { hasTauriRuntime } from '@/utils/tauri'

const LOCAL_API_BASES = ['http://localhost:19099', 'http://127.0.0.1:19099']

function normalizeBase(base: string) {
  return base.replace(/\/$/, '')
}

function normalizePath(path: string) {
  return path.startsWith('/') ? path : `/${path}`
}

function looksLikeEmbeddedTauriShell() {
  if (typeof window === 'undefined') {
    return false
  }

  if (hasTauriRuntime()) {
    return true
  }

  const { hostname, protocol } = window.location
  return hostname === 'tauri.localhost' || !/^https?:$/.test(protocol)
}

function buildApiCandidates(path: string) {
  const normalizedPath = normalizePath(path)
  const envBase = normalizeBase(import.meta.env.VITE_API_BASE ?? '')
  const candidates: string[] = []

  if (looksLikeEmbeddedTauriShell()) {
    for (const base of LOCAL_API_BASES) {
      candidates.push(`${base}${normalizedPath}`)
    }
  }

  if (envBase) {
    candidates.push(`${envBase}${normalizedPath}`)
  } else if (
    typeof window !== 'undefined' &&
    /^https?:$/.test(window.location.protocol) &&
    window.location.origin !== 'null'
  ) {
    candidates.push(`${normalizeBase(window.location.origin)}${normalizedPath}`)
  } else if (!looksLikeEmbeddedTauriShell()) {
    candidates.push(normalizedPath)
  }

  return [...new Set(candidates)]
}

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

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const candidates = buildApiCandidates(path)
  console.log('[API] Request path:', path, 'candidates:', candidates, 'hasTauri:', hasTauriRuntime())
  let lastError: unknown

  for (const url of candidates) {
    try {
      console.log('[API] Trying URL:', url)
      const response = await fetch(url, init)
      console.log('[API] Response status:', response.status, 'ok:', response.ok)
      return await parseResponse<T>(response)
    } catch (error) {
      console.error('[API] Failed URL:', url, 'error:', error)
      lastError = error
    }
  }

  console.error('[API] All candidates failed. Last error:', lastError)
  throw lastError instanceof Error ? lastError : new Error('Failed to fetch API')
}

export async function apiGet<T>(path: string): Promise<T> {
  return request<T>(path)
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

  return request<T>(path, {
    ...init,
    headers,
    body,
  })
}
