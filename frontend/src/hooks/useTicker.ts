import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { marketApi } from '@/lib/api'
import { TickerResponse, TickerUpdateData } from '@/lib/types'
import { useWsContext } from '@/lib/context'
import { toPathSymbol } from '@/lib/symbols'

export function useTicker(symbol: string) {
  // symbol is display format e.g. "BTC/USDT"
  const pathSymbol = toPathSymbol(symbol)

  useEffect(() => {
    setTicker(null);
  }, [symbol]);

  const { data: initial } = useQuery<TickerResponse>({
    queryKey: ['ticker', symbol],
    queryFn: () => marketApi.getTicker(pathSymbol) as Promise<TickerResponse>,
    staleTime: 5_000,
    retry: false,
  })

  const [ticker, setTicker] = useState<TickerResponse | null>(null)

  useEffect(() => {
    if (initial) setTicker(initial)
  }, [initial])

  const { subscribe } = useWsContext()

  useEffect(() => {
    if (!symbol) return
    const channel = `ticker:${symbol}`
    const unsub = subscribe(channel, (data) => {
      const d = data as TickerUpdateData
      setTicker({
        symbol: d.symbol,
        last_price: d.last_price,
        high_24h: d.high_24h,
        low_24h: d.low_24h,
        volume_24h: d.volume_24h,
        price_change_pct: d.price_change_pct,
      })
    })
    return unsub
  }, [symbol, subscribe])

  return ticker
}

export function useAllTickers() {
  const { data: tickers } = useQuery<TickerResponse[]>({
    queryKey: ['ticker', 'all'],
    queryFn: () => marketApi.getTicker() as Promise<TickerResponse[]>,
    refetchInterval: 10_000,
  })

  const [liveMap, setLiveMap] = useState<Map<string, TickerResponse>>(new Map())
  const { subscribe } = useWsContext()

  useEffect(() => {
    if (!tickers) return
    const map = new Map<string, TickerResponse>()
    tickers.forEach((t) => map.set(t.symbol, t))
    setLiveMap(map)
  }, [tickers])

  // Subscribe to each pair's ticker via WS (handled per pair in components)

  return liveMap
}
