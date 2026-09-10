'use client';

import Link from 'next/link';
import { useCallback, useEffect, useState } from 'react';
import { cn, copyToClipboard, shortenHash } from '@/lib/utils';

interface CopyButtonProps {
  value: string;
  className?: string;
  label?: string;
}

export function CopyButton({ value, className, label = 'Copy' }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const timer = setTimeout(() => setCopied(false), 1500);
    return () => clearTimeout(timer);
  }, [copied]);

  const onCopy = useCallback(
    async (event: React.MouseEvent) => {
      event.preventDefault();
      event.stopPropagation();
      const ok = await copyToClipboard(value);
      if (ok) setCopied(true);
    },
    [value]
  );

  return (
    <button
      type="button"
      onClick={onCopy}
      title={copied ? 'Copied' : label}
      aria-label={copied ? 'Copied' : label}
      className={cn(
        'inline-flex flex-shrink-0 items-center text-mute transition-colors hover:text-accent',
        className
      )}
    >
      {copied ? (
        <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
        </svg>
      ) : (
        <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h8a2 2 0 002-2v-2M8 5a2 2 0 002 2h4a2 2 0 002-2M8 5a2 2 0 012-2h4a2 2 0 012 2m0 0h2a2 2 0 012 2v3"
          />
        </svg>
      )}
    </button>
  );
}

interface HashProps {
  value: string | null | undefined;
  /** When set the hash renders as a link to this href. */
  href?: string | null;
  /** Render the full value instead of a shortened one. */
  full?: boolean;
  start?: number;
  end?: number;
  copyable?: boolean;
  className?: string;
}

/** Monospace hash/address with optional link and copy-to-clipboard button. */
export function Hash({
  value,
  href,
  full = false,
  start = 8,
  end = 6,
  copyable = true,
  className,
}: HashProps) {
  if (!value) return <span className="text-mute">—</span>;

  const text = full ? value : shortenHash(value, start, end);

  const body = href ? (
    <Link href={href} title={value} className={cn('link font-mono', full && 'break-all')}>
      {text}
    </Link>
  ) : (
    <span title={value} className={cn('font-mono text-soft', full && 'break-all')}>
      {text}
    </span>
  );

  return (
    <span className={cn('inline-flex min-w-0 items-center gap-1.5', className)}>
      {body}
      {copyable && <CopyButton value={value} />}
    </span>
  );
}
