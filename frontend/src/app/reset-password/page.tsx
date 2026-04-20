'use client'

import { Suspense } from 'react'
import { useState } from 'react'
import Link from 'next/link'
import { useRouter, useSearchParams } from 'next/navigation'
import { authApi } from '@/lib/api'
import { Eye, EyeOff } from 'lucide-react'
import { toast } from 'sonner'

function ResetPasswordForm() {
  const searchParams = useSearchParams()
  const token = searchParams.get('token') ?? ''
  const router = useRouter()

  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!password || password.length < 8) {
      setError('Password must be at least 8 characters')
      return
    }
    if (!token) {
      setError('Invalid reset link')
      return
    }

    setIsLoading(true)
    setError('')
    try {
      await authApi.resetPassword(token, password)
      toast.success('Password updated')
      router.push('/login')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to reset password')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="w-full max-w-sm px-4">
      <div className="bg-bg-surface border border-bg-border rounded-xl p-8">
        <Link href="/trade" className="font-bold text-xl text-text-primary tracking-tight">
          Excentra
        </Link>

        <h1 className="text-2xl font-semibold text-text-primary mt-6">Set new password</h1>
        <p className="text-sm text-text-secondary mt-1 mb-8">
          Enter your new password below
        </p>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="text-xs text-text-muted block mb-1">New Password</label>
            <div className="relative">
              <input
                type={showPassword ? 'text' : 'password'}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
                autoComplete="new-password"
                className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 pr-10 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150"
              />
              <button
                type="button"
                onClick={() => setShowPassword((v) => !v)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-secondary transition-colors duration-150"
              >
                {showPassword ? <EyeOff size={14} /> : <Eye size={14} />}
              </button>
            </div>
            <p className="text-text-muted text-xs mt-1">Minimum 8 characters</p>
            {error && <p className="text-sell text-xs mt-1">{error}</p>}
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="w-full py-2.5 text-sm font-medium bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {isLoading ? 'Updating...' : 'Reset Password'}
          </button>
        </form>
      </div>
    </div>
  )
}

export default function ResetPasswordPage() {
  return (
    <div className="min-h-screen bg-bg-base flex flex-col items-center justify-start pt-24 animate-fade-in">
      <Suspense fallback={<div className="w-5 h-5 border-2 border-accent border-t-transparent rounded-full animate-spin" />}>
        <ResetPasswordForm />
      </Suspense>
    </div>
  )
}
