'use client'

export const dynamic = 'force-dynamic'

import { useSearchParams } from 'next/navigation'
import { useQuery } from '@tanstack/react-query'
import { adminApi } from '@/lib/api'
import { AdminStats, AssetResponse } from '@/lib/types'
import { MetricCard } from '@/components/admin/MetricCard'
import { UsersTable } from '@/components/admin/UsersTable'
import { PairsManager } from '@/components/admin/PairsManager'
import { Users, Wifi, TrendingUp, Clock, BarChart2 } from 'lucide-react'
import { formatCompact } from '@/lib/symbols'
import { useState } from 'react'
import { toast } from 'sonner'
import { useQueryClient } from '@tanstack/react-query'

export default function AdminPage() {
  const searchParams = useSearchParams()
  const tab = searchParams.get('tab')

  return (
    <div className="p-6">
      {!tab && <AdminDashboard />}
      {tab === 'users' && (
        <div className="space-y-4">
          <h1 className="text-xl font-semibold text-text-primary">Users</h1>
          <UsersTable />
        </div>
      )}
      {tab === 'pairs' && (
        <div className="space-y-4">
          <h1 className="text-xl font-semibold text-text-primary">Trading Pairs</h1>
          <PairsManager />
        </div>
      )}
      {tab === 'assets' && <AssetsAdmin />}
    </div>
  )
}

function AdminDashboard() {
  const { data: stats } = useQuery<AdminStats>({
    queryKey: ['admin', 'stats'],
    queryFn: () => adminApi.getStats(),
    refetchInterval: 10_000,
  })

  const uptimeHours = stats ? Math.floor(stats.uptime_seconds / 3600) : 0
  const uptimeMinutes = stats ? Math.floor((stats.uptime_seconds % 3600) / 60) : 0

  return (
    <div className="space-y-6">
      <h1 className="text-xl font-semibold text-text-primary">Admin Dashboard</h1>

      <div className="grid grid-cols-3 gap-4">
        <MetricCard
          label="Total Users"
          value={stats?.total_users ?? '—'}
          icon={<Users size={18} />}
        />
        <MetricCard
          label="Active WS Connections"
          value={stats?.active_ws_connections ?? '—'}
          icon={<Wifi size={18} />}
        />
        <MetricCard
          label="Orders Processed"
          value={stats ? formatCompact(stats.orders_processed) : '—'}
          icon={<TrendingUp size={18} />}
        />
        <MetricCard
          label="Uptime"
          value={stats ? `${uptimeHours}h ${uptimeMinutes}m` : '—'}
          icon={<Clock size={18} />}
        />
        {stats?.volume_24h.slice(0, 2).map((pv) => (
          <MetricCard
            key={pv.symbol}
            label={`${pv.symbol} 24h Volume`}
            value={formatCompact(pv.volume)}
            icon={<BarChart2 size={18} />}
          />
        ))}
      </div>

      {stats?.volume_24h && stats.volume_24h.length > 0 && (
        <div className="bg-bg-surface border border-bg-border rounded-lg overflow-hidden">
          <div className="px-4 py-3 border-b border-bg-border">
            <h2 className="text-sm font-semibold text-text-primary">Pair Volumes (24h)</h2>
          </div>
          <table className="w-full">
            <thead>
              <tr className="border-b border-bg-border">
                <th className="px-4 py-2 text-xs uppercase tracking-wider text-text-muted font-normal text-left">Symbol</th>
                <th className="px-4 py-2 text-xs uppercase tracking-wider text-text-muted font-normal text-right">Volume</th>
              </tr>
            </thead>
            <tbody>
              {stats.volume_24h.map((pv) => (
                <tr key={pv.symbol} className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100">
                  <td className="px-4 py-2 text-sm text-text-secondary">{pv.symbol}</td>
                  <td className="px-4 py-2 text-sm font-mono text-text-secondary text-right">
                    {formatCompact(pv.volume)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}

function AssetsAdmin() {
  const qc = useQueryClient()
  const { data: assets, isLoading } = useQuery<AssetResponse[]>({
    queryKey: ['admin', 'assets'],
    queryFn: () => adminApi.getAssets(),
  })
  const [symbol, setSymbol] = useState('')
  const [name, setName] = useState('')
  const [isAdding, setIsAdding] = useState(false)

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault()
    if (!symbol.trim() || !name.trim()) {
      toast.error('Both symbol and name are required')
      return
    }
    setIsAdding(true)
    try {
      await adminApi.createAsset(symbol.toUpperCase(), name)
      toast.success(`Asset ${symbol.toUpperCase()} created`)
      setSymbol('')
      setName('')
      qc.invalidateQueries({ queryKey: ['admin', 'assets'] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create asset')
    } finally {
      setIsAdding(false)
    }
  }

  return (
    <div className="space-y-6">
      <h1 className="text-xl font-semibold text-text-primary">Assets</h1>

      <div className="bg-bg-surface border border-bg-border rounded-lg overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-bg-border">
              <th className="px-4 py-2 text-xs uppercase tracking-wider text-text-muted font-normal text-left">Symbol</th>
              <th className="px-4 py-2 text-xs uppercase tracking-wider text-text-muted font-normal text-left">Name</th>
            </tr>
          </thead>
          <tbody>
            {assets?.map((a) => (
              <tr key={a.symbol} className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100">
                <td className="px-4 py-2 text-sm font-medium text-text-primary">{a.symbol}</td>
                <td className="px-4 py-2 text-sm text-text-secondary">{a.name}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="bg-bg-surface border border-bg-border rounded-lg p-5">
        <h3 className="text-sm font-semibold text-text-primary mb-4">Add Asset</h3>
        <form onSubmit={handleAdd} className="flex items-end gap-3">
          <div className="flex-1">
            <label className="text-xs text-text-muted mb-1 block">Symbol</label>
            <input
              type="text"
              value={symbol}
              onChange={(e) => setSymbol(e.target.value)}
              placeholder="BTC"
              className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150 uppercase"
            />
          </div>
          <div className="flex-1">
            <label className="text-xs text-text-muted mb-1 block">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Bitcoin"
              className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150"
            />
          </div>
          <button
            type="submit"
            disabled={isAdding}
            className="px-4 py-2.5 text-sm font-medium bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {isAdding ? 'Adding...' : 'Add Asset'}
          </button>
        </form>
      </div>
    </div>
  )
}
