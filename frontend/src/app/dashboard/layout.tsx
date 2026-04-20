import { Suspense } from 'react'
import { Sidebar } from '@/components/shared/Sidebar'
import { ProtectedRoute } from '@/components/shared/ProtectedRoute'

export const dynamic = 'force-dynamic'

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <ProtectedRoute>
      <div className="flex h-screen bg-bg-base overflow-hidden">
        <Suspense fallback={<div className="w-[220px] bg-bg-surface border-r border-bg-border" />}>
          <Sidebar />
        </Suspense>
        <main className="flex-1 overflow-auto min-w-0 animate-fade-in">
          {children}
        </main>
      </div>
    </ProtectedRoute>
  )
}
