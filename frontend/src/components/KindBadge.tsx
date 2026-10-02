'use client';

import { useT } from '@/i18n/client';
import { cn, getKindBadgeClass, getKindLabel, getKindShortLabel } from '@/lib/utils';

interface KindBadgeProps {
  kind: string;
  /** Use the short label ("Call") rather than the full one ("Call (confidential)"). */
  short?: boolean;
  className?: string;
}

export function KindBadge({ kind, short = false, className }: KindBadgeProps) {
  const { get } = useT();
  // A kind from a newer node has no dictionary entry: fall back to the English table, which
  // itself falls back to the raw kind name.
  const label = get<string | undefined>(`kinds.${kind}.label`) ?? getKindLabel(kind);
  const shortLabel = get<string | undefined>(`kinds.${kind}.short`) ?? getKindShortLabel(kind);
  return (
    <span className={cn(getKindBadgeClass(kind), className)} title={label}>
      {short ? shortLabel : label}
    </span>
  );
}
