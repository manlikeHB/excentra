import { Suspense } from 'react'
import { ProtectedRoute } from '@/components/shared/ProtectedRoute'
import { AdminNavbar } from './AdminNavbar'

export const dynamic = 'force-dynamic'

export default function AdminLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <ProtectedRoute requireAdmin>
      <div className="flex flex-col h-screen bg-bg-base overflow-hidden">
        <Suspense fallback={<div className="h-11 bg-bg-surface border-b border-bg-border" />}>
          <AdminNavbar />
        </Suspense>
        <main className="flex-1 overflow-auto animate-fade-in">
          {children}
        </main>
      </div>
    </ProtectedRoute>
  )
}
