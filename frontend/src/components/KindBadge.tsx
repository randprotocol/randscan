import { cn, getKindBadgeClass, getKindLabel, getKindShortLabel } from '@/lib/utils';

interface KindBadgeProps {
  kind: string;
  /** Use the short label ("Call") rather than the full one ("Call (confidential)"). */
  short?: boolean;
  className?: string;
}

export function KindBadge({ kind, short = false, className }: KindBadgeProps) {
  return (
    <span className={cn(getKindBadgeClass(kind), className)} title={getKindLabel(kind)}>
      {short ? getKindShortLabel(kind) : getKindLabel(kind)}
    </span>
  );
}
