'use client'

import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react'
import { authApi, setAccessToken, setOnRefreshFail } from './api'
import { UserResponse } from './types'

// ─── Auth Context ─────────────────────────────────────────────────────────────

interface AuthContextValue {
  user: UserResponse | null
  token: string | null
  isLoading: boolean
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  updateUser: (user: UserResponse) => void
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}

// ─── WebSocket Context ────────────────────────────────────────────────────────

type WsCallback = (data: unknown) => void

interface WsContextValue {
  subscribe: (channel: string, cb: WsCallback) => () => void
  isConnected: boolean
}

const WsContext = createContext<WsContextValue | null>(null)

export function useWsContext(): WsContextValue {
  const ctx = useContext(WsContext)
  if (!ctx) throw new Error('useWsContext must be used within WsProvider')
  return ctx
}

// ─── WS Manager (singleton) ───────────────────────────────────────────────────

function getChannelFromEvent(data: Record<string, unknown>): string | null {
  if (data.OrderBookUpdate) {
    const inner = data.OrderBookUpdate as { symbol: string }
    return `orderbook:${inner.symbol}`
  }
  if (data.TradeEvent) {
    const inner = data.TradeEvent as { symbol: string }
    return `trades:${inner.symbol}`
  }
  if (data.TickerUpdate) {
    const inner = data.TickerUpdate as { symbol: string }
    return `ticker:${inner.symbol}`
  }
  if (data.OrderStatusUpdate) {
    const inner = data.OrderStatusUpdate as { user_id: string }
    return `orders:${inner.user_id}`
  }
  return null
}

// ─── Providers ────────────────────────────────────────────────────────────────

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <AuthProviderInner>
      <WsProviderInner>{children}</WsProviderInner>
    </AuthProviderInner>
  )
}

function AuthProviderInner({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<UserResponse | null>(null)
  const [token, setToken] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  const handleToken = useCallback((t: string | null) => {
    setToken(t)
    setAccessToken(t)
  }, [])

  // On mount: try silent refresh
  useEffect(() => {
    setOnRefreshFail(() => {
      handleToken(null)
      setUser(null)
    })

    authApi
      .refresh()
      .then(async (data) => {
        handleToken(data.access_token)
        // Fetch user info
        const { usersApi } = await import('./api')
        const me = await usersApi.me()
        setUser(me)
      })
      .catch(() => {
        // No valid session
      })
      .finally(() => setIsLoading(false))
  }, [handleToken])

  const login = useCallback(
    async (email: string, password: string) => {
      const data = await authApi.login(email, password)
      handleToken(data.access_token)
      const { usersApi } = await import('./api')
      const me = await usersApi.me()
      setUser(me)
    },
    [handleToken]
  )

  const register = useCallback(
    async (email: string, password: string) => {
      const data = await authApi.register(email, password)
      handleToken(data.access_token)
      const { usersApi } = await import('./api')
      const me = await usersApi.me()
      setUser(me)
    },
    [handleToken]
  )

  const logout = useCallback(async () => {
    try {
      await authApi.logout()
    } catch {
      // ignore
    }
    handleToken(null)
    setUser(null)
  }, [handleToken])

  return (
    <AuthContext.Provider value={{ user, token, isLoading, login, register, logout, updateUser: setUser }}>
      {children}
    </AuthContext.Provider>
  )
}

const WS_URL = process.env.NEXT_PUBLIC_WS_URL ?? 'ws://localhost:5098/ws'

function WsProviderInner({ children }: { children: React.ReactNode }) {
  const wsRef = useRef<WebSocket | null>(null)
  const subscribersRef = useRef<Map<string, Set<WsCallback>>>(new Map())
  const channelsRef = useRef<Set<string>>(new Set())
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const reconnectDelayRef = useRef(1000)
  const [isConnected, setIsConnected] = useState(false)
  const { token } = useAuth()
  const tokenRef = useRef(token)

  useEffect(() => {
    tokenRef.current = token
    // Re-auth if connected
    if (wsRef.current?.readyState === WebSocket.OPEN && token) {
      wsRef.current.send(JSON.stringify({ action: 'auth', token }))
    }
  }, [token])

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return

    const ws = new WebSocket(WS_URL)
    wsRef.current = ws

    ws.onopen = () => {
      setIsConnected(true)
      reconnectDelayRef.current = 1000

      // Auth if we have a token
      if (tokenRef.current) {
        ws.send(JSON.stringify({ action: 'auth', token: tokenRef.current }))
      }

      // Re-subscribe to all channels
      channelsRef.current.forEach((ch) => {
        ws.send(JSON.stringify({ action: 'subscribe', channel: ch }))
      })
    }

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data as string)

        if (msg.type === 'event' && msg.data) {
          const data = msg.data as Record<string, unknown>
          const channel = getChannelFromEvent(data)
          if (!channel) return

          // Get inner data
          const inner =
            (data.OrderBookUpdate as unknown) ||
            (data.TradeEvent as unknown) ||
            (data.TickerUpdate as unknown) ||
            (data.OrderStatusUpdate as unknown)

          const callbacks = subscribersRef.current.get(channel)
          if (callbacks) {
            callbacks.forEach((cb) => cb(inner))
          }
        }
      } catch {
        // ignore malformed messages
      }
    }

    ws.onclose = () => {
      setIsConnected(false)
      wsRef.current = null

      // Exponential backoff reconnect
      const delay = Math.min(reconnectDelayRef.current, 30000)
      reconnectDelayRef.current = delay * 2

      reconnectTimeoutRef.current = setTimeout(connect, delay)
    }

    ws.onerror = () => {
      ws.close()
    }
  }, [])

  useEffect(() => {
    connect()
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      wsRef.current?.close()
    }
  }, [connect])

  const subscribe = useCallback((channel: string, cb: WsCallback): (() => void) => {
    if (!subscribersRef.current.has(channel)) {
      subscribersRef.current.set(channel, new Set())
    }
    subscribersRef.current.get(channel)!.add(cb)

    // Subscribe if not already
    if (!channelsRef.current.has(channel)) {
      channelsRef.current.add(channel)
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ action: 'subscribe', channel }))
      }
    }

    return () => {
      const cbs = subscribersRef.current.get(channel)
      if (cbs) {
        cbs.delete(cb)
        if (cbs.size === 0) {
          subscribersRef.current.delete(channel)
          channelsRef.current.delete(channel)
          if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify({ action: 'unsubscribe', channel }))
          }
        }
      }
    }
  }, [])

  return (
    <WsContext.Provider value={{ subscribe, isConnected }}>
      {children}
    </WsContext.Provider>
  )
}
