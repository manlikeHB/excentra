import { cn } from '@/lib/utils'

interface SkeletonProps {
  className?: string
}

export function Skeleton({ className }: SkeletonProps) {
  return (
    <div
      className={cn(
        'skeleton rounded',
        className
      )}
    />
  )
}

export function SkeletonText({ className }: SkeletonProps) {
  return <Skeleton className={cn('h-4 w-full', className)} />
}

export function SkeletonRow() {
  return (
    <div className="flex items-center gap-2 py-1.5 px-3">
      <Skeleton className="h-3 w-20" />
      <Skeleton className="h-3 w-16" />
      <Skeleton className="h-3 w-16" />
    </div>
  )
}
