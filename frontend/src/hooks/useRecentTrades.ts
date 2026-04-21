import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { marketApi } from '@/lib/api'
import { TradeResponse, WsEvent } from '@/lib/types'
import { useWsContext } from '@/lib/context'
import { toPathSymbol } from '@/lib/symbols'

const MAX_TRADES = 50

export function useRecentTrades(symbol: string) {
  // symbol is display format e.g. "BTC/USDT"
  const pathSymbol = toPathSymbol(symbol)

  const { data: initial } = useQuery<TradeResponse[]>({
    queryKey: ['trades', symbol],
    queryFn: () => marketApi.getTrades(pathSymbol, MAX_TRADES),
    staleTime: 5_000,
  })

  const [trades, setTrades] = useState<TradeResponse[]>([])

  useEffect(() => {
    if (initial) setTrades(initial)
  }, [initial])

  const { subscribe } = useWsContext()

  useEffect(() => {
    if (!symbol) return
    const channel = `trades:${symbol}`
    const unsub = subscribe(channel, (data) => {
      const d = data as Extract<WsEvent, { type: 'trade' }>
      const newTrade: TradeResponse = {
        id: `${d.created_at}-${Math.random()}`,
        symbol: d.symbol,
        taker_side: d.taker_side,
        price: d.price,
        quantity: d.quantity,
        created_at: d.created_at,
      }
      setTrades((prev) => [newTrade, ...prev].slice(0, MAX_TRADES))
    })
    return unsub
  }, [symbol, subscribe])

  return trades
}
