'use client'

import Link from 'next/link'
import { usePathname, useSearchParams } from 'next/navigation'
import { useAuth } from '@/lib/context'
import { useRouter } from 'next/navigation'
import { cn } from '@/lib/utils'

const NAV_LINKS = [
  { label: 'Dashboard', tab: null },
  { label: 'Users', tab: 'users' },
  { label: 'Pairs', tab: 'pairs' },
  { label: 'Assets', tab: 'assets' },
]

export function AdminNavbar() {
  const searchParams = useSearchParams()
  const tab = searchParams.get('tab')
  const { user, logout } = useAuth()
  const router = useRouter()

  async function handleLogout() {
    await logout()
    router.push('/trade')
  }

  return (
    <nav className="h-11 flex items-center justify-between px-4 border-b border-bg-border bg-bg-surface flex-shrink-0">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <span className="font-bold text-lg text-text-primary tracking-tight">Excentra</span>
          <span className="text-xs px-1.5 py-0.5 bg-accent/15 text-accent rounded-full">Admin</span>
        </div>

        <div className="flex items-center gap-1">
          {NAV_LINKS.map((link) => (
            <Link
              key={link.label}
              href={link.tab ? `/admin?tab=${link.tab}` : '/admin'}
              className={cn(
                'px-3 py-1.5 text-sm rounded transition-all duration-150',
                (tab === link.tab || (!tab && link.tab === null))
                  ? 'text-text-primary bg-bg-elevated'
                  : 'text-text-secondary hover:text-text-primary hover:bg-bg-elevated/50'
              )}
            >
              {link.label}
            </Link>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-3">
        <Link href="/trade" className="text-sm text-text-secondary hover:text-text-primary transition-colors duration-150">
          Trade
        </Link>
        <span className="text-xs text-text-muted">{user?.email}</span>
        <button
          onClick={handleLogout}
          className="text-xs text-text-muted hover:text-sell transition-colors duration-150"
        >
          Sign Out
        </button>
      </div>
    </nav>
  )
}
