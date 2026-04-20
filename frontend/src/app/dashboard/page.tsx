'use client'

export const dynamic = 'force-dynamic'

import { useSearchParams } from 'next/navigation'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { balancesApi, usersApi, ordersApi, tradesApi } from '@/lib/api'
import { BalanceResponse, UserResponse, OrderResponse, TradeResponse, PaginatedResponse } from '@/lib/types'
import { formatPrice, formatDecimal } from '@/lib/symbols'
import { Skeleton } from '@/components/shared/Skeleton'
import { X } from 'lucide-react'
import { PortfolioCard } from '@/components/dashboard/PortfolioCard'
import { AssetCard } from '@/components/dashboard/AssetCard'
import { DepositWithdrawModal } from '@/components/dashboard/DepositWithdrawModal'
import { useState } from "react";
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { useAuth } from '@/lib/context'
export default function DashboardPage() {
  const searchParams = useSearchParams()
  const tab = searchParams.get('tab') ?? 'dashboard'
  const { user, isLoading: authLoading } = useAuth()

  return (
    <div className="p-6">
      {tab === 'dashboard' && <DashboardView />}
      {tab === 'assets' && <AssetsView />}
      {tab === 'orders' && (
        authLoading
          ? <div className="flex items-center justify-center h-40 text-text-muted text-sm">Loading...</div>
          : user
            ? <OrdersView user={user} />
            : <div className="flex items-center justify-center h-40 text-text-muted text-sm">Sign in to view orders</div>
      )}
      {tab === 'settings' && <SettingsView />}
    </div>
  )
}

const SUPPORTED_ASSETS = ['BTC', 'ETH', 'SOL', 'USDT']

function mergeBalances(balances: BalanceResponse[] | undefined): BalanceResponse[] {
  return SUPPORTED_ASSETS.map((asset) => {
    const found = balances?.find((b) => b.asset === asset)
    return found ?? { asset, available: '0', held: '0', updated_at: '' }
  })
}

function DashboardView() {
  const { data: balances } = useQuery<BalanceResponse[]>({
    queryKey: ['balances'],
    queryFn: () => balancesApi.list(),
  })

  const merged = mergeBalances(balances)

  return (
    <div className="max-w-4xl space-y-6">
      <h1 className="text-xl font-semibold text-text-primary">Dashboard</h1>
      <PortfolioCard />
      <div>
        <h2 className="text-sm font-semibold text-text-primary mb-3">Assets</h2>
        <div className="grid grid-cols-2 gap-4">
          {merged.map((b) => (
            <AssetCard key={b.asset} balance={b} />
          ))}
        </div>
      </div>
    </div>
  )
}

