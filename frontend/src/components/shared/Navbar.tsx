'use client'

import { useState, useRef, useEffect } from 'react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { ChevronDown, LayoutDashboard, ClipboardList, Settings, LogOut, Shield } from 'lucide-react'
import { useAuth } from '@/lib/context'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'

export function Navbar() {
  const { user, logout } = useAuth()
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  async function handleLogout() {
    setOpen(false)
    await logout()
    router.push('/trade')
  }

  return (
    <nav className="h-11 flex items-center justify-between px-4 border-b border-bg-border bg-bg-surface flex-shrink-0">
      <Link href="/trade" className="font-bold text-lg text-text-primary tracking-tight">
        Excentra
      </Link>

      <div className="flex items-center gap-2">
        {user ? (
          <div className="relative" ref={dropdownRef}>
            <button
              onClick={() => setOpen((v) => !v)}
              className="flex items-center gap-1.5 text-sm text-text-secondary hover:text-text-primary transition-colors duration-150 px-2 py-1 rounded-md hover:bg-bg-elevated"
            >
              <div className="w-6 h-6 rounded-full bg-accent/20 flex items-center justify-center text-xs font-medium text-accent">
                {(user.username || user.email)[0].toUpperCase()}
              </div>
              <span className="max-w-[140px] truncate">{user.username ?? user.email}</span>
              <ChevronDown
                size={14}
                className={cn('transition-transform duration-150', open && 'rotate-180')}
              />
            </button>

            {open && (
              <div className="absolute right-0 top-full mt-1 w-52 bg-bg-surface border border-bg-border rounded-lg shadow-xl py-1 z-50 animate-scale-in">
                <div className="px-3 py-2 border-b border-bg-border mb-1">
                  <p className="text-xs font-medium text-text-primary truncate">
                    {user.username ?? user.email}
                  </p>
                  {user.username && (
                    <p className="text-xs text-text-muted truncate">{user.email}</p>
                  )}
                  {user.role === 'admin' && (
                    <span className="text-xs bg-accent/15 text-accent px-1.5 py-0.5 rounded-full mt-1 inline-block">
                      Admin
                    </span>
                  )}
                </div>

                <NavDropdownItem href="/dashboard" icon={<LayoutDashboard size={14} />} onClick={() => setOpen(false)}>
                  Dashboard
                </NavDropdownItem>
                <NavDropdownItem href="/dashboard?tab=orders" icon={<ClipboardList size={14} />} onClick={() => setOpen(false)}>
                  Orders
                </NavDropdownItem>
                <NavDropdownItem href="/dashboard?tab=settings" icon={<Settings size={14} />} onClick={() => setOpen(false)}>
                  Settings
                </NavDropdownItem>
                {user.role === 'admin' && (
                  <NavDropdownItem href="/admin" icon={<Shield size={14} />} onClick={() => setOpen(false)}>
                    Admin Panel
                  </NavDropdownItem>
                )}

                <div className="border-t border-bg-border mt-1 pt-1">
                  <button
                    onClick={handleLogout}
                    className="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-text-secondary hover:text-sell hover:bg-sell/5 transition-colors duration-150"
                  >
                    <LogOut size={14} />
                    Sign Out
                  </button>
                </div>
              </div>
            )}
          </div>
        ) : (
          <>
            <Link
              href="/login"
              className="px-3 py-1.5 text-sm text-text-secondary border border-bg-border rounded-md hover:bg-bg-elevated hover:text-text-primary transition-all duration-150"
            >
              Sign In
            </Link>
            <Link
              href="/register"
              className="px-3 py-1.5 text-sm font-medium bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98]"
            >
              Create Account
            </Link>
          </>
        )}
      </div>
    </nav>
  )
}

function NavDropdownItem({
  href,
  icon,
  children,
  onClick,
}: {
  href: string
  icon: React.ReactNode
  children: React.ReactNode
  onClick?: () => void
}) {
  return (
    <Link
      href={href}
      onClick={onClick}
      className="flex items-center gap-2 px-3 py-1.5 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-elevated transition-colors duration-150"
    >
      {icon}
      {children}
    </Link>
  )
}
