'use client'

import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { adminApi } from '@/lib/api'
import { UserResponse } from '@/lib/types'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { Skeleton } from '@/components/shared/Skeleton'

export function UsersTable() {
  const qc = useQueryClient()
  const [confirmAction, setConfirmAction] = useState<{
    type: 'role' | 'suspend'
    user: UserResponse
    newRole?: 'user' | 'admin'
  } | null>(null)

  const { data: users, isLoading } = useQuery<UserResponse[]>({
    queryKey: ['admin', 'users'],
    queryFn: () => adminApi.getUsers(),
  })

  async function handleRoleChange(user: UserResponse, role: 'user' | 'admin') {
    try {
      await adminApi.updateRole(user.id, role)
      toast.success(`${user.email} is now ${role}`)
      qc.invalidateQueries({ queryKey: ['admin', 'users'] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to update role')
    }
    setConfirmAction(null)
  }

  async function handleSuspend(user: UserResponse) {
    try {
      await adminApi.suspend(user.id)
      toast.success(`${user.email} status updated`)
      qc.invalidateQueries({ queryKey: ['admin', 'users'] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to update status')
    }
    setConfirmAction(null)
  }

  return (
    <>
      <div className="bg-bg-surface border border-bg-border rounded-lg overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-bg-border">
              <Th>Email</Th>
              <Th>Username</Th>
              <Th>Role</Th>
              <Th>Status</Th>
              <Th>Joined</Th>
              <Th>Actions</Th>
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
              : users?.map((user) => (
                  <tr key={user.id} className="border-b border-bg-border/40 hover:bg-bg-elevated/30 transition-colors duration-100">
                    <td className="px-3 py-2 text-xs text-text-secondary">{user.email}</td>
                    <td className="px-3 py-2 text-xs text-text-muted">{user.username ?? '—'}</td>
                    <td className="px-3 py-2">
                      <span
                        className={cn(
                          'text-xs px-2 py-0.5 rounded-full',
                          user.role === 'admin'
                            ? 'bg-accent/15 text-accent'
                            : 'bg-bg-elevated text-text-muted'
                        )}
                      >
                        {user.role}
                      </span>
                    </td>
                    <td className="px-3 py-2">
                      <span
                        className={cn(
                          'text-xs px-2 py-0.5 rounded-full',
                          user.is_suspended
                            ? 'bg-sell/15 text-sell'
                            : 'bg-buy/15 text-buy'
                        )}
                      >
                        {user.is_suspended ? 'Suspended' : 'Active'}
                      </span>
                    </td>
                    <td className="px-3 py-2 text-xs text-text-muted font-mono">
                      {new Date(user.created_at).toLocaleDateString()}
                    </td>
                    <td className="px-3 py-2">
                      <div className="flex items-center gap-1.5">
                        <button
                          onClick={() =>
                            setConfirmAction({
                              type: 'role',
                              user,
                              newRole: user.role === 'admin' ? 'user' : 'admin',
                            })
                          }
                          className="text-xs px-2 py-1 border border-bg-border rounded text-text-secondary hover:bg-bg-elevated hover:text-text-primary transition-all duration-150"
                        >
                          {user.role === 'admin' ? 'Demote' : 'Promote'}
                        </button>
                        <button
                          onClick={() => setConfirmAction({ type: 'suspend', user })}
                          className={cn(
                            'text-xs px-2 py-1 border rounded transition-all duration-150',
                            user.is_suspended
                              ? 'border-buy/30 text-buy hover:bg-buy/10'
                              : 'border-sell/30 text-sell hover:bg-sell/10'
                          )}
                        >
                          {user.is_suspended ? 'Unsuspend' : 'Suspend'}
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
          </tbody>
        </table>
      </div>

      {/* Confirm dialog */}
      {confirmAction && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 animate-fade-in">
          <div className="bg-bg-surface border border-bg-border rounded-xl p-6 w-full max-w-sm mx-4 animate-scale-in">
            <h3 className="text-base font-semibold text-text-primary mb-2">Confirm Action</h3>
            <p className="text-sm text-text-secondary mb-6">
              {confirmAction.type === 'role'
                ? `Change ${confirmAction.user.email} role to ${confirmAction.newRole}?`
                : `${confirmAction.user.is_suspended ? 'Unsuspend' : 'Suspend'} ${confirmAction.user.email}?`}
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
                    handleRoleChange(confirmAction.user, confirmAction.newRole)
                  } else {
                    handleSuspend(confirmAction.user)
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

function Th({ children }: { children: React.ReactNode }) {
  return (
    <th className="px-3 py-2 text-xs uppercase tracking-wider text-text-muted font-normal text-left">
      {children}
    </th>
  )
}
