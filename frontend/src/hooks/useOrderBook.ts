import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { marketApi } from '@/lib/api'
import { OrderBookResponse, OrderBookUpdateData } from '@/lib/types'
import { useWsContext } from '@/lib/context'
import { toPathSymbol } from '@/lib/symbols'

export function useOrderBook(symbol: string) {
  // symbol is display format e.g. "BTC/USDT"
  const pathSymbol = toPathSymbol(symbol)

  const { data: initial } = useQuery<OrderBookResponse>({
    queryKey: ['orderbook', symbol],
    queryFn: () => marketApi.getOrderBook(pathSymbol),
    staleTime: 0,
  })

  const [orderBook, setOrderBook] = useState<OrderBookResponse>({
    symbol: '',
    bids: [],
    asks: [],
  })

  useEffect(() => {
    if (initial && Array.isArray(initial.bids) && Array.isArray(initial.asks)) {
      setOrderBook(initial)
    }
  }, [initial])

  const { subscribe } = useWsContext()

  useEffect(() => {
    if (!symbol) return
    const channel = `orderbook:${symbol}`
    const unsub = subscribe(channel, (data) => {
      const d = data as OrderBookUpdateData
      if (d.snapshot && Array.isArray(d.snapshot.bids) && Array.isArray(d.snapshot.asks)) {
        setOrderBook((prev) => ({ ...prev, ...d.snapshot }))
      }
    })
    return unsub
  }, [symbol, subscribe])

  return orderBook
}
