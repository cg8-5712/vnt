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
  const embeddedTauriShell = looksLikeEmbeddedTauriShell()

  if (embeddedTauriShell) {
    for (const base of LOCAL_API_BASES) {
      candidates.push(`${base}${normalizedPath}`)
    }
  }

  if (envBase) {
    candidates.push(`${envBase}${normalizedPath}`)
  } else if (
    !embeddedTauriShell &&
    typeof window !== 'undefined' &&
    /^https?:$/.test(window.location.protocol) &&
    window.location.origin !== 'null'
  ) {
    candidates.push(`${normalizeBase(window.location.origin)}${normalizedPath}`)
  } else if (!embeddedTauriShell) {
    candidates.push(normalizedPath)
  }

  return [...new Set(candidates)]
}

async function parseResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`)
  }

  const contentType = response.headers.get('content-type') ?? ''
  if (!contentType.includes('application/json')) {
    const body = await response.text()
    const preview = body.slice(0, 80).replace(/\s+/g, ' ').trim()
    throw new Error(`Expected JSON response, got: ${preview || '<empty response>'}`)
  }

  const payload = (await response.json()) as ApiResponse<T>
  if (payload.code !== 0) {
    throw new Error(payload.msg || 'Request failed')
  }

  return payload.data as T
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const candidates = buildApiCandidates(path)
  let lastError: unknown

  for (const url of candidates) {
    try {
      const response = await fetch(url, init)
      return await parseResponse<T>(response)
    } catch (error) {
      if (error instanceof Error && error.name !== 'TypeError') {
        throw error
      }
      lastError = error
    }
  }

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
