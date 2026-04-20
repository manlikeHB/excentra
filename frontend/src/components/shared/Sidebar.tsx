'use client'

import Link from 'next/link'
import { usePathname, useSearchParams } from 'next/navigation'
import {
  LayoutDashboard,
  Wallet,
  ClipboardList,
  Settings,
  ArrowLeftRight,
} from 'lucide-react'
import { useAuth } from '@/lib/context'
import { cn } from '@/lib/utils'

const navItems = [
  { label: 'Dashboard', href: '/dashboard', icon: LayoutDashboard, tab: null },
  { label: 'Assets', href: '/dashboard?tab=assets', icon: Wallet, tab: 'assets' },
  { label: 'Orders', href: '/dashboard?tab=orders', icon: ClipboardList, tab: 'orders' },
  { label: 'Settings', href: '/dashboard?tab=settings', icon: Settings, tab: 'settings' },
]

export function Sidebar() {
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const tab = searchParams.get('tab')
  const { user } = useAuth()

  function isActive(item: (typeof navItems)[0]) {
    if (pathname !== '/dashboard') return false
    if (item.tab === null) return !tab
    return tab === item.tab
  }

  return (
    <aside className="w-[220px] flex-shrink-0 bg-bg-surface border-r border-bg-border flex flex-col h-full">
      <div className="h-14 flex items-center px-4 border-b border-bg-border">
        <Link href="/trade" className="font-bold text-lg text-text-primary tracking-tight">
          Excentra
        </Link>
      </div>

      <nav className="flex-1 py-3 px-2 flex flex-col gap-0.5">
        {navItems.map((item) => (
          <Link
            key={item.label}
            href={item.href}
            className={cn(
              'flex items-center gap-2.5 px-3 py-2 text-sm rounded-md transition-all duration-150',
              isActive(item)
                ? 'border-l-2 border-accent bg-bg-elevated text-text-primary pl-[10px]'
                : 'text-text-secondary hover:text-text-primary hover:bg-bg-elevated border-l-2 border-transparent pl-[10px]'
            )}
          >
            <item.icon size={15} />
            {item.label}
          </Link>
        ))}

        <div className="border-t border-bg-border mt-2 pt-2">
          <Link
            href="/trade"
            className="flex items-center gap-2.5 px-3 py-2 text-sm rounded-md text-text-secondary hover:text-text-primary hover:bg-bg-elevated transition-all duration-150 border-l-2 border-transparent pl-[10px]"
          >
            <ArrowLeftRight size={15} />
            Trade
          </Link>
        </div>
      </nav>

      {user && (
        <div className="p-3 border-t border-bg-border">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 rounded-full bg-accent/20 flex items-center justify-center text-xs font-medium text-accent flex-shrink-0">
              {(user.username || user.email)[0].toUpperCase()}
            </div>
            <div className="min-w-0">
              <p className="text-xs text-text-primary truncate">{user.username ?? user.email}</p>
              <span
                className={cn(
                  'text-xs px-1.5 py-0.5 rounded-full',
                  user.role === 'admin'
                    ? 'bg-accent/15 text-accent'
                    : 'bg-bg-elevated text-text-muted'
                )}
              >
                {user.role}
              </span>
            </div>
          </div>
        </div>
      )}
    </aside>
  )
}
