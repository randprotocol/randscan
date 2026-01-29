import { cn } from '@/lib/utils';

interface LoadingProps {
  className?: string;
  size?: 'sm' | 'md' | 'lg';
}

export function Loading({ className, size = 'md' }: LoadingProps) {
  const sizeClasses = {
    sm: 'h-4 w-4 border-2',
    md: 'h-8 w-8 border-2',
    lg: 'h-12 w-12 border-3',
  };

  return (
    <div className={cn('flex items-center justify-center', className)}>
      <div
        className={cn(
          'animate-spin rounded-full border-dark-600 border-t-primary-500',
          sizeClasses[size]
        )}
      />
    </div>
  );
}

export function PageLoading() {
  return (
    <div className="flex h-96 items-center justify-center">
      <Loading size="lg" />
    </div>
  );
}

export function CardSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn('animate-pulse rounded-xl border border-dark-700 bg-dark-800 p-6', className)}>
      <div className="space-y-3">
        <div className="h-4 w-3/4 rounded bg-dark-700" />
        <div className="h-4 w-1/2 rounded bg-dark-700" />
      </div>
    </div>
  );
}

export function ListSkeleton({ count = 5 }: { count?: number }) {
  return (
    <div className="space-y-3">
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className="animate-pulse rounded-lg border border-dark-700 bg-dark-800 p-4"
        >
          <div className="flex items-center gap-4">
            <div className="h-10 w-10 rounded-full bg-dark-700" />
            <div className="flex-1 space-y-2">
              <div className="h-4 w-1/4 rounded bg-dark-700" />
              <div className="h-3 w-1/2 rounded bg-dark-700" />
            </div>
            <div className="h-4 w-16 rounded bg-dark-700" />
          </div>
        </div>
      ))}
    </div>
  );
}

export function DetailSkeleton() {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="animate-pulse">
        <div className="h-8 w-48 rounded bg-dark-700" />
        <div className="mt-2 h-4 w-96 rounded bg-dark-700" />
      </div>

      {/* Info cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <div
            key={i}
            className="animate-pulse rounded-xl border border-dark-700 bg-dark-800 p-6"
          >
            <div className="h-4 w-20 rounded bg-dark-700" />
            <div className="mt-3 h-6 w-32 rounded bg-dark-700" />
          </div>
        ))}
      </div>

      {/* Content */}
      <div className="animate-pulse rounded-xl border border-dark-700 bg-dark-800 p-6">
        <div className="space-y-4">
          {Array.from({ length: 6 }).map((_, i) => (
            <div key={i} className="flex justify-between border-b border-dark-700 pb-4">
              <div className="h-4 w-24 rounded bg-dark-700" />
              <div className="h-4 w-48 rounded bg-dark-700" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
