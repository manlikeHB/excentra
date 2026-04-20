'use client'

import { useTicker } from '@/hooks/useTicker'
import { formatPrice } from '@/lib/symbols'
import { cn } from '@/lib/utils'

const PAIRS = [
  { symbol: 'BTC/USDT', base: 'BTC', quote: 'USDT' },
  { symbol: 'ETH/USDT', base: 'ETH', quote: 'USDT' },
  { symbol: 'SOL/USDT', base: 'SOL', quote: 'USDT' },
]

interface MarketsPanelProps {
  activePair: string
  onPairChange: (pair: string) => void
}

export function MarketsPanel({ activePair, onPairChange }: MarketsPanelProps) {
  return (
    <div className="flex flex-col h-full">
      <div className="px-3 py-2 border-b border-bg-border flex-shrink-0">
        <span className="text-sm font-semibold text-text-primary">Markets</span>
      </div>
      <div className="flex-1 overflow-y-auto">
        {PAIRS.map((pair) => (
          <MarketRow
            key={pair.symbol}
            symbol={pair.symbol}
            isActive={activePair === pair.symbol}
            onClick={() => onPairChange(pair.symbol)}
          />
        ))}
      </div>
    </div>
  )
}

function MarketRow({
  symbol,
  isActive,
  onClick,
}: {
  symbol: string
  isActive: boolean
  onClick: () => void
}) {
  const ticker = useTicker(symbol)
  const changePct = ticker ? parseFloat(ticker.price_change_pct) : 0
  const isPositive = changePct >= 0

  return (
    <button
      onClick={onClick}
      className={cn(
        'w-full flex items-center justify-between px-3 py-2.5 transition-colors duration-100 text-left',
        isActive ? 'bg-bg-elevated' : 'hover:bg-bg-elevated/50'
      )}
    >
      <div>
        <p className="text-sm font-medium text-text-primary">{symbol}</p>
      </div>
      <div className="text-right">
        <p className="text-sm font-mono text-text-secondary">
          {ticker ? formatPrice(ticker.last_price, 'USDT') : '—'}
        </p>
        {ticker && (
          <p
            className={cn(
              'text-xs font-mono',
              isPositive ? 'text-buy' : 'text-sell'
            )}
          >
            {isPositive ? '+' : ''}{changePct.toFixed(2)}%
          </p>
        )}
      </div>
    </button>
  )
}
