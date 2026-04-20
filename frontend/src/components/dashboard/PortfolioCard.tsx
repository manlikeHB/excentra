'use client'

import { useQuery } from '@tanstack/react-query'
import { balancesApi, marketApi } from '@/lib/api'
import { BalanceResponse, TickerResponse } from '@/lib/types'
import { formatDecimal } from '@/lib/symbols'
import { Skeleton } from '@/components/shared/Skeleton'

const FALLBACK_PRICES: Record<string, number> = {
  USDT: 1,
  BTC: 65000,
  ETH: 3200,
  SOL: 150,
}

export function PortfolioCard() {
  const { data: balances, isLoading: balancesLoading } = useQuery<BalanceResponse[]>({
    queryKey: ['balances'],
    queryFn: () => balancesApi.list(),
  })

  const { data: btcTicker, isLoading: btcLoading } = useQuery<TickerResponse>({
    queryKey: ['ticker', 'BTC/USDT'],
    queryFn: () => marketApi.getTicker('BTC/USDT') as Promise<TickerResponse>,
  })

  const { data: ethTicker, isLoading: ethLoading } = useQuery<TickerResponse>({
    queryKey: ['ticker', 'ETH/USDT'],
    queryFn: () => marketApi.getTicker('ETH/USDT') as Promise<TickerResponse>,
  })

  const { data: solTicker, isLoading: solLoading } = useQuery<TickerResponse>({
    queryKey: ['ticker', 'SOL/USDT'],
    queryFn: () => marketApi.getTicker('SOL/USDT') as Promise<TickerResponse>,
  })

  const isLoading = balancesLoading || btcLoading || ethLoading || solLoading

  const prices: Record<string, number> = {
    USDT: 1,
    BTC: btcTicker ? parseFloat(btcTicker.last_price) : FALLBACK_PRICES.BTC,
    ETH: ethTicker ? parseFloat(ethTicker.last_price) : FALLBACK_PRICES.ETH,
    SOL: solTicker ? parseFloat(solTicker.last_price) : FALLBACK_PRICES.SOL,
  }

  const totalValue = balances?.reduce((sum, b) => {
    const price = prices[b.asset] ?? 0
    const total = (parseFloat(b.available) + parseFloat(b.held)) * price
    return sum + total
  }, 0) ?? 0

  return (
    <div className="bg-bg-surface border border-bg-border rounded-lg p-6">
      <p className="text-xs uppercase tracking-wider text-text-muted mb-2">
        Total Portfolio Value
      </p>
      {isLoading ? (
        <Skeleton className="h-8 w-40 mb-2" />
      ) : (
        <p className="text-3xl font-semibold font-mono text-text-primary">
          ${formatDecimal(totalValue, 2)}
        </p>
      )}
      <p className="text-xs text-text-muted mt-1">
        Estimated value in USD
      </p>
    </div>
  )
}
