'use client'

import Link from 'next/link'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { ordersApi } from '@/lib/api'
import { OrderResponse, PaginatedResponse, UserResponse } from '@/lib/types'
import { formatPrice, formatDecimal } from '@/lib/symbols'
import { useWsContext } from '@/lib/context'
import { useEffect } from 'react'
import { toast } from 'sonner'
import { X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Skeleton } from '@/components/shared/Skeleton'

interface OpenOrdersPanelProps {
  user: UserResponse | null
  symbol: string
}

export function OpenOrdersPanel({ user, symbol }: OpenOrdersPanelProps) {
  const qc = useQueryClient()
  const { subscribe } = useWsContext()

  const { data, isLoading, refetch } = useQuery<PaginatedResponse<OrderResponse>>({
    queryKey: ["orders", "open,partially_filled", 1],
    queryFn: () =>
      ordersApi.list({
        status: "open,partially_filled",
        page: 1,
        limit: 10,
        order: "desc",
      }),
    enabled: !!user,
  });

  // Subscribe to order status updates for this user
  useEffect(() => {
    if (!user) return
    const channel = `orders:${user.id}`
    const unsub = subscribe(channel, () => {
      refetch()
    })
    return unsub
  }, [user, subscribe, refetch])

  async function handleCancel(id: string) {
    try {
      await ordersApi.cancel(id)
      toast.success('Order cancelled')
      qc.invalidateQueries({ queryKey: ['orders'] })
      qc.invalidateQueries({ queryKey: ['balances'] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to cancel order')
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-bg-border flex-shrink-0">
        <span className="text-xs font-semibold text-text-primary uppercase tracking-wider">Open Orders</span>
        <Link
          href="/dashboard?tab=orders"
          className="text-xs text-accent hover:text-accent-hover transition-colors duration-150"
        >
          View all →
        </Link>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto min-h-0">
        {!user ? (
          <div className="flex items-center justify-center h-full text-text-muted text-sm">
            Sign in to view orders
          </div>
        ) : isLoading ? (
          <div className="space-y-0">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="px-3 py-2 border-b border-bg-border/40">
                <Skeleton className="h-3 w-full" />
              </div>
            ))}
          </div>
        ) : !data?.data?.length ? (
          <div className="flex items-center justify-center h-full text-text-muted text-sm">
            No open orders
          </div>
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b border-bg-border">
                <Th>Pair</Th>
                <Th>Side</Th>
                <Th align="right">Price</Th>
                <Th align="right">Qty</Th>
                <Th align="right">Filled</Th>
                <Th></Th>
              </tr>
            </thead>
            <tbody>
              {data.data.map((order) => {
                const quoteAsset = order.symbol.split('/')[1] ?? 'USDT'
                const filledQty = parseFloat(order.quantity) - parseFloat(order.remaining_quantity)
                const filledPct = parseFloat(order.quantity) > 0
                  ? ((filledQty / parseFloat(order.quantity)) * 100).toFixed(0)
                  : '0'

                return (
                  <tr
                    key={order.id}
                    className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100"
                  >
                    <td className="px-3 py-1.5 text-xs text-text-secondary">{order.symbol}</td>
                    <td className="px-3 py-1.5">
                      <span
                        className={cn(
                          'text-xs font-medium',
                          order.side === 'buy' ? 'text-buy' : 'text-sell'
                        )}
                      >
                        {order.side}
                      </span>
                    </td>
                    <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                      {order.price ? formatPrice(order.price, quoteAsset) : 'Market'}
                    </td>
                    <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                      {formatDecimal(order.quantity, 4)}
                    </td>
                    <td className="px-3 py-1.5 text-xs font-mono text-text-muted text-right">
                      {filledPct}%
                    </td>
                    <td className="px-3 py-1.5">
                      <button
                        onClick={() => handleCancel(order.id)}
                        className="text-text-muted hover:text-sell hover:bg-sell/10 rounded p-0.5 transition-all duration-150"
                        title="Cancel order"
                      >
                        <X size={12} />
                      </button>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        )}
      </div>
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
