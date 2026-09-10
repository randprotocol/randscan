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

/**
 * A single cell of a hairline-separated stat row: big number, small label,
 * smaller muted sub-label. Wrap a group in <StatsRow> to get the rules.
 */
export function StatsCard({ title, value, subtitle, trend, className }: StatsCardProps) {
  return (
    <div className={cn('stat', className)}>
      <p className="stat-value">{value}</p>
      <p className="stat-label">{title}</p>
      {subtitle && <p className="stat-sub">{subtitle}</p>}
      {trend && (
        <p className={cn('stat-sub', trend.isPositive ? 'text-accent' : 'text-accent-3')}>
          {trend.isPositive ? '+' : '−'}
          {Math.abs(trend.value)}%
        </p>
      )}
    </div>
  );
}

interface StatsRowProps {
  children: React.ReactNode;
  /** Number of columns on large screens; cells stack below `sm`. */
  columns?: 3 | 4 | 5;
  className?: string;
}

const columnClasses: Record<number, string> = {
  3: 'sm:grid-cols-3',
  4: 'sm:grid-cols-2 lg:grid-cols-4',
  5: 'sm:grid-cols-2 lg:grid-cols-5',
};

export function StatsRow({ children, columns = 4, className }: StatsRowProps) {
  return (
    <div className={cn('stat-row grid-cols-1', columnClasses[columns], className)}>{children}</div>
  );
}

export function StatsCardSkeleton() {
  return (
    <div className="stat">
      <div className="h-8 w-28 animate-pulse rounded bg-bg-soft" />
      <div className="mt-2 h-3 w-20 animate-pulse rounded bg-bg-soft" />
    </div>
  );
}
