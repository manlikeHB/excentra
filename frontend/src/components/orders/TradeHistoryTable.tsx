'use client'

import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { tradesApi } from '@/lib/api'
import { TradeResponse, PaginatedResponse } from '@/lib/types'
import { formatPrice, formatDecimal } from '@/lib/symbols'
import { cn } from '@/lib/utils'
import { Skeleton } from '@/components/shared/Skeleton'

interface TradeHistoryTableProps {
  compact?: boolean
}

export function TradeHistoryTable({ compact = false }: TradeHistoryTableProps) {
  const [page, setPage] = useState(1)

  const { data, isLoading } = useQuery<PaginatedResponse<TradeResponse>>({
    queryKey: ['trades', 'me', page],
    queryFn: () =>
      tradesApi.mine({ page, limit: compact ? 10 : 20, order: 'desc' }),
  })

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-bg-border">
              <Th>Pair</Th>
              <Th>Side</Th>
              <Th align="right">Price</Th>
              <Th align="right">Amount</Th>
              <Th align="right">Total</Th>
              <Th align="right">Date</Th>
            </tr>
          </thead>
          <tbody>
            {isLoading
              ? Array.from({ length: 5 }).map((_, i) => (
                  <tr key={i} className="border-b border-bg-border/40">
                    {Array.from({ length: 6 }).map((_, j) => (
                      <td key={j} className="px-3 py-1.5">
                        <Skeleton className="h-3 w-16" />
                      </td>
                    ))}
                  </tr>
                ))
              : data?.data?.map((trade) => (
                  <TradeRow key={trade.id} trade={trade} />
                ))}
          </tbody>
        </table>

        {!isLoading && data?.data?.length === 0 && (
          <div className="flex items-center justify-center py-8 text-text-muted text-sm">
            No trade history
          </div>
        )}
      </div>

      {!compact && data?.total != null && data.total > data.limit && (
        <div className="flex items-center justify-between px-3 py-2 border-t border-bg-border flex-shrink-0">
          <span className="text-xs text-text-muted">{data.total} total</span>
          <div className="flex gap-1">
            <button
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page === 1}
              className="px-2 py-1 text-xs border border-bg-border rounded text-text-secondary hover:bg-bg-elevated disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Prev
            </button>
            <span className="px-2 py-1 text-xs text-text-muted">{page}</span>
            <button
              onClick={() => setPage((p) => p + 1)}
              disabled={page * data.limit >= data.total}
              className="px-2 py-1 text-xs border border-bg-border rounded text-text-secondary hover:bg-bg-elevated disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

function Th({ children, align = 'left' }: { children?: React.ReactNode; align?: 'left' | 'right' }) {
  return (
    <th
      className={cn(
        'px-3 py-1.5 text-xs uppercase tracking-wider text-text-muted font-normal',
        align === 'right' ? 'text-right' : 'text-left'
      )}
    >
      {children}
    </th>
  )
}

function TradeRow({ trade }: { trade: TradeResponse }) {
  const quoteAsset = trade.symbol.split('/')[1] ?? 'USDT'
  const total = parseFloat(trade.price) * parseFloat(trade.quantity)

  return (
    <tr className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100">
      <td className="px-3 py-1.5 text-xs text-text-secondary">{trade.symbol}</td>
      <td className="px-3 py-1.5">
        <span
          className={cn(
            'text-xs px-1.5 py-0.5 rounded font-medium',
            trade.side === 'buy' ? 'bg-buy/10 text-buy' : 'bg-sell/10 text-sell'
          )}
        >
          {trade.side}
        </span>
      </td>
      <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
        {formatPrice(trade.price, quoteAsset)}
      </td>
      <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
        {formatDecimal(trade.quantity, 6)}
      </td>
      <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
        {formatPrice(total, quoteAsset)}
      </td>
      <td className="px-3 py-1.5 text-xs font-mono text-text-muted text-right">
        {new Date(trade.created_at).toLocaleString('en-US', {
          month: 'short',
          day: 'numeric',
          hour: '2-digit',
          minute: '2-digit',
        })}
      </td>
    </tr>
  )
}
