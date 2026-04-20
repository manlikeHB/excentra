'use client'

import { useTicker } from '@/hooks/useTicker'
import { formatPrice, formatCompact } from '@/lib/symbols'
import { cn } from '@/lib/utils'

const PAIRS = [
  { symbol: 'BTC/USDT', base: 'BTC', quote: 'USDT' },
  { symbol: 'ETH/USDT', base: 'ETH', quote: 'USDT' },
  { symbol: 'SOL/USDT', base: 'SOL', quote: 'USDT' },
]

interface PairSelectorProps {
  activePair: string
  onPairChange: (pair: string) => void
}

export function PairSelector({ activePair, onPairChange }: PairSelectorProps) {
  const ticker = useTicker(activePair)
  const changePct = ticker ? parseFloat(ticker.price_change_pct) : 0
  const isPositive = changePct >= 0

  return (
    <div className="h-14 flex items-center gap-6 px-4 border-b border-bg-border bg-bg-surface flex-shrink-0">
      {/* Pair tabs */}
      <div className="flex items-center gap-1">
        {PAIRS.map((pair) => (
          <button
            key={pair.symbol}
            onClick={() => onPairChange(pair.symbol)}
            className={cn(
              'px-3 py-1.5 text-sm rounded-full transition-all duration-150',
              activePair === pair.symbol
                ? 'bg-bg-elevated text-text-primary font-medium'
                : 'text-text-secondary hover:text-text-primary'
            )}
          >
            {pair.symbol}
          </button>
        ))}
      </div>

      {/* Divider */}
      <div className="h-5 w-px bg-bg-border" />

      {/* Ticker strip */}
      {ticker && (
        <div className="flex items-center gap-6">
          <div className="flex items-baseline gap-1.5">
            <span
              className={cn(
                'text-xl font-semibold font-mono tabular-nums',
                isPositive ? 'text-buy' : 'text-sell'
              )}
            >
              {formatPrice(ticker.last_price, 'USDT')}
            </span>
            <span
              className={cn(
                'text-xs font-mono',
                isPositive ? 'text-buy' : 'text-sell'
              )}
            >
              {isPositive ? '+' : ''}{changePct.toFixed(2)}%
            </span>
          </div>

          <TickerStat label="24h High" value={formatPrice(ticker.high_24h, 'USDT')} />
          <TickerStat label="24h Low" value={formatPrice(ticker.low_24h, 'USDT')} />
          <TickerStat
            label="24h Volume"
            value={`${formatCompact(ticker.volume_24h)} ${activePair.split('/')[0]}`}
          />
        </div>
      )}
    </div>
  )
}

function TickerStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-xs text-text-muted">{label}</span>
      <span className="text-sm font-mono text-text-secondary">{value}</span>
    </div>
  )
}
