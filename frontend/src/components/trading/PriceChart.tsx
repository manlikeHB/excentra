'use client'

import { useEffect, useRef, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { createChart, CandlestickSeries, IChartApi, ISeriesApi, CandlestickData, UTCTimestamp } from 'lightweight-charts'
import { marketApi } from '@/lib/api'
import { useWsContext } from '@/lib/context'
import { WsEvent, Candle, TradeResponse } from '@/lib/types'
import { toPathSymbol } from '@/lib/symbols'
import { cn } from '@/lib/utils'

interface PriceChartProps {
  symbol: string  // "BTC/USDT"
}

type TimeframeKey = '1m' | '5m' | '15m' | '1h' | '1D'

const TIMEFRAMES: { label: string; key: TimeframeKey; seconds: number }[] = [
  { label: '1m', key: '1m', seconds: 60 },
  { label: '5m', key: '5m', seconds: 300 },
  { label: '15m', key: '15m', seconds: 900 },
  { label: '1h', key: '1h', seconds: 3600 },
  { label: '1D', key: '1D', seconds: 86400 },
]

const TIMEFRAME_LIMITS: Record<TimeframeKey, number> = {
  '1m':  500,
  '5m':  1500,
  '15m': 3000,
  '1h':  5000,
  '1D':  10000,
}

function aggregateCandles(
  trades: { price: string; created_at: string }[],
  intervalSeconds: number
): Candle[] {
  if (!trades.length) return []

  const buckets = new Map<number, { o: number; h: number; l: number; c: number }>()

  // Sort oldest first
  const sorted = [...trades].sort(
    (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
  )

  for (const trade of sorted) {
    const ts = Math.floor(new Date(trade.created_at).getTime() / 1000)
    const bucket = Math.floor(ts / intervalSeconds) * intervalSeconds
    const price = parseFloat(trade.price)

    if (!buckets.has(bucket)) {
      buckets.set(bucket, { o: price, h: price, l: price, c: price })
    } else {
      const b = buckets.get(bucket)!
      b.h = Math.max(b.h, price)
      b.l = Math.min(b.l, price)
      b.c = price
    }
  }

  return Array.from(buckets.entries())
    .sort(([a], [b]) => a - b)
    .map(([time, { o, h, l, c }]) => ({ time, open: o, high: h, low: l, close: c }))
}

export function PriceChart({ symbol }: PriceChartProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const chartRef = useRef<IChartApi | null>(null)
  const seriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null)
  const [timeframe, setTimeframe] = useState<TimeframeKey>('1m')
  const { subscribe } = useWsContext()

  const pathSymbol = toPathSymbol(symbol)
  const intervalSeconds = TIMEFRAMES.find((t) => t.key === timeframe)!.seconds

  const { data: trades = [] } = useQuery<TradeResponse[]>({
    queryKey: ['chart-trades', symbol, timeframe],
    queryFn: () => marketApi.getTrades(pathSymbol, TIMEFRAME_LIMITS[timeframe]),
    staleTime: 10_000,
  })

  // Initialize chart
  useEffect(() => {
    if (!containerRef.current) return

    const chart = createChart(containerRef.current, {
      width: containerRef.current.clientWidth,
      height: containerRef.current.clientHeight,
      layout: {
        background: { color: '#0A0B0D' },
        textColor: '#94A3B8',
      },
      grid: {
        vertLines: { color: '#1F232C' },
        horzLines: { color: '#1F232C' },
      },
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        borderColor: '#1F232C',
      },
      rightPriceScale: {
        borderColor: '#1F232C',
      },
      crosshair: {
        vertLine: { color: '#475569', width: 1, style: 1 },
        horzLine: { color: '#475569', width: 1, style: 1 },
      },
    })

    const candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: '#10B981',
      downColor: '#EF4444',
      borderUpColor: '#10B981',
      borderDownColor: '#EF4444',
      wickUpColor: '#10B981',
      wickDownColor: '#EF4444',
    })

    chartRef.current = chart
    seriesRef.current = candleSeries

    // ResizeObserver
    const ro = new ResizeObserver(() => {
      if (containerRef.current && chartRef.current) {
        chartRef.current.applyOptions({
          width: containerRef.current.clientWidth,
          height: containerRef.current.clientHeight,
        })
      }
    })
    ro.observe(containerRef.current)

    return () => {
      ro.disconnect()
      chart.remove()
      chartRef.current = null
      seriesRef.current = null
    }
  }, [])

  // Update candles when trades or timeframe changes
  useEffect(() => {
    if (!seriesRef.current || !trades.length) return
    const candles = aggregateCandles(trades, intervalSeconds)
    const chartData: CandlestickData[] = candles.map((c) => ({
      time: c.time as UTCTimestamp,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
    }))
    seriesRef.current.setData(chartData)
    chartRef.current?.timeScale().fitContent()
  }, [trades, intervalSeconds])

  // Subscribe to live trades
  useEffect(() => {
    if (!symbol) return
    const channel = `trades:${symbol}`
    const unsub = subscribe(channel, (data) => {
      const d = data as Extract<WsEvent, { type: 'trade' }>
      if (!seriesRef.current) return
      const ts = Math.floor(new Date(d.created_at).getTime() / 1000)
      const bucket = (Math.floor(ts / intervalSeconds) * intervalSeconds) as UTCTimestamp
      const price = parseFloat(d.price)

      try {
        seriesRef.current.update({
          time: bucket,
          open: price,
          high: price,
          low: price,
          close: price,
        })
      } catch {
        // ignore update errors
      }
    })
    return unsub
  }, [symbol, subscribe, intervalSeconds])

  return (
    <div className="flex flex-col h-full">
      {/* Timeframe selector */}
      <div className="flex items-center gap-1 px-3 py-2 border-b border-bg-border flex-shrink-0">
        {TIMEFRAMES.map((tf) => (
          <button
            key={tf.key}
            onClick={() => setTimeframe(tf.key)}
            className={cn(
              'px-2.5 py-1 text-xs rounded transition-all duration-150',
              timeframe === tf.key
                ? 'bg-bg-elevated text-text-primary font-medium'
                : 'text-text-muted hover:text-text-secondary'
            )}
          >
            {tf.label}
          </button>
        ))}
      </div>

      {/* Chart */}
      <div ref={containerRef} className="flex-1 min-h-0" />
    </div>
  )
}
