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
import { UserResponse, WsEvent } from './types'

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

function getChannelFromEvent(data: WsEvent): string | null {
  if (data.type === 'order_book') return `orderbook:${data.symbol}`
  if (data.type === 'trade') return `trades:${data.symbol}`
  if (data.type === 'ticker') return `ticker:${data.symbol}`
  if (data.type === 'order_status') return `orders:${data.user_id}`
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
    const existing = wsRef.current
    if (existing && (existing.readyState === WebSocket.CONNECTING || existing.readyState === WebSocket.OPEN)) {
      return
    }

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
          const data = msg.data as WsEvent
          const channel = getChannelFromEvent(data)
          if (!channel) return

          const callbacks = subscribersRef.current.get(channel)
          if (callbacks) {
              console.log("[WS dispatch]", channel, "callbacks firing:", callbacks.size);
            callbacks.forEach((cb) => cb(data))
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
        reconnectTimeoutRef.current = null
      }
      const ws = wsRef.current
      wsRef.current = null
      if (ws) {
        ws.onclose = null
        ws.onerror = null
        ws.onmessage = null
        ws.onopen = null
        ws.close()
      }
    }
  }, [connect])

  const subscribe = useCallback((channel: string, cb: WsCallback): (() => void) => {
    console.log(
      "[WS subscribe]",
      channel,
      "total subs for channel:",
      (subscribersRef.current.get(channel)?.size ?? 0) + 1,
    );

    if (!subscribersRef.current.has(channel)) {
      subscribersRef.current.set(channel, new Set());
    }
    subscribersRef.current.get(channel)!.add(cb);

    if (!channelsRef.current.has(channel)) {
      channelsRef.current.add(channel);
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        console.log("[WS send subscribe]", channel);
        wsRef.current.send(JSON.stringify({ action: "subscribe", channel }));
      } else {
        console.log("[WS subscribe queued — ws not open]", channel);
      }
    }

    return () => {
      console.log("[WS unsubscribe]", channel);
      const cbs = subscribersRef.current.get(channel);
      if (cbs) {
        cbs.delete(cb);
        console.log("[WS unsubscribe] remaining subs:", cbs.size);
        if (cbs.size === 0) {
          subscribersRef.current.delete(channel);
          channelsRef.current.delete(channel);
          if (wsRef.current?.readyState === WebSocket.OPEN) {
            console.log("[WS send unsubscribe]", channel);
            wsRef.current.send(JSON.stringify({ action: "unsubscribe", channel }));
          }
        }
      }
    };
  }, []);

  return (
    <WsContext.Provider value={{ subscribe, isConnected }}>
      {children}
    </WsContext.Provider>
  )
}
