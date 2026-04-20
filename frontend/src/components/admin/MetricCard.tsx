import { cn } from '@/lib/utils'

interface MetricCardProps {
  label: string
  value: string | number
  icon: React.ReactNode
  className?: string
}

export function MetricCard({ label, value, icon, className }: MetricCardProps) {
  return (
    <div className={cn('bg-bg-surface border border-bg-border rounded-lg p-5', className)}>
      <div className="flex items-center gap-3 mb-3">
        <div className="text-text-muted">{icon}</div>
        <span className="text-xs uppercase tracking-wider text-text-muted">{label}</span>
      </div>
      <p className="text-2xl font-semibold font-mono text-text-primary">{value}</p>
    </div>
  )
}
