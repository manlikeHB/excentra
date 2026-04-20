'use client'

import { useState } from 'react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useAuth } from '@/lib/context'
import { Eye, EyeOff } from 'lucide-react'
import { toast } from 'sonner'

export default function RegisterPage() {
  const { register } = useAuth()
  const router = useRouter()

  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [errors, setErrors] = useState<{ email?: string; password?: string; form?: string }>({})

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setErrors({})

    const newErrors: typeof errors = {}
    if (!email) newErrors.email = 'Email is required'
    if (!password) newErrors.password = 'Password is required'
    if (password && password.length < 8) newErrors.password = 'Password must be at least 8 characters'
    if (Object.keys(newErrors).length) {
      setErrors(newErrors)
      return
    }

    setIsLoading(true)
    try {
      await register(email, password)
      toast.success('Welcome to Excentra')
      router.push('/trade')
    } catch (err) {
      setErrors({ form: err instanceof Error ? err.message : 'Registration failed' })
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

          <h1 className="text-2xl font-semibold text-text-primary mt-6">Create your account</h1>
          <p className="text-sm text-text-secondary mt-1 mb-8">Start trading on Excentra</p>

          {errors.form && (
            <div className="mb-4 p-3 bg-sell/10 border border-sell/20 rounded-md">
              <p className="text-xs text-sell">{errors.form}</p>
            </div>
          )}

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
              {errors.email && <p className="text-sell text-xs mt-1">{errors.email}</p>}
            </div>

            <div>
              <label className="text-xs text-text-muted block mb-1">Password</label>
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
              {errors.password && <p className="text-sell text-xs mt-1">{errors.password}</p>}
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="w-full mt-6 py-2.5 text-sm font-medium bg-accent text-white rounded-md hover:bg-accent-hover transition-all duration-150 active:scale-[0.98] disabled:opacity-60 disabled:cursor-not-allowed"
            >
              {isLoading ? 'Creating account...' : 'Create Account'}
            </button>
          </form>
        </div>

        <p className="text-sm text-text-muted text-center mt-4">
          Already have an account?{' '}
          <Link href="/login" className="text-accent hover:text-accent-hover transition-colors duration-150">
            Sign in
          </Link>
        </p>
      </div>
    </div>
  )
}
