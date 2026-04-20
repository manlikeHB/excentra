'use client'

import { useState } from 'react'
import Link from 'next/link'
import { authApi } from '@/lib/api'

export default function ForgotPasswordPage() {
  const [email, setEmail] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [sent, setSent] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!email) {
      setError('Email is required')
      return
    }
    setIsLoading(true)
    setError('')
    try {
      await authApi.forgotPassword(email)
      setSent(true)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="min-h-screen bg-bg-base flex flex-col items-center justify-start pt-24 animate-fade-in">
      <div className="w-full max-w-sm px-4">
        <div className="bg-bg-surface border border-bg-border rounded-xl p-8">
          <Link href="/trade" className="font-bold text-xl text-text-primary tracking-tight">
            Excentra
          </Link>

          <h1 className="text-2xl font-semibold text-text-primary mt-6">Reset your password</h1>
          <p className="text-sm text-text-secondary mt-1 mb-8">
            Enter your email and we&apos;ll send you a reset link
          </p>

          {sent ? (
            <div className="p-4 bg-buy/10 border border-buy/20 rounded-md">
              <p className="text-sm text-buy">
                If an account exists for {email}, a reset link has been sent.
              </p>
            </div>
          ) : (
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="text-xs text-text-muted block mb-1">Email</label>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="you@example.com"
                  autoComplete="email"
                  className="w-full bg-bg-elevated border border-bg-border rounded-md py-2.5 px-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none focus:shadow-[0_0_0_3px_rgb(59_130_246_/_0.15)] transition-all duration-150"
                />
                {error && <p className="text-sell text-xs mt-1">{error}</p>}
              </div>

              <button
                type="submit"
                disabled={isLoading}
                className="w-full py-2.5 text-sm font-medium bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed"
              >
                {isLoading ? 'Sending...' : 'Send Reset Link'}
              </button>
            </form>
          )}
        </div>

        <p className="text-sm text-text-muted text-center mt-4">
          <Link href="/login" className="text-accent hover:text-accent-hover transition-colors duration-150">
            Back to sign in
          </Link>
        </p>
      </div>
    </div>
  )
}
