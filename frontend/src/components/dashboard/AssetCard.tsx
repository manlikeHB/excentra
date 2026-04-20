'use client'

import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { marketApi } from '@/lib/api'
import { BalanceResponse, TickerResponse } from '@/lib/types'
import { formatDecimal } from '@/lib/symbols'
import { DepositWithdrawModal } from './DepositWithdrawModal'
import { cn } from '@/lib/utils'

const FALLBACK_PRICES: Record<string, number> = {
  USDT: 1,
  BTC: 65000,
  ETH: 3200,
  SOL: 150,
}

const ASSET_COLORS: Record<string, string> = {
  BTC: 'bg-orange-500/20 text-orange-400',
  ETH: 'bg-blue-500/20 text-blue-400',
  SOL: 'bg-purple-500/20 text-purple-400',
  USDT: 'bg-green-500/20 text-green-400',
}

interface AssetCardProps {
  balance: BalanceResponse
}

export function AssetCard({ balance }: AssetCardProps) {
  const [modal, setModal] = useState<'deposit' | 'withdraw' | null>(null)

  const symbol = balance.asset !== 'USDT' ? `${balance.asset}/USDT` : null
  const { data: ticker } = useQuery<TickerResponse>({
    queryKey: ['ticker', symbol],
    queryFn: () => marketApi.getTicker(symbol!) as Promise<TickerResponse>,
    enabled: symbol !== null,
  })

  const livePrice = ticker ? parseFloat(ticker.last_price) : null
  const price = balance.asset === 'USDT' ? 1 : (livePrice ?? FALLBACK_PRICES[balance.asset] ?? 0)
  const total = parseFloat(balance.available) + parseFloat(balance.held)
  const usdValue = total * price

  const canWithdraw = parseFloat(balance.available) > 0

  return (
    <>
      <div className="bg-bg-surface border border-bg-border rounded-lg p-4 flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div
              className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold ${ASSET_COLORS[balance.asset] ?? 'bg-bg-elevated text-text-secondary'}`}
            >
              {balance.asset[0]}
            </div>
            <span className="text-sm font-semibold text-text-primary">{balance.asset}</span>
          </div>
          <span className="text-sm font-mono text-text-muted">
            ${formatDecimal(usdValue, 2)}
          </span>
        </div>

        <div className="space-y-1">
          <div className="flex items-center justify-between">
            <span className="text-xs text-text-muted">Available</span>
            <span className="text-xs font-mono text-text-secondary">
              {formatDecimal(balance.available, balance.asset === 'USDT' ? 2 : 6)}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-xs text-text-muted">In Orders</span>
            <span className="text-xs font-mono text-text-secondary">
              {formatDecimal(balance.held, balance.asset === 'USDT' ? 2 : 6)}
            </span>
          </div>
        </div>

        <div className="flex gap-2">
          <button
            onClick={() => setModal('deposit')}
            className="flex-1 py-1.5 text-xs font-medium bg-accent/10 text-accent border border-accent/20 rounded-md hover:bg-accent/20 transition-all duration-150 active:scale-[0.98]"
          >
            Deposit
          </button>

          <button
            onClick={() => canWithdraw && setModal('withdraw')}
            disabled={!canWithdraw}
            className={cn(
              'flex-1 py-1.5 text-xs font-medium bg-sell/10 text-sell border border-sell/20 rounded-md transition-all duration-150',
              canWithdraw
                ? 'hover:bg-sell/20 active:scale-[0.98]'
                : 'opacity-40 cursor-not-allowed'
            )}
          >
            Withdraw
          </button>
        </div>
      </div>

      {modal && (
        <DepositWithdrawModal
          asset={balance.asset}
          mode={modal}
          onClose={() => setModal(null)}
          availableBalance={balance.available}
        />
      )}
    </>
  )
}
