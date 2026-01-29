import { cn } from '@/lib/utils';

interface StatsCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon?: React.ReactNode;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  className?: string;
}

export function StatsCard({ title, value, subtitle, icon, trend, className }: StatsCardProps) {
  return (
    <div
      className={cn(
        'rounded-xl border border-dark-700 bg-dark-800 p-6',
        className
      )}
    >
      <div className="flex items-start justify-between">
        <div>
          <p className="text-sm font-medium text-dark-400">{title}</p>
          <p className="mt-2 text-2xl font-bold text-white">{value}</p>
          {subtitle && (
            <p className="mt-1 text-sm text-dark-400">{subtitle}</p>
          )}
          {trend && (
            <div
              className={cn(
                'mt-2 inline-flex items-center gap-1 text-sm font-medium',
                trend.isPositive ? 'text-green-500' : 'text-red-500'
              )}
            >
              <svg
                className={cn('h-4 w-4', !trend.isPositive && 'rotate-180')}
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M5 10l7-7m0 0l7 7m-7-7v18"
                />
              </svg>
              {Math.abs(trend.value)}%
            </div>
          )}
        </div>
        {icon && (
          <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-dark-700 text-primary-400">
            {icon}
          </div>
        )}
      </div>
    </div>
  );
}

// Skeleton version for loading states
export function StatsCardSkeleton() {
  return (
    <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="h-4 w-24 animate-pulse rounded bg-dark-700" />
          <div className="mt-3 h-8 w-32 animate-pulse rounded bg-dark-700" />
          <div className="mt-2 h-4 w-20 animate-pulse rounded bg-dark-700" />
        </div>
        <div className="h-12 w-12 animate-pulse rounded-lg bg-dark-700" />
      </div>
    </div>
  );
}