function AssetsView() {
  const { data: balances } = useQuery<BalanceResponse[]>({
    queryKey: ['balances'],
    queryFn: () => balancesApi.list(),
  })

  const merged = mergeBalances(balances)

  return (
    <div className="max-w-4xl space-y-6">
      <h1 className="text-xl font-semibold text-text-primary">Assets</h1>
      <div className="bg-bg-surface border border-bg-border rounded-lg overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-bg-border">
              {['Coin', 'Available', 'In Orders', 'Total', 'Actions'].map((h) => (
                <th key={h} className="px-4 py-3 text-xs uppercase tracking-wider text-text-muted font-normal text-left">
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {merged.map((b) => {
              const total = parseFloat(b.available) + parseFloat(b.held)
              const decimals = b.asset === 'USDT' ? 2 : 6
              return (
                <tr key={b.asset} className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100">
                  <td className="px-4 py-3 text-sm font-medium text-text-primary">{b.asset}</td>
                  <td className="px-4 py-3 text-sm font-mono text-text-secondary">{formatDecimal(b.available, decimals)}</td>
                  <td className="px-4 py-3 text-sm font-mono text-text-secondary">{formatDecimal(b.held, decimals)}</td>
                  <td className="px-4 py-3 text-sm font-mono text-text-secondary">{formatDecimal(total, decimals)}</td>
                  <td className="px-4 py-3">
                    <AssetActionsInline asset={b.asset} available={b.available} />
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </div>
  )
}

function AssetActionsInline({ asset, available }: { asset: string; available: string }) {
  const [modal, setModal] = useState<'deposit' | 'withdraw' | null>(null)

  const canWithdraw = parseFloat(available) > 0

  return (
    <>
      <div className="flex gap-2">
        <button
          onClick={() => setModal('deposit')}
          className="text-xs px-2.5 py-1 bg-accent/10 text-accent border border-accent/20 rounded-md hover:bg-accent/20 transition-all duration-150"
        >
          Deposit
        </button>
        <button
          onClick={() => canWithdraw && setModal('withdraw')}
          disabled={!canWithdraw}
          className={cn(
            'text-xs px-2.5 py-1 bg-sell/10 text-sell border border-sell/20 rounded-md transition-all duration-150',
            canWithdraw ? 'hover:bg-sell/20' : 'opacity-40 cursor-not-allowed'
          )}
        >
          Withdraw
        </button>
      </div>
      {modal && (
        <DepositWithdrawModal asset={asset} mode={modal} onClose={() => setModal(null)} availableBalance={available} />
      )}
    </>
  )
}

type DashOrderTab = 'open' | 'history' | 'trades'

function OrdersView({ user: _user }: { user: UserResponse }) {
  const [tab, setTab] = useState<DashOrderTab>("open");
  const [openPage, setOpenPage] = useState(1);
  const [historyPage, setHistoryPage] = useState(1);
  const [tradesPage, setTradesPage] = useState(1);
  const qc = useQueryClient();

  const { data: openData, isLoading: openLoading } = useQuery<
    PaginatedResponse<OrderResponse>
  >({
    queryKey: ["dashboard", "orders", "open,partially_filled", openPage],
    queryFn: () =>
      ordersApi.list({
        status: "open,partially_filled",
        page: openPage,
        limit: 20,
        order: "desc",
      }),
  });

  const { data: historyData, isLoading: historyLoading } = useQuery<
    PaginatedResponse<OrderResponse>
  >({
    queryKey: ["dashboard", "orders", "history", historyPage],
    queryFn: () => ordersApi.list({ page: historyPage, limit: 20, order: 'desc' }),
  });

  const { data: tradesData, isLoading: tradesLoading } = useQuery<
    PaginatedResponse<TradeResponse>
  >({
    queryKey: ["dashboard", "trades", tradesPage],
    queryFn: () => tradesApi.mine({ page: tradesPage, limit: 20, order: "desc" }),
  });

  async function handleCancel(id: string) {
    try {
      await ordersApi.cancel(id);
      toast.success("Order cancelled");
      qc.invalidateQueries({ queryKey: ["dashboard", "orders"] });
      qc.invalidateQueries({ queryKey: ["balances"] });
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to cancel order");
    }
  }

  const tabs: { key: DashOrderTab; label: string }[] = [
    { key: "open", label: "Open Orders" },
    { key: "history", label: "Order History" },
    { key: "trades", label: "Trade History" },
  ];

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-semibold text-text-primary">Orders</h1>
      <div
        className="bg-bg-surface border border-bg-border rounded-lg flex flex-col"
        style={{ height: "calc(100vh - 120px)" }}
      >
        {/* Tab headers */}
        <div className="flex border-b border-bg-border flex-shrink-0">
          {tabs.map((t) => (
            <button
              key={t.key}
              onClick={() => setTab(t.key)}
              className={cn(
                "px-4 py-2.5 text-xs font-medium transition-all duration-150",
                tab === t.key
                  ? "text-text-primary border-b-2 border-accent -mb-px"
                  : "text-text-muted hover:text-text-secondary",
              )}
            >
              {t.label}
            </button>
          ))}
        </div>

        {/* Tab content */}
        <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
          {/* Open Orders */}
          {tab === "open" && (
            <>
              <div className="flex-1 overflow-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-bg-border">
                      {[
                        "Pair",
                        "Type",
                        "Side",
                        "Price",
                        "Amount",
                        "Filled",
                        "Status",
                        "Date",
                        "",
                      ].map((h) => (
                        <th
                          key={h}
                          className={cn(
                            "px-3 py-1.5 text-xs uppercase tracking-wider text-text-muted font-normal",
                            ["Price", "Amount", "Filled"].includes(h)
                              ? "text-right"
                              : "text-left",
                            h === "" && "w-px whitespace-nowrap",
                          )}
                        >
                          {h}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {openLoading
                      ? Array.from({ length: 5 }).map((_, i) => (
                          <tr key={i} className="border-b border-bg-border/40">
                            {Array.from({ length: 9 }).map((_, j) => (
                              <td key={j} className="px-3 py-1.5">
                                <Skeleton className="h-3 w-16" />
                              </td>
                            ))}
                          </tr>
                        ))
                      : openData?.data?.map((order) => {
                          const quote = order.symbol.split("/")[1] ?? "USDT";
                          const filled = (
                            parseFloat(order.quantity) -
                            parseFloat(order.remaining_quantity)
                          ).toFixed(6);
                          return (
                            <tr
                              key={order.id}
                              className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100"
                            >
                              <td className="px-3 py-1.5 text-xs text-text-secondary">
                                {order.symbol}
                              </td>
                              <td className="px-3 py-1.5 text-xs text-text-secondary capitalize">
                                {order.order_type}
                              </td>
                              <td className="px-3 py-1.5">
                                <span
                                  className={cn(
                                    "text-xs px-1.5 py-0.5 rounded font-medium",
                                    order.side === "buy"
                                      ? "bg-buy/10 text-buy"
                                      : "bg-sell/10 text-sell",
                                  )}
                                >
                                  {order.side}
                                </span>
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                                {order.price ? formatPrice(order.price, quote) : "Market"}
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                                {formatDecimal(order.quantity, 6)}
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                                {filled}
                              </td>
                              <td className="px-3 py-1.5">
                                <OrderStatusBadge status={order.status} />
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-muted whitespace-nowrap">
                                {new Date(order.created_at).toLocaleString("en-US", {
                                  month: "short",
                                  day: "numeric",
                                  hour: "2-digit",
                                  minute: "2-digit",
                                })}
                              </td>
                              <td className="px-3 py-1.5 w-px whitespace-nowrap">
                                <button
                                  onClick={() => handleCancel(order.id)}
                                  className="text-text-muted hover:text-sell hover:bg-sell/10 rounded p-0.5 transition-all duration-150"
                                  title="Cancel order"
                                >
                                  <X size={12} />
                                </button>
                              </td>
                            </tr>
                          );
                        })}
                  </tbody>
                </table>
                {!openLoading && !openData?.data?.length && (
                  <div className="flex items-center justify-center py-10 text-text-muted text-sm">
                    No open orders
                  </div>
                )}
              </div>
              <DashPagination
                page={openPage}
                total={openData?.total}
                limit={openData?.limit}
                onPrev={() => setOpenPage((p) => Math.max(1, p - 1))}
                onNext={() => setOpenPage((p) => p + 1)}
              />
            </>
          )}

          {/* Order History */}
          {tab === "history" && (
            <>
              <div className="flex-1 overflow-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-bg-border">
                      {["Pair", "Type", "Side", "Price", "Amount", "Status", "Date"].map(
                        (h) => (
                          <th
                            key={h}
                            className={cn(
                              "px-3 py-1.5 text-xs uppercase tracking-wider text-text-muted font-normal",
                              ["Price", "Amount"].includes(h)
                                ? "text-right"
                                : "text-left",
                            )}
                          >
                            {h}
                          </th>
                        ),
                      )}
                    </tr>
                  </thead>
                  <tbody>
                    {historyLoading
                      ? Array.from({ length: 5 }).map((_, i) => (
                          <tr key={i} className="border-b border-bg-border/40">
                            {Array.from({ length: 7 }).map((_, j) => (
                              <td key={j} className="px-3 py-1.5">
                                <Skeleton className="h-3 w-16" />
                              </td>
                            ))}
                          </tr>
                        ))
                      : historyData?.data?.map((order) => {
                          const quote = order.symbol.split("/")[1] ?? "USDT";
                          return (
                            <tr
                              key={order.id}
                              className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100"
                            >
                              <td className="px-3 py-1.5 text-xs text-text-secondary">
                                {order.symbol}
                              </td>
                              <td className="px-3 py-1.5 text-xs text-text-secondary capitalize">
                                {order.order_type}
                              </td>
                              <td className="px-3 py-1.5">
                                <span
                                  className={cn(
                                    "text-xs px-1.5 py-0.5 rounded font-medium",
                                    order.side === "buy"
                                      ? "bg-buy/10 text-buy"
                                      : "bg-sell/10 text-sell",
                                  )}
                                >
                                  {order.side}
                                </span>
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                                {order.price ? formatPrice(order.price, quote) : "Market"}
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                                {formatDecimal(order.quantity, 6)}
                              </td>
                              <td className="px-3 py-1.5">
                                <OrderStatusBadge status={order.status} />
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-muted whitespace-nowrap">
                                {new Date(order.created_at).toLocaleString("en-US", {
                                  month: "short",
                                  day: "numeric",
                                  hour: "2-digit",
                                  minute: "2-digit",
                                })}
                              </td>
                            </tr>
                          );
                        })}
                  </tbody>
                </table>
                {!historyLoading && !historyData?.data?.length && (
                  <div className="flex items-center justify-center py-10 text-text-muted text-sm">
                    No order history
                  </div>
                )}
              </div>
              <DashPagination
                page={historyPage}
                total={historyData?.total}
                limit={historyData?.limit}
                onPrev={() => setHistoryPage((p) => Math.max(1, p - 1))}
                onNext={() => setHistoryPage((p) => p + 1)}
              />
            </>
          )}

          {/* Trade History */}
          {tab === "trades" && (
            <>
              <div className="flex-1 overflow-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-bg-border">
                      {["Pair", "Side", "Price", "Amount", "Total", "Date"].map((h) => (
                        <th
                          key={h}
                          className={cn(
                            "px-3 py-1.5 text-xs uppercase tracking-wider text-text-muted font-normal",
                            ["Price", "Amount", "Total"].includes(h)
                              ? "text-right"
                              : "text-left",
                          )}
                        >
                          {h}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {tradesLoading
                      ? Array.from({ length: 5 }).map((_, i) => (
                          <tr key={i} className="border-b border-bg-border/40">
                            {Array.from({ length: 6 }).map((_, j) => (
                              <td key={j} className="px-3 py-1.5">
                                <Skeleton className="h-3 w-16" />
                              </td>
                            ))}
                          </tr>
                        ))
                      : tradesData?.data?.map((trade) => {
                          const quote = trade.symbol.split("/")[1] ?? "USDT";
                          const total =
                            parseFloat(trade.price) * parseFloat(trade.quantity);
                          return (
                            <tr
                              key={trade.id}
                              className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100"
                            >
                              <td className="px-3 py-1.5 text-xs text-text-secondary">
                                {trade.symbol}
                              </td>
                              <td className="px-3 py-1.5">
                                <span
                                  className={cn(
                                    "text-xs px-1.5 py-0.5 rounded font-medium",
                                    trade.side === "buy"
                                      ? "bg-buy/10 text-buy"
                                      : "bg-sell/10 text-sell",
                                  )}
                                >
                                  {trade.side}
                                </span>
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                                {formatPrice(trade.price, quote)}
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                                {formatDecimal(trade.quantity, 6)}
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-secondary text-right">
                                {formatDecimal(total, 2)}
                              </td>
                              <td className="px-3 py-1.5 text-xs font-mono text-text-muted whitespace-nowrap">
                                {new Date(trade.created_at).toLocaleString("en-US", {
                                  month: "short",
                                  day: "numeric",
                                  hour: "2-digit",
                                  minute: "2-digit",
                                })}
                              </td>
                            </tr>
                          );
                        })}
                  </tbody>
                </table>
                {!tradesLoading && !tradesData?.data?.length && (
                  <div className="flex items-center justify-center py-10 text-text-muted text-sm">
                    No trade history
                  </div>
                )}
              </div>
              <DashPagination
                page={tradesPage}
                total={tradesData?.total}
                limit={tradesData?.limit}
                onPrev={() => setTradesPage((p) => Math.max(1, p - 1))}
                onNext={() => setTradesPage((p) => p + 1)}
              />
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function OrderStatusBadge({ status }: { status: OrderResponse['status'] }) {
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
  return <span className={cn('text-xs', styles[status])}>{labels[status]}</span>
}

function DashPagination({
  page,
  total,
  limit,
  onPrev,
  onNext,
}: {
  page: number
  total?: number
  limit?: number
  onPrev: () => void
  onNext: () => void
}) {
  if (total == null || limit == null || total <= limit) return null
  return (
    <div className="flex items-center justify-between px-3 py-2 border-t border-bg-border flex-shrink-0">
      <span className="text-xs text-text-muted">{total} total</span>
      <div className="flex gap-1">
        <button
          onClick={onPrev}
          disabled={page === 1}
          className="px-2 py-1 text-xs border border-bg-border rounded text-text-secondary hover:bg-bg-elevated disabled:opacity-40 disabled:cursor-not-allowed transition-all duration-150"
        >
          Prev
        </button>
        <span className="px-2 py-1 text-xs text-text-muted">{page}</span>
        <button
          onClick={onNext}
          disabled={page * limit >= total}
          className="px-2 py-1 text-xs border border-bg-border rounded text-text-secondary hover:bg-bg-elevated disabled:opacity-40 disabled:cursor-not-allowed transition-all duration-150"
        >
          Next
        </button>
      </div>
    </div>
  )
}

function SettingsView() {
  const qc = useQueryClient()
  const { updateUser } = useAuth()
  const { data: user } = useQuery<UserResponse>({
    queryKey: ['user', 'me'],
    queryFn: () => usersApi.me(),
  })

  const [username, setUsername] = useState('')
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [isSavingProfile, setIsSavingProfile] = useState(false)
  const [isSavingSecurity, setIsSavingSecurity] = useState(false)

  async function handleSaveProfile(e: React.FormEvent) {
    e.preventDefault()
    if (!username.trim()) return
    setIsSavingProfile(true)
    try {
      const updated = await usersApi.update({ username })
      updateUser(updated)
      toast.success('Username updated')
      qc.invalidateQueries({ queryKey: ['user', 'me'] })
      setUsername('')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Update failed')
    } finally {
      setIsSavingProfile(false)
    }
  }

  async function handleSaveSecurity(e: React.FormEvent) {
    e.preventDefault()
    if (!currentPassword || !newPassword) {
      toast.error('Both passwords are required')
      return
    }
    if (newPassword.length < 8) {
      toast.error('New password must be at least 8 characters')
      return
    }
    setIsSavingSecurity(true)
    try {
      await usersApi.update({ current_password: currentPassword, new_password: newPassword })
      toast.success('Password updated')
      setCurrentPassword('')
      setNewPassword('')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Update failed')
    } finally {
      setIsSavingSecurity(false)
    }
  }

  return (
    <div className="max-w-lg space-y-6">
      <h1 className="text-xl font-semibold text-text-primary">Settings</h1>

      {/* Profile */}
      <div className="bg-bg-surface border border-bg-border rounded-lg p-6">
        <h2 className="text-sm font-semibold text-text-primary mb-4">Profile</h2>
        <form onSubmit={handleSaveProfile} className="space-y-4">
          <div>
            <label className="text-xs text-text-muted block mb-1">Email</label>
            <div className="bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-muted">
              {user?.email ?? '—'}
            </div>
          </div>
          <div>
            <label className="text-xs text-text-muted block mb-1">Username</label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder={user?.username ?? 'Set a username'}
              className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150"
            />
            <p className="text-xs text-text-muted mt-1">Alphanumeric and underscore only, min 3 chars</p>
          </div>
          <button
            type="submit"
            disabled={isSavingProfile || !username.trim()}
            className="px-4 py-2 text-sm font-medium bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {isSavingProfile ? 'Saving...' : 'Save Profile'}
          </button>
        </form>
      </div>

      {/* Security */}
      <div className="bg-bg-surface border border-bg-border rounded-lg p-6">
        <h2 className="text-sm font-semibold text-text-primary mb-4">Security</h2>
        <form onSubmit={handleSaveSecurity} className="space-y-4">
          <div>
            <label className="text-xs text-text-muted block mb-1">Current Password</label>
            <input
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              placeholder="••••••••"
              className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150"
            />
          </div>
          <div>
            <label className="text-xs text-text-muted block mb-1">New Password</label>
            <input
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              placeholder="••••••••"
              className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150"
            />
            <p className="text-xs text-text-muted mt-1">Minimum 8 characters</p>
          </div>
          <button
            type="submit"
            disabled={isSavingSecurity}
            className="px-4 py-2 text-sm font-medium bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {isSavingSecurity ? 'Saving...' : 'Change Password'}
          </button>
        </form>
      </div>
    </div>
  )
}
