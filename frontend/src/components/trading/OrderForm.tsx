'use client'

import { useState, useEffect, useRef, useMemo } from 'react'
import { useRouter } from 'next/navigation'
import { toast } from 'sonner'
import { useAuth } from '@/lib/context'
import { useOrderBook } from '@/hooks/useOrderBook'
import { usePairs } from '@/hooks/usePairs'
import { ordersApi, balancesApi } from '@/lib/api'
import { formatDecimal, formatPrice } from '@/lib/symbols'
import { cn } from '@/lib/utils'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { BalanceResponse } from '@/lib/types'

interface OrderFormProps {
  symbol: string      // "BTC/USDT"
  baseAsset: string   // "BTC"
  quoteAsset: string  // "USDT"
  initialPrice?: string
}

type Side = 'buy' | 'sell'
type OrderType = 'limit' | 'market'

const PERCENTAGES = [25, 50, 75, 100]

export function OrderForm({ symbol, baseAsset, quoteAsset, initialPrice }: OrderFormProps) {
  const { user } = useAuth()
  const router = useRouter()
  const qc = useQueryClient()
  const ob = useOrderBook(symbol)
  const { data: pairs } = usePairs()

  const [side, setSide] = useState<Side>('buy')
  const [orderType, setOrderType] = useState<OrderType>('limit')
  const [price, setPrice] = useState('')
  const [amount, setAmount] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const hasUserEdited = useRef(false)

  const pair = pairs?.find((p) => p.symbol === symbol)

  const { data: balances } = useQuery<BalanceResponse[]>({
    queryKey: ['balances'],
    queryFn: () => balancesApi.list(),
    enabled: !!user,
  })

  const baseBalance = balances?.find((b) => b.asset === baseAsset)
  const quoteBalance = balances?.find((b) => b.asset === quoteAsset)

  // When initialPrice arrives from an order book click, treat it as user-edited (Fix 3)
  useEffect(() => {
    if (initialPrice) {
      setPrice(initialPrice)
      hasUserEdited.current = true
    }
  }, [initialPrice])

  // Pre-fill price from order book; skip if user has manually edited (Fix 2)
  useEffect(() => {
    if (hasUserEdited.current) return
    if (side === "buy" && ob.asks.length > 0) {
      setPrice(ob.asks[0].price);
    } else if (side === "sell" && ob.bids.length > 0) {
      setPrice(ob.bids[0].price);
    }
  }, [side, ob.asks, ob.bids, orderType])

  const total = useMemo(() => {
    const p = parseFloat(price)
    const a = parseFloat(amount)
    if (isNaN(p) || isNaN(a)) return 0
    return p * a
  }, [price, amount])

  function handlePercentage(pct: number) {
    if (!user) return
    if (side === 'buy' && quoteBalance) {
      const available = parseFloat(quoteBalance.available)
      if (orderType === 'market') {
        const qty = (available * pct) / 100
        setAmount(qty > 0 ? qty.toFixed(2) : '')
      } else {
        const p = parseFloat(price) || 1
        const qty = (available * pct) / 100 / p
        setAmount(qty > 0 ? qty.toFixed(6) : '')
      }
    } else if (side === 'sell' && baseBalance) {
      const available = parseFloat(baseBalance.available)
      const qty = (available * pct) / 100
      setAmount(qty > 0 ? qty.toFixed(6) : '')
    }
  }

  function fillMaxBalance() {
    handlePercentage(100)
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!pair || !user) return

    const qty = parseFloat(amount)
    if (isNaN(qty) || qty <= 0) {
      toast.error('Enter a valid amount')
      return
    }

    if (orderType === 'limit') {
      const p = parseFloat(price)
      if (isNaN(p) || p <= 0) {
        toast.error('Enter a valid price')
        return
      }
    }

    setIsSubmitting(true)
    try {
      await ordersApi.place({
        symbol,
        side,
        order_type: orderType,
        price: orderType === "limit" ? price : undefined,
        quantity: amount,
      });
      toast.success(`${side === 'buy' ? 'Buy' : 'Sell'} order placed`)
      setAmount('')
      qc.invalidateQueries({ queryKey: ['balances'] })
      qc.invalidateQueries({ queryKey: ['orders'] })
      qc.invalidateQueries({ queryKey: ['orderbook', symbol] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Order failed')
    } finally {
      setIsSubmitting(false)
    }
  }

  if (!user) {
    return (
      <div className="flex items-center justify-center h-full px-4">
        <button
          onClick={() => router.push('/login?from=/trade')}
          className="w-full py-2.5 text-sm text-text-secondary border border-bg-border rounded-md hover:bg-bg-elevated hover:text-text-primary transition-all duration-150"
        >
          Sign in to Trade
        </button>
      </div>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3 px-3 py-3 h-full overflow-y-auto">
      {/* Buy/Sell toggle */}
      <div className="grid grid-cols-2 gap-1 p-1 bg-bg-elevated rounded-md">
        <button
          type="button"
          onClick={() => { hasUserEdited.current = false; setSide('buy') }}
          className={cn(
            'py-1.5 text-sm font-semibold rounded transition-all duration-150',
            side === 'buy'
              ? 'bg-buy text-black'
              : 'text-text-secondary hover:text-text-primary'
          )}
        >
          Buy
        </button>
        <button
          type="button"
          onClick={() => { hasUserEdited.current = false; setSide('sell') }}
          className={cn(
            'py-1.5 text-sm font-semibold rounded transition-all duration-150',
            side === 'sell'
              ? 'bg-sell text-white'
              : 'text-text-secondary hover:text-text-primary'
          )}
        >
          Sell
        </button>
      </div>

      {/* Order type tabs */}
      <div className="flex gap-4 border-b border-bg-border">
        {(['limit', 'market'] as const).map((t) => (
          <button
            key={t}
            type="button"
            onClick={() => setOrderType(t)}
            className={cn(
              'pb-1.5 text-xs capitalize transition-all duration-150',
              orderType === t
                ? 'text-text-primary border-b-2 border-accent -mb-px'
                : 'text-text-muted hover:text-text-secondary'
            )}
          >
            {t.charAt(0).toUpperCase() + t.slice(1)}
          </button>
        ))}
      </div>

      {/* Price input (limit only) */}
      {orderType === 'limit' && (
        <div>
          <label className="text-xs text-text-muted mb-1 block">
            Price ({quoteAsset})
          </label>
          <input
            type="number"
            value={price}
            onChange={(e) => {
              hasUserEdited.current = e.target.value !== ''
              setPrice(e.target.value)
            }}
            placeholder="0.00"
            step="any"
            className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm font-mono text-right text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:ring-0 focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150"
          />
        </div>
      )}

      {/* Amount input */}
      <div>
        <label className="text-xs text-text-muted mb-1 block">
          Amount ({orderType === 'market' && side === 'buy' ? quoteAsset : baseAsset})
        </label>
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="0.000000"
          step="any"
          className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm font-mono text-right text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:ring-0 focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150"
        />
      </div>

      {/* Percentage pills */}
      <div className="flex gap-1.5">
        {PERCENTAGES.map((pct) => (
          <button
            key={pct}
            type="button"
            onClick={() => handlePercentage(pct)}
            className="flex-1 text-xs border border-bg-border rounded-full px-2 py-0.5 text-text-muted hover:border-accent hover:text-accent transition-all duration-150"
          >
            {pct}%
          </button>
        ))}
      </div>

      {/* Total (limit only) */}
      {orderType === 'limit' && (
        <div className="flex items-center justify-between">
          <span className="text-xs text-text-muted">Total</span>
          <span className="text-xs font-mono text-text-secondary">
            {formatPrice(total, quoteAsset)} {quoteAsset}
          </span>
        </div>
      )}

      {/* Available balance */}
      <button
        type="button"
        onClick={fillMaxBalance}
        className="flex items-center justify-between text-xs text-text-muted hover:text-text-secondary transition-colors duration-150 text-left"
      >
        <span>Available</span>
        <span className="font-mono">
          {side === 'buy'
            ? `${formatDecimal(quoteBalance?.available ?? '0', 2)} ${quoteAsset}`
            : `${formatDecimal(baseBalance?.available ?? '0', 6)} ${baseAsset}`}
        </span>
      </button>

      {/* Submit button */}
      <button
        type="submit"
        disabled={isSubmitting}
        className={cn(
          'w-full py-2.5 text-sm font-semibold rounded-md transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed',
          side === 'buy'
            ? 'bg-buy text-black hover:bg-buy/90'
            : 'bg-sell text-white hover:bg-sell/90'
        )}
      >
        {isSubmitting
          ? 'Placing...'
          : `${side === 'buy' ? 'Buy' : 'Sell'} ${baseAsset}`}
      </button>
    </form>
  )
}
