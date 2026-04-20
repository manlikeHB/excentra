'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { useAuth } from '@/lib/context'

interface ProtectedRouteProps {
  children: React.ReactNode
  requireAdmin?: boolean
  redirectTo?: string
}

export function ProtectedRoute({
  children,
  requireAdmin = false,
  redirectTo = '/login',
}: ProtectedRouteProps) {
  const { user, isLoading } = useAuth()
  const router = useRouter()

  useEffect(() => {
    if (isLoading) return
    if (!user) {
      router.replace(redirectTo)
      return
    }
    if (requireAdmin && user.role !== 'admin') {
      router.replace('/trade')
    }
  }, [user, isLoading, requireAdmin, redirectTo, router])

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="w-5 h-5 border-2 border-accent border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  if (!user) return null
  if (requireAdmin && user.role !== 'admin') return null

  return <>{children}</>
}
