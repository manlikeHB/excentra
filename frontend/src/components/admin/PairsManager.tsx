'use client'

import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { adminApi } from '@/lib/api'
import { PairResponse } from '@/lib/types'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { Skeleton } from '@/components/shared/Skeleton'

export function PairsManager() {
  const qc = useQueryClient()
  const [baseAsset, setBaseAsset] = useState('')
  const [quoteAsset, setQuoteAsset] = useState('')
  const [isAdding, setIsAdding] = useState(false)

  const { data: pairs, isLoading } = useQuery<PairResponse[]>({
    queryKey: ['admin', 'pairs'],
    queryFn: () => adminApi.getAllPairs(),
  })

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault()
    if (!baseAsset.trim() || !quoteAsset.trim()) {
      toast.error('Both assets are required')
      return
    }
    setIsAdding(true)
    try {
      await adminApi.createPair(baseAsset.toUpperCase(), quoteAsset.toUpperCase())
      toast.success(`${baseAsset.toUpperCase()}/${quoteAsset.toUpperCase()} pair created`)
      setBaseAsset('')
      setQuoteAsset('')
      qc.invalidateQueries({ queryKey: ['admin', 'pairs'] })
      qc.invalidateQueries({ queryKey: ['pairs'] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create pair')
    } finally {
      setIsAdding(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* Table */}
      <div className="bg-bg-surface border border-bg-border rounded-lg overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-bg-border">
              <Th>Symbol</Th>
              <Th>Base</Th>
              <Th>Quote</Th>
              <Th>Status</Th>
            </tr>
          </thead>
          <tbody>
            {isLoading
              ? Array.from({ length: 4 }).map((_, i) => (
                  <tr key={i} className="border-b border-bg-border/40">
                    {Array.from({ length: 4 }).map((_, j) => (
                      <td key={j} className="px-3 py-2">
                        <Skeleton className="h-3 w-20" />
                      </td>
                    ))}
                  </tr>
                ))
              : pairs?.map((pair) => (
                  <tr key={pair.id} className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100">
                    <td className="px-3 py-2 text-xs font-medium text-text-primary">{pair.symbol}</td>
                    <td className="px-3 py-2 text-xs text-text-secondary">{pair.base_asset}</td>
                    <td className="px-3 py-2 text-xs text-text-secondary">{pair.quote_asset}</td>
                    <td className="px-3 py-2">
                      <span
                        className={cn(
                          'text-xs px-2 py-0.5 rounded-full',
                          pair.is_active
                            ? 'bg-buy/15 text-buy'
                            : 'bg-bg-elevated text-text-muted'
                        )}
                      >
                        {pair.is_active ? 'Active' : 'Inactive'}
                      </span>
                    </td>
                  </tr>
                ))}
          </tbody>
        </table>
      </div>

      {/* Add pair form */}
      <div className="bg-bg-surface border border-bg-border rounded-lg p-5">
        <h3 className="text-sm font-semibold text-text-primary mb-4">Add Trading Pair</h3>
        <form onSubmit={handleAdd} className="flex items-end gap-3">
          <div className="flex-1">
            <label className="text-xs text-text-muted mb-1 block">Base Asset</label>
            <input
              type="text"
              value={baseAsset}
              onChange={(e) => setBaseAsset(e.target.value)}
              placeholder="BTC"
              className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150 uppercase"
            />
          </div>
          <div className="flex-1">
            <label className="text-xs text-text-muted mb-1 block">Quote Asset</label>
            <input
              type="text"
              value={quoteAsset}
              onChange={(e) => setQuoteAsset(e.target.value)}
              placeholder="USDT"
              className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150 uppercase"
            />
          </div>
          <button
            type="submit"
            disabled={isAdding}
            className="px-4 py-2.5 text-sm font-medium bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {isAdding ? 'Adding...' : 'Add Pair'}
          </button>
        </form>
      </div>
    </div>
  )
}

function Th({ children }: { children: React.ReactNode }) {
  return (
    <th className="px-3 py-2 text-xs uppercase tracking-wider text-text-muted font-normal text-left">
      {children}
    </th>
  )
}
