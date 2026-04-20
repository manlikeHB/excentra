'use client'

import { useState } from 'react'
import { Navbar } from '@/components/shared/Navbar'
import { PairSelector } from '@/components/trading/PairSelector'
import { OrderBook } from '@/components/trading/OrderBook'
import { PriceChart } from '@/components/trading/PriceChart'
import { OrderForm } from '@/components/trading/OrderForm'
import { RecentTrades } from '@/components/trading/RecentTrades'
import { MarketsPanel } from '@/components/trading/MarketsPanel'
import { OpenOrdersPanel } from '@/components/trading/OpenOrdersPanel'
import { useAuth } from '@/lib/context'

const PAIR_INFO: Record<string, { base: string; quote: string }> = {
  'BTC/USDT': { base: 'BTC', quote: 'USDT' },
  'ETH/USDT': { base: 'ETH', quote: 'USDT' },
  'SOL/USDT': { base: 'SOL', quote: 'USDT' },
}

export default function TradePage() {
  const [activePair, setActivePair] = useState('BTC/USDT')
  const [formPrice, setFormPrice] = useState<string | undefined>()
  const { user } = useAuth()

  const { base, quote } = PAIR_INFO[activePair] ?? { base: 'BTC', quote: 'USDT' }

  return (
    <div className="flex flex-col h-screen overflow-hidden bg-bg-base">
      {/* Navbar */}
      <Navbar />

      {/* Pair selector + ticker */}
      <PairSelector activePair={activePair} onPairChange={setActivePair} />

      {/* Main trading grid */}
      <div className="flex flex-1 overflow-hidden min-h-0">
        {/* Left: Order Book */}
        <div className="w-[260px] flex-shrink-0 border-r border-bg-border bg-bg-surface overflow-hidden">
          <OrderBook
            symbol={activePair}
            quoteAsset={quote}
            onPriceClick={setFormPrice}
          />
        </div>

        {/* Center: Chart + Order form + Open orders */}
        <div className="flex-1 flex flex-col overflow-hidden min-w-0">
          {/* Price chart */}
          <div className="border-b border-bg-border bg-bg-base" style={{ height: '55%', minHeight: 0 }}>
            <PriceChart symbol={activePair} />
          </div>

          {/* Order form + Open orders split */}
          <div className="flex flex-1 min-h-0">
            <div className="w-[260px] flex-shrink-0 border-r border-bg-border bg-bg-surface overflow-y-auto">
              <OrderForm
                symbol={activePair}
                baseAsset={base}
                quoteAsset={quote}
              />
            </div>
            <div className="flex-1 min-h-0 overflow-hidden bg-bg-surface">
              <OpenOrdersPanel user={user} symbol={activePair} />
            </div>
          </div>
        </div>

        {/* Right: Recent Trades + Markets */}
        <div className="w-[240px] flex-shrink-0 border-l border-bg-border bg-bg-surface flex flex-col overflow-hidden">
          <div className="flex-1 overflow-hidden border-b border-bg-border">
            <RecentTrades symbol={activePair} quoteAsset={quote} baseAsset={base} />
          </div>
          <div style={{ height: '160px', flexShrink: 0 }}>
            <MarketsPanel activePair={activePair} onPairChange={setActivePair} />
          </div>
        </div>
      </div>
    </div>
  )
}
