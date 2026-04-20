'use client'

import { useState } from 'react'
import { toast } from 'sonner'
import { useQueryClient } from '@tanstack/react-query'
import { balancesApi } from '@/lib/api'
import { X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { formatDecimal } from '@/lib/symbols'

interface DepositWithdrawModalProps {
  asset: string
  mode: 'deposit' | 'withdraw'
  onClose: () => void
  availableBalance?: string
}

export function DepositWithdrawModal({ asset, mode, onClose, availableBalance }: DepositWithdrawModalProps) {
  const [amount, setAmount] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [validationError, setValidationError] = useState('')
  const qc = useQueryClient()

  const DEPOSIT_LIMITS: Record<string, number> = {
    USDT: 1000,
    BTC: 0.05,
    ETH: 0.5,
    SOL: 5,
  }

  const depositMax = DEPOSIT_LIMITS[asset]

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setValidationError('')
    const n = parseFloat(amount)
    if (isNaN(n) || n <= 0) {
      toast.error('Enter a valid amount')
      return
    }
    if (mode === 'deposit' && depositMax !== undefined && n > depositMax) {
      setValidationError(`Maximum deposit is ${depositMax} ${asset}`)
      return
    }

    setIsLoading(true)
    try {
      if (mode === 'deposit') {
        await balancesApi.deposit(asset, amount)
        toast.success(`Deposited ${amount} ${asset}`)
      } else {
        await balancesApi.withdraw(asset, amount)
        toast.success(`Withdrew ${amount} ${asset}`)
      }
      qc.invalidateQueries({ queryKey: ['balances'] })
      onClose()
    } catch (err) {
      toast.error(err instanceof Error ? err.message : `${mode} failed`)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 animate-fade-in">
      <div className="bg-bg-surface border border-bg-border rounded-xl w-full max-w-sm mx-4 p-6 animate-scale-in">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-base font-semibold text-text-primary capitalize">
            {mode} {asset}
          </h2>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text-secondary transition-colors duration-150"
          >
            <X size={18} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div>
            <label className="text-xs text-text-muted mb-1 block">
              Asset
            </label>
            <div className="bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-secondary">
              {asset}
            </div>
            {mode === 'deposit' && depositMax !== undefined && (
              <p className="text-text-muted text-xs mt-1">Max deposit: {depositMax} {asset}</p>
            )}
          </div>

          {mode === 'withdraw' && availableBalance !== undefined && (
            <>
              <div className="flex items-center justify-between">
                <span className="text-xs text-text-muted">
                  Available: {formatDecimal(availableBalance, asset === 'USDT' ? 2 : 6)} {asset}
                </span>
                <div className="flex gap-1">
                  {[25, 50, 75, 100].map((pct) => (
                    <button
                      key={pct}
                      type="button"
                      onClick={() => {
                        const decimals = asset === 'USDT' ? 2 : 6
                        const val = (parseFloat(availableBalance) * pct) / 100
                        setAmount(val.toFixed(decimals))
                        setValidationError('')
                      }}
                      className="px-1.5 py-0.5 text-xs border border-bg-border rounded text-text-muted hover:text-text-secondary hover:border-accent/50 transition-all duration-150"
                    >
                      {pct}%
                    </button>
                  ))}
                </div>
              </div>
            </>
          )}

          <div>
            <label className="text-xs text-text-muted mb-1 block">
              Amount
            </label>
            <input
              type="number"
              value={amount}
              onChange={(e) => { setAmount(e.target.value); setValidationError('') }}
              placeholder="0.00"
              step="any"
              autoFocus
              className={cn(
                'w-full bg-bg-elevated border rounded-md py-2.5 px-3 text-sm font-mono text-right text-text-primary placeholder:text-text-muted focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150',
                validationError ? 'border-sell focus:border-sell' : 'border-bg-border focus:border-accent'
              )}
            />
            {validationError && (
              <p className="text-sell text-xs mt-1">{validationError}</p>
            )}
            {mode === 'deposit' && asset !== 'USDT' && (
              <p className="text-text-muted text-xs mt-1">
                You can acquire more {asset} by trading on the order book.
              </p>
            )}
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className={cn(
              'w-full py-2.5 text-sm font-medium rounded-md transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed',
              mode === 'deposit'
                ? 'bg-accent text-white hover:bg-accent-hover'
                : 'bg-sell/10 text-sell border border-sell/20 hover:bg-sell/20'
            )}
          >
            {isLoading ? 'Processing...' : `Confirm ${mode.charAt(0).toUpperCase() + mode.slice(1)}`}
          </button>
        </form>
      </div>
    </div>
  )
}
