'use client'

import { useEffect, useRef, useState } from 'react'
import { useRecentTrades } from '@/hooks/useRecentTrades'
import { formatPrice } from '@/lib/symbols'
import { TradeResponse } from '@/lib/types'

interface RecentTradesProps {
  symbol: string      // "BTC/USDT"
  quoteAsset: string
  baseAsset: string
}

export function RecentTrades({ symbol, quoteAsset, baseAsset }: RecentTradesProps) {
  const trades = useRecentTrades(symbol)
  const prevPriceRef = useRef<number | null>(null)

  return (
    <div className="flex flex-col h-full">
      <div className="px-3 py-2 border-b border-bg-border flex-shrink-0">
        <span className="text-sm font-semibold text-text-primary">Market Trades</span>
      </div>

      {/* Column headers */}
      <div className="grid grid-cols-3 px-3 py-1 border-b border-bg-border flex-shrink-0">
        <span className="text-xs uppercase tracking-wider text-text-muted">Price</span>
        <span className="text-xs uppercase tracking-wider text-text-muted text-right">Amount</span>
        <span className="text-xs uppercase tracking-wider text-text-muted text-right">Time</span>
      </div>

      <div className="flex-1 overflow-y-auto">
        {trades.map((trade, i) => {
          const price = parseFloat(trade.price)
          const prevPrice = i < trades.length - 1 ? parseFloat(trades[i + 1].price) : price
          const isUp = price >= prevPrice
          return (
            <TradeRow
              key={trade.id}
              trade={trade}
              isUp={isUp}
              quoteAsset={quoteAsset}
            />
          )
        })}
      </div>
    </div>
  )
}

function TradeRow({
  trade,
  isUp,
  quoteAsset,
}: {
  trade: TradeResponse
  isUp: boolean
  quoteAsset: string
}) {
  const time = new Date(trade.created_at)
  const timeStr = `${time.getHours().toString().padStart(2, '0')}:${time
    .getMinutes()
    .toString()
    .padStart(2, '0')}:${time.getSeconds().toString().padStart(2, '0')}`

  return (
    <div className="grid grid-cols-3 px-3 py-[3px] hover:bg-bg-elevated/40 transition-colors duration-100">
      <span
        className={`text-xs font-mono ${isUp ? 'text-buy' : 'text-sell'}`}
      >
        {formatPrice(trade.price, quoteAsset)}
      </span>
      <span className="text-xs font-mono text-text-secondary text-right">
        {parseFloat(trade.quantity).toFixed(4)}
      </span>
      <span className="text-xs font-mono text-text-muted text-right">{timeStr}</span>
    </div>
  )
}
