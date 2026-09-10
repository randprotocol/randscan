import { cn } from '@/lib/utils';

interface LoadingProps {
  className?: string;
  size?: 'sm' | 'md' | 'lg';
}

export function Loading({ className, size = 'md' }: LoadingProps) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-6 w-6',
    lg: 'h-8 w-8',
  };

  return (
    <div className={cn('flex items-center justify-center', className)}>
      <div
        className={cn(
          'animate-spin rounded-full border-2 border-border border-t-accent',
          sizeClasses[size]
        )}
      />
    </div>
  );
}

export function PageLoading() {
  return (
    <div className="flex h-64 items-center justify-center">
      <Loading size="lg" />
    </div>
  );
}

export function CardSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn('card-padded animate-pulse', className)}>
      <div className="space-y-3">
        <div className="h-3.5 w-3/4 rounded bg-bg-soft" />
        <div className="h-3.5 w-1/2 rounded bg-bg-soft" />
      </div>
    </div>
  );
}

export function ListSkeleton({ count = 5 }: { count?: number }) {
  return (
    <div className="card divide-y divide-border-soft">
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="animate-pulse p-4">
          <div className="flex items-center gap-4">
            <div className="h-8 w-8 rounded bg-bg-soft" />
            <div className="flex-1 space-y-2">
              <div className="h-3.5 w-1/4 rounded bg-bg-soft" />
              <div className="h-3 w-1/2 rounded bg-bg-soft" />
            </div>
            <div className="h-3.5 w-16 rounded bg-bg-soft" />
          </div>
        </div>
      ))}
    </div>
  );
}

export function DetailSkeleton() {
  return (
    <div className="space-y-8">
      <div className="animate-pulse">
        <div className="h-9 w-56 rounded bg-bg-soft" />
        <div className="mt-3 h-3.5 w-96 max-w-full rounded bg-bg-soft" />
      </div>

      <div className="stat-row grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="stat animate-pulse">
            <div className="h-8 w-24 rounded bg-bg-soft" />
            <div className="mt-2 h-3 w-16 rounded bg-bg-soft" />
          </div>
        ))}
      </div>

      <div className="card-padded animate-pulse">
        <div className="space-y-4">
          {Array.from({ length: 6 }).map((_, i) => (
            <div key={i} className="flex justify-between border-b border-border-soft pb-4 last:border-0">
              <div className="h-3.5 w-24 rounded bg-bg-soft" />
              <div className="h-3.5 w-48 rounded bg-bg-soft" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
