'use client'

import { useMemo } from 'react'
import { useOrderBook } from '@/hooks/useOrderBook'
import { formatPrice } from '@/lib/symbols'
import { Skeleton } from '@/components/shared/Skeleton'

interface OrderBookProps {
  symbol: string       // "BTC/USDT"
  quoteAsset: string   // "USDT"
  onPriceClick?: (price: string) => void
}

const ROWS = 14

export function OrderBook({ symbol, quoteAsset, onPriceClick }: OrderBookProps) {
  const ob = useOrderBook(symbol)

  const asks = useMemo(() => (ob.asks ?? []).slice(0, ROWS), [ob.asks])
  const bids = useMemo(() => (ob.bids ?? []).slice(0, ROWS), [ob.bids])

  const maxAskQty = useMemo(
    () => asks.reduce((max, entry) => Math.max(max, parseFloat(entry.quantity)), 0),
    [asks],
  );
  const maxBidQty = useMemo(
    () => bids.reduce((max, entry) => Math.max(max, parseFloat(entry.quantity)), 0),
    [bids],
  );

  const spread = useMemo(() => {
    if (!asks.length || !bids.length) return null
    const bestAsk = parseFloat(asks[0].price);
    const bestBid = parseFloat(bids[0].price);
    const spreadVal = bestAsk - bestBid
    const spreadPct = (spreadVal / bestAsk) * 100
    return { value: spreadVal, pct: spreadPct }
  }, [asks, bids])

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-bg-border flex-shrink-0">
        <span className="text-sm font-semibold text-text-primary">Order Book</span>
        {spread && (
          <span className="text-xs text-text-muted font-mono">
            {formatPrice(spread.value, quoteAsset)} ({spread.pct.toFixed(3)}%)
          </span>
        )}
      </div>

      {/* Column headers */}
      <div className="grid grid-cols-3 px-3 py-1 border-b border-bg-border flex-shrink-0">
        <span className="text-xs uppercase tracking-wider text-text-muted">Price</span>
        <span className="text-xs uppercase tracking-wider text-text-muted text-right">Amount</span>
        <span className="text-xs uppercase tracking-wider text-text-muted text-right">Total</span>
      </div>

      {/* Asks (displayed bottom to top, best ask closest to spread) */}
      <div className="flex flex-col-reverse overflow-hidden" style={{ flex: '1 1 0' }}>
        {asks.length === 0
          ? Array.from({ length: ROWS }).map((_, i) => (
              <div key={i} className="px-3 py-1">
                <Skeleton className="h-3 w-full" />
              </div>
            ))
          : asks.map((entry, i) => {
              const price = entry.price;
              const qty = entry.quantity;
              const total = parseFloat(price) * parseFloat(qty)
              const barWidth = maxAskQty > 0 ? (parseFloat(qty) / maxAskQty) * 100 : 0
              return (
                <OrderRow
                  key={`ask-${i}`}
                  price={price}
                  qty={qty}
                  total={total}
                  side="ask"
                  barWidth={barWidth}
                  quoteAsset={quoteAsset}
                  onPriceClick={onPriceClick}
                />
              )
            })}
      </div>

      {/* Spread row */}
      <div className="flex items-center justify-center px-3 py-1 border-y border-bg-border flex-shrink-0">
        {spread ? (
          <span className="text-xs text-text-muted font-mono">
            Spread: {formatPrice(spread.value, quoteAsset)} ({spread.pct.toFixed(3)}%)
          </span>
        ) : (
          <span className="text-xs text-text-muted">—</span>
        )}
      </div>

      {/* Bids */}
      <div className="overflow-hidden" style={{ flex: '1 1 0' }}>
        {bids.length === 0
          ? Array.from({ length: ROWS }).map((_, i) => (
              <div key={i} className="px-3 py-1">
                <Skeleton className="h-3 w-full" />
              </div>
            ))
          : bids.map((entry, i) => {
              const price = entry.price;
              const qty = entry.quantity;
              const total = parseFloat(price) * parseFloat(qty)
              const barWidth = maxBidQty > 0 ? (parseFloat(qty) / maxBidQty) * 100 : 0
              return (
                <OrderRow
                  key={`bid-${i}`}
                  price={price}
                  qty={qty}
                  total={total}
                  side="bid"
                  barWidth={barWidth}
                  quoteAsset={quoteAsset}
                  onPriceClick={onPriceClick}
                />
              )
            })}
      </div>
    </div>
  )
}

function OrderRow({
  price,
  qty,
  total,
  side,
  barWidth,
  quoteAsset,
  onPriceClick,
}: {
  price: string
  qty: string
  total: number
  side: 'bid' | 'ask'
  barWidth: number
  quoteAsset: string
  onPriceClick?: (price: string) => void
}) {
  const priceColor = side === 'ask' ? 'text-sell' : 'text-buy'
  const barColor = side === 'ask' ? 'bg-sell' : 'bg-buy'

  return (
    <div
      className="relative grid grid-cols-3 px-3 py-[3px] cursor-pointer hover:bg-bg-elevated/50 transition-colors duration-100"
      onClick={() => onPriceClick?.(price)}
    >
      {/* Depth bar */}
      <div
        className={`absolute right-0 top-0 bottom-0 ${barColor} opacity-[0.12]`}
        style={{ width: `${barWidth}%` }}
      />
      <span className={`text-xs font-mono relative z-10 ${priceColor}`}>
        {formatPrice(price, quoteAsset)}
      </span>
      <span className="text-xs font-mono relative z-10 text-text-secondary text-right">
        {parseFloat(qty).toFixed(4)}
      </span>
      <span className="text-xs font-mono relative z-10 text-text-muted text-right">
        {formatPrice(total, quoteAsset)}
      </span>
    </div>
  )
}
