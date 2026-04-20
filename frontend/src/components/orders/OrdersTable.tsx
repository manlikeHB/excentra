'use client'

import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { ordersApi } from '@/lib/api'
import { OrderResponse, PaginatedResponse } from '@/lib/types'
import { formatPrice, formatDecimal } from '@/lib/symbols'
import { toast } from 'sonner'
import { X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Skeleton } from '@/components/shared/Skeleton'

type TabKey = 'open' | 'history'

interface OrdersTableProps {
  compact?: boolean
  statusFilter?: TabKey
}

export function OrdersTable({ compact = false, statusFilter }: OrdersTableProps) {
  const [tab, setTab] = useState<TabKey>(statusFilter ?? 'open')
  const [page, setPage] = useState(1)
  const qc = useQueryClient()

  // When statusFilter is provided externally, use it directly
  const activeTab = statusFilter ?? tab

  const statusParam =
    activeTab === 'open' ? 'open' : undefined

  const { data, isLoading } = useQuery<PaginatedResponse<OrderResponse>>({
    queryKey: ['orders', activeTab, page],
    queryFn: () =>
      ordersApi.list({
        status: statusParam,
        page,
        limit: compact ? 10 : 20,
        order: 'desc',
      }),
  })

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
      {/* Tabs — only shown when not controlled externally */}
      {!statusFilter && (
        <div className="flex border-b border-bg-border flex-shrink-0">
          {(['open', 'history'] as TabKey[]).map((t) => (
            <button
              key={t}
              onClick={() => { setTab(t); setPage(1) }}
              className={cn(
                'px-4 py-2 text-xs font-medium transition-all duration-150',
                activeTab === t
                  ? 'text-text-primary border-b-2 border-accent -mb-px'
                  : 'text-text-muted hover:text-text-secondary'
              )}
            >
              {t === 'open' ? 'Open Orders' : 'Order History'}
            </button>
          ))}
        </div>
      )}

      {/* Table */}
      <div className="flex-1 overflow-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-bg-border">
              <Th>Pair</Th>
              <Th>Type</Th>
              <Th>Side</Th>
              <Th align="right">Price</Th>
              <Th align="right">Amount</Th>
              {activeTab === 'open' && <Th align="right">Filled</Th>}
              <Th>Status</Th>
              <Th align="right">Date</Th>
              {activeTab === 'open' && <Th></Th>}
            </tr>
          </thead>
          <tbody>
            {isLoading
              ? Array.from({ length: 5 }).map((_, i) => (
                  <tr key={i} className="border-b border-bg-border/40">
                    {Array.from({ length: activeTab === 'open' ? 9 : 7 }).map((_, j) => (
                      <td key={j} className="px-3 py-1.5">
                        <Skeleton className="h-3 w-16" />
                      </td>
                    ))}
                  </tr>
                ))
              : data?.data?.map((order) => (
                  <OrderRow
                    key={order.id}
                    order={order}
                    showFilled={activeTab === 'open'}
                    showCancel={activeTab === 'open'}
                    onCancel={() => handleCancel(order.id)}
                  />
                ))}
          </tbody>
        </table>

        {!isLoading && data?.data?.length === 0 && (
          <div className="flex items-center justify-center py-8 text-text-muted text-sm">
            No {activeTab === 'open' ? 'open orders' : 'order history'}
          </div>
        )}
      </div>

      {/* Pagination */}
      {!compact && data?.total != null && data.total > data.limit && (
        <div className="flex items-center justify-between px-3 py-2 border-t border-bg-border flex-shrink-0">
          <span className="text-xs text-text-muted">
            {data.total} total
          </span>
          <div className="flex gap-1">
            <button
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page === 1}
              className="px-2 py-1 text-xs border border-bg-border rounded text-text-secondary hover:bg-bg-elevated disabled:opacity-40 disabled:cursor-not-allowed transition-all duration-150"
            >
              Prev
            </button>
            <span className="px-2 py-1 text-xs text-text-muted">{page}</span>
            <button
              onClick={() => setPage((p) => p + 1)}
              disabled={page * data.limit >= data.total}
              className="px-2 py-1 text-xs border border-bg-border rounded text-text-secondary hover:bg-bg-elevated disabled:opacity-40 disabled:cursor-not-allowed transition-all duration-150"
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

function OrderRow({
  order,
  showFilled,
  showCancel,
  onCancel,
}: {
  order: OrderResponse
  showFilled: boolean
  showCancel: boolean
  onCancel: () => void
}) {
  const quoteAsset = order.symbol.split('/')[1] ?? 'USDT'
  const filled = parseFloat(order.quantity) - parseFloat(order.remaining_quantity)

  return (
    <tr className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100">
      <td className="px-3 py-1.5 text-xs text-text-secondary">{order.symbol}</td>
      <td className="px-3 py-1.5 text-xs text-text-secondary capitalize">{order.order_type}</td>
      <td className="px-3 py-1.5">
        <span
          className={cn(
            'text-xs px-1.5 py-0.5 rounded font-medium',
            order.side === 'buy'
              ? 'bg-buy/10 text-buy'
              : 'bg-sell/10 text-sell'
          )}
        >
          {order.side}
        </span>
      </td>
      <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
        {order.price ? formatPrice(order.price, quoteAsset) : 'Market'}
      </td>
      <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
        {formatDecimal(order.quantity, 6)}
      </td>
      {showFilled && (
        <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
          {formatDecimal(filled, 6)}
        </td>
      )}
      <td className="px-3 py-1.5">
        <StatusBadge status={order.status} />
      </td>
      <td className="px-3 py-1.5 text-xs font-mono text-text-muted text-right">
        {new Date(order.created_at).toLocaleString('en-US', {
          month: 'short',
          day: 'numeric',
          hour: '2-digit',
          minute: '2-digit',
        })}
      </td>
      {showCancel && (
        <td className="px-3 py-1.5">
          <button
            onClick={onCancel}
            className="text-text-muted hover:text-sell hover:bg-sell/10 rounded p-0.5 transition-all duration-150"
            title="Cancel order"
          >
            <X size={12} />
          </button>
        </td>
      )}
    </tr>
  )
}

function StatusBadge({ status }: { status: OrderResponse['status'] }) {
  const styles: Record<OrderResponse['status'], string> = {
    open: 'text-accent',
    partially_filled: 'text-yellow-400',
    filled: 'text-buy',
    cancelled: 'text-text-muted',
  }
  const labels: Record<OrderResponse['status'], string> = {
    open: 'Open',
    partially_filled: 'Partial',
    filled: 'Filled',
    cancelled: 'Cancelled',
  }
  return (
    <span className={cn('text-xs', styles[status])}>
      {labels[status]}
    </span>
  )
}
