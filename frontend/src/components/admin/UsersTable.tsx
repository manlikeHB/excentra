'use client'

import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { adminApi } from '@/lib/api'
import { PaginatedUserSummary, UserSummary } from '@/lib/types'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { Skeleton } from '@/components/shared/Skeleton'
import { formatPrice } from '@/lib/symbols'

export function UsersTable() {
  const qc = useQueryClient()
  const [selectedUser, setSelectedUser] = useState<UserSummary | null>(null)
  const [confirmAction, setConfirmAction] = useState<{
    type: 'role' | 'suspend'
    newRole?: 'user' | 'admin'
  } | null>(null)

  const { data, isLoading } = useQuery<PaginatedUserSummary>({
    queryKey: ['admin', 'users'],
    queryFn: () => adminApi.getUsers(),
  })

  async function handleRoleChange(role: 'user' | 'admin') {
    if (!selectedUser) return
    try {
      await adminApi.updateRole(selectedUser.id, role)
      toast.success(`${selectedUser.email} is now ${role}`)
      qc.invalidateQueries({ queryKey: ['admin', 'users'] })
      setSelectedUser((prev) => (prev ? { ...prev, role } : null))
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to update role')
    }
    setConfirmAction(null)
  }

  async function handleSuspend() {
    if (!selectedUser) return
    try {
      await adminApi.suspend(selectedUser.id)
      toast.success(`${selectedUser.email} status updated`)
      qc.invalidateQueries({ queryKey: ['admin', 'users'] })
      setSelectedUser((prev) =>
        prev ? { ...prev, is_suspended: !prev.is_suspended } : null,
      )
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to update status')
    }
    setConfirmAction(null)
  }

  return (
    <>
      {/* Table */}
      <div className="bg-bg-surface border border-bg-border rounded-lg overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-bg-border">
              <Th>Email</Th>
              <Th>Username</Th>
              <Th>Role</Th>
              <Th>Status</Th>
              <Th>Joined</Th>
              <Th></Th>
            </tr>
          </thead>
          <tbody>
            {isLoading
              ? Array.from({ length: 5 }).map((_, i) => (
                  <tr key={i} className="border-b border-bg-border/40">
                    {Array.from({ length: 6 }).map((_, j) => (
                      <td key={j} className="px-3 py-2">
                        <Skeleton className="h-3 w-20" />
                      </td>
                    ))}
                  </tr>
                ))
              : data?.data?.map((user) => (
                  <tr
                    key={user.id}
                    onClick={() => setSelectedUser(user)}
                    className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-all duration-150 cursor-pointer"
                  >
                    <td className="px-3 py-2 text-xs text-text-secondary">{user.email}</td>
                    <td className="px-3 py-2 text-xs text-text-muted">{user.username ?? '—'}</td>
                    <td className="px-3 py-2">
                      <RoleBadge role={user.role} />
                    </td>
                    <td className="px-3 py-2">
                      <StatusBadge is_suspended={user.is_suspended} />
                    </td>
                    <td className="px-3 py-2 text-xs text-text-muted font-mono">
                      {new Date(user.created_at).toLocaleDateString()}
                    </td>
                    <td className="px-3 py-2 text-right">
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          setSelectedUser(user)
                        }}
                        className="text-xs px-2 py-1 border border-bg-border rounded-md text-text-secondary hover:bg-bg-elevated hover:text-text-primary transition-all duration-150"
                      >
                        View
                      </button>
                    </td>
                  </tr>
                ))}
          </tbody>
        </table>
      </div>

      {/* Backdrop */}
      {selectedUser && (
        <div
          className="fixed inset-0 bg-black/50 z-30 animate-fade-in"
          onClick={() => setSelectedUser(null)}
        />
      )}

      {/* Drawer panel — always in DOM so transition-transform plays on open */}
      <div
        className={cn(
          'fixed top-0 right-0 h-full w-[400px] bg-bg-surface border-l border-bg-border z-40 flex flex-col transition-transform duration-200 overflow-y-auto',
          selectedUser ? 'translate-x-0' : 'translate-x-full',
        )}
      >
        {selectedUser && (
          <>
            {/* Header */}
            <div className="flex items-start justify-between px-5 py-4 border-b border-bg-border">
              <div className="flex flex-col gap-1.5">
                <span className="text-base font-semibold text-text-primary">
                  {selectedUser.email}
                </span>
                <span className="text-sm text-text-muted">
                  {selectedUser.username ?? '—'}
                </span>
                <div className="flex items-center gap-2 mt-0.5">
                  <RoleBadge role={selectedUser.role} />
                  <StatusBadge is_suspended={selectedUser.is_suspended} />
                </div>
                <span className="text-xs text-text-muted">
                  Member since {new Date(selectedUser.created_at).toLocaleDateString()}
                </span>
              </div>
              <button
                onClick={() => setSelectedUser(null)}
                aria-label="Close"
                className="text-text-muted hover:text-text-primary transition-all duration-150 mt-0.5 flex-shrink-0"
              >
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                  <path
                    d="M12 4L4 12M4 4l8 8"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                  />
                </svg>
              </button>
            </div>

            {/* Balances */}
            <div className="px-5 py-4 border-b border-bg-border">
              <p className="text-xs uppercase tracking-wider text-text-muted mb-3">Balances</p>
              {selectedUser.balances.length === 0 ? (
                <p className="text-sm text-text-muted">No balances</p>
              ) : (
                <table className="w-full">
                  <thead>
                    <tr>
                      <th className="text-xs text-text-muted font-normal text-left pb-1.5">
                        Asset
                      </th>
                      <th className="text-xs text-text-muted font-normal text-right pb-1.5">
                        Available
                      </th>
                      <th className="text-xs text-text-muted font-normal text-right pb-1.5">
                        Held
                      </th>
                      <th className="text-xs text-text-muted font-normal text-right pb-1.5">
                        Total
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {selectedUser.balances.map((b) => {
                      const held = parseFloat(b.held)
                      const total = parseFloat(b.available) + held
                      return (
                        <tr key={b.asset}>
                          <td className="text-xs text-text-primary py-1">{b.asset}</td>
                          <td className="text-xs font-mono text-text-secondary py-1 text-right">
                            {formatPrice(parseFloat(b.available), b.asset)}
                          </td>
                          <td className="text-xs font-mono text-text-secondary py-1 text-right">
                            {formatPrice(held, b.asset)}
                          </td>
                          <td className="text-xs font-mono text-text-secondary py-1 text-right">
                            {formatPrice(total, b.asset)}
                          </td>
                        </tr>
                      )
                    })}
                  </tbody>
                </table>
              )}
            </div>

            {/* Actions */}
            <div className="px-5 py-4">
              <p className="text-xs uppercase tracking-wider text-text-muted mb-3">Actions</p>
              <div className="flex gap-2">
                <button
                  onClick={() =>
                    setConfirmAction({
                      type: 'role',
                      newRole: selectedUser.role === 'admin' ? 'user' : 'admin',
                    })
                  }
                  className="text-xs px-3 py-1.5 border border-bg-border rounded-md text-text-secondary hover:bg-bg-elevated hover:text-text-primary transition-all duration-150"
                >
                  {selectedUser.role === 'admin' ? 'Demote' : 'Promote'}
                </button>
                <button
                  onClick={() => setConfirmAction({ type: 'suspend' })}
                  className={cn(
                    'text-xs px-3 py-1.5 border rounded-md transition-all duration-150',
                    selectedUser.is_suspended
                      ? 'border-buy/30 text-buy hover:bg-buy/10'
                      : 'border-sell/30 text-sell hover:bg-sell/10',
                  )}
                >
                  {selectedUser.is_suspended ? 'Unsuspend' : 'Suspend'}
                </button>
              </div>
            </div>
          </>
        )}
      </div>

      {/* Confirm dialog */}
      {confirmAction && selectedUser && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 animate-fade-in">
          <div className="bg-bg-surface border border-bg-border rounded-xl p-6 w-full max-w-sm mx-4 animate-scale-in">
            <h3 className="text-base font-semibold text-text-primary mb-2">Confirm Action</h3>
            <p className="text-sm text-text-secondary mb-6">
              {confirmAction.type === 'role'
                ? `Change ${selectedUser.email} role to ${confirmAction.newRole}?`
                : `${selectedUser.is_suspended ? 'Unsuspend' : 'Suspend'} ${selectedUser.email}?`}
            </p>
            <div className="flex gap-2">
              <button
                onClick={() => setConfirmAction(null)}
                className="flex-1 py-2 text-sm border border-bg-border rounded-md text-text-secondary hover:bg-bg-elevated transition-all duration-150"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  if (confirmAction.type === 'role' && confirmAction.newRole) {
                    handleRoleChange(confirmAction.newRole)
                  } else {
                    handleSuspend()
                  }
                }}
                className="flex-1 py-2 text-sm bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98]"
              >
                Confirm
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}

function Th({ children }: { children?: React.ReactNode }) {
  return (
    <th className="px-3 py-2 text-xs uppercase tracking-wider text-text-muted font-normal text-left">
      {children}
    </th>
  )
}

function RoleBadge({ role }: { role: string }) {
  return (
    <span
      className={cn(
        'text-xs px-2 py-0.5 rounded-full',
        role === 'admin' ? 'bg-accent/15 text-accent' : 'bg-bg-elevated text-text-muted',
      )}
    >
      {role}
    </span>
  )
}

function StatusBadge({ is_suspended }: { is_suspended: boolean }) {
  return (
    <span
      className={cn(
        'text-xs px-2 py-0.5 rounded-full',
        is_suspended ? 'bg-sell/15 text-sell' : 'bg-buy/15 text-buy',
      )}
    >
      {is_suspended ? 'Suspended' : 'Active'}
    </span>
  )
}
